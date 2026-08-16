use crate::defs::{
    scale_turret, upgrade_cost_for, BuildKind, FireMode, StrikeKind, BUILD_CATALOG, STRIKE_CATALOG,
};
use crate::snapshot::{CatalogItem, StrikeItem};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackDoc {
    #[serde(default)]
    pub slug: String,
    #[serde(default = "default_pack_name")]
    pub name: String,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub guns: Vec<GunPatch>,
    #[serde(default)]
    pub strikes: Vec<StrikePatch>,
}

fn default_pack_name() -> String {
    "Custom loadout".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GunPatch {
    pub id: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blurb: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fire_interval: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub splash: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proj_speed: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hits_ground: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hits_air: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fire: Option<FireMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volley: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slow: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slow_ttl: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct StrikePatch {
    pub id: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blurb: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slow: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slow_ttl: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hits_ground: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hits_air: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackError {
    Parse(String),
    BadGun,
    BadStrike,
    DuplicateGun,
    DuplicateStrike,
    Range,
    EmptyTray,
}

impl PackError {
    pub fn message(&self) -> String {
        match self {
            Self::Parse(s) => format!("Bad JSON: {s}"),
            Self::BadGun => "Gun id must be 1–10 (barricade through siege rail)".into(),
            Self::BadStrike => "Strike id must be 1–3 (satchel, overload, orbital)".into(),
            Self::DuplicateGun => "Duplicate gun id in pack".into(),
            Self::DuplicateStrike => "Duplicate strike id in pack".into(),
            Self::Range => "A pack value is out of range".into(),
            Self::EmptyTray => "A pack must leave at least one buildable enabled".into(),
        }
    }
}

#[derive(Clone, Debug)]
struct GunMeta {
    name: String,
    role: String,
    blurb: String,
    enabled: bool,
}

#[derive(Clone, Debug)]
struct StrikeMeta {
    name: String,
    blurb: String,
    enabled: bool,
}

/// Turret stats by `BuildKind as usize`. `Copy`, so it can be mirrored cheaply.
pub type GunTable = [Option<crate::defs::TurretStats>; 11];

/// `Loadout::scaled` against a bare table. Same values, no borrow of the whole Loadout.
pub fn scaled_from(
    table: &GunTable,
    kind: BuildKind,
    tier: u8,
) -> Option<crate::defs::TurretStats> {
    Some(scale_turret(
        table.get(kind as usize).copied().flatten()?,
        tier,
    ))
}

#[derive(Clone, Debug)]
pub struct Loadout {
    guns: [Option<crate::defs::TurretStats>; 11],
    gun_meta: [Option<GunMeta>; 11],
    strikes: [Option<crate::defs::StrikeStats>; 4],
    strike_meta: [Option<StrikeMeta>; 4],
}

fn mix(h: u64, v: u64) -> u64 {
    let mut x = h ^ v.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 31)
}

fn mix_f(h: u64, v: f32) -> u64 {
    mix(h, v.to_bits() as u64)
}

fn mix_str(mut h: u64, s: &str) -> u64 {
    h = mix(h, s.len() as u64);
    for b in s.bytes() {
        h = mix(h, b as u64);
    }
    h
}

impl Loadout {
    pub fn stock() -> Self {
        let mut guns = [None; 11];
        let mut gun_meta = [
            None, None, None, None, None, None, None, None, None, None, None,
        ];
        for s in BUILD_CATALOG {
            let i = s.kind as usize;
            guns[i] = Some(*s);
            gun_meta[i] = Some(GunMeta {
                name: s.name.into(),
                role: s.role.into(),
                blurb: s.blurb.into(),
                enabled: true,
            });
        }
        let mut strikes = [None; 4];
        let mut strike_meta = [None, None, None, None];
        for s in STRIKE_CATALOG {
            let i = s.kind as usize;
            strikes[i] = Some(*s);
            strike_meta[i] = Some(StrikeMeta {
                name: s.name.into(),
                blurb: s.blurb.into(),
                enabled: true,
            });
        }
        Self {
            guns,
            gun_meta,
            strikes,
            strike_meta,
        }
    }

    pub fn from_doc(doc: &PackDoc) -> Result<Self, PackError> {
        let mut load = Self::stock();
        let mut seen_g = [false; 11];
        for patch in &doc.guns {
            let kind = BuildKind::from_u8(patch.id);
            if kind == BuildKind::Inspect || patch.id != kind as u8 {
                return Err(PackError::BadGun);
            }
            let i = kind as usize;
            if seen_g[i] {
                return Err(PackError::DuplicateGun);
            }
            seen_g[i] = true;
            apply_gun(&mut load, i, patch)?;
        }
        let mut seen_s = [false; 4];
        for patch in &doc.strikes {
            let kind = StrikeKind::from_u8(patch.id);
            if kind == StrikeKind::None || patch.id != kind as u8 {
                return Err(PackError::BadStrike);
            }
            let i = kind as usize;
            if seen_s[i] {
                return Err(PackError::DuplicateStrike);
            }
            seen_s[i] = true;
            apply_strike(&mut load, i, patch)?;
        }
        // A tray with nothing in it is an unplayable match, not a valid loadout.
        if load.catalog_items().is_empty() {
            return Err(PackError::EmptyTray);
        }
        Ok(load)
    }

    pub fn gun(&self, kind: BuildKind) -> Option<crate::defs::TurretStats> {
        let i = kind as usize;
        if !self
            .gun_meta
            .get(i)
            .and_then(|m| m.as_ref())
            .map(|m| m.enabled)
            .unwrap_or(false)
        {
            return None;
        }
        self.guns.get(i).copied().flatten()
    }

    pub fn gun_even_disabled(&self, kind: BuildKind) -> Option<crate::defs::TurretStats> {
        self.guns.get(kind as usize).copied().flatten()
    }

    /// The `Copy` half of the loadout. `tick_towers` reads only this, so mirroring it on
    /// `Game` avoids cloning the whole `String`-bearing Loadout every tick.
    pub fn guns_table(&self) -> GunTable {
        self.guns
    }

    pub fn gun_enabled(&self, kind: BuildKind) -> bool {
        self.gun_meta
            .get(kind as usize)
            .and_then(|m| m.as_ref())
            .map(|m| m.enabled)
            .unwrap_or(false)
    }

    pub fn gun_name(&self, kind: BuildKind) -> &str {
        self.gun_meta
            .get(kind as usize)
            .and_then(|m| m.as_ref())
            .map(|m| m.name.as_str())
            .unwrap_or("Unknown")
    }

    pub fn scaled(&self, kind: BuildKind, tier: u8) -> Option<crate::defs::TurretStats> {
        Some(scale_turret(self.gun_even_disabled(kind)?, tier))
    }

    pub fn upgrade_cost(&self, kind: BuildKind, tier: u8) -> Option<i32> {
        upgrade_cost_for(self.gun_even_disabled(kind)?.cost, tier)
    }

    pub fn strike(&self, kind: StrikeKind) -> Option<crate::defs::StrikeStats> {
        let i = kind as usize;
        if !self
            .strike_meta
            .get(i)
            .and_then(|m| m.as_ref())
            .map(|m| m.enabled)
            .unwrap_or(false)
        {
            return None;
        }
        self.strikes.get(i).copied().flatten()
    }

    pub fn fingerprint(&self) -> u64 {
        let mut h = 0x10AD_0000_u64;
        for i in 1..11 {
            let Some(s) = self.guns[i] else { continue };
            let meta = self.gun_meta[i].as_ref();
            let enabled = meta.map(|m| m.enabled).unwrap_or(true);
            h = mix(h, i as u64);
            h = mix(h, enabled as u64);
            h = mix_str(h, meta.map(|m| m.name.as_str()).unwrap_or(""));
            h = mix(h, s.cost as u64);
            h = mix_f(h, s.range);
            h = mix_f(h, s.fire_interval);
            h = mix_f(h, s.damage);
            h = mix_f(h, s.splash);
            h = mix_f(h, s.proj_speed);
            h = mix(h, s.hits_ground as u64);
            h = mix(h, s.hits_air as u64);
            h = mix(h, s.homing as u64);
            h = mix(
                h,
                match s.fire {
                    FireMode::Shell => 0,
                    FireMode::Cone => 1,
                    FireMode::Line => 2,
                    FireMode::Pulse => 3,
                    FireMode::Beam => 4,
                },
            );
            h = mix(h, s.volley as u64);
            h = mix_f(h, s.slow);
            h = mix_f(h, s.slow_ttl);
        }
        for i in 1..4 {
            let Some(s) = self.strikes[i] else { continue };
            let meta = self.strike_meta[i].as_ref();
            let enabled = meta.map(|m| m.enabled).unwrap_or(true);
            h = mix(h, 100 + i as u64);
            h = mix(h, enabled as u64);
            h = mix(h, s.cost as u64);
            h = mix_f(h, s.radius);
            h = mix_f(h, s.damage);
            h = mix_f(h, s.slow);
            h = mix_f(h, s.slow_ttl);
            h = mix_f(h, s.cooldown);
            h = mix(h, s.hits_ground as u64);
            h = mix(h, s.hits_air as u64);
        }
        h
    }

    pub fn is_stock(&self) -> bool {
        self.fingerprint() == Self::stock().fingerprint()
    }

    pub fn catalog_items(&self) -> Vec<CatalogItem> {
        BUILD_CATALOG
            .iter()
            .filter(|s| self.gun_enabled(s.kind))
            .filter_map(|s| {
                let stats = self.gun_even_disabled(s.kind)?;
                let meta = self.gun_meta[s.kind as usize].as_ref()?;
                Some(CatalogItem {
                    id: s.kind as u8,
                    hotkey: s.kind.hotkey().to_string(),
                    name: meta.name.clone(),
                    role: meta.role.clone(),
                    blurb: meta.blurb.clone(),
                    cost: stats.cost,
                    range: stats.range,
                    hits_ground: stats.hits_ground,
                    hits_air: stats.hits_air,
                    detects: s.kind.detects(),
                    fire: stats.fire,
                })
            })
            .collect()
    }

    pub fn strike_items(&self) -> Vec<StrikeItem> {
        STRIKE_CATALOG
            .iter()
            .filter(|s| {
                self.strike_meta
                    .get(s.kind as usize)
                    .and_then(|m| m.as_ref())
                    .map(|m| m.enabled)
                    .unwrap_or(false)
            })
            .filter_map(|s| {
                let stats = self.strikes[s.kind as usize]?;
                let meta = self.strike_meta[s.kind as usize].as_ref()?;
                Some(StrikeItem {
                    id: s.kind as u8,
                    hotkey: s.hotkey.to_string(),
                    name: meta.name.clone(),
                    blurb: meta.blurb.clone(),
                    cost: stats.cost,
                    radius: stats.radius,
                    cooldown: stats.cooldown,
                })
            })
            .collect()
    }

    pub fn strike_hud(&self) -> Vec<crate::snapshot::StrikeHud> {
        STRIKE_CATALOG
            .iter()
            .map(|s| crate::snapshot::StrikeHud {
                id: s.kind as u8,
                ready: false,
                cooldown: 0.0,
                cost: self
                    .strikes
                    .get(s.kind as usize)
                    .copied()
                    .flatten()
                    .map(|st| st.cost)
                    .unwrap_or(s.cost),
            })
            .collect()
    }

    pub fn to_doc(&self, name: &str, slug: &str, blurb: &str) -> PackDoc {
        PackDoc {
            slug: slug.into(),
            name: name.into(),
            blurb: blurb.into(),
            guns: BUILD_CATALOG
                .iter()
                .filter_map(|s| {
                    let stats = self.gun_even_disabled(s.kind)?;
                    let meta = self.gun_meta[s.kind as usize].as_ref()?;
                    Some(GunPatch {
                        id: s.kind as u8,
                        name: Some(meta.name.clone()),
                        role: Some(meta.role.clone()),
                        blurb: Some(meta.blurb.clone()),
                        enabled: Some(meta.enabled),
                        cost: Some(stats.cost),
                        range: Some(stats.range),
                        fire_interval: Some(stats.fire_interval),
                        damage: Some(stats.damage),
                        splash: Some(stats.splash),
                        proj_speed: Some(stats.proj_speed),
                        hits_ground: Some(stats.hits_ground),
                        hits_air: Some(stats.hits_air),
                        homing: Some(stats.homing),
                        fire: Some(stats.fire),
                        volley: Some(stats.volley),
                        slow: Some(stats.slow),
                        slow_ttl: Some(stats.slow_ttl),
                    })
                })
                .collect(),
            strikes: STRIKE_CATALOG
                .iter()
                .filter_map(|s| {
                    let stats = self.strikes[s.kind as usize]?;
                    let meta = self.strike_meta[s.kind as usize].as_ref()?;
                    Some(StrikePatch {
                        id: s.kind as u8,
                        name: Some(meta.name.clone()),
                        blurb: Some(meta.blurb.clone()),
                        enabled: Some(meta.enabled),
                        cost: Some(stats.cost),
                        radius: Some(stats.radius),
                        damage: Some(stats.damage),
                        slow: Some(stats.slow),
                        slow_ttl: Some(stats.slow_ttl),
                        cooldown: Some(stats.cooldown),
                        hits_ground: Some(stats.hits_ground),
                        hits_air: Some(stats.hits_air),
                    })
                })
                .collect(),
        }
    }
}

fn clamp_ok(ok: bool) -> Result<(), PackError> {
    if ok {
        Ok(())
    } else {
        Err(PackError::Range)
    }
}

fn apply_gun(load: &mut Loadout, i: usize, p: &GunPatch) -> Result<(), PackError> {
    let Some(stats) = load.guns[i].as_mut() else {
        return Err(PackError::BadGun);
    };
    if let Some(v) = p.cost {
        clamp_ok((0..=250_000).contains(&v))?;
        stats.cost = v;
    }
    if let Some(v) = p.range {
        clamp_ok((0.0..=40.0).contains(&v))?;
        stats.range = v;
    }
    if let Some(v) = p.fire_interval {
        clamp_ok((0.0..=12.0).contains(&v))?;
        stats.fire_interval = v;
    }
    if let Some(v) = p.damage {
        clamp_ok((0.0..=20_000.0).contains(&v))?;
        stats.damage = v;
    }
    if let Some(v) = p.splash {
        clamp_ok((0.0..=24.0).contains(&v))?;
        stats.splash = v;
    }
    if let Some(v) = p.proj_speed {
        clamp_ok((0.0..=80.0).contains(&v))?;
        stats.proj_speed = v;
    }
    if let Some(v) = p.hits_ground {
        stats.hits_ground = v;
    }
    if let Some(v) = p.hits_air {
        stats.hits_air = v;
    }
    if let Some(v) = p.homing {
        stats.homing = v;
    }
    if let Some(v) = p.fire {
        stats.fire = v;
    }
    if let Some(v) = p.volley {
        clamp_ok((0..=16).contains(&v))?;
        stats.volley = v;
    }
    if let Some(v) = p.slow {
        clamp_ok((0.0..=1.0).contains(&v))?;
        stats.slow = v;
    }
    if let Some(v) = p.slow_ttl {
        clamp_ok((0.0..=30.0).contains(&v))?;
        stats.slow_ttl = v;
    }
    let Some(meta) = load.gun_meta[i].as_mut() else {
        return Err(PackError::BadGun);
    };
    if let Some(v) = &p.name {
        clamp_ok((1..=32).contains(&v.len()))?;
        meta.name = v.clone();
    }
    if let Some(v) = &p.role {
        clamp_ok(v.len() <= 48)?;
        meta.role = v.clone();
    }
    if let Some(v) = &p.blurb {
        clamp_ok(v.len() <= 160)?;
        meta.blurb = v.clone();
    }
    if let Some(v) = p.enabled {
        meta.enabled = v;
    }
    Ok(())
}

fn apply_strike(load: &mut Loadout, i: usize, p: &StrikePatch) -> Result<(), PackError> {
    let Some(stats) = load.strikes[i].as_mut() else {
        return Err(PackError::BadStrike);
    };
    if let Some(v) = p.cost {
        clamp_ok((0..=250_000).contains(&v))?;
        stats.cost = v;
    }
    if let Some(v) = p.radius {
        clamp_ok((0.1..=20.0).contains(&v))?;
        stats.radius = v;
    }
    if let Some(v) = p.damage {
        clamp_ok((0.0..=20_000.0).contains(&v))?;
        stats.damage = v;
    }
    if let Some(v) = p.slow {
        clamp_ok((0.0..=1.0).contains(&v))?;
        stats.slow = v;
    }
    if let Some(v) = p.slow_ttl {
        clamp_ok((0.0..=30.0).contains(&v))?;
        stats.slow_ttl = v;
    }
    if let Some(v) = p.cooldown {
        clamp_ok((0.0..=60.0).contains(&v))?;
        stats.cooldown = v;
    }
    if let Some(v) = p.hits_ground {
        stats.hits_ground = v;
    }
    if let Some(v) = p.hits_air {
        stats.hits_air = v;
    }
    let Some(meta) = load.strike_meta[i].as_mut() else {
        return Err(PackError::BadStrike);
    };
    if let Some(v) = &p.name {
        clamp_ok((1..=32).contains(&v.len()))?;
        meta.name = v.clone();
    }
    if let Some(v) = &p.blurb {
        clamp_ok(v.len() <= 160)?;
        meta.blurb = v.clone();
    }
    if let Some(v) = p.enabled {
        meta.enabled = v;
    }
    Ok(())
}

pub fn stock_doc() -> PackDoc {
    Loadout::stock().to_doc("Stock", "stock", "Default frontier loadout.")
}

pub fn parse_pack_json(raw: &str) -> Result<PackDoc, PackError> {
    serde_json::from_str(raw).map_err(|e| PackError::Parse(e.to_string()))
}

pub fn parse_and_resolve(raw: &str) -> Result<(PackDoc, Loadout), PackError> {
    let doc = parse_pack_json(raw)?;
    let load = Loadout::from_doc(&doc)?;
    Ok((doc, load))
}

#[allow(dead_code)]
pub fn resolve_doc(doc: &PackDoc) -> Result<PackDoc, PackError> {
    let load = Loadout::from_doc(doc)?;
    Ok(load.to_doc(&doc.name, &doc.slug, &doc.blurb))
}

pub fn validate_pack_json(raw: &str) -> String {
    match parse_and_resolve(raw) {
        Ok((doc, load)) => serde_json::json!({
            "ok": true,
            "name": doc.name,
            "slug": doc.slug,
            "guns": load.catalog_items().len(),
            "strikes": load.strike_items().len(),
            "stock": load.is_stock(),
            "error": null
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "error": e.message()
        })
        .to_string(),
    }
}

pub fn resolve_pack_json(raw: &str) -> String {
    match parse_and_resolve(raw) {
        Ok((doc, load)) => serde_json::json!({
            "ok": true,
            "pack": load.to_doc(&doc.name, &doc.slug, &doc.blurb),
            "error": null
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "pack": null,
            "error": e.message()
        })
        .to_string(),
    }
}

pub fn stock_pack_json() -> String {
    serde_json::to_string(&stock_doc()).expect("stock pack")
}

fn scale_gun(id: u8, cost: f32, range: f32, damage: f32, interval: f32) -> GunPatch {
    let stock = BUILD_CATALOG
        .iter()
        .find(|s| s.kind as u8 == id)
        .expect("gun");
    GunPatch {
        id,
        cost: Some(((stock.cost as f32) * cost).round() as i32),
        range: Some(stock.range * range),
        damage: Some(stock.damage * damage),
        fire_interval: Some((stock.fire_interval * interval).max(0.05)),
        ..GunPatch {
            id,
            ..Default::default()
        }
    }
}

pub fn presets() -> Vec<PackDoc> {
    vec![
        PackDoc {
            slug: "stock".into(),
            name: "Stock".into(),
            blurb: "Default numbers. The yard you already know.".into(),
            guns: vec![],
            strikes: vec![],
        },
        PackDoc {
            slug: "glass".into(),
            name: "Glass Cannons".into(),
            blurb: "Cheap, short, mean. The walk has to be long or you fold.".into(),
            guns: (2..=10)
                .map(|id| scale_gun(id, 0.72, 0.82, 1.65, 0.92))
                .collect(),
            strikes: vec![],
        },
        PackDoc {
            slug: "fortress".into(),
            name: "Fortress".into(),
            blurb: "Walls are a vow. Guns reach, they do not rush.".into(),
            guns: {
                let mut g = vec![GunPatch {
                    id: 1,
                    cost: Some(14),
                    ..Default::default()
                }];
                g.extend((2..=10).map(|id| scale_gun(id, 1.15, 1.22, 0.84, 1.12)));
                g
            },
            strikes: vec![StrikePatch {
                id: 3,
                cost: Some(200),
                ..Default::default()
            }],
        },
        PackDoc {
            slug: "skywatch".into(),
            name: "Skywatch".into(),
            blurb: "The air is the war. Ground guns pay rent.".into(),
            guns: vec![
                scale_gun(3, 1.25, 0.92, 0.85, 1.0),
                scale_gun(4, 0.78, 1.18, 1.35, 0.9),
                scale_gun(5, 1.2, 1.0, 0.9, 1.0),
                scale_gun(8, 0.85, 1.12, 1.28, 0.92),
                scale_gun(9, 0.82, 1.1, 1.22, 0.9),
            ],
            strikes: vec![],
        },
        PackDoc {
            slug: "bargain".into(),
            name: "Bargain Bin".into(),
            blurb: "Everything is cheaper and thinner. Spend it twice.".into(),
            guns: (1..=10)
                .map(|id| scale_gun(id, 0.55, 1.0, 0.68, 1.0))
                .collect(),
            strikes: vec![
                StrikePatch {
                    id: 1,
                    cost: Some(45),
                    damage: Some(70.0),
                    ..Default::default()
                },
                StrikePatch {
                    id: 2,
                    cost: Some(70),
                    ..Default::default()
                },
                StrikePatch {
                    id: 3,
                    cost: Some(160),
                    damage: Some(180.0),
                    ..Default::default()
                },
            ],
        },
    ]
}

pub fn presets_json() -> String {
    serde_json::to_string(&presets()).expect("presets")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_roundtrip_is_identity() {
        let load = Loadout::from_doc(&stock_doc()).unwrap();
        assert!(load.is_stock());
        assert_eq!(
            load.gun(BuildKind::Autocannon).unwrap().cost,
            crate::defs::stats_for(BuildKind::Autocannon).unwrap().cost
        );
    }

    #[test]
    fn glass_is_cheaper_and_meaner() {
        let doc = presets().into_iter().find(|p| p.slug == "glass").unwrap();
        let load = Loadout::from_doc(&doc).unwrap();
        let ac = load.gun(BuildKind::Autocannon).unwrap();
        let stock = crate::defs::stats_for(BuildKind::Autocannon).unwrap();
        assert!(ac.cost < stock.cost);
        assert!(ac.damage > stock.damage);
        assert!(ac.range < stock.range);
        assert_ne!(load.fingerprint(), Loadout::stock().fingerprint());
    }

    #[test]
    fn bad_id_rejected() {
        let doc = PackDoc {
            slug: "x".into(),
            name: "x".into(),
            blurb: String::new(),
            guns: vec![GunPatch {
                id: 99,
                cost: Some(1),
                ..Default::default()
            }],
            strikes: vec![],
        };
        assert!(matches!(Loadout::from_doc(&doc), Err(PackError::BadGun)));
    }

    #[test]
    fn disable_drops_from_catalog() {
        let doc = PackDoc {
            slug: "no-ac".into(),
            name: "No AC".into(),
            blurb: String::new(),
            guns: vec![GunPatch {
                id: 2,
                enabled: Some(false),
                ..Default::default()
            }],
            strikes: vec![],
        };
        let load = Loadout::from_doc(&doc).unwrap();
        assert!(load.gun(BuildKind::Autocannon).is_none());
        assert!(!load.catalog_items().iter().any(|c| c.id == 2));
    }

    #[test]
    fn empty_tray_is_rejected() {
        let guns = (1u8..=10)
            .map(|id| GunPatch {
                id,
                enabled: Some(false),
                ..Default::default()
            })
            .collect();
        let doc = PackDoc {
            slug: "none".into(),
            name: "None".into(),
            blurb: String::new(),
            guns,
            strikes: vec![],
        };
        assert!(matches!(Loadout::from_doc(&doc), Err(PackError::EmptyTray)));
    }
}
