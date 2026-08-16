use crate::defs::{
    apply_armor, armor_pen, creep_stats, strike_pen, tier_name, wave_bounty_mul, wave_hp_mul,
    wave_speed_mul, BuildKind, CreepKind, FireMode, StrikeKind, TargetMode, COLOSSUS_ROAR_PERIOD,
    COLOSSUS_ROAR_RADIUS, COLOSSUS_STUN, DT, FIRST_WAVE_DELAY, FLICKER_PERIOD, FLICKER_STEPS,
    HELIOS_CONVERT_COST, MAX_TIER, MEDIC_HEAL_PER_SEC, MEDIC_HEAL_RADIUS, MOVE_COST,
    OVERCHARGE_COST, OVERCHARGE_FIRE_MUL, OVERCHARGE_TTL, REPAIR_COST, SELL_RATIO,
    STARTING_INTEGRITY, WAVE_DELAY,
};
use crate::director::{WavePlan, WaveScript};
use crate::geom::Vec2;
use crate::grid::{Grid, Occupant};
use crate::mapdoc::{grid_to_doc, MapDoc, MapError, WORKSHOP_MAP_ID};
use crate::modifiers::{MatchRules, Modifier};
use crate::orders::{replay_hash, Order, OrderOp, ReplayFile, RunOutcome};
use crate::pack::{Loadout, PackDoc, PackError};
use crate::path::FlowField;
use crate::rng::Rng;
use crate::snapshot::{
    AfterAction, BeamView, CreepView, FxView, GunScore, HoverInfo, KindCount, MapStatic, ProjView,
    SelectedInfo, Snapshot, StrikeHud, TowerView, WaveIntel,
};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    OutOfBounds,
    Occupied,
    CantAfford,
    BlocksPath,
    TrapsCreeps,
    NotATurret,
    MaxTier,
    NotHelios,
    AlreadyAir,
    TurretCap,
    NotInLoadout,
    Intact,
    RelayDown,
    NothingToMove,
}

impl PlaceError {
    pub fn message(self) -> &'static str {
        match self {
            Self::OutOfBounds => "Off the grid",
            Self::Occupied => "Can't build there",
            Self::CantAfford => "Not enough scrap",
            Self::BlocksPath => "That would cut off the relay",
            Self::TrapsCreeps => "That would trap someone with nowhere to walk",
            Self::NotATurret => "Nothing to upgrade",
            Self::MaxTier => "Already maxed",
            Self::NotHelios => "Only Helios converts",
            Self::AlreadyAir => "Already air-tuned",
            Self::TurretCap => "Turret ceiling reached",
            Self::NotInLoadout => "Not in this loadout",
            Self::Intact => "Relay is intact",
            Self::RelayDown => "Relay is down",
            Self::NothingToMove => "Select a structure first",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Fortify { remaining: f32 },
    Incoming,
    Defeat,
}

struct Tower {
    id: u32,
    kind: BuildKind,
    x: i32,
    y: i32,
    tier: u8,
    cooldown: f32,
    aim: f32,
    target: Option<u32>,
    invested: i32,
    kills: u32,
    damage_dealt: f32,
    target_mode: TargetMode,
    air_focus: bool,
    stun_ttl: f32,
    overcharge_ttl: f32,
}

struct Creep {
    id: u32,
    kind: CreepKind,
    pos: Vec2,
    vel: Vec2,
    hp: f32,
    hp_max: f32,
    speed: f32,
    bounty: i32,
    leak: i32,
    armor: f32,
    flying: bool,
    camo: bool,
    radius: f32,
    heading: f32,
    weave: f32,
    slow_mul: f32,
    slow_ttl: f32,
    split_gen: u8,
    blink_cd: f32,
    roar_cd: f32,
    hit_by: Option<u32>,
}

struct Projectile {
    pos: Vec2,
    vel: Vec2,
    target: Option<u32>,
    owner: u32,
    damage: f32,
    splash: f32,
    hits_ground: bool,
    hits_air: bool,
    detects: bool,
    kind: BuildKind,
    ttl: f32,
    homing: bool,
}

struct Fx {
    kind: &'static str,
    pos: Vec2,
    ttl: f32,
    max: f32,
    mag: f32,
    heading: f32,
}

struct Beam {
    a: Vec2,
    b: Vec2,
    kind: BuildKind,
    ttl: f32,
    max: f32,
}

struct SpawnOrder {
    kind: CreepKind,
    spawn: (i32, i32),
}

#[derive(Clone, Copy)]
struct Hand {
    build: BuildKind,
    strike: StrikeKind,
    hover: Option<(i32, i32)>,
    selected: Option<u32>,
    lift: Option<u32>,
}

impl Hand {
    fn fresh() -> Self {
        Self {
            build: BuildKind::Inspect,
            strike: StrikeKind::None,
            hover: None,
            selected: None,
            lift: None,
        }
    }
}

pub struct Game {
    grid: Grid,
    flow: FlowField,
    /// Relay centroids, cached at construction. Terrain never changes during a match, and
    /// `Grid::nearest_core` was rescanning all cells + allocating a HashSet per call —
    /// once per flying creep per tower inside `acquire`.
    core_clusters: Vec<Vec2>,
    core_mid: Vec2,
    /// Copy mirror of `loadout.guns`, refreshed on pack apply. `tick_towers` reads only
    /// this, so it no longer clones the String-bearing Loadout 60x/second.
    guns_hot: crate::pack::GunTable,
    rng: Rng,
    towers: Vec<Tower>,
    creeps: Vec<Creep>,
    projectiles: Vec<Projectile>,
    fx: VecDeque<Fx>,
    beams: VecDeque<Beam>,
    spawn_q: VecDeque<SpawnOrder>,
    spawn_cd: f32,
    next_id: u32,
    credits: i32,
    integrity: i32,
    integrity_max: i32,
    wave: u32,
    phase: Phase,
    time: f32,
    tick: u64,
    hand: Hand,
    strike_cd: [f32; 4],
    banner: Option<(String, f32)>,
    message: Option<(String, f32)>,
    hurt_flash: f32,
    kills: u32,
    leaks: u32,
    map_name: String,
    map_id: u8,
    modifier: Modifier,
    seed: u64,
    recording: bool,
    orders: Vec<Order>,
    hold_until: Option<u32>,
    objective_cleared: bool,
    mission_id: Option<u8>,
    challenge_id: Option<u8>,
    mission_name: Option<String>,
    loadout: Loadout,
    pack: Option<PackDoc>,
    spent: i32,
    last_interest: i32,
    kill_kinds: [u32; 9],
    leak_kinds: [u32; 9],
    wave_script: WaveScript,
    order_idx: usize,
}

impl Game {
    pub fn kilo_outpost() -> Self {
        Self::theater(0)
    }

    pub fn theater(id: u8) -> Self {
        Self::start(id, Modifier::Standard, None)
    }

    pub fn start(map_id: u8, modifier: Modifier, seed_override: Option<u64>) -> Self {
        // Resolve the id and the theater together. Storing an unresolved id while playing
        // the fallback theater desynchronises best-wave keys and the replay bundle — and
        // id 255 is WORKSHOP_MAP_ID, so it would emit a forged workshop replay.
        let resolved = if crate::maps::theater_by_id(map_id).is_some() {
            map_id
        } else {
            0
        };
        let (grid, name, seed) = crate::maps::theater_by_id(resolved).expect("kilo theater");
        Self::with_modifier(
            grid,
            name,
            seed_override.unwrap_or(seed),
            resolved,
            modifier,
        )
    }

    pub fn daily(utc_day: u32) -> Self {
        let pick = crate::modifiers::daily_pick(utc_day);
        Self::start(
            pick.map_id,
            Modifier::from_u8(pick.modifier_id),
            Some(pick.seed),
        )
    }

    pub fn mission(id: u8) -> Option<Self> {
        let m = crate::campaign::mission_by_id(id)?;
        let mut g = Self::start(m.map_id, Modifier::from_u8(m.modifier_id), Some(m.seed));
        g.hold_until = Some(m.hold_until_wave);
        g.mission_id = Some(m.id);
        g.mission_name = Some(m.name.clone());
        g.banner = Some((
            format!(
                "{}  ·  {}",
                m.name.to_uppercase(),
                m.objective.to_uppercase()
            ),
            4.2,
        ));
        Some(g)
    }

    pub fn challenge(id: u8) -> Option<Self> {
        let c = crate::campaign::challenge_by_id(id)?;
        let mut g = Self::start(c.map_id, Modifier::from_u8(c.modifier_id), Some(c.seed));
        g.hold_until = c.hold_until_wave;
        g.challenge_id = Some(c.id);
        g.mission_name = Some(c.name.clone());
        g.banner = Some((c.name.to_uppercase(), 3.4));
        Some(g)
    }

    pub fn from_doc(doc: MapDoc, modifier: Modifier) -> Result<Self, MapError> {
        let name = doc.name.clone();
        let seed = doc.seed;
        let grid = crate::mapdoc::validate_map(&doc)?;
        Ok(Self::with_modifier(
            grid,
            &name,
            seed,
            WORKSHOP_MAP_ID,
            modifier,
        ))
    }

    pub fn from_map_json(raw: &str, modifier: Modifier) -> Result<Self, MapError> {
        let (doc, _) = crate::mapdoc::parse_and_validate(raw)?;
        Self::from_doc(doc, modifier)
    }

    pub fn from_replay(file: ReplayFile) -> Result<Self, MapError> {
        let mut game = if let Some(doc) = file.map {
            Self::from_doc(doc, Modifier::from_u8(file.modifier_id))?
        } else {
            Self::start(
                file.map_id.unwrap_or(0),
                Modifier::from_u8(file.modifier_id),
                Some(file.seed),
            )
        };
        game.recording = false;
        game.orders = file.orders;
        game.order_idx = 0;
        if let Some(pack) = file.pack {
            game.apply_pack(pack)
                .map_err(|e| MapError::Parse(e.message()))?;
        }
        Ok(game)
    }

    pub fn from_replay_json(raw: &str) -> Result<Self, MapError> {
        let file: ReplayFile =
            serde_json::from_str(raw).map_err(|e| MapError::Parse(e.to_string()))?;
        Self::from_replay(file)
    }

    pub fn apply_pack(&mut self, doc: PackDoc) -> Result<(), PackError> {
        let loadout = Loadout::from_doc(&doc)?;
        self.pack = if loadout.is_stock() { None } else { Some(doc) };
        self.guns_hot = loadout.guns_table();
        self.loadout = loadout;
        Ok(())
    }

    pub fn apply_pack_json(&mut self, raw: &str) -> Result<(), PackError> {
        let doc = crate::pack::parse_pack_json(raw)?;
        self.apply_pack(doc)
    }

    pub fn match_catalog_json(&self) -> String {
        serde_json::to_string(&self.loadout.catalog_items()).expect("catalog")
    }

    pub fn match_strikes_json(&self) -> String {
        serde_json::to_string(&self.loadout.strike_items()).expect("strikes")
    }

    pub fn new(grid: Grid, name: &str, seed: u64, map_id: u8) -> Self {
        Self::with_modifier(grid, name, seed, map_id, Modifier::Standard)
    }

    pub fn with_modifier(
        grid: Grid,
        name: &str,
        seed: u64,
        map_id: u8,
        modifier: Modifier,
    ) -> Self {
        let flow = FlowField::compute(&grid);
        let rules = modifier.rules();
        let core_clusters = grid.core_clusters();
        let core_mid = grid.core_center();
        let stock = Loadout::stock();
        let guns_hot = stock.guns_table();
        let mut game = Self {
            grid,
            flow,
            core_clusters,
            core_mid,
            guns_hot,
            rng: Rng::new(seed),
            towers: Vec::new(),
            creeps: Vec::new(),
            projectiles: Vec::new(),
            fx: VecDeque::new(),
            beams: VecDeque::new(),
            spawn_q: VecDeque::new(),
            spawn_cd: 0.0,
            next_id: 1,
            credits: rules.starting_credits,
            integrity: STARTING_INTEGRITY,
            integrity_max: STARTING_INTEGRITY,
            wave: 1,
            phase: Phase::Fortify {
                remaining: FIRST_WAVE_DELAY,
            },
            time: 0.0,
            tick: 0,
            hand: Hand::fresh(),
            strike_cd: [0.0; 4],
            banner: Some((modifier.opening_banner().into(), 3.2)),
            message: None,
            hurt_flash: 0.0,
            kills: 0,
            leaks: 0,
            map_name: name.to_string(),
            map_id,
            modifier,
            seed,
            recording: true,
            orders: Vec::new(),
            hold_until: None,
            objective_cleared: false,
            mission_id: None,
            challenge_id: None,
            mission_name: None,
            loadout: stock,
            pack: None,
            spent: 0,
            last_interest: 0,
            kill_kinds: [0; 9],
            leak_kinds: [0; 9],
            wave_script: WaveScript::Mixed,
            order_idx: 0,
        };
        game.recompute_flow();
        game
    }

