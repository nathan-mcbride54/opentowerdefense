use crate::grid::{Grid, Terrain};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TheaterInfo {
    pub id: u8,
    pub slug: String,
    pub name: String,
    pub blurb: String,
    pub hazard: String,
}

pub fn theaters() -> Vec<TheaterInfo> {
    vec![
        TheaterInfo {
            id: 0,
            slug: "kilo".into(),
            name: "Kilo Outpost".into(),
            blurb: "Wide scrub. Ingress from the north and the east. Two fronts are the lesson."
                .into(),
            hazard: "Open field — you must invent the maze.".into(),
        },
        TheaterInfo {
            id: 1,
            slug: "redoubt".into(),
            name: "Redoubt".into(),
            blurb: "Courtyard core. Hostiles enter from the north-west and the south-east.".into(),
            hazard: "Two doors into the same yard. Merge them or die covering both.".into(),
        },
        TheaterInfo {
            id: 2,
            slug: "dust".into(),
            name: "Dust Cut".into(),
            blurb: "Rock pass. The stones already pinched the walk — you finish the job.".into(),
            hazard: "Less room to dream. Every tile is a fight.".into(),
        },
        TheaterInfo {
            id: 3,
            slug: "split".into(),
            name: "Split Relay".into(),
            blurb: "A stone spine splits the field. Two walks. One merge. One core.".into(),
            hazard: "Cover both corridors or they leak while you admire the other.".into(),
        },
        TheaterInfo {
            id: 4,
            slug: "enclave".into(),
            name: "Enclave".into(),
            blurb: "Tight yard. Ingress is already in your pocket.".into(),
            hazard: "Every tile is a fight. Waste one and the relay hears it.".into(),
        },
        TheaterInfo {
            id: 5,
            slug: "twin".into(),
            name: "Twin Cores".into(),
            blurb: "Two relays, one integrity pool. Ground takes the nearest. Air does too.".into(),
            hazard: "Abandon a core and the sky still finds it.".into(),
        },
        TheaterInfo {
            id: 6,
            slug: "trigate".into(),
            name: "Tri-Gate".into(),
            blurb: "Three doors. North, west, and east all walk south to one relay.".into(),
            hazard: "Wall one door and the other two teach you manners.".into(),
        },
        TheaterInfo {
            id: 7,
            slug: "oxbow".into(),
            name: "Oxbow".into(),
            blurb:
                "A rock U already folded the walk. Force them through the cup, or they go around."
                    .into(),
            hazard: "The sides are short. The cup is the killbox if you seal the banks.".into(),
        },
        TheaterInfo {
            id: 8,
            slug: "mossfold".into(),
            name: "Mossfold".into(),
            blurb: "Forest enclaves. West ingress, relay nested in the north-east canopy.".into(),
            hazard: "The plateaus already pinch the walk. Finish the maze in the moss.".into(),
        },
        TheaterInfo {
            id: 9,
            slug: "labyrinth".into(),
            name: "Labyrinth".into(),
            blurb: "The coil ends in the south-east. Ground walks every switchback to the relay."
                .into(),
            hazard: "The relay is the last cell of the maze. Air still cuts the short way.".into(),
        },
    ]
}

pub fn theater_by_id(id: u8) -> Option<(Grid, &'static str, u64)> {
    match id {
        0 => Some((kilo_outpost(), "Kilo Outpost", 0xA11CE5)),
        1 => Some((redoubt(), "Redoubt", 0xBEEF42)),
        2 => Some((dust_cut(), "Dust Cut", 0xC0FFEE)),
        3 => Some((split_relay(), "Split Relay", 0x5B117)),
        4 => Some((enclave(), "Enclave", 0xE4C1A4)),
        5 => Some((twin_cores(), "Twin Cores", 0x7C0DE)),
        6 => Some((tri_gate(), "Tri-Gate", 0x7116_A7E0)),
        7 => Some((oxbow(), "Oxbow", 0x000B_5011)),
        8 => Some((mossfold(), "Mossfold", 0xF01E57)),
        9 => Some((labyrinth(), "Labyrinth", 0x1AB7_1175)),
        _ => None,
    }
}

pub fn theater_by_slug(slug: &str) -> Option<u8> {
    theaters()
        .into_iter()
        .find(|t| t.slug == slug)
        .map(|t| t.id)
}

