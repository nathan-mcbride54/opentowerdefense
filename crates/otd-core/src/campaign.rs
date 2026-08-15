use crate::maps::theaters;
use crate::modifiers::{modifiers, Modifier};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mission {
    pub id: u8,
    pub slug: String,
    pub name: String,
    pub briefing: String,
    pub objective: String,
    pub hold_until_wave: u32,
    pub map_id: u8,
    pub map_name: String,
    pub modifier_id: u8,
    pub modifier_name: String,
    pub hazard: String,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: u8,
    pub slug: String,
    pub name: String,
    pub blurb: String,
    pub map_id: u8,
    pub map_name: String,
    pub modifier_id: u8,
    pub modifier_name: String,
    pub seed: u64,
    pub hold_until_wave: Option<u32>,
}

fn theater_name(id: u8) -> String {
    theaters()
        .into_iter()
        .find(|t| t.id == id)
        .map(|t| t.name)
        .unwrap_or_else(|| "Unknown".into())
}

fn mod_name(id: u8) -> String {
    Modifier::from_u8(id).name().into()
}

fn hazard(map_id: u8, modifier_id: u8) -> String {
    let map = theaters()
        .into_iter()
        .find(|t| t.id == map_id)
        .map(|t| t.hazard)
        .unwrap_or_default();
    let m = modifiers()
        .into_iter()
        .find(|m| m.id == modifier_id)
        .map(|m| m.hazard)
        .unwrap_or_default();
    format!("{map} {m}").trim().to_string()
}

pub fn campaign() -> Vec<Mission> {
    let spec = [
        (
            0u8,
            "open-ground",
            "Open Ground",
            "Wide scrub. Two ingresses. Build a kill corridor before the sky learns your name.",
            "Hold through wave 8",
            8u32,
            0u8,
            0u8,
            0xC010_0001u64,
        ),
        (
            1,
            "yard-doors",
            "Yard Doors",
            "The courtyard has two doors and no Wasps. The walk is the whole war.",
            "Hold through wave 10",
            10,
            1,
            1,
            0xC010_0002,
        ),
        (
            2,
            "pinch",
            "The Pinch",
            "Dust Cut already pinched the field. You get ten guns. Make them count.",
            "Hold through wave 10",
            10,
            2,
            5,
            0xC010_0003,
        ),
        (
            3,
            "split-attention",
            "Split Attention",
            "A spine splits the walk. They are thin and they are fast. Cover both corridors.",
            "Hold through wave 12",
            12,
            3,
            2,
            0xC010_0004,
        ),
        (
            4,
            "pocket",
            "Pocket Change",
            "Tight yard. Ten thousand scrap. Kills pay nothing. Every turret is a vow.",
            "Hold through wave 10",
            10,
            4,
            4,
            0xC010_0005,
        ),
        (
            5,
            "twin-watch",
            "Twin Watch",
            "Two relays, one integrity pool. Twenty guns. Air still picks the nearest core.",
            "Hold through wave 12",
            12,
            5,
            6,
            0xC010_0006,
        ),
        (
            6,
            "three-gates",
            "Three Gates",
            "North, west, east. Shades walk past guns that cannot see them. Pulse, Arc, and Helios can.",
            "Hold through wave 10",
            10,
            6,
            0,
            0xC010_0007,
        ),
        (
            7,
            "the-bend",
            "The Bend",
            "The oxbow already folded the walk. Seal the banks and the cup is the killbox. The Colossus will silence guns that stand too close.",
            "Hold through wave 12",
            12,
            7,
            0,
            0xC010_0008,
        ),
    ];
    spec.into_iter()
        .map(
            |(id, slug, name, briefing, objective, hold, map_id, modifier_id, seed)| Mission {
                id,
                slug: slug.into(),
                name: name.into(),
                briefing: briefing.into(),
                objective: objective.into(),
                hold_until_wave: hold,
                map_id,
                map_name: theater_name(map_id),
                modifier_id,
                modifier_name: mod_name(modifier_id),
                hazard: hazard(map_id, modifier_id),
                seed,
            },
        )
        .collect()
}

pub fn mission_by_id(id: u8) -> Option<Mission> {
    campaign().into_iter().find(|m| m.id == id)
}

pub fn challenges() -> Vec<Challenge> {
    let spec = [
        (
            0u8,
            "dry-season",
            "Dry Season",
            "Kilo, standard rules, a known seed. Same walk if you issue the same orders.",
            0u8,
            0u8,
            0xD00D_5EED_u64,
            Some(10u32),
        ),
        (
            1,
            "night-watch",
            "Night Watch",
            "Twin Cores, accelerated. The sky will split. Hold ten.",
            5,
            2,
            0xA11_0B17,
            Some(10),
        ),
        (
            2,
            "iron-budget",
            "Iron Budget",
            "Enclave, fixed scrap. You do not get a second economy.",
            4,
            4,
            0xC0DE_B10C,
            Some(8),
        ),
        (
            3,
            "three-doors",
            "Three Doors",
            "Tri-Gate, ten guns. Three walks, one budget.",
            6u8,
            5u8,
            0x7116_A7E1,
            Some(10),
        ),
        (
            4,
            "blind-spot",
            "Blind Spot",
            "Dust Cut, standard. Shades and Flickers in a pinched walk. Hold ten.",
            2u8,
            0u8,
            0x5A1D_E0u64,
            Some(10),
        ),
        (
            5,
            "the-fold",
            "The Fold",
            "Oxbow, ten guns. Seal the banks or they go around. Hold twelve.",
            7u8,
            5u8,
            0x0B50_12u64,
            Some(12),
        ),
    ];
    spec.into_iter()
        .map(|(id, slug, name, blurb, map_id, modifier_id, seed, hold)| Challenge {
            id,
            slug: slug.into(),
            name: name.into(),
            blurb: blurb.into(),
            map_id,
            map_name: theater_name(map_id),
            modifier_id,
            modifier_name: mod_name(modifier_id),
            seed,
            hold_until_wave: hold,
        })
        .collect()
}

pub fn challenge_by_id(id: u8) -> Option<Challenge> {
    challenges().into_iter().find(|c| c.id == id)
}

pub fn campaign_json() -> String {
    serde_json::to_string(&campaign()).expect("campaign json")
}

pub fn challenges_json() -> String {
    serde_json::to_string(&challenges()).expect("challenges json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maps::theater_by_id;
    use crate::path::FlowField;

    #[test]
    fn every_mission_map_opens() {
        for m in campaign() {
            let (grid, _, _) = theater_by_id(m.map_id).expect("map");
            assert!(FlowField::compute(&grid).spawns_reachable(&grid), "{}", m.name);
        }
        for c in challenges() {
            let (grid, _, _) = theater_by_id(c.map_id).expect("challenge map");
            assert!(
                FlowField::compute(&grid).spawns_reachable(&grid),
                "{}",
                c.name
            );
        }
        assert_eq!(campaign().len(), 8);
        assert_eq!(challenges().len(), 6);
    }
}