    fn rules(&self) -> MatchRules {
        self.modifier.rules()
    }

    fn turret_count(&self) -> u32 {
        self.towers.iter().filter(|t| t.kind.is_turret()).count() as u32
    }

    fn alloc(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn recompute_flow(&mut self) {
        self.flow = FlowField::compute(&self.grid);
    }

    pub fn map_static(&self) -> MapStatic {
        MapStatic {
            w: self.grid.w,
            h: self.grid.h,
            id: self.map_id,
            name: self.map_name.clone(),
            slug: crate::maps::theaters()
                .into_iter()
                .find(|t| t.id == self.map_id)
                .map(|t| t.slug.to_string())
                .unwrap_or_else(|| "workshop".to_string()),
            seed: self.seed,
            core: self.grid.cores().into_iter().map(|(x, y)| [x, y]).collect(),
            spawns: self
                .grid
                .spawns()
                .into_iter()
                .map(|(x, y)| [x, y])
                .collect(),
            rocks: self.grid.rocks().into_iter().map(|(x, y)| [x, y]).collect(),
        }
    }

    fn note(&mut self, op: OrderOp) {
        if self.recording {
            self.orders.push(Order {
                tick: self.tick,
                op,
            });
        }
    }

    pub fn set_build(&mut self, kind: u8) {
        self.note(OrderOp::SetBuild { kind });
        let hand = &mut self.hand;
        hand.build = BuildKind::from_u8(kind);
        hand.lift = None;
        if hand.build.is_structure() {
            hand.selected = None;
            hand.strike = StrikeKind::None;
        }
    }

    pub fn set_strike(&mut self, kind: u8) {
        self.note(OrderOp::SetStrike { kind });
        let hand = &mut self.hand;
        hand.strike = StrikeKind::from_u8(kind);
        if hand.strike != StrikeKind::None {
            hand.build = BuildKind::Inspect;
            hand.selected = None;
            hand.lift = None;
        }
    }

    pub fn set_hover(&mut self, x: i32, y: i32) {
        let hover = if self.grid.in_bounds(x, y) {
            Some((x, y))
        } else {
            None
        };
        self.hand.hover = hover;
    }

    pub fn clear_hover(&mut self) {
        self.hand.hover = None;
    }

    pub fn cancel(&mut self) -> bool {
        self.note(OrderOp::Cancel);
        let hand = &mut self.hand;
        let busy = hand.build.is_structure()
            || hand.strike != StrikeKind::None
            || hand.selected.is_some()
            || hand.lift.is_some();
        *hand = Hand::fresh();
        busy
    }

    pub fn click(&mut self, x: i32, y: i32) {
        self.note(OrderOp::Click { x, y });
        if !self.grid.in_bounds(x, y) {
            self.hand.selected = None;
            return;
        }
        if self.hand.lift.is_some() {
            let id = self.hand.lift.unwrap();
            match self.relocate(id, x, y) {
                Ok(()) => self.hand.lift = None,
                Err(err) => {
                    if err == PlaceError::NothingToMove {
                        self.hand.lift = None;
                    }
                    self.toast(err.message());
                }
            }
            return;
        }
        if self.hand.strike != StrikeKind::None {
            let kind = self.hand.strike;
            self.fire_strike_kind(kind, x, y);
            return;
        }
        if self.hand.build.is_structure() {
            let kind = self.hand.build;
            match self.place_for(x, y, kind) {
                Ok(()) => {}
                Err(err) => self.toast(err.message()),
            }
            return;
        }
        self.hand.selected = match self.grid.occupant(x, y) {
            Occupant::Tower(id) => Some(id),
            Occupant::Wall => wall_id_at(&self.towers, x, y),
            Occupant::None => None,
        };
    }

    pub fn place(&mut self, x: i32, y: i32, kind: BuildKind) -> Result<(), PlaceError> {
        self.place_for(x, y, kind)
    }

    fn place_for(&mut self, x: i32, y: i32, kind: BuildKind) -> Result<(), PlaceError> {
        if matches!(self.phase, Phase::Defeat) {
            return Err(PlaceError::Occupied);
        }
        if !kind.is_structure() {
            return Err(PlaceError::Occupied);
        }
        if !self.grid.in_bounds(x, y) {
            return Err(PlaceError::OutOfBounds);
        }
        if !self.grid.buildable(x, y) {
            return Err(PlaceError::Occupied);
        }
        if !self.loadout.gun_enabled(kind) {
            return Err(PlaceError::NotInLoadout);
        }
        let cost = self.loadout.gun(kind).map(|s| s.cost).unwrap_or(0);
        if self.credits < cost {
            return Err(PlaceError::CantAfford);
        }
        if kind.is_turret() {
            if let Some(cap) = self.rules().turret_cap {
                if self.turret_count() >= cap {
                    return Err(PlaceError::TurretCap);
                }
            }
        }

        let occ = if kind == BuildKind::Barricade {
            Occupant::Wall
        } else {
            Occupant::Tower(0)
        };
        self.grid.set_occ(x, y, occ);
        let flow = FlowField::compute(&self.grid);
        let path_ok = flow.spawns_reachable(&self.grid);
        let creeps_ok = self.ground_creeps_reachable(&flow);
        if !path_ok || !creeps_ok {
            self.grid.set_occ(x, y, Occupant::None);
            return Err(if !path_ok {
                PlaceError::BlocksPath
            } else {
                PlaceError::TrapsCreeps
            });
        }

        self.credits -= cost;
        self.spent += cost;
        let id = self.alloc();
        if kind == BuildKind::Barricade {
            self.grid.set_occ(x, y, Occupant::Wall);
        } else {
            self.grid.set_occ(x, y, Occupant::Tower(id));
        }
        self.towers.push(Tower {
            id,
            kind,
            x,
            y,
            tier: 0,
            cooldown: 0.15,
            aim: -std::f32::consts::FRAC_PI_2,
            target: None,
            invested: cost,
            kills: 0,
            damage_dealt: 0.0,
            target_mode: TargetMode::First,
            air_focus: false,
            stun_ttl: 0.0,
            overcharge_ttl: 0.0,
        });
        self.flow = flow;
        self.hand.selected = Some(id);
        self.push_fx("place", Grid::cell_center(x, y), 0.28, 1.0, 0.0);
        Ok(())
    }

    fn ground_creeps_reachable(&self, flow: &FlowField) -> bool {
        self.creeps
            .iter()
            .filter(|c| !c.flying && c.hp > 0.0)
            .all(|c| {
                let (x, y) = Grid::world_to_cell(c.pos.x, c.pos.y);
                if self.grid.blocks_ground(x, y) {
                    return false;
                }
                flow.cell_reachable(&self.grid, x, y)
            })
    }

    pub fn upgrade(&mut self) -> Result<(), PlaceError> {
        self.note(OrderOp::Upgrade);
        let id = self.hand.selected.ok_or(PlaceError::NotATurret)?;
        let idx = self
            .towers
            .iter()
            .position(|t| t.id == id)
            .ok_or(PlaceError::NotATurret)?;
        if !self.towers[idx].kind.is_turret() {
            return Err(PlaceError::NotATurret);
        }
        let kind = self.towers[idx].kind;
        let tier = self.towers[idx].tier;
        let cost = self
            .loadout
            .upgrade_cost(kind, tier)
            .ok_or(PlaceError::MaxTier)?;
        if self.credits < cost {
            self.toast(PlaceError::CantAfford.message());
            return Err(PlaceError::CantAfford);
        }
        self.credits -= cost;
        self.spent += cost;
        let t = &mut self.towers[idx];
        t.tier += 1;
        t.invested += cost;
        t.cooldown = 0.05;
        let pos = Grid::cell_center(t.x, t.y);
        self.push_fx("upgrade", pos, 0.4, 1.2, 0.0);
        Ok(())
    }

    pub fn sell(&mut self) -> Result<(), PlaceError> {
        self.note(OrderOp::Sell);
        let id = self.hand.selected.ok_or(PlaceError::NotATurret)?;
        let idx = self
            .towers
            .iter()
            .position(|t| t.id == id)
            .ok_or(PlaceError::NotATurret)?;
        let t = self.towers.remove(idx);
        let refund = ((t.invested as f32) * SELL_RATIO).round() as i32;
        self.credits += refund;
        self.grid.set_occ(t.x, t.y, Occupant::None);
        self.recompute_flow();
        {
            let hand = &mut self.hand;
            if hand.selected == Some(id) {
                hand.selected = None;
            }
            if hand.lift == Some(id) {
                hand.lift = None;
            }
        }
        self.push_fx("sell", Grid::cell_center(t.x, t.y), 0.3, 0.8, 0.0);
        Ok(())
    }

    pub fn cycle_targeting(&mut self) -> bool {
        self.note(OrderOp::Target);
        let id = match self.hand.selected {
            Some(id) => id,
            None => return false,
        };
        let label = {
            let t = match self
                .towers
                .iter_mut()
                .find(|t| t.id == id && t.kind.is_turret())
            {
                Some(t) => t,
                None => return false,
            };
            t.target_mode = t.target_mode.next();
            t.target = None;
            t.target_mode.label()
        };
        self.toast(label);
        true
    }

    pub fn convert(&mut self) -> Result<(), PlaceError> {
        self.note(OrderOp::Convert);
        let id = self.hand.selected.ok_or(PlaceError::NotHelios)?;
        let idx = self
            .towers
            .iter()
            .position(|t| t.id == id)
            .ok_or(PlaceError::NotHelios)?;
        if self.towers[idx].kind != BuildKind::Helios {
            return Err(PlaceError::NotHelios);
        }
        if self.towers[idx].air_focus {
            return Err(PlaceError::AlreadyAir);
        }
        if self.credits < HELIOS_CONVERT_COST {
            self.toast(PlaceError::CantAfford.message());
            return Err(PlaceError::CantAfford);
        }
        self.credits -= HELIOS_CONVERT_COST;
        self.spent += HELIOS_CONVERT_COST;
        let t = &mut self.towers[idx];
        t.air_focus = true;
        t.target = None;
        t.invested += HELIOS_CONVERT_COST;
        self.toast("Helios air-tuned");
        Ok(())
    }

    pub fn repair(&mut self) -> Result<(), PlaceError> {
        self.note(OrderOp::Repair);
        if matches!(self.phase, Phase::Defeat) || self.integrity <= 0 {
            self.toast(PlaceError::RelayDown.message());
            return Err(PlaceError::RelayDown);
        }
        if self.integrity >= self.integrity_max {
            self.toast(PlaceError::Intact.message());
            return Err(PlaceError::Intact);
        }
        if self.credits < REPAIR_COST {
            self.toast(PlaceError::CantAfford.message());
            return Err(PlaceError::CantAfford);
        }
        self.credits -= REPAIR_COST;
        self.spent += REPAIR_COST;
        self.integrity += 1;
        self.toast("Relay patched");
        Ok(())
    }

    pub fn lift(&mut self) -> Result<(), PlaceError> {
        self.note(OrderOp::Lift);
        if self.hand.lift.is_some() {
            self.hand.lift = None;
            self.toast("Move cancelled");
            return Ok(());
        }
        let id = self.hand.selected.ok_or(PlaceError::NothingToMove)?;
        if !self.towers.iter().any(|t| t.id == id) {
            return Err(PlaceError::NothingToMove);
        }
        self.hand.build = BuildKind::Inspect;
        self.hand.strike = StrikeKind::None;
        self.hand.lift = Some(id);
        self.toast("Click a cell to move");
        Ok(())
    }

    pub fn overcharge(&mut self) -> Result<(), PlaceError> {
        self.note(OrderOp::Overcharge);
        let id = self.hand.selected.ok_or(PlaceError::NotATurret)?;
        let idx = self
            .towers
            .iter()
            .position(|t| t.id == id)
            .ok_or(PlaceError::NotATurret)?;
        if !self.towers[idx].kind.is_turret() {
            return Err(PlaceError::NotATurret);
        }
        if self.credits < OVERCHARGE_COST {
            self.toast(PlaceError::CantAfford.message());
            return Err(PlaceError::CantAfford);
        }
        self.credits -= OVERCHARGE_COST;
        self.spent += OVERCHARGE_COST;
        let t = &mut self.towers[idx];
        t.overcharge_ttl = OVERCHARGE_TTL;
        t.invested += OVERCHARGE_COST;
        let pos = Grid::cell_center(t.x, t.y);
        self.push_fx("overcharge", pos, 0.45, 1.15, 0.0);
        self.toast("Barrels hot");
        Ok(())
    }

    fn relocate(&mut self, id: u32, x: i32, y: i32) -> Result<(), PlaceError> {
        if matches!(self.phase, Phase::Defeat) {
            return Err(PlaceError::Occupied);
        }
        let idx = self
            .towers
            .iter()
            .position(|t| t.id == id)
            .ok_or(PlaceError::NothingToMove)?;
        let old_x = self.towers[idx].x;
        let old_y = self.towers[idx].y;
        let kind = self.towers[idx].kind;
        if old_x == x && old_y == y {
            return Ok(());
        }
        if !self.grid.in_bounds(x, y) {
            return Err(PlaceError::OutOfBounds);
        }
        if !self.grid.buildable(x, y) {
            return Err(PlaceError::Occupied);
        }
        if self.credits < MOVE_COST {
            return Err(PlaceError::CantAfford);
        }
        let old_occ = self.grid.occupant(old_x, old_y);
        self.grid.set_occ(old_x, old_y, Occupant::None);
        let new_occ = if kind == BuildKind::Barricade {
            Occupant::Wall
        } else {
            Occupant::Tower(id)
        };
        self.grid.set_occ(x, y, new_occ);
        let flow = FlowField::compute(&self.grid);
        let path_ok = flow.spawns_reachable(&self.grid);
        let creeps_ok = self.ground_creeps_reachable(&flow);
        if !path_ok || !creeps_ok {
            self.grid.set_occ(old_x, old_y, old_occ);
            self.grid.set_occ(x, y, Occupant::None);
            return Err(if !path_ok {
                PlaceError::BlocksPath
            } else {
                PlaceError::TrapsCreeps
            });
        }
        self.credits -= MOVE_COST;
        self.spent += MOVE_COST;
        self.towers[idx].x = x;
        self.towers[idx].y = y;
        self.towers[idx].target = None;
        self.flow = flow;
        self.push_fx("sell", Grid::cell_center(old_x, old_y), 0.22, 0.8, 0.0);
        self.push_fx("place", Grid::cell_center(x, y), 0.28, 1.0, 0.0);
        Ok(())
    }

    pub fn call_wave(&mut self) -> bool {
        self.note(OrderOp::Call);
        let Phase::Fortify { remaining } = self.phase else {
            return false;
        };
        let bonus = 12 + (self.wave as i32) * 3;
        let extra = ((remaining / WAVE_DELAY).clamp(0.0, 1.0) * bonus as f32).round() as i32;
        self.credits += extra.max(8);
        self.begin_wave();
        true
    }

    pub fn fire_strike(&mut self, x: i32, y: i32) -> bool {
        self.fire_strike_kind(self.hand.strike, x, y)
    }

    fn fire_strike_kind(&mut self, kind: StrikeKind, x: i32, y: i32) -> bool {
        if matches!(self.phase, Phase::Defeat) {
            return false;
        }
        let Some(stats) = self.loadout.strike(kind) else {
            return false;
        };
        let cd_idx = kind as usize;
        if self.strike_cd.get(cd_idx).copied().unwrap_or(0.0) > 0.0 {
            self.toast("Strike recharging");
            return false;
        }
        if self.credits < stats.cost {
            self.toast(PlaceError::CantAfford.message());
            return false;
        }
        self.credits -= stats.cost;
        self.spent += stats.cost;
        self.strike_cd[cd_idx] = stats.cooldown;
        let pos = Grid::cell_center(x, y);
        let fx_kind = match kind {
            StrikeKind::Satchel => "satchel",
            StrikeKind::Overload => "overload",
            StrikeKind::Orbital => "orbital",
            StrikeKind::None => "burst",
        };
        self.push_fx(fx_kind, pos, 0.55, stats.radius, 0.0);
        let pen = strike_pen(kind);
        let mut taken_total = 0.0;
        for c in &mut self.creeps {
            if c.hp <= 0.0 {
                continue;
            }
            if c.flying && !stats.hits_air {
                continue;
            }
            if !c.flying && !stats.hits_ground {
                continue;
            }
            let d = c.pos.dist(pos);
            if d > stats.radius + c.radius {
                continue;
            }
            let falloff = if kind == StrikeKind::Orbital {
                (1.0 - 0.72 * (d / (stats.radius + 0.001)).clamp(0.0, 1.0)).max(0.22)
            } else {
                1.0
            };
            let taken = apply_armor(stats.damage * falloff, c.armor, pen);
            hurt(c, taken, None);
            taken_total += taken;
            if stats.slow > 0.0 {
                apply_slow(c, stats.slow, stats.slow_ttl);
            }
        }
        let _ = taken_total;
        true
    }

    fn toast(&mut self, msg: &str) {
        self.message = Some((msg.to_string(), 1.8));
    }

    fn pay_interest(&mut self) {
        let rules = self.rules();
        if rules.interest_bps == 0 || self.credits <= 0 {
            self.last_interest = 0;
            return;
        }
        let gain = ((self.credits as i64) * (rules.interest_bps as i64) / 10_000) as i32;
        let gain = gain.clamp(0, rules.interest_cap);
        self.last_interest = gain;
        if gain > 0 {
            self.credits += gain;
            self.toast(&format!("Interest +{gain}"));
        }
    }

    fn tick_heals(&mut self) {
        let healers: Vec<(u32, Vec2)> = self
            .creeps
            .iter()
            .filter(|c| c.kind == CreepKind::Medic && c.hp > 0.0)
            .map(|c| (c.id, c.pos))
            .collect();
        if healers.is_empty() {
            return;
        }
        let amount = MEDIC_HEAL_PER_SEC * DT;
        for c in &mut self.creeps {
            if c.hp <= 0.0 || c.flying || c.kind == CreepKind::Medic {
                continue;
            }
            let near = healers
                .iter()
                .any(|(id, pos)| *id != c.id && pos.dist(c.pos) <= MEDIC_HEAL_RADIUS);
            if near {
                c.hp = (c.hp + amount).min(c.hp_max);
            }
        }
    }

    fn push_fx(&mut self, kind: &'static str, pos: Vec2, ttl: f32, mag: f32, heading: f32) {
        if self.fx.len() > 180 {
            self.fx.pop_front();
        }
        self.fx.push_back(Fx {
            kind,
            pos,
            ttl,
            max: ttl,
            mag,
            heading,
        });
    }

    fn push_beam(&mut self, a: Vec2, b: Vec2, kind: BuildKind, ttl: f32) {
        if self.beams.len() > 80 {
            self.beams.pop_front();
        }
        self.beams.push_back(Beam {
            a,
            b,
            kind,
            ttl,
            max: ttl,
        });
    }

    fn begin_wave(&mut self) {
        let ground_only = self.rules().ground_only;
        let plan = WavePlan::for_wave(self.wave, ground_only);
        self.wave_script = plan.script;
        self.spawn_q = compose_wave(plan, &mut self.rng, &self.grid);
        self.spawn_cd = if matches!(plan.script, WaveScript::Swarm | WaveScript::Split) {
            0.22
        } else {
            0.35
        };
        self.phase = Phase::Incoming;
        let banner = format!(
            "WAVE {}  ·  {}",
            self.wave,
            plan.script.label().to_uppercase()
        );
        self.banner = Some((banner, 2.4));
    }

    pub fn step(&mut self) {
        if matches!(self.phase, Phase::Defeat) {
            self.decay_fx();
            return;
        }
        self.tick += 1;
        self.time += DT;
        self.hurt_flash = (self.hurt_flash - DT).max(0.0);
        for cd in &mut self.strike_cd {
            *cd = (*cd - DT).max(0.0);
        }
        if let Some((_, ttl)) = &mut self.banner {
            *ttl -= DT;
            if *ttl <= 0.0 {
                self.banner = None;
            }
        }
        if let Some((_, ttl)) = &mut self.message {
            *ttl -= DT;
            if *ttl <= 0.0 {
                self.message = None;
            }
        }

        match self.phase {
            Phase::Fortify { remaining } => {
                let r = remaining - DT;
                if r <= 0.0 {
                    self.begin_wave();
                } else {
                    self.phase = Phase::Fortify { remaining: r };
                }
            }
            Phase::Incoming => self.tick_incoming(),
            Phase::Defeat => {}
        }

        self.tick_creeps();
        self.tick_flickers();
        self.tick_roars();
        self.tick_heals();
        self.tick_towers();
        self.tick_projectiles();
        self.reap_creeps();
        self.decay_fx();

        if self.integrity <= 0 {
            self.integrity = 0;
            self.phase = Phase::Defeat;
            self.banner = Some(("The relay went dark".into(), 8.0));
        } else if matches!(self.phase, Phase::Incoming)
            && self.spawn_q.is_empty()
            && self.creeps.is_empty()
        {
            self.pay_interest();
            self.wave += 1;
            self.phase = Phase::Fortify {
                remaining: WAVE_DELAY,
            };
            if !self.objective_cleared {
                if let Some(goal) = self.hold_until {
                    if self.wave > goal {
                        self.objective_cleared = true;
                        self.banner = Some(("OBJECTIVE HELD".into(), 4.0));
                    } else {
                        self.banner = Some(("FIELD CLEAR  ·  FORTIFY".into(), 2.0));
                    }
                } else {
                    self.banner = Some(("FIELD CLEAR  ·  FORTIFY".into(), 2.0));
                }
            } else {
                self.banner = Some(("FIELD CLEAR  ·  FORTIFY".into(), 2.0));
            }
        }
    }

    fn tick_incoming(&mut self) {
        if self.spawn_q.is_empty() {
            return;
        }
        self.spawn_cd -= DT;
        if self.spawn_cd > 0.0 {
            return;
        }
        self.spawn_cd = match self.wave_script {
            WaveScript::Swarm | WaveScript::Split => {
                if self.wave >= 20 {
                    0.14
                } else {
                    0.2
                }
            }
            _ => {
                if self.wave >= 20 {
                    0.22
                } else {
                    0.32
                }
            }
        };
        if let Some(order) = self.spawn_q.pop_front() {
            self.spawn_creep(order);
        }
    }

    fn spawn_creep(&mut self, order: SpawnOrder) {
        let stats = creep_stats(order.kind);
        let mul_hp = wave_hp_mul(self.wave) * self.rules().hp_mul;
        let mul_b = wave_bounty_mul(self.wave) * self.rules().bounty_mul;
        let mul_s = wave_speed_mul(self.wave) * self.rules().speed_mul;
        let mut pos = Grid::cell_center(order.spawn.0, order.spawn.1);
        pos.x += self.rng.range_f32(-0.18, 0.18);
        pos.y += self.rng.range_f32(-0.18, 0.18);
        let hp = stats.hp * mul_hp;
        let id = self.alloc();
        let bounty = if self.rules().kill_income {
            ((stats.bounty as f32) * mul_b).round() as i32
        } else {
            0
        };
        let weave = self.rng.range_f32(0.0, std::f32::consts::TAU);
        let blink_cd = if order.kind == CreepKind::Flicker {
            self.rng.range_f32(0.4, FLICKER_PERIOD)
        } else {
            0.0
        };
        let roar_cd = if order.kind == CreepKind::Colossus {
            self.rng.range_f32(1.2, COLOSSUS_ROAR_PERIOD)
        } else {
            0.0
        };
        self.creeps.push(Creep {
            id,
            kind: order.kind,
            pos,
            vel: Vec2::ZERO,
            hp,
            hp_max: hp,
            speed: stats.speed * mul_s,
            bounty,
            leak: stats.leak,
            armor: stats.armor,
            flying: stats.flying,
            camo: stats.camo,
            radius: stats.radius,
            heading: 0.0,
            weave,
            slow_mul: 1.0,
            slow_ttl: 0.0,
            split_gen: 0,
            blink_cd,
            roar_cd,
            hit_by: None,
        });
    }

    fn tick_creeps(&mut self) {
        let mut leaks: Vec<usize> = Vec::new();
        for (i, c) in self.creeps.iter_mut().enumerate() {
            // A creep killed out of step (a strike lands between movement passes) must not
            // move or leak this tick — reap_creeps will score it as a kill instead.
            if c.hp <= 0.0 {
                continue;
            }
            if c.slow_ttl > 0.0 {
                c.slow_ttl -= DT;
                if c.slow_ttl <= 0.0 {
                    c.slow_mul = 1.0;
                    c.slow_ttl = 0.0;
                }
            }
            let spd = c.speed * c.slow_mul;
            let prev = c.pos;
            if c.flying {
                let core = Grid::nearest_of(&self.core_clusters, self.core_mid, c.pos);
                let to = core.sub(c.pos);
                let dir = to.norm();
                let weave = dir.perp().mul((self.time * 3.2 + c.weave).sin() * 0.12);
                c.vel = dir.add(weave).norm().mul(spd);
                c.pos = c.pos.add(c.vel.mul(DT));
                if c.pos.dist(core) < 0.55 {
                    leaks.push(i);
                }
            } else {
                let (cx, cy) = Grid::world_to_cell(c.pos.x, c.pos.y);
                if self.flow.dist_at(&self.grid, cx, cy) == 0 {
                    leaks.push(i);
                    continue;
                }
                if let Some((nx, ny)) = self.flow.next_at(&self.grid, cx, cy) {
                    let target = Grid::cell_center(nx, ny);
                    let to = target.sub(c.pos);
                    if to.length() < 0.06 {
                        c.pos = target;
                    } else {
                        c.vel = to.norm().mul(spd);
                        c.pos = c.pos.add(c.vel.mul(DT));
                    }
                } else if c
                    .pos
                    .dist(Grid::nearest_of(&self.core_clusters, self.core_mid, c.pos))
                    < 0.6
                {
                    leaks.push(i);
                }
            }
            let delta = c.pos.sub(prev);
            if delta.length() > 1e-4 {
                c.heading = delta.y.atan2(delta.x);
            }
        }
        leaks.sort_unstable();
        leaks.dedup();
        for i in leaks.into_iter().rev() {
            let c = self.creeps.remove(i);
            self.integrity -= c.leak;
            self.leaks += 1;
            self.leak_kinds[c.kind.index()] += 1;
            self.hurt_flash = 0.45;
            self.push_fx("leak", c.pos, 0.55, 1.4, 0.0);
        }
    }

    fn tick_flickers(&mut self) {
        let mut hops: Vec<(Vec2, Vec2)> = Vec::new();
        for c in &mut self.creeps {
            if c.kind != CreepKind::Flicker || c.hp <= 0.0 || c.flying {
                continue;
            }
            c.blink_cd -= DT;
            if c.blink_cd > 0.0 {
                continue;
            }
            c.blink_cd = FLICKER_PERIOD;
            let (x, y) = Grid::world_to_cell(c.pos.x, c.pos.y);
            let (nx, ny) = flow_hop(&self.flow, &self.grid, x, y, FLICKER_STEPS);
            let dest = Grid::cell_center(nx, ny);
            if dest.dist(c.pos) < 0.2 {
                continue;
            }
            hops.push((c.pos, dest));
            c.pos = dest;
            c.vel = Vec2::ZERO;
        }
        for (from, to) in hops {
            self.push_fx("blink", from, 0.28, 0.9, 0.0);
            self.push_fx("blink", to, 0.22, 0.7, 0.0);
        }
    }

    fn tick_roars(&mut self) {
        let mut roars: Vec<Vec2> = Vec::new();
        for c in &mut self.creeps {
            if c.kind != CreepKind::Colossus || c.hp <= 0.0 {
                continue;
            }
            c.roar_cd -= DT;
            if c.roar_cd > 0.0 {
                continue;
            }
            c.roar_cd = COLOSSUS_ROAR_PERIOD;
            roars.push(c.pos);
        }
        for pos in roars {
            self.push_fx("roar", pos, 0.55, COLOSSUS_ROAR_RADIUS, 0.0);
            for t in &mut self.towers {
                if !t.kind.is_turret() {
                    continue;
                }
                let origin = Grid::cell_center(t.x, t.y);
                if origin.dist(pos) <= COLOSSUS_ROAR_RADIUS {
                    t.stun_ttl = t.stun_ttl.max(COLOSSUS_STUN);
                }
            }
        }
    }

    fn tick_towers(&mut self) {
        struct Shot {
            owner: u32,
            kind: BuildKind,
            pos: Vec2,
            vel: Vec2,
            target: u32,
            damage: f32,
            splash: f32,
            hits_ground: bool,
            hits_air: bool,
            detects: bool,
            homing: bool,
            muzzle: Vec2,
            ttl: f32,
        }

        let mut shots: Vec<Shot> = Vec::new();
        let mut instants: Vec<InstantFire> = Vec::new();
        let guns = self.guns_hot;

        for t in &mut self.towers {
            t.stun_ttl = (t.stun_ttl - DT).max(0.0);
            t.overcharge_ttl = (t.overcharge_ttl - DT).max(0.0);
            t.cooldown = (t.cooldown - DT).max(0.0);
            if !t.kind.is_turret() {
                continue;
            }
            let Some(mut stats) = crate::pack::scaled_from(&guns, t.kind, t.tier) else {
                continue;
            };
            if t.kind == BuildKind::Helios {
                if t.air_focus {
                    stats.hits_ground = false;
                    stats.hits_air = true;
                } else {
                    stats.hits_ground = true;
                    stats.hits_air = false;
                }
            }
            if t.overcharge_ttl > 0.0 {
                stats.fire_interval *= OVERCHARGE_FIRE_MUL;
            }
            let origin = Grid::cell_center(t.x, t.y);
            if let Some(tid) = t.target {
                let valid = self.creeps.iter().any(|c| {
                    c.id == tid
                        && c.hp > 0.0
                        && targeting_ok(&stats, c)
                        && c.pos.dist(origin) <= stats.range + c.radius
                });
                if !valid {
                    t.target = None;
                }
            }
            if t.target.is_none() {
                t.target = acquire(
                    &self.creeps,
                    &self.flow,
                    &self.grid,
                    &self.core_clusters,
                    self.core_mid,
                    origin,
                    &stats,
                    t.target_mode,
                );
            }
            let Some(tid) = t.target else {
                continue;
            };
            let Some(creep) = self.creeps.iter().find(|c| c.id == tid) else {
                continue;
            };
            let desired = creep.pos.sub(origin).y.atan2(creep.pos.sub(origin).x);
            t.aim = lerp_angle(t.aim, desired, 12.0 * DT);
            if t.stun_ttl > 0.0 {
                continue;
            }
            if t.cooldown > 0.0 {
                continue;
            }
            t.cooldown = stats.fire_interval;

            match stats.fire {
                FireMode::Shell => {
                    let predicted = if stats.proj_speed > 1.0 {
                        let t_hit = origin.dist(creep.pos) / stats.proj_speed;
                        creep.pos.add(creep.vel.mul(t_hit.clamp(0.0, 0.9)))
                    } else {
                        creep.pos
                    };
                    let base_dir = predicted.sub(origin).norm();
                    let volley = stats.volley.max(1);
                    for i in 0..volley {
                        let spread = (i as f32 - (volley as f32 - 1.0) * 0.5) * 0.16;
                        let ang = base_dir.y.atan2(base_dir.x) + spread;
                        let dir = Vec2::new(ang.cos(), ang.sin());
                        shots.push(Shot {
                            owner: t.id,
                            kind: t.kind,
                            pos: origin.add(dir.mul(0.32)),
                            vel: dir.mul(stats.proj_speed),
                            target: tid,
                            damage: stats.damage,
                            splash: stats.splash,
                            hits_ground: stats.hits_ground,
                            hits_air: stats.hits_air,
                            detects: stats.kind.detects(),
                            homing: stats.homing,
                            muzzle: origin.add(dir.mul(0.38)),
                            // Live just past the gun's own reach. A flat 1.8s let a missed
                            // splash shell detonate at full damage several tiles outside range.
                            ttl: ((stats.range * 1.35) / stats.proj_speed.max(1.0)).clamp(0.1, 1.8),
                        });
                    }
                }
                _ => {
                    instants.push(InstantFire {
                        owner: t.id,
                        kind: t.kind,
                        origin,
                        aim: t.aim,
                        target: Some(tid),
                        range: stats.range,
                        splash: stats.splash,
                        damage: stats.damage,
                        hits_ground: stats.hits_ground,
                        hits_air: stats.hits_air,
                        detects: stats.kind.detects(),
                        fire: stats.fire,
                        slow: stats.slow,
                        slow_ttl: stats.slow_ttl,
                        interval_hint: stats.fire_interval,
                    });
                }
            }
        }

        for s in shots {
            self.projectiles.push(Projectile {
                pos: s.pos,
                vel: s.vel,
                target: Some(s.target),
                owner: s.owner,
                damage: s.damage,
                splash: s.splash,
                hits_ground: s.hits_ground,
                hits_air: s.hits_air,
                detects: s.detects,
                kind: s.kind,
                ttl: s.ttl,
                homing: s.homing,
            });
            self.push_fx("muzzle", s.muzzle, 0.08, 0.7, 0.0);
        }

        for fire in instants {
            self.resolve_instant(&fire);
        }
    }

    fn resolve_instant(&mut self, fire: &InstantFire) {
        let origin = fire.origin;
        let aim = fire.aim;
        let dir = Vec2::new(aim.cos(), aim.sin());
        let pen = armor_pen(fire.kind);
        let mut taken_total = 0.0;
        match fire.fire {
            FireMode::Cone => {
                self.push_fx("cone", origin, 0.16, fire.range, aim);
                for c in &mut self.creeps {
                    if !hit_filter(c, fire.hits_ground, fire.hits_air, fire.detects) {
                        continue;
                    }
                    if in_cone(origin, dir, c.pos, fire.range, fire.splash, c.radius) {
                        let taken = apply_armor(fire.damage, c.armor, pen);
                        hurt(c, taken, Some(fire.owner));
                        taken_total += taken;
                    }
                }
            }
            FireMode::Line => {
                let end = origin.add(dir.mul(fire.range));
                self.push_beam(origin, end, fire.kind, 0.12);
                for c in &mut self.creeps {
                    if !hit_filter(c, fire.hits_ground, fire.hits_air, fire.detects) {
                        continue;
                    }
                    if dist_to_segment(c.pos, origin, end) <= fire.splash + c.radius
                        && origin.dist(c.pos) <= fire.range + c.radius
                    {
                        let taken = apply_armor(fire.damage, c.armor, pen);
                        hurt(c, taken, Some(fire.owner));
                        taken_total += taken;
                    }
                }
            }
            FireMode::Pulse => {
                self.push_fx("pulse", origin, 0.28, fire.range, 0.0);
                for c in &mut self.creeps {
                    if !hit_filter(c, fire.hits_ground, fire.hits_air, fire.detects) {
                        continue;
                    }
                    if c.pos.dist(origin) <= fire.range + c.radius {
                        let taken = apply_armor(fire.damage, c.armor, pen);
                        hurt(c, taken, Some(fire.owner));
                        taken_total += taken;
                        if fire.slow > 0.0 {
                            apply_slow(c, fire.slow, fire.slow_ttl);
                        }
                    }
                }
            }
            FireMode::Beam => {
                if let Some(tid) = fire.target {
                    let pos = self.creeps.iter().find(|c| c.id == tid).map(|c| c.pos);
                    if let Some(pos) = pos {
                        self.push_beam(origin, pos, fire.kind, fire.interval_hint.max(0.08));
                        if let Some(c) = self.creeps.iter_mut().find(|c| c.id == tid) {
                            let taken = apply_armor(fire.damage, c.armor, pen);
                            hurt(c, taken, Some(fire.owner));
                            taken_total += taken;
                        }
                    }
                }
            }
            FireMode::Shell => {}
        }
        if let Some(shooter) = self.towers.iter_mut().find(|t| t.id == fire.owner) {
            shooter.damage_dealt += taken_total;
        }
    }

    fn tick_projectiles(&mut self) {
        struct Impact {
            pos: Vec2,
            damage: f32,
            splash: f32,
            hits_ground: bool,
            hits_air: bool,
            detects: bool,
            kind: BuildKind,
            owner: u32,
        }
        let mut impacts: Vec<Impact> = Vec::new();
        for p in &mut self.projectiles {
            p.ttl -= DT;
            if p.homing {
                let alive = p.target.and_then(|tid| {
                    self.creeps
                        .iter()
                        .find(|c| c.id == tid && c.hp > 0.0)
                        .map(|c| c.pos)
                });
                if let Some(pos) = alive {
                    let desired = pos.sub(p.pos).norm().mul(p.vel.length().max(8.0));
                    p.vel = Vec2::new(
                        p.vel.x + (desired.x - p.vel.x) * 0.22,
                        p.vel.y + (desired.y - p.vel.y) * 0.22,
                    );
                } else {
                    p.target = self
                        .creeps
                        .iter()
                        .filter(|c| {
                            c.hp > 0.0 && hit_filter(c, p.hits_ground, p.hits_air, p.detects)
                        })
                        .min_by(|a, b| {
                            a.pos
                                .dist(p.pos)
                                .partial_cmp(&b.pos.dist(p.pos))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .filter(|c| c.pos.dist(p.pos) < 4.5)
                        .map(|c| c.id);
                }
            }
            p.pos = p.pos.add(p.vel.mul(DT));
            let mut hit = false;
            if let Some(tid) = p.target {
                if let Some(c) = self.creeps.iter().find(|c| c.id == tid && c.hp > 0.0) {
                    if p.pos.dist(c.pos) < (0.22 + c.radius) {
                        hit = true;
                    }
                }
            }
            if !hit {
                hit = self.creeps.iter().any(|c| {
                    c.hp > 0.0
                        && hit_filter(c, p.hits_ground, p.hits_air, p.detects)
                        && p.pos.dist(c.pos) < (0.18 + c.radius)
                });
            }
            if hit || (p.ttl <= 0.0 && p.splash > 0.05) {
                impacts.push(Impact {
                    pos: p.pos,
                    damage: p.damage,
                    splash: p.splash,
                    hits_ground: p.hits_ground,
                    hits_air: p.hits_air,
                    detects: p.detects,
                    kind: p.kind,
                    owner: p.owner,
                });
                p.ttl = -1.0;
            } else if p.ttl <= 0.0 {
                p.ttl = -1.0;
            }
        }
        self.projectiles.retain(|p| p.ttl > 0.0);

        for impact in impacts {
            self.push_fx(
                if impact.splash > 0.05 {
                    "burst"
                } else {
                    "spark"
                },
                impact.pos,
                if impact.splash > 0.05 { 0.28 } else { 0.12 },
                0.6 + impact.splash,
                0.0,
            );
            let pen = armor_pen(impact.kind);
            let radius = if impact.splash > 0.05 {
                impact.splash
            } else {
                0.22
            };
            let mut taken_total = 0.0;
            for c in &mut self.creeps {
                if c.hp <= 0.0
                    || !hit_filter(c, impact.hits_ground, impact.hits_air, impact.detects)
                {
                    continue;
                }
                if c.pos.dist(impact.pos) > radius + c.radius {
                    continue;
                }
                let falloff = if impact.splash > 0.05 {
                    let d = c.pos.dist(impact.pos) / (radius + 0.001);
                    (1.0 - 0.45 * d.clamp(0.0, 1.0)).max(0.4)
                } else {
                    1.0
                };
                let taken = apply_armor(impact.damage * falloff, c.armor, pen);
                hurt(c, taken, Some(impact.owner));
                taken_total += taken;
            }
            if let Some(shooter) = self.towers.iter_mut().find(|t| t.id == impact.owner) {
                shooter.damage_dealt += taken_total;
            }
        }
    }

    fn reap_creeps(&mut self) {
        let mut i = 0;
        let mut splits: Vec<Creep> = Vec::new();
        while i < self.creeps.len() {
            if self.creeps[i].hp <= 0.0 {
                let c = self.creeps.remove(i);
                self.credits += c.bounty;
                self.kills += 1;
                self.kill_kinds[c.kind.index()] += 1;
                self.push_fx("kill", c.pos, 0.35, 1.0 + c.radius, 0.0);
                if c.bounty > 0 {
                    self.push_fx("cash", c.pos, 0.7, c.bounty as f32, 0.0);
                }
                if let Some(id) = c.hit_by {
                    if let Some(t) = self.towers.iter_mut().find(|t| t.id == id) {
                        t.kills += 1;
                    }
                }
                if c.kind == CreepKind::Mite && c.split_gen == 0 {
                    splits.extend(self.mite_children(&c));
                }
            } else {
                i += 1;
            }
        }
        self.creeps.extend(splits);
    }

    fn mite_children(&mut self, parent: &Creep) -> [Creep; 2] {
        let stats = creep_stats(CreepKind::Mite);
        let hp = (parent.hp_max * 0.42).max(6.0);
        let bounty = if self.rules().kill_income {
            (parent.bounty / 2).max(1)
        } else {
            0
        };
        let a = parent.heading;
        let dir = Vec2::new(a.cos(), a.sin()).perp();
        let child = |s: &mut Game, off: Vec2| {
            let mut pos = parent.pos.add(off);
            pos.x = pos.x.clamp(0.2, s.grid.w as f32 - 0.2);
            pos.y = pos.y.clamp(0.2, s.grid.h as f32 - 0.2);
            Creep {
                id: s.alloc(),
                kind: CreepKind::Mite,
                pos,
                vel: Vec2::ZERO,
                hp,
                hp_max: hp,
                speed: parent.speed * 1.12,
                bounty,
                leak: stats.leak,
                armor: stats.armor,
                flying: false,
                camo: false,
                radius: stats.radius * 0.78,
                heading: parent.heading,
                weave: s.rng.range_f32(0.0, std::f32::consts::TAU),
                slow_mul: 1.0,
                slow_ttl: 0.0,
                split_gen: 1,
                blink_cd: 0.0,
                roar_cd: 0.0,
                hit_by: None,
            }
        };
        let left = child(self, dir.mul(0.22));
        let right = child(self, dir.mul(-0.22));
        [left, right]
    }

    fn decay_fx(&mut self) {
        for f in &mut self.fx {
            f.ttl -= DT;
        }
        self.fx.retain(|f| f.ttl > 0.0);
        for b in &mut self.beams {
            b.ttl -= DT;
        }
        self.beams.retain(|b| b.ttl > 0.0);
    }

    pub fn snapshot(&self) -> Snapshot {
        let (status, next_wave_in, can_call) = match self.phase {
            Phase::Fortify { remaining } => ("fortify".into(), remaining, true),
            Phase::Incoming => ("incoming".into(), 0.0, false),
            Phase::Defeat => ("defeat".into(), 0.0, false),
        };
        let core = self.grid.core_center();
        Snapshot {
            tick: self.tick,
            time: self.time,
            status,
            defeated: matches!(self.phase, Phase::Defeat),
            credits: self.credits,
            integrity: self.integrity.max(0),
            integrity_max: self.integrity_max,
            wave: self.wave,
            next_wave_in,
            can_call_wave: can_call,
            creeps_alive: self.creeps.len() as u32,
            creeps_remaining: self.spawn_q.len() as u32,
            kills: self.kills,
            leaks: self.leaks,
            banner: self.banner.as_ref().map(|(s, _)| s.clone()),
            banner_life: self.banner.as_ref().map(|(_, t)| *t).unwrap_or(0.0),
            message: self.message.as_ref().map(|(s, _)| s.clone()),
            hurt_flash: self.hurt_flash,
            build: self.hand.build as u8,
            strike: self.hand.strike as u8,
            map_id: self.map_id,
            map_name: self.map_name.clone(),
            modifier_id: self.modifier.id(),
            modifier_name: self.modifier.name().to_string(),
            turret_count: self.turret_count(),
            turret_cap: self.rules().turret_cap,
            hover: self.hover_info(),
            selected: self.selected_info(),
            strikes: self
                .loadout
                .strike_items()
                .into_iter()
                .map(|s| {
                    let idx = s.id as usize;
                    let ready = self.loadout.strike(StrikeKind::from_u8(s.id)).is_some()
                        && self.strike_cd.get(idx).copied().unwrap_or(0.0) <= 0.0
                        && self.credits >= s.cost;
                    StrikeHud {
                        id: s.id,
                        ready,
                        cooldown: self.strike_cd.get(idx).copied().unwrap_or(0.0),
                        cost: s.cost,
                    }
                })
                .collect(),
            walls: self.grid.walls().into_iter().map(|(x, y)| [x, y]).collect(),
            towers: self
                .towers
                .iter()
                .filter(|t| t.kind.is_turret())
                .map(|t| TowerView {
                    id: t.id,
                    kind: t.kind,
                    x: t.x,
                    y: t.y,
                    aim: t.aim,
                    tier: t.tier,
                    air_focus: t.air_focus,
                    stunned: t.stun_ttl > 0.0,
                    overcharged: t.overcharge_ttl > 0.0,
                })
                .collect(),
            creeps: self
                .creeps
                .iter()
                .map(|c| CreepView {
                    id: c.id,
                    kind: c.kind,
                    x: c.pos.x,
                    y: c.pos.y,
                    hp: c.hp,
                    hp_max: c.hp_max,
                    flying: c.flying,
                    heading: c.heading,
                    radius: c.radius,
                    slowed: c.slow_ttl > 0.0,
                })
                .collect(),
            projectiles: self
                .projectiles
                .iter()
                .map(|p| ProjView {
                    x: p.pos.x,
                    y: p.pos.y,
                    vx: p.vel.x,
                    vy: p.vel.y,
                    kind: p.kind,
                })
                .collect(),
            fx: self
                .fx
                .iter()
                .map(|f| FxView {
                    kind: f.kind,
                    x: f.pos.x,
                    y: f.pos.y,
                    life: (f.ttl / f.max).clamp(0.0, 1.0),
                    mag: f.mag,
                    heading: f.heading,
                })
                .collect(),
            beams: self
                .beams
                .iter()
                .map(|b| BeamView {
                    x0: b.a.x,
                    y0: b.a.y,
                    x1: b.b.x,
                    y1: b.b.y,
                    kind: b.kind,
                    life: (b.ttl / b.max).clamp(0.0, 1.0),
                })
                .collect(),
            core: [core.x, core.y],
            cores: self
                .grid
                .core_clusters()
                .into_iter()
                .map(|v| [v.x, v.y])
                .collect(),
            objective_wave: self.hold_until,
            objective_cleared: self.objective_cleared,
            mission_id: self.mission_id,
            challenge_id: self.challenge_id,
            mission_name: self.mission_name.clone(),
            seed_hex: format!("{:016x}", self.seed),
            pack_name: self.pack.as_ref().map(|p| p.name.clone()),
            wave_intel: self.wave_intel(),
            after: self.after_action(),
            interest_paid: self.last_interest,
            interest_bps: self.rules().interest_bps,
            move_cost: MOVE_COST,
            repair_cost: REPAIR_COST,
            overcharge_cost: OVERCHARGE_COST,
            walk: self.flow.max_spawn_dist(&self.grid),
            relocating: self.hand.lift.is_some(),
            walk_paths: self.flow.spawn_paths_ref().to_vec(),
        }
    }

    fn wave_intel(&self) -> WaveIntel {
        let plan = WavePlan::for_wave(self.wave, self.rules().ground_only);
        WaveIntel {
            script: plan.script.label().into(),
            total: plan.total(),
            parts: kind_parts_from_plan(&plan),
        }
    }

    fn after_action(&self) -> AfterAction {
        let mut guns: Vec<GunScore> = self
            .towers
            .iter()
            .filter(|t| t.kind.is_turret() && (t.kills > 0 || t.damage_dealt > 0.0))
            .map(|t| GunScore {
                name: self.loadout.gun_name(t.kind).to_string(),
                kills: t.kills,
                damage: t.damage_dealt,
            })
            .collect();
        guns.sort_by_key(|g| std::cmp::Reverse(g.kills));
        AfterAction {
            spent: self.spent,
            kills: self.kills,
            leaks: self.leaks,
            wave: self.wave,
            kill_kinds: kind_parts_from_counts(&self.kill_kinds),
            leak_kinds: kind_parts_from_counts(&self.leak_kinds),
            guns,
        }
    }

    fn hover_info(&self) -> Option<HoverInfo> {
        let hand = self.hand;
        let (x, y) = hand.hover?;
        if !self.grid.in_bounds(x, y) {
            return None;
        }
        if let Some(id) = hand.lift {
            if let Some(t) = self.towers.iter().find(|t| t.id == id) {
                let mut range = 0.0;
                let mut hg = false;
                let mut ha = false;
                if let Some(s) = self
                    .loadout
                    .scaled(t.kind, t.tier)
                    .or_else(|| self.loadout.gun_even_disabled(t.kind))
                {
                    range = s.range;
                    hg = s.hits_ground;
                    ha = s.hits_air;
                }
                let (valid, reason, walk_after) = match self.preview_relocate(id, x, y) {
                    Ok(walk) => (true, String::new(), Some(walk)),
                    Err(e) => (false, e.message().to_string(), None),
                };
                return Some(HoverInfo {
                    x,
                    y,
                    valid,
                    reason,
                    range,
                    hits_ground: hg,
                    hits_air: ha,
                    strike: false,
                    walk_after,
                });
            }
        }
        if hand.strike != StrikeKind::None {
            if let Some(s) = self.loadout.strike(hand.strike) {
                let cd = self
                    .strike_cd
                    .get(hand.strike as usize)
                    .copied()
                    .unwrap_or(0.0);
                let (valid, reason) = if cd > 0.0 {
                    (false, "Strike recharging".into())
                } else if self.credits < s.cost {
                    (false, PlaceError::CantAfford.message().into())
                } else {
                    (true, String::new())
                };
                return Some(HoverInfo {
                    x,
                    y,
                    valid,
                    reason,
                    range: s.radius,
                    hits_ground: s.hits_ground,
                    hits_air: s.hits_air,
                    strike: true,
                    walk_after: None,
                });
            }
        }
        let mut range = 0.0;
        let mut hg = false;
        let mut ha = false;
        let mut walk_after = None;
        let (valid, reason) = if hand.build.is_structure() {
            if let Some(s) = self
                .loadout
                .scaled(hand.build, 0)
                .or_else(|| self.loadout.gun_even_disabled(hand.build))
            {
                range = s.range;
                hg = s.hits_ground;
                ha = s.hits_air;
            }
            match self.preview_place(x, y, hand.build) {
                Ok(walk) => {
                    walk_after = Some(walk);
                    (true, String::new())
                }
                Err(e) => (false, e.message().to_string()),
            }
        } else {
            (self.grid.buildable(x, y), String::new())
        };
        Some(HoverInfo {
            x,
            y,
            valid,
            reason,
            range,
            hits_ground: hg,
            hits_air: ha,
            strike: false,
            walk_after,
        })
    }

    fn preview_place(&self, x: i32, y: i32, kind: BuildKind) -> Result<u32, PlaceError> {
        // Mirror place_for's first guard, or the hover reports a legal placement on a
        // field the engine will refuse.
        if matches!(self.phase, Phase::Defeat) {
            return Err(PlaceError::Occupied);
        }
        if !kind.is_structure() {
            return Err(PlaceError::Occupied);
        }
        if !self.grid.in_bounds(x, y) {
            return Err(PlaceError::OutOfBounds);
        }
        if !self.grid.buildable(x, y) {
            return Err(PlaceError::Occupied);
        }
        if !self.loadout.gun_enabled(kind) {
            return Err(PlaceError::NotInLoadout);
        }
        let cost = self.loadout.gun(kind).map(|s| s.cost).unwrap_or(0);
        if self.credits < cost {
            return Err(PlaceError::CantAfford);
        }
        if kind.is_turret() {
            if let Some(cap) = self.rules().turret_cap {
                if self.turret_count() >= cap {
                    return Err(PlaceError::TurretCap);
                }
            }
        }
        let mut grid = self.grid.clone();
        grid.set_occ(
            x,
            y,
            if kind == BuildKind::Barricade {
                Occupant::Wall
            } else {
                Occupant::Tower(0)
            },
        );
        let flow = FlowField::compute(&grid);
        if !flow.spawns_reachable(&grid) {
            return Err(PlaceError::BlocksPath);
        }
        let trapped = self
            .creeps
            .iter()
            .filter(|c| !c.flying && c.hp > 0.0)
            .any(|c| {
                let (cx, cy) = Grid::world_to_cell(c.pos.x, c.pos.y);
                (cx == x && cy == y) || !flow.cell_reachable(&grid, cx, cy)
            });
        if trapped {
            return Err(PlaceError::TrapsCreeps);
        }
        Ok(flow.max_spawn_dist(&grid))
    }

    fn preview_relocate(&self, id: u32, x: i32, y: i32) -> Result<u32, PlaceError> {
        if matches!(self.phase, Phase::Defeat) {
            return Err(PlaceError::Occupied);
        }
        let t = self
            .towers
            .iter()
            .find(|t| t.id == id)
            .ok_or(PlaceError::NothingToMove)?;
        if t.x == x && t.y == y {
            return Ok(self.flow.max_spawn_dist(&self.grid));
        }
        if !self.grid.in_bounds(x, y) {
            return Err(PlaceError::OutOfBounds);
        }
        if !self.grid.buildable(x, y) {
            return Err(PlaceError::Occupied);
        }
        if self.credits < MOVE_COST {
            return Err(PlaceError::CantAfford);
        }
        let mut grid = self.grid.clone();
        grid.set_occ(t.x, t.y, Occupant::None);
        grid.set_occ(
            x,
            y,
            if t.kind == BuildKind::Barricade {
                Occupant::Wall
            } else {
                Occupant::Tower(id)
            },
        );
        let flow = FlowField::compute(&grid);
        if !flow.spawns_reachable(&grid) {
            return Err(PlaceError::BlocksPath);
        }
        let trapped = self
            .creeps
            .iter()
            .filter(|c| !c.flying && c.hp > 0.0)
            .any(|c| {
                let (cx, cy) = Grid::world_to_cell(c.pos.x, c.pos.y);
                (cx == x && cy == y) || !flow.cell_reachable(&grid, cx, cy)
            });
        if trapped {
            return Err(PlaceError::TrapsCreeps);
        }
        Ok(flow.max_spawn_dist(&grid))
    }

    fn selected_info(&self) -> Option<SelectedInfo> {
        let id = self.hand.selected?;
        let t = self.towers.iter().find(|t| t.id == id)?;
        let mut stats = self
            .loadout
            .scaled(t.kind, t.tier)
            .or_else(|| self.loadout.gun_even_disabled(t.kind))?;
        if t.kind == BuildKind::Helios {
            if t.air_focus {
                stats.hits_ground = false;
                stats.hits_air = true;
            } else {
                stats.hits_ground = true;
                stats.hits_air = false;
            }
        }
        let name = if t.kind == BuildKind::Helios && t.air_focus {
            format!("{} Air", self.loadout.gun_name(t.kind))
        } else {
            self.loadout.gun_name(t.kind).to_string()
        };
        Some(SelectedInfo {
            id: t.id,
            kind: t.kind,
            name,
            x: t.x,
            y: t.y,
            tier: t.tier,
            max_tier: MAX_TIER,
            tier_name: tier_name(t.tier).to_string(),
            range: stats.range,
            damage: stats.damage,
            fire_interval: stats.fire_interval,
            splash: stats.splash,
            hits_ground: stats.hits_ground,
            hits_air: stats.hits_air,
            detects: t.kind.detects(),
            fire: stats.fire,
            targeting: t.target_mode,
            targeting_label: t.target_mode.label().to_string(),
            can_convert: t.kind == BuildKind::Helios && !t.air_focus,
            convert_cost: if t.kind == BuildKind::Helios && !t.air_focus {
                Some(HELIOS_CONVERT_COST)
            } else {
                None
            },
            invested: t.invested,
            upgrade_cost: if t.kind.is_turret() {
                self.loadout.upgrade_cost(t.kind, t.tier)
            } else {
                None
            },
            sell_value: ((t.invested as f32) * SELL_RATIO).round() as i32,
            kills: t.kills,
            damage_dealt: t.damage_dealt,
        })
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.snapshot()).expect("snapshot")
    }

    pub fn map_static_json(&self) -> String {
        serde_json::to_string(&self.map_static()).expect("map")
    }

    pub fn replay_bundle(&self) -> ReplayFile {
        let mut file = ReplayFile {
            version: 1,
            map_id: if self.map_id == WORKSHOP_MAP_ID {
                None
            } else {
                Some(self.map_id)
            },
            map: if self.map_id == WORKSHOP_MAP_ID {
                Some(grid_to_doc(
                    &self.grid,
                    &self.map_name,
                    "workshop",
                    self.seed,
                ))
            } else {
                None
            },
            modifier_id: self.modifier.id(),
            seed: self.seed,
            orders: self.orders.clone(),
            pack: self.pack.clone(),
            outcome: Some(RunOutcome {
                wave: self.wave,
                integrity: self.integrity.max(0),
                kills: self.kills,
                leaks: self.leaks,
                defeated: matches!(self.phase, Phase::Defeat),
                ticks: self.tick,
            }),
            hash: None,
        };
        file.hash = Some(replay_hash(&file));
        file
    }

    pub fn replay_json(&self) -> String {
        serde_json::to_string(&self.replay_bundle()).expect("replay")
    }

    /// Apply recorded orders whose tick has arrived. No-op while live recording.
    pub fn pump_recorded(&mut self) {
        if self.recording {
            return;
        }
        while self.order_idx < self.orders.len() && self.orders[self.order_idx].tick <= self.tick {
            let order = self.orders[self.order_idx].clone();
            self.apply_order(&order);
            self.order_idx += 1;
        }
    }

    pub fn step_recorded(&mut self) {
        self.pump_recorded();
        self.step();
    }

    pub fn apply_order(&mut self, order: &Order) {
        let rec = self.recording;
        self.recording = false;
        match &order.op {
            OrderOp::SetBuild { kind } => self.set_build(*kind),
            OrderOp::SetStrike { kind } => self.set_strike(*kind),
            OrderOp::Click { x, y } => self.click(*x, *y),
            OrderOp::Cancel => {
                let _ = self.cancel();
            }
            OrderOp::Upgrade => {
                let _ = self.upgrade();
            }
            OrderOp::Sell => {
                let _ = self.sell();
            }
            OrderOp::Call => {
                let _ = self.call_wave();
            }
            OrderOp::Target => {
                let _ = self.cycle_targeting();
            }
            OrderOp::Convert => {
                let _ = self.convert();
            }
            OrderOp::Repair => {
                let _ = self.repair();
            }
            OrderOp::Lift => {
                let _ = self.lift();
            }
            OrderOp::Overcharge => {
                let _ = self.overcharge();
            }
        }
        self.recording = rec;
    }

    /// Step the match, applying recorded orders at the ticks they were issued.
    pub fn run_recorded(&mut self, until_wave: Option<u32>, until_tick: Option<u64>) {
        self.recording = false;
        let cap = until_tick.unwrap_or(u64::MAX).min(1_200_000);
        loop {
            self.pump_recorded();
            if matches!(self.phase, Phase::Defeat) {
                break;
            }
            if self.tick >= cap {
                break;
            }
            if let Some(w) = until_wave {
                if self.wave > w && matches!(self.phase, Phase::Fortify { .. }) {
                    break;
                }
            }
            self.step();
        }
    }
}

struct InstantFire {
    owner: u32,
    kind: BuildKind,
    origin: Vec2,
    aim: f32,
    target: Option<u32>,
    range: f32,
    splash: f32,
    damage: f32,
    hits_ground: bool,
    hits_air: bool,
    detects: bool,
    fire: FireMode,
    slow: f32,
    slow_ttl: f32,
    interval_hint: f32,
}

fn wall_id_at(towers: &[Tower], x: i32, y: i32) -> Option<u32> {
    towers
        .iter()
        .find(|t| t.x == x && t.y == y && t.kind == BuildKind::Barricade)
        .map(|t| t.id)
}

fn targeting_ok(stats: &crate::defs::TurretStats, c: &Creep) -> bool {
    hit_filter(c, stats.hits_ground, stats.hits_air, stats.kind.detects())
}

fn hit_filter(c: &Creep, hits_ground: bool, hits_air: bool, detects: bool) -> bool {
    if c.hp <= 0.0 {
        return false;
    }
    if c.camo && !detects {
        return false;
    }
    (c.flying && hits_air) || (!c.flying && hits_ground)
}

/// The creep carries one slow: the strongest currently active, with its own expiry.
///
/// A weaker slow must not refresh a stronger one's timer — otherwise a Pulse Array
/// ticking every 0.78s keeps an Overload's 0.42 multiplier alive indefinitely and the
/// creep never recovers. A weaker slow is simply ignored while a stronger one holds;
/// once that expires, `tick_creeps` resets `slow_mul` to 1.0 and the weaker slow lands.
fn apply_slow(c: &mut Creep, mul: f32, dur: f32) {
    if mul < c.slow_mul {
        c.slow_mul = mul;
        c.slow_ttl = dur;
    } else if mul == c.slow_mul {
        c.slow_ttl = c.slow_ttl.max(dur);
    }
}

fn hurt(c: &mut Creep, amount: f32, owner: Option<u32>) {
    if amount <= 0.0 {
        return;
    }
    c.hp -= amount;
    if let Some(id) = owner {
        c.hit_by = Some(id);
    }
}

fn in_cone(origin: Vec2, dir: Vec2, pos: Vec2, range: f32, half: f32, radius: f32) -> bool {
    let to = pos.sub(origin);
    let dist = to.length();
    if dist > range + radius {
        return false;
    }
    if dist < 0.15 {
        return true;
    }
    let ang = dir.dot(to.norm()).clamp(-1.0, 1.0).acos();
    ang <= half + (radius / dist.max(0.15)) * 0.35
}

fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b.sub(a);
    let len2 = ab.dot(ab).max(1e-6);
    let t = (p.sub(a).dot(ab) / len2).clamp(0.0, 1.0);
    p.dist(a.add(ab.mul(t)))
}

#[allow(clippy::too_many_arguments)]
fn acquire(
    creeps: &[Creep],
    flow: &FlowField,
    grid: &Grid,
    cores: &[Vec2],
    core_mid: Vec2,
    origin: Vec2,
    stats: &crate::defs::TurretStats,
    mode: TargetMode,
) -> Option<u32> {
    let mut best: Option<(u32, f32)> = None;
    for c in creeps {
        if !targeting_ok(stats, c) {
            continue;
        }
        if c.pos.dist(origin) > stats.range + c.radius {
            continue;
        }
        let first = if c.flying {
            c.pos.dist(Grid::nearest_of(cores, core_mid, c.pos))
        } else {
            let (x, y) = Grid::world_to_cell(c.pos.x, c.pos.y);
            flow.dist_at(grid, x, y) as f32 + c.pos.dist(Grid::cell_center(x, y)) * 0.01
        };
        let rank = match mode {
            TargetMode::First => first,
            TargetMode::Last => -first,
            TargetMode::Strong => -c.hp,
            TargetMode::Weak => c.hp,
            TargetMode::Flying => {
                if c.flying {
                    first - 10_000.0
                } else {
                    first
                }
            }
            TargetMode::Camo => {
                if c.camo {
                    first - 10_000.0
                } else {
                    first
                }
            }
        };
        if best.is_none_or(|(_, r)| rank < r) {
            best = Some((c.id, rank));
        }
    }
    best.map(|(id, _)| id)
}

fn flow_hop(flow: &FlowField, grid: &Grid, mut x: i32, mut y: i32, steps: u32) -> (i32, i32) {
    for _ in 0..steps {
        match flow.next_at(grid, x, y) {
            Some((nx, ny)) => {
                x = nx;
                y = ny;
            }
            None => break,
        }
    }
    (x, y)
}

fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let mut d = to - from;
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    from + d * t.clamp(0.0, 1.0)
}

fn compose_wave(plan: WavePlan, rng: &mut Rng, grid: &Grid) -> VecDeque<SpawnOrder> {
    let spawns = grid.spawns();
    let north: Vec<(i32, i32)> = spawns.iter().copied().filter(|(_, y)| *y == 0).collect();
    let east: Vec<(i32, i32)> = spawns
        .iter()
        .copied()
        .filter(|(x, _)| *x == grid.w - 1)
        .collect();
    let west: Vec<(i32, i32)> = spawns.iter().copied().filter(|(x, _)| *x == 0).collect();
    let south: Vec<(i32, i32)> = spawns
        .iter()
        .copied()
        .filter(|(_, y)| *y == grid.h - 1)
        .collect();
    let mut pools: Vec<&[(i32, i32)]> = Vec::new();
    if !north.is_empty() {
        pools.push(&north);
    }
    if !east.is_empty() {
        pools.push(&east);
    }
    if !west.is_empty() {
        pools.push(&west);
    }
    if !south.is_empty() {
        pools.push(&south);
    }
    if pools.is_empty() {
        pools.push(&spawns);
    }

    let mut kinds = plan.kinds();
    if kinds.len() > 1 {
        for i in (1..kinds.len()).rev() {
            let j = rng.range_i32(0, (i + 1) as i32) as usize;
            kinds.swap(i, j);
        }
    }

    let mut q = VecDeque::new();
    for (i, kind) in kinds.into_iter().enumerate() {
        let pool = pools[i % pools.len()];
        let spawn = rng
            .pick(pool)
            .or_else(|| rng.pick(&spawns))
            .copied()
            .unwrap_or((0, 0));
        q.push_back(SpawnOrder { kind, spawn });
    }
    q
}

fn kind_parts_from_plan(plan: &WavePlan) -> Vec<KindCount> {
    let pairs = [
        (CreepKind::Runner, plan.runners),
        (CreepKind::Lorry, plan.lorries),
        (CreepKind::Bulwark, plan.bulwarks),
        (CreepKind::Wasp, plan.wasps),
        (CreepKind::Mite, plan.mites),
        (CreepKind::Medic, plan.medics),
        (CreepKind::Shade, plan.shades),
        (CreepKind::Flicker, plan.flickers),
        (CreepKind::Colossus, plan.colossus),
    ];
    pairs
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(kind, count)| KindCount {
            kind,
            name: kind.name().into(),
            count,
        })
        .collect()
}