/// Kilo Outpost — wide scrub, two ingresses, south-west relay.
pub fn kilo_outpost() -> Grid {
    let mut g = Grid::new(40, 24);

    g.fill_rect(2, 20, 2, 2, Terrain::Core);

    for x in 8..20 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for x in 27..38 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for y in 1..9 {
        g.set_terrain(39, y, Terrain::Spawn);
    }

    g.stamp_rock(10, 4, 4, 2);
    g.stamp_rock(11, 6, 1, 3);
    g.stamp_rock(4, 11, 3, 2);
    g.stamp_rock(5, 13, 1, 2);

    g.stamp_rock(18, 8, 5, 2);
    g.stamp_rock(18, 10, 2, 2);
    g.stamp_rock(21, 10, 2, 2);

    g.stamp_rock(31, 5, 3, 2);
    g.stamp_rock(33, 7, 1, 3);
    g.stamp_rock(28, 14, 4, 2);
    g.stamp_rock(29, 16, 1, 2);

    g.stamp_rock(8, 18, 2, 2);
    g.stamp_rock(14, 21, 3, 1);

    g
}

/// Courtyard core with two opposite doors.
pub fn redoubt() -> Grid {
    let mut g = Grid::new(36, 24);
    g.fill_rect(17, 11, 2, 2, Terrain::Core);

    for x in 1..8 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for y in 1..6 {
        g.set_terrain(0, y, Terrain::Spawn);
    }
    for x in 28..35 {
        g.set_terrain(x, 23, Terrain::Spawn);
    }
    for y in 18..23 {
        g.set_terrain(35, y, Terrain::Spawn);
    }

    // Courtyard ring with two gaps (NW and SE).
    g.stamp_rock(12, 7, 12, 1);
    g.stamp_rock(12, 16, 12, 1);
    g.stamp_rock(12, 8, 1, 8);
    g.stamp_rock(23, 8, 1, 8);
    // Open the doors: punch gaps
    g.set_terrain(12, 8, Terrain::Empty);
    g.set_terrain(12, 9, Terrain::Empty);
    g.set_terrain(23, 14, Terrain::Empty);
    g.set_terrain(23, 15, Terrain::Empty);

    g.stamp_rock(4, 10, 3, 2);
    g.stamp_rock(29, 12, 3, 2);
    g.stamp_rock(16, 3, 4, 1);
    g.stamp_rock(16, 20, 4, 1);

    g
}

/// Rocky pass, core in the south-east, pinched approaches.
pub fn dust_cut() -> Grid {
    let mut g = Grid::new(40, 24);
    g.fill_rect(36, 20, 2, 2, Terrain::Core);

    for x in 2..14 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for y in 2..12 {
        g.set_terrain(0, y, Terrain::Spawn);
    }

    g.stamp_rock(8, 4, 6, 3);
    g.stamp_rock(10, 7, 2, 4);
    g.stamp_rock(16, 2, 3, 5);
    g.stamp_rock(22, 6, 8, 2);
    g.stamp_rock(24, 8, 2, 5);
    g.stamp_rock(6, 14, 10, 2);
    g.stamp_rock(8, 16, 3, 4);
    g.stamp_rock(18, 14, 5, 4);
    g.stamp_rock(28, 12, 4, 3);
    g.stamp_rock(30, 16, 2, 4);
    g.stamp_rock(33, 8, 3, 6);
    g.stamp_rock(14, 20, 8, 2);

    g
}

/// Two long corridors split by a rock spine; they merge only at the southern relay.
pub fn split_relay() -> Grid {
    let mut g = Grid::new(40, 24);
    g.fill_rect(19, 20, 2, 2, Terrain::Core);

    for x in 1..12 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for x in 28..39 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }

    g.stamp_rock(19, 0, 2, 17);
    g.stamp_rock(6, 5, 4, 2);
    g.stamp_rock(9, 11, 3, 3);
    g.stamp_rock(4, 16, 3, 2);
    g.stamp_rock(30, 4, 4, 2);
    g.stamp_rock(27, 10, 3, 3);
    g.stamp_rock(33, 15, 3, 2);
    g.stamp_rock(14, 18, 4, 1);
    g.stamp_rock(22, 18, 4, 1);

    g
}

