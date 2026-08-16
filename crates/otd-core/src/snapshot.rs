use crate::defs::{BuildKind, CreepKind, FireMode, TargetMode, STRIKE_CATALOG};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: u8,
    pub hotkey: String,
    pub name: String,
    pub role: String,
    pub blurb: String,
    pub cost: i32,
    pub range: f32,
    pub hits_ground: bool,
    pub hits_air: bool,
    pub detects: bool,
    pub fire: FireMode,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeItem {
    pub id: u8,
    pub hotkey: String,
    pub name: String,
    pub blurb: String,
    pub cost: i32,
    pub radius: f32,
    pub cooldown: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapStatic {
    pub w: i32,
    pub h: i32,
    pub id: u8,
    pub name: String,
    /// Visual identity, so the in-match ground matches the thumbnail the player picked.
    pub slug: String,
    pub seed: u64,
    pub core: Vec<[i32; 2]>,
    pub spawns: Vec<[i32; 2]>,
    pub rocks: Vec<[i32; 2]>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverInfo {
    pub x: i32,
    pub y: i32,
    pub valid: bool,
    pub reason: String,
    pub range: f32,
    pub hits_ground: bool,
    pub hits_air: bool,
    pub strike: bool,
    pub walk_after: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedInfo {
    pub id: u32,
    pub kind: BuildKind,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub tier: u8,
    pub max_tier: u8,
    pub tier_name: String,
    pub range: f32,
    pub damage: f32,
    pub fire_interval: f32,
    pub splash: f32,
    pub hits_ground: bool,
    pub hits_air: bool,
    pub detects: bool,
    pub fire: FireMode,
    pub targeting: TargetMode,
    pub targeting_label: String,
    pub can_convert: bool,
    pub convert_cost: Option<i32>,
    pub invested: i32,
    pub upgrade_cost: Option<i32>,
    pub sell_value: i32,
    pub kills: u32,
    pub damage_dealt: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TowerView {
    pub id: u32,
    pub kind: BuildKind,
    pub x: i32,
    pub y: i32,
    pub aim: f32,
    pub tier: u8,
    pub air_focus: bool,
    pub stunned: bool,
    pub overcharged: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreepView {
    pub id: u32,
    pub kind: CreepKind,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub hp_max: f32,
    pub flying: bool,
    pub heading: f32,
    pub radius: f32,
    pub slowed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjView {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: BuildKind,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FxView {
    /// `&'static str`: the sim already stores these as static strings, and building a
    /// fresh String per effect meant up to 180 heap allocations per frame.
    pub kind: &'static str,
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub mag: f32,
    pub heading: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeamView {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub kind: BuildKind,
    pub life: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeHud {
    pub id: u8,
    pub ready: bool,
    pub cooldown: f32,
    pub cost: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindCount {
    pub kind: CreepKind,
    pub name: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveIntel {
    pub script: String,
    pub total: u32,
    pub parts: Vec<KindCount>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GunScore {
    pub name: String,
    pub kills: u32,
    pub damage: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AfterAction {
    pub spent: i32,
    pub kills: u32,
    pub leaks: u32,
    pub wave: u32,
    pub kill_kinds: Vec<KindCount>,
    pub leak_kinds: Vec<KindCount>,
    pub guns: Vec<GunScore>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub tick: u64,
    pub time: f32,
    pub status: String,
    pub defeated: bool,
    pub credits: i32,
    pub integrity: i32,
    pub integrity_max: i32,
    pub wave: u32,
    pub next_wave_in: f32,
    pub can_call_wave: bool,
    pub creeps_alive: u32,
    pub creeps_remaining: u32,
    pub kills: u32,
    pub leaks: u32,
    pub banner: Option<String>,
    pub banner_life: f32,
    pub message: Option<String>,
    pub hurt_flash: f32,
    pub build: u8,
    pub strike: u8,
    pub map_id: u8,
    pub map_name: String,
    pub modifier_id: u8,
    pub modifier_name: String,
    pub turret_count: u32,
    pub turret_cap: Option<u32>,
    pub hover: Option<HoverInfo>,
    pub selected: Option<SelectedInfo>,
    pub strikes: Vec<StrikeHud>,
    pub walls: Vec<[i32; 2]>,
    pub towers: Vec<TowerView>,
    pub creeps: Vec<CreepView>,
    pub projectiles: Vec<ProjView>,
    pub fx: Vec<FxView>,
    pub beams: Vec<BeamView>,
    pub core: [f32; 2],
    pub cores: Vec<[f32; 2]>,
    pub objective_wave: Option<u32>,
    pub objective_cleared: bool,
    pub mission_id: Option<u8>,
    pub challenge_id: Option<u8>,
    pub mission_name: Option<String>,
    pub seed_hex: String,
    pub pack_name: Option<String>,
    pub wave_intel: WaveIntel,
    pub after: AfterAction,
    pub interest_paid: i32,
    pub interest_bps: u32,
    /// Action prices, so the HUD never hardcodes them. A pack retune must not make a
    /// button label lie about what it costs.
    pub move_cost: i32,
    pub repair_cost: i32,
    pub overcharge_cost: i32,
    pub walk: u32,
    pub relocating: bool,
    pub walk_paths: Vec<Vec<[i32; 2]>>,
}

pub fn catalog_json() -> String {
    let items: Vec<CatalogItem> = crate::defs::BUILD_CATALOG
        .iter()
        .map(|s| CatalogItem {
            id: s.kind as u8,
            hotkey: s.kind.hotkey().to_string(),
            name: s.name.to_string(),
            role: s.role.to_string(),
            blurb: s.blurb.to_string(),
            cost: s.cost,
            range: s.range,
            hits_ground: s.hits_ground,
            hits_air: s.hits_air,
            detects: s.kind.detects(),
            fire: s.fire,
        })
        .collect();
    serde_json::to_string(&items).expect("catalog json")
}

pub fn strikes_json() -> String {
    let items: Vec<StrikeItem> = STRIKE_CATALOG
        .iter()
        .map(|s| StrikeItem {
            id: s.kind as u8,
            hotkey: s.hotkey.to_string(),
            name: s.name.to_string(),
            blurb: s.blurb.to_string(),
            cost: s.cost,
            radius: s.radius,
            cooldown: s.cooldown,
        })
        .collect();
    serde_json::to_string(&items).expect("strikes json")
}

pub fn theaters_json() -> String {
    serde_json::to_string(&crate::maps::theaters()).expect("theaters json")
}

pub fn modifiers_json() -> String {
    serde_json::to_string(&crate::modifiers::modifiers()).expect("modifiers json")
}

pub fn daily_json(utc_day: u32) -> String {
    serde_json::to_string(&crate::modifiers::daily_pick(utc_day)).expect("daily json")
}
