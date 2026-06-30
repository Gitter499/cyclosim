#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RideMode {
    #[default]
    Free,
    Erg,
    Sim,
}

#[derive(Debug, Clone, Default)]
pub struct RideState {
    pub mode: RideMode,
    pub distance_m: f64,
    pub speed_mps: f64,
    pub elapsed_s: f64,
    pub grade: f64,
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
}
