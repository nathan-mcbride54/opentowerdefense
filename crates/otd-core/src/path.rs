use crate::grid::Grid;
use std::collections::VecDeque;

pub const INF: u32 = u32::MAX / 4;

#[derive(Clone, Debug)]
pub struct FlowField {
    pub dist: Vec<u32>,
    pub next: Vec<Option<(i32, i32)>>,
}

impl FlowField {
    pub fn compute(grid: &Grid) -> Self {
        let n = (grid.w * grid.h) as usize;
        let mut dist = vec![INF; n];
        let mut next = vec![None; n];
        let mut q = VecDeque::new();

        for (x, y) in grid.cores() {
            let i = grid.idx(x, y);
            dist[i] = 0;
            q.push_back((x, y));
        }

        while let Some((x, y)) = q.pop_front() {
            let d = dist[grid.idx(x, y)];
            for (nx, ny) in grid.neighbors4(x, y) {
                if grid.blocks_ground(nx, ny) {
                    continue;
                }
                let ni = grid.idx(nx, ny);
                if dist[ni] > d + 1 {
                    dist[ni] = d + 1;
                    next[ni] = Some((x, y));
                    q.push_back((nx, ny));
                }
            }
        }

        Self { dist, next }
    }

    pub fn dist_at(&self, grid: &Grid, x: i32, y: i32) -> u32 {
        grid.try_idx(x, y).map(|i| self.dist[i]).unwrap_or(INF)
    }

    pub fn next_at(&self, grid: &Grid, x: i32, y: i32) -> Option<(i32, i32)> {
        grid.try_idx(x, y).and_then(|i| self.next[i])
    }

    pub fn spawns_reachable(&self, grid: &Grid) -> bool {
        grid.spawns()
            .into_iter()
            .all(|(x, y)| self.dist_at(grid, x, y) < INF)
    }

    pub fn cell_reachable(&self, grid: &Grid, x: i32, y: i32) -> bool {
        self.dist_at(grid, x, y) < INF
    }

    pub fn max_spawn_dist(&self, grid: &Grid) -> u32 {
        grid.spawns()
            .into_iter()
            .map(|(x, y)| self.dist_at(grid, x, y))
            .filter(|d| *d < INF)
            .max()
            .unwrap_or(0)
    }

    pub fn path_from(&self, grid: &Grid, mut x: i32, mut y: i32) -> Vec<[i32; 2]> {
        let mut out = vec![[x, y]];
        for _ in 0..512 {
            if self.dist_at(grid, x, y) == 0 {
                break;
            }
            match self.next_at(grid, x, y) {
                Some((nx, ny)) => {
                    if nx == x && ny == y {
                        break;
                    }
                    x = nx;
                    y = ny;
                    out.push([x, y]);
                }
                None => break,
            }
        }
        out
    }

    pub fn spawn_paths(&self, grid: &Grid) -> Vec<Vec<[i32; 2]>> {
        let spawns = grid.spawns();
        let mut used = vec![false; spawns.len()];
        let mut out = Vec::new();
        for i in 0..spawns.len() {
            if used[i] {
                continue;
            }
            let (sx, sy) = spawns[i];
            let mut stack = vec![i];
            used[i] = true;
            while let Some(j) = stack.pop() {
                let (jx, jy) = spawns[j];
                for (k, &(kx, ky)) in spawns.iter().enumerate() {
                    if used[k] {
                        continue;
                    }
                    if (kx - jx).abs() + (ky - jy).abs() == 1 {
                        used[k] = true;
                        stack.push(k);
                    }
                }
            }
            let p = self.path_from(grid, sx, sy);
            if p.len() > 1 {
                out.push(p);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Occupant, Terrain};

    fn hallway() -> Grid {
        let mut g = Grid::new(8, 3);
        g.set_terrain(0, 1, Terrain::Spawn);
        g.set_terrain(7, 1, Terrain::Core);
        for x in 0..8 {
            g.set_terrain(x, 0, Terrain::Rock);
            g.set_terrain(x, 2, Terrain::Rock);
        }
        g
    }

    #[test]
    fn open_hallway_reaches() {
        let g = hallway();
        let f = FlowField::compute(&g);
        assert!(f.spawns_reachable(&g));
        assert_eq!(f.dist_at(&g, 0, 1), 7);
    }

    #[test]
    fn wall_seals_hallway() {
        let mut g = hallway();
        g.set_occ(4, 1, Occupant::Wall);
        let f = FlowField::compute(&g);
        assert!(!f.spawns_reachable(&g));
    }

    #[test]
    fn spawn_path_walks_to_the_core() {
        let g = hallway();
        let f = FlowField::compute(&g);
        let paths = f.spawn_paths(&g);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].first().copied(), Some([0, 1]));
        assert_eq!(paths[0].last().copied(), Some([7, 1]));
    }
}
