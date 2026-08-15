use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Terrain {
    Empty,
    Rock,
    Spawn,
    Core,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occupant {
    None,
    Wall,
    Tower(u32),
}

#[derive(Clone, Debug)]
pub struct Grid {
    pub w: i32,
    pub h: i32,
    terrain: Vec<Terrain>,
    occ: Vec<Occupant>,
}

impl Grid {
    pub fn new(w: i32, h: i32) -> Self {
        let n = (w * h) as usize;
        Self {
            w,
            h,
            terrain: vec![Terrain::Empty; n],
            occ: vec![Occupant::None; n],
        }
    }

    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    pub fn try_idx(&self, x: i32, y: i32) -> Option<usize> {
        if self.in_bounds(x, y) {
            Some(self.idx(x, y))
        } else {
            None
        }
    }

    pub fn terrain_at(&self, x: i32, y: i32) -> Option<Terrain> {
        Some(self.terrain[self.try_idx(x, y)?])
    }

    pub fn occupant(&self, x: i32, y: i32) -> Occupant {
        self.try_idx(x, y)
            .map(|i| self.occ[i])
            .unwrap_or(Occupant::None)
    }

    pub fn set_terrain(&mut self, x: i32, y: i32, t: Terrain) {
        if let Some(i) = self.try_idx(x, y) {
            self.terrain[i] = t;
        }
    }

    pub fn set_occ(&mut self, x: i32, y: i32, o: Occupant) {
        if let Some(i) = self.try_idx(x, y) {
            self.occ[i] = o;
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, t: Terrain) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set_terrain(xx, yy, t);
            }
        }
    }

    pub fn stamp_rock(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.fill_rect(x, y, w, h, Terrain::Rock);
    }

    pub fn blocks_ground(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return true;
        }
        let i = self.idx(x, y);
        self.terrain[i] == Terrain::Rock || !matches!(self.occ[i], Occupant::None)
    }

    pub fn buildable(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        let i = self.idx(x, y);
        self.terrain[i] == Terrain::Empty && matches!(self.occ[i], Occupant::None)
    }

    pub fn neighbors4(&self, x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> {
        let w = self.w;
        let h = self.h;
        [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .filter_map(move |(dx, dy)| {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && ny >= 0 && nx < w && ny < h {
                    Some((nx, ny))
                } else {
                    None
                }
            })
    }

    pub fn cores(&self) -> Vec<(i32, i32)> {
        self.filter_terrain(Terrain::Core)
    }

    pub fn spawns(&self) -> Vec<(i32, i32)> {
        self.filter_terrain(Terrain::Spawn)
    }

    pub fn rocks(&self) -> Vec<(i32, i32)> {
        self.filter_terrain(Terrain::Rock)
    }

    fn filter_terrain(&self, want: Terrain) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.terrain[self.idx(x, y)] == want {
                    out.push((x, y));
                }
            }
        }
        out
    }

    pub fn walls(&self) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if matches!(self.occ[self.idx(x, y)], Occupant::Wall) {
                    out.push((x, y));
                }
            }
        }
        out
    }

    pub fn core_center(&self) -> crate::geom::Vec2 {
        let cells = self.cores();
        if cells.is_empty() {
            return crate::geom::Vec2::new(self.w as f32 * 0.5, self.h as f32 * 0.5);
        }
        let n = cells.len() as f32;
        let sx: f32 = cells.iter().map(|(x, _)| *x as f32 + 0.5).sum();
        let sy: f32 = cells.iter().map(|(_, y)| *y as f32 + 0.5).sum();
        crate::geom::Vec2::new(sx / n, sy / n)
    }

    pub fn nearest_core(&self, pos: crate::geom::Vec2) -> crate::geom::Vec2 {
        let clusters = self.core_clusters();
        clusters
            .into_iter()
            .min_by(|a, b| {
                a.dist(pos)
                    .partial_cmp(&b.dist(pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(|| self.core_center())
    }

    /// Centroids of 4-connected core cell groups (one entry per relay).
    pub fn core_clusters(&self) -> Vec<crate::geom::Vec2> {
        let cells = self.cores();
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for &(sx, sy) in &cells {
            if !seen.insert((sx, sy)) {
                continue;
            }
            let mut stack = vec![(sx, sy)];
            let mut group = vec![(sx, sy)];
            while let Some((x, y)) = stack.pop() {
                for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if cells.iter().any(|&c| c == (nx, ny)) && seen.insert((nx, ny)) {
                        stack.push((nx, ny));
                        group.push((nx, ny));
                    }
                }
            }
            let n = group.len() as f32;
            let cx = group.iter().map(|(x, _)| *x as f32 + 0.5).sum::<f32>() / n;
            let cy = group.iter().map(|(_, y)| *y as f32 + 0.5).sum::<f32>() / n;
            out.push(crate::geom::Vec2::new(cx, cy));
        }
        out
    }

    pub fn cell_center(x: i32, y: i32) -> crate::geom::Vec2 {
        crate::geom::Vec2::new(x as f32 + 0.5, y as f32 + 0.5)
    }

    pub fn world_to_cell(x: f32, y: f32) -> (i32, i32) {
        (x.floor() as i32, y.floor() as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_and_buildable() {
        let mut g = Grid::new(4, 3);
        assert!(g.buildable(1, 1));
        g.set_terrain(1, 1, Terrain::Rock);
        assert!(!g.buildable(1, 1));
        assert!(g.blocks_ground(1, 1));
        assert!(!g.in_bounds(4, 0));
    }
}
