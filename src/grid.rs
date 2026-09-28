use crate::math::V2;
use std::collections::HashMap;

/// Spatial hash for neighbor queries on an unbounded map, rebuilt every step.
pub struct Grid {
    cell: f32,
    cells: HashMap<(i32, i32), Vec<u32>>,
}

impl Grid {
    pub fn new(cell: f32) -> Self {
        Grid { cell, cells: HashMap::new() }
    }

    /// Empties every cell but keeps the allocations for the next rebuild.
    pub fn clear(&mut self) {
        for cell in self.cells.values_mut() {
            cell.clear();
        }
        // Forget cells far behind the player once they pile up.
        if self.cells.len() > 4096 {
            self.cells.retain(|_, v| !v.is_empty());
        }
    }

    fn coords(&self, pos: V2) -> (i32, i32) {
        ((pos.x / self.cell).floor() as i32, (pos.y / self.cell).floor() as i32)
    }

    pub fn insert(&mut self, id: u32, pos: V2) {
        let key = self.coords(pos);
        self.cells.entry(key).or_default().push(id);
    }

    /// Calls `f` with every id in the cells overlapping the circle. Callers still
    /// check the exact distance: this is a broad phase only.
    pub fn for_each_near(&self, pos: V2, radius: f32, mut f: impl FnMut(u32)) {
        let (x0, y0) = self.coords(pos - V2::new(radius, radius));
        let (x1, y1) = self.coords(pos + V2::new(radius, radius));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                if let Some(cell) = self.cells.get(&(cx, cy)) {
                    for &id in cell {
                        f(id);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_neighbors_at_any_coordinate() {
        let mut grid = Grid::new(80.0);
        grid.insert(1, V2::new(-10_000.0, 25_000.0));
        grid.insert(2, V2::new(10.0, 10.0));
        let mut found = Vec::new();
        grid.for_each_near(V2::new(-10_010.0, 25_010.0), 30.0, |id| found.push(id));
        assert_eq!(found, vec![1]);
    }
}
