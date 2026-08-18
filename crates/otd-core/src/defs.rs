use serde::{Deserialize, Serialize};

pub const DT: f32 = 1.0 / 60.0;
pub const STARTING_CREDITS: i32 = 1000;
pub const STARTING_INTEGRITY: i32 = 20;
pub const SELL_RATIO: f32 = 0.65;
pub const MAX_TIER: u8 = 3;
pub const FIRST_WAVE_DELAY: f32 = 14.0;
pub const WAVE_DELAY: f32 = 8.5;
pub const HELIOS_CONVERT_COST: i32 = 1;
pub const MEDIC_HEAL_PER_SEC: f32 = 16.0;
pub const MEDIC_HEAL_RADIUS: f32 = 1.9;
pub const INTEREST_BPS: u32 = 400;
pub const INTEREST_CAP: i32 = 48;
pub const REPAIR_COST: i32 = 35;
pub const MOVE_COST: i32 = 6;
pub const FLICKER_PERIOD: f32 = 2.2;
pub const FLICKER_STEPS: u32 = 3;
pub const OVERCHARGE_COST: i32 = 40;
pub const OVERCHARGE_TTL: f32 = 6.5;
pub const OVERCHARGE_FIRE_MUL: f32 = 0.52;
pub const COLOSSUS_ROAR_PERIOD: f32 = 4.8;
pub const COLOSSUS_ROAR_RADIUS: f32 = 2.55;
pub const COLOSSUS_STUN: f32 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildKind {
    Inspect = 0,
    Barricade = 1,
    Autocannon = 2,
    Howitzer = 3,
    Skystinger = 4,
    Inferno = 5,
    ArcLance = 6,
    PulseArray = 7,
    Helios = 8,
    SwarmRack = 9,
    SiegeRail = 10,
}

