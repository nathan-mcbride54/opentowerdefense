use otd_core::{
    modifiers, theater_by_slug, theaters, verify_replay, Game, Modifier, ReplayFile,
    WORKSHOP_MAP_ID,
};
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!(
            "otd-bench — headless Open Tower Defense\n\n\
             Usage:\n  otd-bench --validate FILE.json\n  otd-bench --validate-pack FILE.json\n  otd-bench --verify FILE.json\n  otd-bench --map kilo [--mod standard] [--pack FILE.json] [--until-wave 20]\n  otd-bench --mission 0 [--until-wave N]\n  otd-bench --challenge 0\n  otd-bench --map-json FILE.json [--orders FILE.json] [--until-wave N] [--until-tick N]\n"
        );
        return ExitCode::SUCCESS;
    }

    let mut map_slug: Option<String> = None;
    let mut map_json: Option<String> = None;
    let mut orders_path: Option<String> = None;
    let mut validate_path: Option<String> = None;
    let mut verify_path: Option<String> = None;
    let mut pack_path: Option<String> = None;
    let mut validate_pack_path: Option<String> = None;
    let mut mission_id: Option<u8> = None;
    let mut challenge_id: Option<u8> = None;
    let mut mod_slug = "standard".to_string();
    let mut until_wave: Option<u32> = None;
    let mut until_tick: Option<u64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--map" => {
                i += 1;
                map_slug = args.get(i).cloned();
            }
            "--map-json" => {
                i += 1;
                map_json = args.get(i).cloned();
            }
            "--orders" => {
                i += 1;
                orders_path = args.get(i).cloned();
            }
            "--validate" => {
                i += 1;
                validate_path = args.get(i).cloned();
            }
            "--verify" => {
                i += 1;
                verify_path = args.get(i).cloned();
            }
            "--pack" => {
                i += 1;
                pack_path = args.get(i).cloned();
            }
            "--validate-pack" => {
                i += 1;
                validate_pack_path = args.get(i).cloned();
            }
            "--mission" => {
                i += 1;
                mission_id = args.get(i).and_then(|s| s.parse().ok());
            }
            "--challenge" => {
                i += 1;
                challenge_id = args.get(i).and_then(|s| s.parse().ok());
            }
            "--mod" => {
                i += 1;
                mod_slug = args.get(i).cloned().unwrap_or(mod_slug);
            }
            "--until-wave" => {
                i += 1;
                until_wave = args.get(i).and_then(|s| s.parse().ok());
            }
            "--until-tick" => {
                i += 1;
                until_tick = args.get(i).and_then(|s| s.parse().ok());
            }
            other => {
                eprintln!("Unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    if let Some(path) = validate_path {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::from(1);
            }
        };
        match otd_core::parse_and_validate(&raw) {
            Ok((doc, _)) => {
                println!(
                    "ok  {}  {}x{}  cores={} spawns={} rocks={}",
                    doc.name,
                    doc.w,
                    doc.h,
                    doc.cores.len(),
                    doc.spawns.len(),
                    doc.rocks.len()
                );
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("{}", e.message());
                return ExitCode::from(1);
            }
        }
    }

    if let Some(path) = validate_pack_path {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::from(1);
            }
        };
        match otd_core::parse_and_resolve(&raw) {
            Ok((doc, load)) => {
                println!(
                    "ok  {}  guns={} strikes={} {}",
                    doc.name,
                    load.catalog_items().len(),
                    load.strike_items().len(),
                    if load.is_stock() { "STOCK" } else { "CUSTOM" }
                );
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("{}", e.message());
                return ExitCode::from(1);
            }
        }
    }

    if let Some(path) = verify_path {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::from(1);
            }
        };
        let file: ReplayFile = match serde_json::from_str(&raw) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("replay JSON: {e}");
                return ExitCode::from(1);
            }
        };
        let report = verify_replay(file);
        println!(
            "ok={} hashOk={} outcomeOk={} hash={} wave={} ticks={} kills={} leaks={} integrity={} {}{}",
            report.ok,
            report.hash_ok,
            report.outcome_ok,
            report.hash,
            report.wave,
            report.ticks,
            report.kills,
            report.leaks,
            report.integrity,
            if report.defeated { "DEFEAT" } else { "LIVE" },
            report
                .error
                .as_ref()
                .map(|e| format!("  err={e}"))
                .unwrap_or_default()
        );
        return if report.ok {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }

    let modifier = parse_mod(&mod_slug);
    let mut game = if let Some(id) = mission_id {
        match Game::mission(id) {
            Some(g) => g,
            None => {
                eprintln!("unknown mission {id}");
                return ExitCode::from(1);
            }
        }
    } else if let Some(id) = challenge_id {
        match Game::challenge(id) {
            Some(g) => g,
            None => {
                eprintln!("unknown challenge {id}");
                return ExitCode::from(1);
            }
        }
    } else if let Some(path) = map_json {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::from(1);
            }
        };
        match Game::from_map_json(&raw, modifier) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("{}", e.message());
                return ExitCode::from(1);
            }
        }
    } else {
        let id = parse_map(map_slug.as_deref().unwrap_or("kilo"));
        Game::start(id, modifier, None)
    };

    if let Some(path) = orders_path {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::from(1);
            }
        };
        let file: ReplayFile = match serde_json::from_str(&raw) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("orders JSON: {e}");
                return ExitCode::from(1);
            }
        };
        game = match Game::from_replay(file) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("{}", e.message());
                return ExitCode::from(1);
            }
        };
    }

    if let Some(path) = pack_path {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::from(1);
            }
        };
        if let Err(e) = game.apply_pack_json(&raw) {
            eprintln!("{}", e.message());
            return ExitCode::from(1);
        }
    }

    let tick_cap = until_tick.or(Some(if until_wave.is_some() {
        400_000
    } else {
        90_000
    }));
    game.run_recorded(until_wave, tick_cap);
    let snap = game.snapshot();
    println!(
        "map={} id={} mod={} wave={} ticks={} kills={} leaks={} integrity={} pack={} hash={} {}",
        snap.mission_name.as_deref().unwrap_or(&snap.map_name),
        if snap.map_id == WORKSHOP_MAP_ID {
            "workshop".into()
        } else {
            snap.map_id.to_string()
        },
        snap.modifier_name,
        snap.wave,
        snap.tick,
        snap.kills,
        snap.leaks,
        snap.integrity,
        snap.pack_name.as_deref().unwrap_or("-"),
        snap.seed_hex,
        if snap.defeated {
            "DEFEAT"
        } else if snap.objective_cleared {
            "HELD"
        } else {
            "LIVE"
        }
    );
    ExitCode::SUCCESS
}

fn parse_map(s: &str) -> u8 {
    if let Ok(id) = s.parse::<u8>() {
        return id;
    }
    theater_by_slug(s)
        .or_else(|| {
            theaters()
                .into_iter()
                .find(|t| t.name.eq_ignore_ascii_case(s))
                .map(|t| t.id)
        })
        .unwrap_or(0)
}

fn parse_mod(s: &str) -> Modifier {
    if let Ok(id) = s.parse::<u8>() {
        return Modifier::from_u8(id);
    }
    modifiers()
        .into_iter()
        .find(|m| m.slug.eq_ignore_ascii_case(s) || m.name.eq_ignore_ascii_case(s))
        .map(|m| Modifier::from_u8(m.id))
        .unwrap_or(Modifier::Standard)
}
