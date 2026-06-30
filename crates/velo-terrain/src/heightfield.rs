/// Regular grid heightfield in local ENU meters (east/north) with elevation samples.
#[derive(Debug, Clone)]
pub struct Heightfield {
    pub cols: usize,
    pub rows: usize,
    pub cell_m: f64,
    pub origin_east_m: f64,
    pub origin_north_m: f64,
    pub elevations: Vec<f32>,
}

impl Heightfield {
    pub fn elevation_at(&self, col: usize, row: usize) -> f32 {
        self.elevations[row * self.cols + col]
    }

    pub fn set(&mut self, col: usize, row: usize, elev: f32) {
        self.elevations[row * self.cols + col] = elev;
    }

    pub fn east_at(&self, col: usize) -> f64 {
        self.origin_east_m + col as f64 * self.cell_m
    }

    pub fn north_at(&self, row: usize) -> f64 {
        self.origin_north_m + row as f64 * self.cell_m
    }
}