/// Small yard. Hostiles are already close.
pub fn enclave() -> Grid {
    let mut g = Grid::new(28, 18);
    g.fill_rect(13, 8, 2, 2, Terrain::Core);

    for x in 3..10 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for x in 18..25 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for y in 6..12 {
        g.set_terrain(0, y, Terrain::Spawn);
    }

    g.stamp_rock(11, 1, 6, 1);
    g.stamp_rock(3, 3, 3, 2);
    g.stamp_rock(22, 3, 3, 2);
    g.stamp_rock(8, 13, 4, 2);
    g.stamp_rock(16, 12, 4, 2);
    g.stamp_rock(2, 13, 2, 2);
    g.stamp_rock(24, 13, 2, 2);
    g.stamp_rock(6, 7, 2, 2);
    g.stamp_rock(20, 7, 2, 2);

    g
}

/// Two relays, one integrity pool. West and east basins, north can choose.
pub fn twin_cores() -> Grid {
    let mut g = Grid::new(42, 24);
    g.fill_rect(1, 11, 2, 2, Terrain::Core);
    g.fill_rect(39, 11, 2, 2, Terrain::Core);

    for y in 2..10 {
        g.set_terrain(0, y, Terrain::Spawn);
    }
    for y in 2..10 {
        g.set_terrain(41, y, Terrain::Spawn);
    }
    for x in 16..26 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }

    g.stamp_rock(20, 4, 2, 16);
    g.stamp_rock(7, 5, 3, 2);
    g.stamp_rock(6, 16, 4, 2);
    g.stamp_rock(32, 5, 3, 2);
    g.stamp_rock(32, 16, 4, 2);
    g.stamp_rock(14, 8, 2, 2);
    g.stamp_rock(26, 8, 2, 2);
    g.stamp_rock(12, 20, 4, 1);
    g.stamp_rock(26, 20, 4, 1);

    g
}

/// Three ingresses, one southern relay. You cannot cover a single door.
pub fn tri_gate() -> Grid {
    let mut g = Grid::new(40, 24);
    g.fill_rect(19, 20, 2, 2, Terrain::Core);

    for x in 16..24 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }
    for y in 8..17 {
        g.set_terrain(0, y, Terrain::Spawn);
        g.set_terrain(39, y, Terrain::Spawn);
    }

    g.stamp_rock(8, 5, 8, 2);
    g.stamp_rock(24, 5, 8, 2);
    g.stamp_rock(10, 10, 2, 7);
    g.stamp_rock(28, 10, 2, 7);
    g.stamp_rock(4, 18, 4, 2);
    g.stamp_rock(32, 18, 4, 2);
    g.stamp_rock(17, 12, 2, 2);
    g.stamp_rock(21, 12, 2, 2);
    g.stamp_rock(6, 2, 3, 2);
    g.stamp_rock(31, 2, 3, 2);

    g
}

/// Rock U opening north. Side banks are short; the cup is the long walk if you seal them.
pub fn oxbow() -> Grid {
    let mut g = Grid::new(36, 22);
    g.fill_rect(16, 19, 2, 2, Terrain::Core);

    for x in 10..26 {
        g.set_terrain(x, 0, Terrain::Spawn);
    }

    g.stamp_rock(0, 0, 5, 22);
    g.stamp_rock(31, 0, 5, 22);

    g.stamp_rock(12, 4, 2, 12);
    g.stamp_rock(22, 4, 2, 12);
    g.stamp_rock(12, 15, 12, 2);

    g.stamp_rock(6, 8, 2, 2);
    g.stamp_rock(28, 10, 2, 2);
    g.stamp_rock(8, 17, 3, 1);
    g.stamp_rock(25, 17, 3, 1);

    g
}

