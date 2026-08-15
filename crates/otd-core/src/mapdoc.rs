use crate::grid::{Grid, Terrain};
use crate::path::FlowField;
use serde::{Deserialize, Serialize};

pub const WORKSHOP_MAP_ID: u8 = 255;
pub const MIN_MAP: i32 = 8;
pub const MAX_MAP_W: i32 = 64;
pub const MAX_MAP_H: i32 = 48;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapDoc {
    #[serde(default)]
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub hazard: String,
    pub w: i32,
    pub h: i32,
    #[serde(default = "default_seed")]
    pub seed: u64,
    pub cores: Vec<[i32; 2]>,
    pub spawns: Vec<[i32; 2]>,
    #[serde(default)]
    pub rocks: Vec<[i32; 2]>,
}

fn default_seed() -> u64 {
    0xA11CE5
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapError {
    Parse(String),
    Size,
    NoCore,
    NoSpawn,
    OutOfBounds,
    Overlap,
    Unreachable,
}

impl MapError {
    pub fn message(&self) -> String {
        match self {
            Self::Parse(s) => format!("Bad JSON: {s}"),
            Self::Size => "Map must be 8–64 wide and 8–48 tall".into(),
            Self::NoCore => "Need at least one relay (core) cell".into(),
            Self::NoSpawn => "Need at least one ingress (spawn) cell".into(),
            Self::OutOfBounds => "A marked cell is off the grid".into(),
            Self::Overlap => "Core, spawn, and rock cannot share a cell".into(),
            Self::Unreachable => "A spawn cannot walk to a relay — punch a path".into(),
        }
    }
}

pub fn parse_map_json(raw: &str) -> Result<MapDoc, MapError> {
    serde_json::from_str(raw).map_err(|e| MapError::Parse(e.to_string()))
}

pub fn validate_map(doc: &MapDoc) -> Result<Grid, MapError> {
    if doc.w < MIN_MAP || doc.h < MIN_MAP || doc.w > MAX_MAP_W || doc.h > MAX_MAP_H {
        return Err(MapError::Size);
    }
    if doc.cores.is_empty() {
        return Err(MapError::NoCore);
    }
    if doc.spawns.is_empty() {
        return Err(MapError::NoSpawn);
    }

    let mut g = Grid::new(doc.w, doc.h);
    let mut seen = std::collections::HashSet::new();

    let stamp = |g: &mut Grid,
                 seen: &mut std::collections::HashSet<(i32, i32)>,
                 cells: &[[i32; 2]],
                 t: Terrain|
     -> Result<(), MapError> {
        for [x, y] in cells {
            if *x < 0 || *y < 0 || *x >= doc.w || *y >= doc.h {
                return Err(MapError::OutOfBounds);
            }
            if !seen.insert((*x, *y)) {
                return Err(MapError::Overlap);
            }
            g.set_terrain(*x, *y, t);
        }
        Ok(())
    };

    stamp(&mut g, &mut seen, &doc.cores, Terrain::Core)?;
    stamp(&mut g, &mut seen, &doc.spawns, Terrain::Spawn)?;
    stamp(&mut g, &mut seen, &doc.rocks, Terrain::Rock)?;

    let flow = FlowField::compute(&g);
    if !flow.spawns_reachable(&g) {
        return Err(MapError::Unreachable);
    }
    Ok(g)
}

pub fn parse_and_validate(raw: &str) -> Result<(MapDoc, Grid), MapError> {
    let doc = parse_map_json(raw)?;
    let grid = validate_map(&doc)?;
    Ok((doc, grid))
}

pub fn grid_to_doc(grid: &Grid, name: &str, slug: &str, seed: u64) -> MapDoc {
    MapDoc {
        slug: slug.into(),
        name: name.into(),
        blurb: String::new(),
        hazard: String::new(),
        w: grid.w,
        h: grid.h,
        seed,
        cores: grid.cores().into_iter().map(|(x, y)| [x, y]).collect(),
        spawns: grid.spawns().into_iter().map(|(x, y)| [x, y]).collect(),
        rocks: grid.rocks().into_iter().map(|(x, y)| [x, y]).collect(),
    }
}

pub fn theater_to_doc(id: u8) -> Option<MapDoc> {
    let (grid, name, seed) = crate::maps::theater_by_id(id)?;
    let info = crate::maps::theaters().into_iter().find(|t| t.id == id)?;
    let mut doc = grid_to_doc(&grid, name, &info.slug, seed);
    doc.blurb = info.blurb;
    doc.hazard = info.hazard;
    Some(doc)
}

pub fn theater_doc_json(id: u8) -> String {
    theater_to_doc(id)
        .map(|d| serde_json::to_string(&d).unwrap_or_else(|_| "{}".into()))
        .unwrap_or_else(|| "{}".into())
}

pub fn validate_json_report(raw: &str) -> String {
    match parse_and_validate(raw) {
        Ok((doc, _)) => serde_json::json!({
            "ok": true,
            "name": doc.name,
            "w": doc.w,
            "h": doc.h,
            "cores": doc.cores.len(),
            "spawns": doc.spawns.len(),
            "rocks": doc.rocks.len(),
        })
        .to_string(),
        Err(e) => serde_json::json!({ "ok": false, "error": e.message() }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_theaters_export_and_validate() {
        for t in crate::maps::theaters() {
            let doc = theater_to_doc(t.id).expect("doc");
            validate_map(&doc).unwrap_or_else(|e| panic!("{}: {}", t.name, e.message()));
        }
    }

    #[test]
    fn sealed_spawn_is_rejected() {
        let doc = MapDoc {
            slug: "bad".into(),
            name: "Bad".into(),
            blurb: String::new(),
            hazard: String::new(),
            w: 10,
            h: 8,
            seed: 1,
            cores: vec![[9, 4]],
            spawns: vec![[0, 4]],
            rocks: (0..8).map(|y| [1, y]).collect(),
        };
        assert_eq!(validate_map(&doc).unwrap_err(), MapError::Unreachable);
    }

    #[test]
    fn overlap_rejected() {
        let doc = MapDoc {
            slug: "o".into(),
            name: "O".into(),
            blurb: String::new(),
            hazard: String::new(),
            w: 10,
            h: 8,
            seed: 1,
            cores: vec![[2, 2]],
            spawns: vec![[2, 2]],
            rocks: vec![],
        };
        assert_eq!(validate_map(&doc).unwrap_err(), MapError::Overlap);
    }

    #[test]
    fn json_roundtrip() {
        let doc = theater_to_doc(0).unwrap();
        let raw = serde_json::to_string(&doc).unwrap();
        let (again, grid) = parse_and_validate(&raw).unwrap();
        assert_eq!(again.w, 40);
        assert!(!grid.cores().is_empty());
    }
}
