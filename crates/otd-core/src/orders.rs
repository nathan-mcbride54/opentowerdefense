use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub tick: u64,
    pub op: OrderOp,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum OrderOp {
    SetBuild { kind: u8 },
    SetStrike { kind: u8 },
    Click { x: i32, y: i32 },
    Cancel,
    Upgrade,
    Sell,
    Call,
    Target,
    Convert,
    Repair,
    Lift,
    Overcharge,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunOutcome {
    pub wave: u32,
    pub integrity: i32,
    pub kills: u32,
    pub leaks: u32,
    pub defeated: bool,
    pub ticks: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFile {
    pub version: u8,
    #[serde(default)]
    pub map_id: Option<u8>,
    #[serde(default)]
    pub map: Option<crate::mapdoc::MapDoc>,
    pub modifier_id: u8,
    pub seed: u64,
    #[serde(default)]
    pub orders: Vec<Order>,
    #[serde(default)]
    pub pack: Option<crate::pack::PackDoc>,
    #[serde(default)]
    pub outcome: Option<RunOutcome>,
    #[serde(default)]
    pub hash: Option<String>,
}

fn mix(h: u64, v: u64) -> u64 {
    let mut x = h ^ v.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 31)
}

fn op_code(op: &OrderOp) -> u64 {
    match op {
        OrderOp::SetBuild { kind } => 1u64 << 8 | *kind as u64,
        OrderOp::SetStrike { kind } => 2u64 << 8 | *kind as u64,
        OrderOp::Click { x, y } => 3u64 << 32 | (*x as u64 & 0xFFFF) << 16 | (*y as u64 & 0xFFFF),
        OrderOp::Cancel => 4,
        OrderOp::Upgrade => 5,
        OrderOp::Sell => 6,
        OrderOp::Call => 7,
        OrderOp::Target => 8,
        OrderOp::Convert => 9,
        OrderOp::Repair => 10,
        OrderOp::Lift => 11,
        OrderOp::Overcharge => 12,
    }
}

/// Stable fingerprint of seed + map + modifier + orders. Outcome and hash fields are ignored.
pub fn replay_hash(file: &ReplayFile) -> String {
    let mut h = 0x0D7_CA11_u64;
    h = mix(h, file.version as u64);
    h = mix(h, file.seed);
    h = mix(h, file.modifier_id as u64);
    h = mix(h, file.map_id.unwrap_or(255) as u64);
    if let Some(m) = &file.map {
        h = mix(h, m.w as u64);
        h = mix(h, m.h as u64);
        h = mix(h, m.seed);
        h = mix(h, m.cores.len() as u64);
        h = mix(h, m.spawns.len() as u64);
        h = mix(h, m.rocks.len() as u64);
        // Cell positions, not just counts: otherwise any two workshop maps with the same
        // dimensions and the same number of rocks fingerprint identically.
        for c in m.cores.iter().chain(&m.spawns).chain(&m.rocks) {
            h = mix(h, c[0] as u32 as u64);
            h = mix(h, c[1] as u32 as u64);
        }
    }
    h = mix(h, file.orders.len() as u64);
    for o in &file.orders {
        h = mix(h, o.tick);
        h = mix(h, op_code(&o.op));
    }
    if let Some(p) = &file.pack {
        match crate::pack::Loadout::from_doc(p) {
            Ok(load) => h = mix(h, load.fingerprint()),
            Err(_) => h = mix(h, 0xBAD_FACE),
        }
    }
    format!("{h:016x}")
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReport {
    pub ok: bool,
    pub hash: String,
    pub hash_ok: bool,
    pub outcome_ok: bool,
    pub error: Option<String>,
    pub wave: u32,
    pub integrity: i32,
    pub kills: u32,
    pub leaks: u32,
    pub defeated: bool,
    pub ticks: u64,
}

pub fn verify_replay(file: ReplayFile) -> VerifyReport {
    let claimed_hash = file.hash.clone();
    let claimed_outcome = file.outcome.clone();
    let last_order_tick = file.orders.last().map(|o| o.tick);
    let hash = replay_hash(&file);
    let hash_ok = claimed_hash.as_ref().map(|h| h == &hash).unwrap_or(true);
    let tick_cap = Some(
        claimed_outcome
            .as_ref()
            .map(|o| o.ticks)
            .or(last_order_tick)
            .unwrap_or(0),
    );
    match crate::sim::Game::from_replay(file) {
        Ok(mut game) => {
            game.run_recorded(None, tick_cap);
            let snap = game.snapshot();
            let outcome_ok = claimed_outcome
                .as_ref()
                .map(|o| {
                    o.wave == snap.wave
                        && o.integrity == snap.integrity
                        && o.kills == snap.kills
                        && o.leaks == snap.leaks
                        && o.defeated == snap.defeated
                })
                .unwrap_or(true);
            VerifyReport {
                ok: hash_ok && outcome_ok,
                hash,
                hash_ok,
                outcome_ok,
                error: None,
                wave: snap.wave,
                integrity: snap.integrity,
                kills: snap.kills,
                leaks: snap.leaks,
                defeated: snap.defeated,
                ticks: snap.tick,
            }
        }
        Err(e) => VerifyReport {
            ok: false,
            hash,
            hash_ok,
            outcome_ok: false,
            error: Some(e.message()),
            wave: 0,
            integrity: 0,
            kills: 0,
            leaks: 0,
            defeated: false,
            ticks: 0,
        },
    }
}

pub fn verify_replay_json(raw: &str) -> String {
    match serde_json::from_str::<ReplayFile>(raw) {
        Ok(file) => serde_json::to_string(&verify_replay(file)).expect("verify json"),
        Err(e) => serde_json::json!({
            "ok": false,
            "hash": "",
            "hashOk": false,
            "outcomeOk": false,
            "error": format!("Bad replay JSON: {e}"),
            "wave": 0,
            "integrity": 0,
            "kills": 0,
            "leaks": 0,
            "defeated": false,
            "ticks": 0
        })
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_sensitive() {
        let a = ReplayFile {
            version: 1,
            map_id: Some(0),
            map: None,
            modifier_id: 0,
            seed: 99,
            orders: vec![Order {
                tick: 0,
                op: OrderOp::SetBuild { kind: 2 },
            }],
            pack: None,
            outcome: None,
            hash: None,
        };
        let mut b = a.clone();
        assert_eq!(replay_hash(&a), replay_hash(&b));
        b.seed = 100;
        assert_ne!(replay_hash(&a), replay_hash(&b));
        let mut c = a.clone();
        c.orders[0].op = OrderOp::SetBuild { kind: 3 };
        assert_ne!(replay_hash(&a), replay_hash(&c));
    }
}