/// Forested enclaves: organic plateaus pinch a west-to-north-east walk.
pub fn mossfold() -> Grid {
    let mut g = Grid::new(42, 24);

    g.fill_rect(38, 1, 2, 2, Terrain::Core);

    for y in 5..20 {
        g.set_terrain(0, y, Terrain::Spawn);
    }

    // Relay nest in the north-east canopy.
    g.stamp_blob(41.0, 0.0, 3.6, 2.8);
    g.stamp_blob(34.2, 0.4, 4.4, 2.4);
    g.stamp_blob(41.0, 7.2, 4.8, 3.6);

    // Upper-middle plateau.
    g.stamp_blob(18.2, 5.4, 6.6, 4.0);
    g.stamp_blob(21.6, 8.0, 4.2, 3.0);
    g.stamp_blob(15.0, 7.2, 3.4, 2.6);

    // Lower plateaus as separate enclaves, not one fused wall.
    g.stamp_blob(15.2, 19.6, 4.0, 2.5);
    g.stamp_blob(23.2, 17.4, 3.1, 2.4);

    // Bottom-right mass, held off the west so a south corridor can exist.
    g.stamp_blob(40.6, 22.8, 8.0, 5.0);
    g.stamp_blob(36.4, 21.6, 4.0, 3.0);

    // Mid-right bottleneck island.
    g.stamp_blob(29.2, 11.2, 4.4, 3.5);
    g.stamp_blob(31.4, 13.6, 3.0, 2.4);

    // Scattered islets.
    g.stamp_blob(8.0, 7.2, 2.7, 2.3);
    g.stamp_blob(6.4, 15.2, 2.5, 2.2);
    g.stamp_blob(12.2, 12.0, 2.2, 1.9);
    g.stamp_blob(10.4, 20.6, 1.9, 1.6);
    g.stamp_blob(25.2, 3.6, 2.3, 1.8);
    g.stamp_blob(27.2, 20.6, 1.8, 1.5);
    g.stamp_blob(14.8, 3.2, 2.0, 1.6);

    // Narrow bottom passes (1–2 cells). Keep the plateaus as killboxes.
    g.carve_gap(13, 20, 2, 1);
    g.carve_gap(19, 21, 2, 1);
    g.carve_gap(22, 18, 2, 1);
    g.carve_gap(31, 20, 2, 1);
    g.carve_gap(35, 18, 2, 1);
    g.carve_gap(38, 14, 1, 4);

    // Trim the ragged spurs the blobs left on the upper plateau's west lip and along
    // its southern edge, so the approach reads as a clean shelf rather than a fringe.
    g.carve_gap(11, 5, 1, 2);
    g.carve_gap(13, 9, 2, 1);
    g.carve_gap(25, 8, 2, 1);
    g.carve_gap(24, 9, 2, 1);

    g
}

