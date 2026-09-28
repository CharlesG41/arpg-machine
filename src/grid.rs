use crate::math::V2;

/// Uniform spatial grid over the map, rebuilt every step, for neighbor queries.
pub struct Grid {
    cell: f32,
    cols: i32,
    rows: i32,
    cells: Vec<Vec<u32>>,
}

impl Grid {
    pub fn new(width: f32, height: f32, cell: f32) -> Self {
        let cols = (width / cell).ceil() as i32 + 1;
        let rows = (height / cell).ceil() as i32 + 1;
        Grid { cell, cols, rows, cells: vec![Vec::new(); (cols * rows) as usize] }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    fn coords(&self, pos: V2) -> (i32, i32) {
        (
            ((pos.x / self.cell) as i32).clamp(0, self.cols - 1),
            ((pos.y / self.cell) as i32).clamp(0, self.rows - 1),
        )
    }

    pub fn insert(&mut self, id: u32, pos: V2) {
        let (cx, cy) = self.coords(pos);
        self.cells[(cy * self.cols + cx) as usize].push(id);
    }

    /// Calls `f` with every id in the cells overlapping the circle. Callers still
    /// check the exact distance: this is a broad phase only.
    pub fn for_each_near(&self, pos: V2, radius: f32, mut f: impl FnMut(u32)) {
        let (x0, y0) = self.coords(pos - V2::new(radius, radius));
        let (x1, y1) = self.coords(pos + V2::new(radius, radius));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                for &id in &self.cells[(cy * self.cols + cx) as usize] {
                    f(id);
                }
            }
        }
    }
}