impl BuildKind {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Barricade,
            2 => Self::Autocannon,
            3 => Self::Howitzer,
            4 => Self::Skystinger,
            5 => Self::Inferno,
            6 => Self::ArcLance,
            7 => Self::PulseArray,
            8 => Self::Helios,
            9 => Self::SwarmRack,
            10 => Self::SiegeRail,
            _ => Self::Inspect,
        }
    }

    pub fn is_structure(self) -> bool {
        !matches!(self, Self::Inspect)
    }

    pub fn is_turret(self) -> bool {
        !matches!(self, Self::Inspect | Self::Barricade)
    }

    /// Camo hulls are skipped unless the shooter detects. Packs cannot override this.
    pub fn detects(self) -> bool {
        matches!(
            self,
            Self::Skystinger | Self::ArcLance | Self::PulseArray | Self::Helios | Self::SwarmRack
        )
    }

    pub fn hotkey(self) -> &'static str {
        match self {
            Self::Inspect => "Esc",
            Self::Barricade => "1",
            Self::Autocannon => "2",
            Self::Howitzer => "3",
            Self::Skystinger => "4",
            Self::Inferno => "5",
            Self::ArcLance => "6",
            Self::PulseArray => "7",
            Self::Helios => "8",
            Self::SwarmRack => "9",
            Self::SiegeRail => "0",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CreepKind {
    Runner,
    Lorry,
    Bulwark,
    Wasp,
    Colossus,
    Mite,
    Medic,
    Shade,
    Flicker,
}

impl CreepKind {
    pub const ALL: [Self; 9] = [
        Self::Runner,
        Self::Lorry,
        Self::Bulwark,
        Self::Wasp,
        Self::Colossus,
        Self::Mite,
        Self::Medic,
        Self::Shade,
        Self::Flicker,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Runner => "Runner",
            Self::Lorry => "Lorry",
            Self::Bulwark => "Bulwark",
            Self::Wasp => "Wasp",
            Self::Colossus => "Colossus",
            Self::Mite => "Mite",
            Self::Medic => "Medic",
            Self::Shade => "Shade",
            Self::Flicker => "Flicker",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Runner => 0,
            Self::Lorry => 1,
            Self::Bulwark => 2,
            Self::Wasp => 3,
            Self::Colossus => 4,
            Self::Mite => 5,
            Self::Medic => 6,
            Self::Shade => 7,
            Self::Flicker => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FireMode {
    Shell,
    Cone,
    Line,
    Pulse,
    Beam,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetMode {
    #[default]
    First,
    Last,
    Strong,
    Weak,
    Flying,
    Camo,
}

impl TargetMode {
    pub fn next(self) -> Self {
        match self {
            Self::First => Self::Last,
            Self::Last => Self::Strong,
            Self::Strong => Self::Weak,
            Self::Weak => Self::Flying,
            Self::Flying => Self::Camo,
            Self::Camo => Self::First,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::First => "First",
            Self::Last => "Last",
            Self::Strong => "Strong",
            Self::Weak => "Weak",
            Self::Flying => "Flying",
            Self::Camo => "Camo",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StrikeKind {
    None = 0,
    Satchel = 1,
    Overload = 2,
    Orbital = 3,
}

impl StrikeKind {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Satchel,
            2 => Self::Overload,
            3 => Self::Orbital,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TurretStats {
    pub kind: BuildKind,
    pub name: &'static str,
    pub role: &'static str,
    pub blurb: &'static str,
    pub cost: i32,
    pub range: f32,
    pub fire_interval: f32,
    pub damage: f32,
    pub splash: f32,
    pub proj_speed: f32,
    pub hits_ground: bool,
    pub hits_air: bool,
    pub homing: bool,
    pub fire: FireMode,
    pub volley: u8,
    pub slow: f32,
    pub slow_ttl: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct CreepStats {
    #[allow(dead_code)]
    pub kind: CreepKind,
    #[allow(dead_code)]
    pub name: &'static str,
    pub hp: f32,
    pub speed: f32,
    pub bounty: i32,
    pub leak: i32,
    pub armor: f32,
    pub flying: bool,
    pub camo: bool,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct StrikeStats {
    pub kind: StrikeKind,
    pub name: &'static str,
    pub hotkey: &'static str,
    pub blurb: &'static str,
    pub cost: i32,
    pub radius: f32,
    pub damage: f32,
    pub slow: f32,
    pub slow_ttl: f32,
    pub cooldown: f32,
    pub hits_ground: bool,
    pub hits_air: bool,
}

pub const TIER_NAMES: [&str; 4] = ["Mark I", "Mark II", "Mark III", "Apex"];

pub const BUILD_CATALOG: &[TurretStats] = &[
    TurretStats {
        kind: BuildKind::Barricade,
        name: "Barricade",
        role: "Maze",
        blurb: "Cheap block. Stretch the walk. Towers also block — save these for filler.",
        cost: 8,
        range: 0.0,
        fire_interval: 0.0,
        damage: 0.0,
        splash: 0.0,
        proj_speed: 0.0,
        hits_ground: false,
        hits_air: false,
        homing: false,
        fire: FireMode::Shell,
        volley: 0,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::Autocannon,
        name: "Autocannon",
        role: "Dual",
        blurb: "Fast tracers. Hits ground and air. Blind to Shades. The gun you start with and eventually sell.",
        cost: 50,
        range: 2.70,
        fire_interval: 0.16,
        damage: 5.5,
        splash: 0.0,
        proj_speed: 26.0,
        hits_ground: true,
        hits_air: true,
        homing: false,
        fire: FireMode::Shell,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::Howitzer,
        name: "Howitzer",
        role: "Ground",
        blurb: "Slow shells, fat splash. Blind to Shades. Wants a packed corridor — not a lonely approach.",
        cost: 110,
        range: 3.20,
        fire_interval: 0.92,
        damage: 34.0,
        splash: 0.78,
        proj_speed: 9.5,
        hits_ground: true,
        hits_air: false,
        homing: false,
        fire: FireMode::Shell,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::Skystinger,
        name: "Skystinger",
        role: "Air",
        blurb: "Long-reach bursts. Sees camo, but still air-only. Wasps ignore your maze; this is why they do not ignore you.",
        cost: 90,
        range: 3.85,
        fire_interval: 0.24,
        damage: 13.0,
        splash: 0.42,
        proj_speed: 14.0,
        hits_ground: false,
        hits_air: true,
        homing: true,
        fire: FireMode::Shell,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::Inferno,
        name: "Inferno",
        role: "Cone",
        blurb: "Ground cone. Blind to Shades. Park it at a corner where the maze folds back on itself.",
        cost: 140,
        range: 2.55,
        fire_interval: 0.14,
        damage: 7.5,
        splash: 0.58,
        proj_speed: 0.0,
        hits_ground: true,
        hits_air: false,
        homing: false,
        fire: FireMode::Cone,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::ArcLance,
        name: "Arc Lance",
        role: "Line",
        blurb: "Burns the whole corridor. Sees Shades. One tile that sees a long straight is worth three that do not.",
        cost: 175,
        range: 4.10,
        fire_interval: 0.62,
        damage: 22.0,
        splash: 0.28,
        proj_speed: 0.0,
        hits_ground: true,
        hits_air: false,
        homing: false,
        fire: FireMode::Line,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::PulseArray,
        name: "Pulse Array",
        role: "Slow",
        blurb: "Chip plus slow in a disc. Sees Shades. Stack them along a lane so nothing walks out at full speed.",
        cost: 130,
        range: 2.35,
        fire_interval: 0.78,
        damage: 14.0,
        splash: 0.0,
        proj_speed: 0.0,
        hits_ground: true,
        hits_air: true,
        homing: false,
        fire: FireMode::Pulse,
        volley: 1,
        slow: 0.62,
        slow_ttl: 1.7,
    },
    TurretStats {
        kind: BuildKind::Helios,
        name: "Helios",
        role: "Beam",
        blurb: "Dwell beam. Sees Shades. Ground by default. Convert to air for 1 scrap — no converting back except sell.",
        cost: 200,
        range: 3.55,
        fire_interval: 0.07,
        damage: 9.0,
        splash: 0.0,
        proj_speed: 0.0,
        hits_ground: true,
        hits_air: false,
        homing: false,
        fire: FireMode::Beam,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::SwarmRack,
        name: "Swarm Rack",
        role: "Air",
        blurb: "Volley of seekers. Sees camo, still air-only. If a Wasp dies mid-flight, the rest pick a new in-range target.",
        cost: 160,
        range: 4.20,
        fire_interval: 0.95,
        damage: 11.0,
        splash: 0.22,
        proj_speed: 11.5,
        hits_ground: false,
        hits_air: true,
        homing: true,
        fire: FireMode::Shell,
        volley: 3,
        slow: 0.0,
        slow_ttl: 0.0,
    },
    TurretStats {
        kind: BuildKind::SiegeRail,
        name: "Siege Rail",
        role: "Pierce",
        blurb: "Slow, brutal, armour-blind. Blind to Shades. For Bulwarks and the Colossus, not for Runners.",
        cost: 240,
        range: 4.40,
        fire_interval: 1.85,
        damage: 155.0,
        splash: 0.0,
        proj_speed: 22.0,
        hits_ground: true,
        hits_air: false,
        homing: false,
        fire: FireMode::Shell,
        volley: 1,
        slow: 0.0,
        slow_ttl: 0.0,
    },
];

pub const STRIKE_CATALOG: &[StrikeStats] = &[
    StrikeStats {
        kind: StrikeKind::Satchel,
        name: "Satchel",
        hotkey: "Q",
        blurb: "Small boom. Emergency clear on a packed tile.",
        cost: 75,
        radius: 1.35,
        damage: 95.0,
        slow: 0.0,
        slow_ttl: 0.0,
        cooldown: 0.45,
        hits_ground: true,
        hits_air: true,
    },
    StrikeStats {
        kind: StrikeKind::Overload,
        name: "Overload",
        hotkey: "W",
        blurb: "Slow field. Buy time for the killbox when the walk is too short.",
        cost: 110,
        radius: 2.45,
        damage: 18.0,
        slow: 0.42,
        slow_ttl: 2.8,
        cooldown: 1.15,
        hits_ground: true,
        hits_air: true,
    },
    StrikeStats {
        kind: StrikeKind::Orbital,
        name: "Orbital",
        hotkey: "E",
        blurb: "Wide radial, hardest at the centre. Last resort, not a farming tool.",
        cost: 260,
        radius: 3.80,
        damage: 240.0,
        slow: 0.0,
        slow_ttl: 0.0,
        cooldown: 2.2,
        hits_ground: true,
        hits_air: true,
    },
];

#[allow(dead_code)]
pub fn stats_for(kind: BuildKind) -> Option<&'static TurretStats> {
    BUILD_CATALOG.iter().find(|s| s.kind == kind)
}

#[allow(dead_code)]
pub fn strike_stats(kind: StrikeKind) -> Option<&'static StrikeStats> {
    STRIKE_CATALOG.iter().find(|s| s.kind == kind)
}

pub fn scale_turret(mut s: TurretStats, tier: u8) -> TurretStats {
    if s.range <= 0.0 {
        return s;
    }
    match tier.min(MAX_TIER) {
        0 => {}
        1 => {
            s.damage *= 1.55;
            s.range *= 1.12;
        }
        2 => {
            s.damage *= 2.25;
            s.range *= 1.22;
            s.fire_interval /= 1.18;
            s.splash *= 1.18;
            s.slow = if s.slow > 0.0 {
                (s.slow * 0.92).max(0.38)
            } else {
                0.0
            };
        }
        _ => {
            s.damage *= 3.45;
            s.range *= 1.38;
            s.fire_interval /= 1.32;
            s.splash *= 1.42;
            s.slow = if s.slow > 0.0 {
                (s.slow * 0.85).max(0.32)
            } else {
                0.0
            };
            if s.volley > 1 {
                s.volley += 1;
            }
        }
    }
    s
}

#[allow(dead_code)]
pub fn scaled_turret(kind: BuildKind, tier: u8) -> Option<TurretStats> {
    Some(scale_turret(*stats_for(kind)?, tier))
}

pub fn upgrade_cost_for(base_cost: i32, current_tier: u8) -> Option<i32> {
    if current_tier >= MAX_TIER {
        return None;
    }
    let base = base_cost as f32;
    let step = current_tier as f32 + 1.0;
    Some((base * 0.78 * step * (1.0 + 0.12 * current_tier as f32)).round() as i32)
}

#[allow(dead_code)]
pub fn upgrade_cost(kind: BuildKind, current_tier: u8) -> Option<i32> {
    upgrade_cost_for(stats_for(kind)?.cost, current_tier)
}

pub fn tier_name(tier: u8) -> &'static str {
    TIER_NAMES[tier.min(MAX_TIER) as usize]
}

pub fn creep_stats(kind: CreepKind) -> CreepStats {
    match kind {
        CreepKind::Runner => CreepStats {
            kind,
            name: "Runner",
            hp: 34.0,
            speed: 2.55,
            bounty: 5,
            leak: 1,
            armor: 0.0,
            flying: false,
            camo: false,
            radius: 0.18,
        },
        CreepKind::Lorry => CreepStats {
            kind,
            name: "Lorry",
            hp: 110.0,
            speed: 1.55,
            bounty: 9,
            leak: 2,
            armor: 0.08,
            flying: false,
            camo: false,
            radius: 0.26,
        },
        CreepKind::Bulwark => CreepStats {
            kind,
            name: "Bulwark",
            hp: 380.0,
            speed: 0.92,
            bounty: 22,
            leak: 3,
            armor: 0.32,
            flying: false,
            camo: false,
            radius: 0.34,
        },
        CreepKind::Wasp => CreepStats {
            kind,
            name: "Wasp",
            hp: 78.0,
            speed: 2.15,
            bounty: 12,
            leak: 2,
            armor: 0.0,
            flying: true,
            camo: false,
            radius: 0.22,
        },
        CreepKind::Colossus => CreepStats {
            kind,
            name: "Colossus",
            hp: 1650.0,
            speed: 0.72,
            bounty: 90,
            leak: 8,
            armor: 0.38,
            flying: false,
            camo: false,
            radius: 0.48,
        },
        CreepKind::Mite => CreepStats {
            kind,
            name: "Mite",
            hp: 22.0,
            speed: 2.85,
            bounty: 3,
            leak: 1,
            armor: 0.0,
            flying: false,
            camo: false,
            radius: 0.13,
        },
        CreepKind::Medic => CreepStats {
            kind,
            name: "Medic",
            hp: 160.0,
            speed: 1.12,
            bounty: 18,
            leak: 2,
            armor: 0.04,
            flying: false,
            camo: false,
            radius: 0.24,
        },
        CreepKind::Shade => CreepStats {
            kind,
            name: "Shade",
            hp: 48.0,
            speed: 2.28,
            bounty: 8,
            leak: 1,
            armor: 0.0,
            flying: false,
            camo: true,
            radius: 0.16,
        },
        CreepKind::Flicker => CreepStats {
            kind,
            name: "Flicker",
            hp: 62.0,
            speed: 2.05,
            bounty: 11,
            leak: 1,
            armor: 0.0,
            flying: false,
            camo: false,
            radius: 0.18,
        },
    }
}

/// Hull HP per wave. 13% through wave 20, then 9%.
///
/// A finished maze has a hard DPS cap: every buildable tile is already a gun.
/// Compounding 13% forever made wave 50 a ~400× sponge. The late rate is still
/// exponential, just slow enough that a packed killbox can hold the walk.
pub fn wave_hp_mul(wave: u32) -> f32 {
    let grown = wave.saturating_sub(1);
    const EARLY_WAVES: u32 = 19;
    const EARLY: f32 = 1.13;
    const LATE: f32 = 1.09;
    if grown <= EARLY_WAVES {
        EARLY.powi(grown as i32)
    } else {
        EARLY.powi(EARLY_WAVES as i32) * LATE.powi((grown - EARLY_WAVES) as i32)
    }
}

pub fn wave_bounty_mul(wave: u32) -> f32 {
    1.0 + 0.085 * wave.saturating_sub(1) as f32
}

pub fn wave_speed_mul(wave: u32) -> f32 {
    1.0 + 0.011 * wave.saturating_sub(1) as f32
}

pub fn armor_pen(kind: BuildKind) -> f32 {
    match kind {
        BuildKind::Howitzer => 0.55,
        BuildKind::Autocannon => 0.05,
        BuildKind::Skystinger => 0.15,
        BuildKind::Inferno => 0.10,
        BuildKind::ArcLance => 0.20,
        BuildKind::PulseArray => 0.08,
        BuildKind::Helios => 0.25,
        BuildKind::SwarmRack => 0.12,
        BuildKind::SiegeRail => 0.90,
        _ => 0.0,
    }
}

pub fn strike_pen(kind: StrikeKind) -> f32 {
    match kind {
        StrikeKind::Satchel => 0.20,
        StrikeKind::Overload => 0.05,
        StrikeKind::Orbital => 0.35,
        StrikeKind::None => 0.0,
    }
}

pub fn apply_armor(damage: f32, armor: f32, pen: f32) -> f32 {
    let effective = (armor * (1.0 - pen)).clamp(0.0, 0.85);
    (damage * (1.0 - effective)).max(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hulls_thicken_then_ease() {
        assert!((wave_hp_mul(1) - 1.0).abs() < 1e-5);
        assert!((wave_hp_mul(20) - 1.13_f32.powi(19)).abs() < 1e-4);
        // Same as the old 13% curve through the campaign / midgame.
        assert!((wave_hp_mul(10) - 1.13_f32.powi(9)).abs() < 1e-4);
        let late = wave_hp_mul(50);
        let old_late = 1.13_f32.powi(49);
        assert!(late < 160.0, "wave 50 mul {late}");
        assert!(late > 90.0, "wave 50 mul {late}");
        assert!(
            late < old_late * 0.45,
            "late curve should be well under unbounded 13%"
        );
    }
}