/// Four switchbacks. Spawn north-west; the relay is the last cell in the south-east corner.
pub fn labyrinth() -> Grid {
    let mut g = Grid::new(40, 24);
    g.stamp_rock(0, 0, 40, 24);

    for y in 2..4 {
        g.set_terrain(0, y, Terrain::Spawn);
    }

    // Lane 1 east, drop south on the east wall.
    g.carve_gap(1, 2, 37, 2);
    g.carve_gap(36, 4, 2, 4);

    // Lane 2 west, drop south on the west wall.
    g.carve_gap(3, 7, 35, 2);
    g.carve_gap(3, 9, 2, 4);

    // Lane 3 east, drop into lane 4 on the east wall.
    g.carve_gap(3, 12, 35, 2);
    g.carve_gap(36, 14, 2, 5);

    // Lane 4 west, drop into the floor on the west wall, then east to the corner.
    g.carve_gap(3, 17, 35, 2);
    g.carve_gap(3, 19, 2, 4);
    g.carve_gap(3, 21, 37, 2);

    g.fill_rect(38, 21, 2, 2, Terrain::Core);

    // Gun pockets. Staggered so they never join two lanes.
    for x in [4, 12, 20, 28] {
        g.carve_gap(x, 0, 2, 2);
        g.carve_gap(x, 4, 2, 2);
    }
    for x in [8, 16, 24, 32] {
        g.carve_gap(x, 5, 2, 2);
        g.carve_gap(x, 9, 2, 2);
    }
    for x in [6, 14, 22, 30] {
        g.carve_gap(x, 14, 2, 2);
    }
    for x in [10, 18, 26] {
        g.carve_gap(x, 15, 2, 2);
    }
    for x in [8, 16, 24, 32] {
        g.carve_gap(x, 23, 2, 1);
    }

    g.stamp_rock(11, 2, 1, 1);
    g.stamp_rock(21, 3, 1, 1);
    g.stamp_rock(14, 8, 1, 1);
    g.stamp_rock(25, 7, 1, 1);
    g.stamp_rock(10, 13, 1, 1);
    g.stamp_rock(22, 12, 1, 1);
    g.stamp_rock(16, 18, 1, 1);
    g.stamp_rock(28, 17, 1, 1);
    g.stamp_rock(20, 22, 1, 1);

    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::FlowField;

    fn assert_open(g: Grid) {
        assert!(!g.cores().is_empty());
        assert!(!g.spawns().is_empty());
        let f = FlowField::compute(&g);
        assert!(f.spawns_reachable(&g));
    }

    #[test]
    fn all_theaters_start_open() {
        for t in theaters() {
            let (grid, _, _) = theater_by_id(t.id).expect("theater");
            assert_open(grid);
        }
    }

    #[test]
    fn twin_cores_has_two_relays() {
        let g = twin_cores();
        assert_eq!(g.core_clusters().len(), 2);
    }

    #[test]
    fn tri_gate_has_three_doors() {
        let g = tri_gate();
        let spawns = g.spawns();
        assert!(spawns.iter().any(|(_, y)| *y == 0));
        assert!(spawns.iter().any(|(x, _)| *x == 0));
        assert!(spawns.iter().any(|(x, _)| *x == g.w - 1));
        assert_eq!(g.core_clusters().len(), 1);
    }

    #[test]
    fn oxbow_has_a_cup_and_two_banks() {
        let g = oxbow();
        assert!(g.spawns().iter().all(|(_, y)| *y == 0));
        assert_eq!(g.core_clusters().len(), 1);
        assert!(g.blocks_ground(12, 8));
        assert!(g.blocks_ground(22, 8));
        assert!(g.blocks_ground(16, 15));
        assert!(!g.blocks_ground(8, 8));
        assert!(!g.blocks_ground(27, 8));
        assert!(!g.blocks_ground(16, 8));
    }

    #[test]
    fn mossfold_is_forest_enclaves() {
        let g = mossfold();
        assert_eq!(g.w, 42);
        assert_eq!(g.h, 24);
        assert!(g.spawns().iter().all(|(x, _)| *x == 0));
        assert!(g.cores().iter().all(|(x, y)| *x >= 36 && *y <= 4));
        assert_eq!(g.core_clusters().len(), 1);
        assert!(g.blocks_ground(18, 6));
        assert!(g.blocks_ground(40, 22));
        assert!(!g.blocks_ground(4, 12));
        assert!(!g.blocks_ground(19, 21));
        assert!(!g.blocks_ground(31, 20));
        assert!(!g.blocks_ground(38, 15));
        assert!(g.blocks_ground(15, 19));
    }

    #[test]
    fn labyrinth_is_a_long_maze() {
        let g = labyrinth();
        assert_eq!(g.w, 40);
        assert_eq!(g.h, 24);
        assert!(g.spawns().iter().all(|(x, _)| *x == 0));
        assert!(g.cores().iter().all(|(x, y)| *x >= 36 && *y >= 20));
        assert_eq!(g.core_clusters().len(), 1);
        let f = FlowField::compute(&g);
        let walk = f.max_spawn_dist(&g);
        assert!(walk >= 140, "labyrinth walk too short: {walk}");
        assert!(walk <= 220, "labyrinth walk too long: {walk}");
        let path = f.path_from(&g, 0, 2);
        assert!(
            path.iter().any(|p| p[0] <= 6 && p[1] >= 21),
            "walk must use the west floor before the relay"
        );
        assert!(
            path.iter().any(|p| p[0] >= 34 && p[1] >= 21),
            "walk must finish in the south-east"
        );
        assert!(
            f.dist_at(&g, 8, 21) > f.dist_at(&g, 30, 21),
            "floor runs east to the relay"
        );
        assert!(g.blocks_ground(2, 5), "lane 1/2 wall");
        assert!(g.blocks_ground(2, 10), "lane 2/3 wall");
        assert!(g.blocks_ground(2, 15), "lane 3/4 wall");
        assert!(g.blocks_ground(10, 5), "pocket gap");
        assert!(!g.blocks_ground(18, 2), "lane 1");
        assert!(!g.blocks_ground(18, 7), "lane 2");
        assert!(!g.blocks_ground(18, 12), "lane 3");
        assert!(!g.blocks_ground(18, 17), "lane 4");
        assert!(!g.blocks_ground(4, 4), "gun pocket");
        assert!(!g.blocks_ground(36, 8), "east drop");
        assert!(!g.blocks_ground(3, 20), "west drop to floor");
        assert!(g.blocks_ground(39, 20), "nothing past the relay");
    }
}
