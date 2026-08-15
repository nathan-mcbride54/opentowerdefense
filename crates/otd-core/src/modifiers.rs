use crate::defs::STARTING_CREDITS;
use crate::maps::theaters;
use crate::rng::Rng;
use serde::Serialize;

pub const FIXED_SCRAP: i32 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Modifier {
    Standard = 0,
    GroundOnly = 1,
    Accelerated = 2,
    RichBounties = 3,
    FixedScrap = 4,
    Cap10 = 5,
    Cap20 = 6,
}

impl Modifier {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::GroundOnly,
            2 => Self::Accelerated,
            3 => Self::RichBounties,
            4 => Self::FixedScrap,
            5 => Self::Cap10,
            6 => Self::Cap20,
            _ => Self::Standard,
        }
    }

    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::GroundOnly => "ground",
            Self::Accelerated => "accelerated",
            Self::RichBounties => "rich",
            Self::FixedScrap => "fixed",
            Self::Cap10 => "cap10",
            Self::Cap20 => "cap20",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::GroundOnly => "Ground only",
            Self::Accelerated => "Accelerated",
            Self::RichBounties => "Rich bounties",
            Self::FixedScrap => "Fixed scrap",
            Self::Cap10 => "Ten guns",
            Self::Cap20 => "Twenty guns",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Self::Standard => "Default rules. Invent the maze, cover the sky. Unspent scrap pays interest.",
            Self::GroundOnly => "No air. The walk is the whole war.",
            Self::Accelerated => "Thin hulls, long gait. They close before barrels settle.",
            Self::RichBounties => "Kills pay extra. Leaks still cost the relay.",
            Self::FixedScrap => "Ten thousand scrap. Kills pay nothing. No interest.",
            Self::Cap10 => "At most ten turrets. Barricades ignore the ceiling.",
            Self::Cap20 => "Twenty turret ceiling. Still a ceiling.",
        }
    }

    pub fn hazard(self) -> &'static str {
        match self {
            Self::Standard => "No extra hazard.",
            Self::GroundOnly => "Skystinger and Helios-air have nothing to hunt.",
            Self::Accelerated => "Dwell beams and slow matter more than raw DPS.",
            Self::RichBounties => "The economy is drunk. Don't get greedy and leak.",
            Self::FixedScrap => "Every placement is a vow. Sell-back is the only refund.",
            Self::Cap10 => "Quality over spray. Apex a few, wall the rest.",
            Self::Cap20 => "Enough guns to get sloppy. The cap still bites late.",
        }
    }

    pub fn opening_banner(self) -> &'static str {
        match self {
            Self::Standard => "FORTIFY THE RELAY",
            Self::GroundOnly => "GROUND ONLY — NO AIR",
            Self::Accelerated => "ACCELERATED INGRESS",
            Self::RichBounties => "RICH BOUNTIES",
            Self::FixedScrap => "FIXED SCRAP — KILLS PAY NOTHING",
            Self::Cap10 => "TEN GUNS",
            Self::Cap20 => "TWENTY GUNS",
        }
    }

    pub fn rules(self) -> MatchRules {
        match self {
            Self::Standard => MatchRules::standard(),
            Self::GroundOnly => MatchRules {
                ground_only: true,
                ..MatchRules::standard()
            },
            Self::Accelerated => MatchRules {
                hp_mul: 0.55,
                speed_mul: 1.85,
                ..MatchRules::standard()
            },
            Self::RichBounties => MatchRules {
                bounty_mul: 2.25,
                ..MatchRules::standard()
            },
            Self::FixedScrap => MatchRules {
                kill_income: false,
                bounty_mul: 0.0,
                starting_credits: FIXED_SCRAP,
                interest_bps: 0,
                interest_cap: 0,
                ..MatchRules::standard()
            },
            Self::Cap10 => MatchRules {
                turret_cap: Some(10),
                ..MatchRules::standard()
            },
            Self::Cap20 => MatchRules {
                turret_cap: Some(20),
                ..MatchRules::standard()
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MatchRules {
    pub ground_only: bool,
    pub hp_mul: f32,
    pub speed_mul: f32,
    pub bounty_mul: f32,
    pub kill_income: bool,
    pub turret_cap: Option<u32>,
    pub starting_credits: i32,
    pub interest_bps: u32,
    pub interest_cap: i32,
}

impl MatchRules {
    pub fn standard() -> Self {
        Self {
            ground_only: false,
            hp_mul: 1.0,
            speed_mul: 1.0,
            bounty_mul: 1.0,
            kill_income: true,
            turret_cap: None,
            starting_credits: STARTING_CREDITS,
            interest_bps: crate::defs::INTEREST_BPS,
            interest_cap: crate::defs::INTEREST_CAP,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierInfo {
    pub id: u8,
    pub slug: String,
    pub name: String,
    pub blurb: String,
    pub hazard: String,
}

pub fn modifiers() -> Vec<ModifierInfo> {
    (0..=6)
        .map(|id| {
            let m = Modifier::from_u8(id);
            ModifierInfo {
                id: m.id(),
                slug: m.slug().into(),
                name: m.name().into(),
                blurb: m.blurb().into(),
                hazard: m.hazard().into(),
            }
        })
        .collect()
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPick {
    pub utc_day: u32,
    pub map_id: u8,
    pub modifier_id: u8,
    pub map_name: String,
    pub map_hazard: String,
    pub modifier_name: String,
    pub modifier_hazard: String,
    pub seed: u64,
}

/// Deterministic theater + modifier for a UTC day number (`unix_ms / 86400000`).
pub fn daily_pick(utc_day: u32) -> DailyPick {
    let mut rng = Rng::new(0xD411u64.wrapping_mul((utc_day as u64) ^ 0x0D7D));
    let list = theaters();
    let map = &list[rng.range_i32(0, list.len() as i32) as usize];
    let modifier = if rng.range_i32(0, 5) == 0 {
        Modifier::Standard
    } else {
        Modifier::from_u8(rng.range_i32(1, 7) as u8)
    };
    let base = crate::maps::theater_by_id(map.id)
        .map(|(_, _, seed)| seed)
        .unwrap_or(0xA11CE5);
    let seed = base ^ (utc_day as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    DailyPick {
        utc_day,
        map_id: map.id,
        modifier_id: modifier.id(),
        map_name: map.name.clone(),
        map_hazard: map.hazard.clone(),
        modifier_name: modifier.name().into(),
        modifier_hazard: modifier.hazard().into(),
        seed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_is_stable() {
        let a = daily_pick(20_000);
        let b = daily_pick(20_000);
        assert_eq!(a.map_id, b.map_id);
        assert_eq!(a.modifier_id, b.modifier_id);
        assert_eq!(a.seed, b.seed);
        assert_ne!(daily_pick(20_001).seed, a.seed);
    }

    #[test]
    fn catalog_covers_enum() {
        assert_eq!(modifiers().len(), 7);
        assert_eq!(Modifier::from_u8(99), Modifier::Standard);
    }
}
