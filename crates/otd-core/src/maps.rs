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
            blurb: "Two relays, one integrity pool. Ground takes the nearest. Air does too."
                .into(),
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
            blurb: "A rock U already folded the walk. Force them through the cup, or they go around.".into(),
            hazard: "The sides are short. The cup is the killbox if you seal the banks.".into(),
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
        7 => Some((oxbow(), "Oxbow", 0x0B50_11)),
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
}