fn kind_parts_from_counts(counts: &[u32; 9]) -> Vec<KindCount> {
    CreepKind::ALL
        .into_iter()
        .filter(|k| counts[k.index()] > 0)
        .map(|kind| KindCount {
            kind,
            name: kind.name().into(),
            count: counts[kind.index()],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Terrain;

    fn tiny() -> Game {
        let mut g = Grid::new(10, 6);
        g.set_terrain(0, 2, Terrain::Spawn);
        g.set_terrain(9, 2, Terrain::Core);
        Game::new(g, "Test", 1, 0)
    }

    #[test]
    fn cannot_seal_core() {
        let mut g = tiny();
        for y in 0..6 {
            if y == 2 {
                continue;
            }
            let _ = g.place(4, y, BuildKind::Barricade);
        }
        g.credits = 9999;
        let err = g.place(4, 2, BuildKind::Barricade).unwrap_err();
        assert_eq!(err, PlaceError::BlocksPath);
    }

    #[test]
    fn place_and_sell_refunds() {
        let mut g = tiny();
        g.credits = 50;
        g.place(3, 2, BuildKind::Barricade).unwrap();
        assert_eq!(g.credits, 42);
        g.hand.selected = g.towers.last().map(|t| t.id);
        g.sell().unwrap();
        assert!(g.credits > 42);
        assert!(g.grid.buildable(3, 2));
    }

    #[test]
    fn autocannon_kills_runner() {
        let mut g = tiny();
        g.credits = 500;
        g.place(7, 2, BuildKind::Autocannon).unwrap();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(6.4, 2.5);
        g.creeps[0].speed = 0.0;
        for _ in 0..180 {
            g.step();
            if g.creeps.is_empty() {
                break;
            }
        }
        assert!(g.kills >= 1);
    }

    #[test]
    fn flying_ignores_wall() {
        let mut g = tiny();
        g.credits = 999;
        for y in 0..6 {
            if y == 2 {
                continue;
            }
            g.place(5, y, BuildKind::Barricade).unwrap();
        }
        g.place(5, 1, BuildKind::Barricade).ok();
        g.place(5, 3, BuildKind::Barricade).ok();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Wasp,
            spawn: (0, 2),
        });
        let start = g.creeps[0].pos;
        for _ in 0..30 {
            g.step();
        }
        assert!(g.creeps[0].pos.x > start.x);
    }

    #[test]
    fn upgrade_requires_cash() {
        let mut g = tiny();
        g.credits = 50;
        g.place(3, 1, BuildKind::Autocannon).unwrap();
        g.credits = 0;
        let err = g.upgrade().unwrap_err();
        assert_eq!(err, PlaceError::CantAfford);
    }

    #[test]
    fn pulse_slows_ground() {
        let mut g = tiny();
        g.credits = 9999;
        g.place(6, 2, BuildKind::PulseArray).unwrap();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(6.5, 2.5);
        g.creeps[0].speed = 0.0;
        for _ in 0..90 {
            g.step();
        }
        assert!(g.creeps[0].slow_ttl > 0.0 || g.kills >= 1);
    }

    #[test]
    fn satchel_costs_and_kills() {
        let mut g = tiny();
        g.credits = 200;
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(5.5, 2.5);
        g.hand.strike = StrikeKind::Satchel;
        assert!(g.fire_strike(5, 2));
        assert!(g.credits < 200);
        g.reap_creeps();
        assert!(g.creeps.is_empty() || g.creeps[0].hp < g.creeps[0].hp_max);
    }

    #[test]
    fn helios_converts_once() {
        let mut g = tiny();
        g.credits = 999;
        g.place(4, 1, BuildKind::Helios).unwrap();
        g.convert().unwrap();
        assert!(g.towers[0].air_focus);
        assert_eq!(g.convert().unwrap_err(), PlaceError::AlreadyAir);
    }

    #[test]
    fn four_tiers_then_max() {
        let mut g = tiny();
        g.credits = 50_000;
        g.place(3, 1, BuildKind::Autocannon).unwrap();
        for _ in 0..3 {
            g.upgrade().unwrap();
        }
        assert_eq!(g.towers[0].tier, 3);
        assert_eq!(g.upgrade().unwrap_err(), PlaceError::MaxTier);
    }

    #[test]
    fn targeting_last_picks_further() {
        let mut g = tiny();
        g.credits = 999;
        g.place(5, 2, BuildKind::Autocannon).unwrap();
        g.towers[0].target_mode = TargetMode::Last;
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(6.2, 2.5);
        g.creeps[1].pos = Vec2::new(4.2, 2.5);
        g.creeps[0].speed = 0.0;
        g.creeps[1].speed = 0.0;
        g.step();
        let stats = crate::defs::scaled_turret(BuildKind::Autocannon, 0).unwrap();
        let id = acquire(
            &g.creeps,
            &g.flow,
            &g.grid,
            &g.core_clusters,
            g.core_mid,
            Grid::cell_center(5, 2),
            &stats,
            TargetMode::Last,
        )
        .unwrap();
        assert_eq!(id, g.creeps[1].id);
    }

    fn roomy(modifier: Modifier) -> Game {
        let mut g = Grid::new(16, 8);
        g.set_terrain(0, 4, Terrain::Spawn);
        g.set_terrain(15, 4, Terrain::Core);
        Game::with_modifier(g, "Test", 1, 0, modifier)
    }

    #[test]
    fn ground_only_skips_wasps() {
        let mut g = roomy(Modifier::GroundOnly);
        g.wave = 12;
        g.phase = Phase::Fortify { remaining: 4.0 };
        assert!(g.call_wave());
        assert!(g.spawn_q.iter().all(|o| o.kind != CreepKind::Wasp));
        assert!(g.spawn_q.iter().any(|o| o.kind == CreepKind::Lorry));
    }

    #[test]
    fn turret_cap_blocks_eleventh_gun() {
        let mut g = roomy(Modifier::Cap10);
        g.credits = 50_000;
        let mut placed = 0;
        for y in 0..8 {
            for x in 2..14 {
                if y == 4 {
                    continue;
                }
                if g.place(x, y, BuildKind::Autocannon).is_ok() {
                    placed += 1;
                }
                if placed >= 10 {
                    break;
                }
            }
            if placed >= 10 {
                break;
            }
        }
        assert_eq!(placed, 10);
        let err = g.place(2, 1, BuildKind::Autocannon).unwrap_err();
        assert_eq!(err, PlaceError::TurretCap);
        assert!(g.place(2, 1, BuildKind::Barricade).is_ok());
    }

    #[test]
    fn fixed_scrap_kills_pay_nothing() {
        let mut g = roomy(Modifier::FixedScrap);
        let start = g.credits;
        assert_eq!(start, crate::modifiers::FIXED_SCRAP);
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 4),
        });
        assert_eq!(g.creeps[0].bounty, 0);
        g.creeps[0].hp = 0.0;
        g.reap_creeps();
        assert_eq!(g.credits, start);
        assert_eq!(g.kills, 1);
    }

    #[test]
    fn accelerated_is_fast_and_thin() {
        let mut g = roomy(Modifier::Accelerated);
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 4),
        });
        let stats = creep_stats(CreepKind::Runner);
        assert!(g.creeps[0].speed > stats.speed * 1.5);
        assert!(g.creeps[0].hp_max < stats.hp * 0.7);
    }

    #[test]
    fn air_steers_to_nearest_core() {
        let mut g = Game::start(5, Modifier::Standard, Some(1));
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Wasp,
            spawn: (0, 5),
        });
        let start = g.creeps[0].pos;
        for _ in 0..40 {
            g.step();
        }
        let pos = g.creeps[0].pos;
        let west = g.grid.nearest_core(start);
        assert!(pos.dist(west) < start.dist(west));
        assert!(pos.x < 12.0);
    }

    #[test]
    fn replay_repeats_placements() {
        let mut g = tiny();
        g.credits = 999;
        g.set_build(BuildKind::Autocannon as u8);
        g.click(3, 1);
        assert_eq!(g.towers.len(), 1);
        let bundle = g.replay_bundle();
        assert!(!bundle.orders.is_empty());
        let mut g2 = Game::from_replay(bundle).unwrap();
        g2.run_recorded(None, Some(1));
        assert_eq!(g2.towers.len(), 1);
        assert_eq!(g2.towers[0].kind, BuildKind::Autocannon);
    }

    #[test]
    fn mission_clears_objective_after_hold_wave() {
        let mut g = Game::mission(0).expect("open ground");
        assert_eq!(g.hold_until, Some(8));
        assert!(!g.objective_cleared);
        g.wave = 8;
        g.spawn_q.clear();
        g.creeps.clear();
        g.phase = Phase::Incoming;
        g.step();
        assert!(g.objective_cleared);
        assert_eq!(g.wave, 9);
        assert!(matches!(g.phase, Phase::Fortify { .. }));
        let snap = g.snapshot();
        assert_eq!(snap.mission_id, Some(0));
        assert_eq!(snap.objective_wave, Some(8));
        assert!(snap.objective_cleared);
        assert!(!snap.seed_hex.is_empty());
    }

    #[test]
    fn replay_bundle_verifies() {
        let mut g = tiny();
        g.credits = 999;
        g.set_build(BuildKind::Barricade as u8);
        g.click(3, 1);
        let bundle = g.replay_bundle();
        let hash = bundle.hash.clone().expect("hash");
        assert_eq!(hash, crate::replay_hash(&bundle));
        let report = crate::verify_replay(bundle);
        assert!(report.ok, "{:?}", report.error);
        assert!(report.hash_ok);
        assert!(report.outcome_ok);
    }

    #[test]
    fn bargain_pack_cheapens_walls() {
        let mut g = tiny();
        g.credits = 4;
        assert_eq!(
            g.place(3, 2, BuildKind::Barricade).unwrap_err(),
            PlaceError::CantAfford
        );
        g.apply_pack(
            crate::pack::presets()
                .into_iter()
                .find(|p| p.slug == "bargain")
                .unwrap(),
        )
        .unwrap();
        g.place(3, 2, BuildKind::Barricade).unwrap();
        assert!(g.pack.is_some());
        let snap = g.snapshot();
        assert_eq!(snap.pack_name.as_deref(), Some("Bargain Bin"));
        let bundle = g.replay_bundle();
        assert!(bundle.pack.is_some());
        let stock_hash = tiny().replay_bundle();
        assert_ne!(crate::replay_hash(&bundle), crate::replay_hash(&stock_hash));
        let report = crate::verify_replay(bundle);
        assert!(report.ok, "{:?}", report.error);
    }

    #[test]
    fn mite_splits_on_kill() {
        let mut g = tiny();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Mite,
            spawn: (0, 2),
        });
        g.creeps[0].hp = 0.0;
        g.reap_creeps();
        assert_eq!(g.creeps.len(), 2);
        assert!(g
            .creeps
            .iter()
            .all(|c| c.kind == CreepKind::Mite && c.split_gen == 1));
        g.creeps[0].hp = 0.0;
        g.creeps[1].hp = 0.0;
        g.reap_creeps();
        assert!(g.creeps.is_empty());
        assert_eq!(g.kills, 3);
    }

    #[test]
    fn mite_leak_does_not_split() {
        let mut g = tiny();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Mite,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Grid::cell_center(9, 2);
        g.tick_creeps();
        assert!(g.creeps.is_empty());
        assert_eq!(g.leaks, 1);
        assert_eq!(g.kills, 0);
    }

    #[test]
    fn wave_two_is_swarm_with_mites() {
        let mut g = roomy(Modifier::Standard);
        g.wave = 2;
        g.phase = Phase::Fortify { remaining: 4.0 };
        assert!(g.call_wave());
        assert!(g.spawn_q.iter().any(|o| o.kind == CreepKind::Mite));
        let intel = g.snapshot().wave_intel;
        assert_eq!(intel.script, "Swarm");
        assert!(intel.total > 0);
    }

    #[test]
    fn after_action_tracks_spend() {
        let mut g = tiny();
        g.credits = 200;
        g.place(3, 1, BuildKind::Barricade).unwrap();
        let after = g.snapshot().after;
        assert_eq!(after.spent, 8);
    }

    #[test]
    fn interest_pays_on_clear() {
        let mut g = tiny();
        g.credits = 1000;
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.step();
        assert_eq!(g.last_interest, 40);
        assert_eq!(g.credits, 1040);
        assert_eq!(g.snapshot().interest_bps, 400);
    }

    #[test]
    fn fixed_scrap_pays_no_interest() {
        let mut g = roomy(Modifier::FixedScrap);
        g.credits = 1000;
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.step();
        assert_eq!(g.last_interest, 0);
        assert_eq!(g.credits, 1000);
    }

    #[test]
    fn medic_heals_nearby_ground() {
        let mut g = tiny();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Medic,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(4.0, 2.5);
        g.creeps[1].pos = Vec2::new(4.2, 2.5);
        g.creeps[0].speed = 0.0;
        g.creeps[1].speed = 0.0;
        g.creeps[0].hp = 10.0;
        for _ in 0..60 {
            g.tick_heals();
        }
        assert!(g.creeps[0].hp > 10.0);
        assert!(g.creeps[0].hp <= g.creeps[0].hp_max);
    }

    #[test]
    fn autocannon_cannot_see_shade() {
        let mut g = tiny();
        g.credits = 500;
        g.place(7, 2, BuildKind::Autocannon).unwrap();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Shade,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(6.4, 2.5);
        g.creeps[0].speed = 0.0;
        let hp = g.creeps[0].hp;
        for _ in 0..180 {
            g.step();
        }
        assert_eq!(g.kills, 0);
        assert!(g
            .creeps
            .iter()
            .any(|c| c.kind == CreepKind::Shade && c.hp == hp));
    }

    #[test]
    fn pulse_kills_shade() {
        let mut g = tiny();
        g.credits = 500;
        g.place(7, 2, BuildKind::PulseArray).unwrap();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Shade,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Vec2::new(6.4, 2.5);
        g.creeps[0].speed = 0.0;
        for _ in 0..240 {
            g.step();
            if g.creeps.is_empty() {
                break;
            }
        }
        assert!(g.kills >= 1);
    }

    #[test]
    fn strike_hits_shade() {
        let mut g = tiny();
        g.credits = 500;
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Shade,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Grid::cell_center(5, 2);
        g.creeps[0].speed = 0.0;
        assert!(g.fire_strike_kind(StrikeKind::Satchel, 5, 2));
        g.reap_creeps();
        assert!(g.kills >= 1);
    }

    #[test]
    fn repair_restores_integrity() {
        let mut g = tiny();
        g.integrity = 15;
        g.credits = 100;
        g.repair().unwrap();
        assert_eq!(g.integrity, 16);
        assert_eq!(g.credits, 100 - REPAIR_COST);
        g.integrity = STARTING_INTEGRITY;
        assert_eq!(g.repair().unwrap_err(), PlaceError::Intact);
        g.integrity = 0;
        g.phase = Phase::Defeat;
        assert_eq!(g.repair().unwrap_err(), PlaceError::RelayDown);
        let mut live = tiny();
        live.integrity = 12;
        live.credits = 200;
        live.repair().unwrap();
        assert!(live
            .replay_bundle()
            .orders
            .iter()
            .any(|o| matches!(o.op, OrderOp::Repair)));
    }

    #[test]
    fn walk_grows_when_the_maze_folds() {
        let mut g = tiny();
        g.credits = 999;
        let open = g.snapshot().walk;
        assert_eq!(open, 9);
        for y in 0..5 {
            g.place(4, y, BuildKind::Barricade).unwrap();
        }
        assert!(g.snapshot().walk > open);
    }

    #[test]
    fn lift_moves_a_wall() {
        let mut g = tiny();
        g.credits = 100;
        g.place(3, 2, BuildKind::Barricade).unwrap();
        let id = g.towers.last().unwrap().id;
        g.hand.selected = Some(id);
        g.lift().unwrap();
        assert!(g.hand.lift.is_some());
        g.click(3, 1);
        assert_eq!(g.towers.last().unwrap().x, 3);
        assert_eq!(g.towers.last().unwrap().y, 1);
        assert!(g.grid.buildable(3, 2));
        assert_eq!(g.credits, 100 - 8 - MOVE_COST);
        assert!(g.hand.lift.is_none());
    }

    #[test]
    fn sell_clears_a_pending_lift() {
        let mut g = tiny();
        g.credits = 100;
        g.place(3, 2, BuildKind::Barricade).unwrap();
        let id = g.towers.last().unwrap().id;
        g.hand.selected = Some(id);
        g.lift().unwrap();
        g.sell().unwrap();
        assert!(g.hand.lift.is_none());
        assert!(!g.snapshot().relocating);
        g.click(5, 1);
        assert!(g.hand.selected.is_none());
        assert!(g.towers.is_empty());
    }

    #[test]
    fn kill_credit_goes_to_the_shooter() {
        let mut g = tiny();
        g.credits = 500;
        g.place(7, 2, BuildKind::Autocannon).unwrap();
        let far = g.towers.last().unwrap().id;
        g.place(3, 2, BuildKind::Autocannon).unwrap();
        let near = g.towers.last().unwrap().id;
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Runner,
            spawn: (0, 2),
        });
        g.creeps[0].hp = 0.0;
        g.creeps[0].hit_by = Some(far);
        g.reap_creeps();
        assert_eq!(g.towers.iter().find(|t| t.id == far).unwrap().kills, 1);
        assert_eq!(g.towers.iter().find(|t| t.id == near).unwrap().kills, 0);
    }

    #[test]
    fn unknown_mission_is_none() {
        assert!(Game::mission(99).is_none());
        assert!(Game::challenge(99).is_none());
        assert!(Game::mission(0).is_some());
    }

    #[test]
    fn walk_paths_reach_the_relay() {
        let g = tiny();
        let paths = g.snapshot().walk_paths;
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].first().copied(), Some([0, 2]));
        assert_eq!(paths[0].last().copied(), Some([9, 2]));
    }

    #[test]
    fn colossus_stuns_nearby_guns() {
        let mut g = tiny();
        g.credits = 500;
        g.place(6, 2, BuildKind::Autocannon).unwrap();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Colossus,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Grid::cell_center(5, 2);
        g.creeps[0].speed = 0.0;
        g.creeps[0].roar_cd = 0.0;
        g.tick_roars();
        let gun = g.towers.iter().find(|t| t.kind.is_turret()).unwrap();
        assert!(gun.stun_ttl > 0.0);
        assert!(g.snapshot().towers.iter().any(|t| t.stunned));
    }

    #[test]
    fn overcharge_heats_a_gun() {
        let mut g = tiny();
        g.credits = 200;
        g.place(3, 2, BuildKind::Autocannon).unwrap();
        let id = g.towers.last().unwrap().id;
        g.hand.selected = Some(id);
        g.overcharge().unwrap();
        assert!(g.towers.last().unwrap().overcharge_ttl > 0.0);
        assert_eq!(g.credits, 200 - 50 - OVERCHARGE_COST);
        assert!(g
            .replay_bundle()
            .orders
            .iter()
            .any(|o| matches!(o.op, OrderOp::Overcharge)));
        g.place(4, 1, BuildKind::Barricade).unwrap();
        g.hand.selected = g
            .towers
            .iter()
            .find(|t| t.kind == BuildKind::Barricade)
            .map(|t| t.id);
        assert_eq!(g.overcharge().unwrap_err(), PlaceError::NotATurret);
    }

    #[test]
    fn flicker_hops_along_the_walk() {
        let mut g = tiny();
        g.spawn_q.clear();
        g.phase = Phase::Incoming;
        g.spawn_creep(SpawnOrder {
            kind: CreepKind::Flicker,
            spawn: (0, 2),
        });
        g.creeps[0].pos = Grid::cell_center(1, 2);
        g.creeps[0].speed = 0.0;
        g.creeps[0].blink_cd = 0.0;
        let start = g.creeps[0].pos;
        g.tick_flickers();
        assert!(g.creeps[0].pos.x > start.x + 0.5);
    }
}
