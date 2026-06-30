use velo_platform::{SensorSource, TelemetrySample, TrainerControl};
use velo_units::{Grade, MetersPerSecond, Watts};

use crate::physics::{integrate_step, PhysicsConfig};
use crate::ride::{RideMode, RideState};
use crate::ride_session::{RideSample, RideSession, RideSummary};
use crate::route::RouteModel;
use crate::workout::{Workout, WorkoutEngine};

const DT: f32 = 1.0 / 100.0;

/// Application state owned by Rust, driven from the Swift shell via FFI.
pub struct VeloApp {
    pub toggle_count: u32,
    pub ride: RideState,
    pub ride_session: RideSession,
    pub route: Option<RouteModel>,
    pub active_route_id: Option<String>,
    pub workout_engine: Option<WorkoutEngine>,
    log: Vec<String>,
    tick: u64,
    target_power: Watts,
    physics: PhysicsConfig,
    speed: MetersPerSecond,
    clock_unix: u64,
}

impl VeloApp {
    pub fn new() -> Self {
        Self {
            toggle_count: 0,
            ride: RideState::default(),
            ride_session: RideSession::new(),
            route: None,
            active_route_id: None,
            workout_engine: None,
            log: Vec::new(),
            tick: 0,
            target_power: Watts::new(150.0),
            physics: PhysicsConfig::default(),
            speed: MetersPerSecond::new(0.0),
            clock_unix: 1_700_000_000,
        }
    }

    pub fn set_clock_unix(&mut self, unix_secs: u64) {
        self.clock_unix = unix_secs;
    }

    pub fn is_ride_recording(&self) -> bool {
        self.ride_session.is_active()
    }

    pub fn start_ride(&mut self) {
        if !self.ride_session.is_active() {
            self.ride_session.start(self.clock_unix);
            self.push_log("ride started".into());
        }
    }

    pub fn stop_ride(&mut self) -> Option<RideSummary> {
        let summary = self.ride_session.stop();
        if summary.is_some() {
            self.push_log("ride stopped".into());
        }
        summary
    }

    pub fn export_fit(&self) -> Result<Vec<u8>, velo_fit::FitEncodeError> {
        self.ride_session.export_fit()
    }

    pub fn last_ride_summary(&self) -> Option<RideSummary> {
        self.ride_session.last_summary()
    }

    pub fn toggle(&mut self) -> u32 {
        self.toggle_count = self.toggle_count.wrapping_add(1);
        self.toggle_count
    }

    pub fn toggle_count(&self) -> u32 {
        self.toggle_count
    }

    pub fn set_ride_mode(&mut self, mode: RideMode) {
        self.ride.mode = mode;
    }

    pub fn set_target_power(&mut self, watts: f64) {
        self.target_power = Watts::new(watts);
    }

    pub fn set_ftp(&mut self, ftp_w: f64) {
        self.physics.ftp_w = ftp_w;
    }

    pub fn ftp(&self) -> f64 {
        self.physics.ftp_w
    }

    pub fn start_workout(&mut self, workout: Workout) {
        let engine = WorkoutEngine::new(workout, self.physics.ftp_w);
        self.workout_engine = Some(engine);
        self.set_ride_mode(RideMode::Erg);
        self.push_log("workout started".into());
    }

    pub fn clear_workout(&mut self) {
        self.workout_engine = None;
        self.push_log("workout cleared".into());
    }

    pub fn workout_active(&self) -> bool {
        self.workout_engine.is_some()
    }

    pub fn workout_state(&self) -> Option<&crate::workout::WorkoutState> {
        self.workout_engine.as_ref().map(WorkoutEngine::state)
    }

    pub fn target_power(&self) -> f64 {
        self.target_power.0
    }

    pub fn set_grade(&mut self, grade: f64) {
        if self.route.is_none() {
            self.ride.grade = grade;
        }
    }

    pub fn load_route(&mut self, route: RouteModel) {
        self.active_route_id = Some(route.meta.route_id.clone());
        self.ride.distance_m = 0.0;
        self.ride.grade = route.grade_at(0.0);
        self.route = Some(route);
        self.push_log("route loaded".into());
    }

    pub fn clear_route(&mut self) {
        self.route = None;
        self.active_route_id = None;
        self.push_log("route cleared".into());
    }

    pub fn active_route_id(&self) -> Option<&str> {
        self.active_route_id.as_deref()
    }

    pub fn route_position_enu(&self) -> Option<(f64, f64, f64)> {
        self.route
            .as_ref()
            .map(|r| r.position_enu_at(self.ride.distance_m))
    }

    fn sync_grade_from_route(&mut self) {
        if let Some(route) = &self.route {
            self.ride.grade = route.grade_at(self.ride.distance_m);
        }
    }

    fn sync_workout_targets(&mut self) {
        let Some(engine) = self.workout_engine.as_mut() else {
            return;
        };
        if engine.state().finished {
            self.workout_engine = None;
            self.push_log("workout finished".into());
            return;
        }
        if engine.is_free_ride_interval() {
            self.ride.mode = RideMode::Sim;
        } else if let Some(w) = engine.target_watts() {
            self.ride.mode = RideMode::Erg;
            self.target_power = w;
        }
        engine.tick(DT as f64);
    }

    /// Fixed-step sim tick: drain sensor samples, integrate, emit trainer commands.
    pub fn tick<S: SensorSource, T: TrainerControl>(
        &mut self,
        sensors: &mut S,
        trainer: &T,
    ) {
        self.tick = self.tick.wrapping_add(1);
        let samples = sensors.drain_samples();

        for sample in samples {
            self.apply_sample(&sample);
        }

        self.sync_grade_from_route();
        self.sync_workout_targets();

        let grade = Grade::new(self.ride.grade);
        let power = self
            .ride
            .power_w
            .map(Watts::new)
            .unwrap_or(self.target_power);

        let snap = integrate_step(&self.physics, grade, power, self.speed, DT);
        self.speed = snap.speed;
        self.ride.distance_m += snap.distance.0;
        self.ride.speed_mps = self.speed.0;
        self.ride.elapsed_s += DT as f64;

        if self.ride_session.is_active() {
            self.ride_session.record_tick(RideSample {
                elapsed_s: self.ride.elapsed_s,
                distance_m: self.ride.distance_m,
                speed_mps: self.ride.speed_mps,
                power_w: self.ride.power_w,
                cadence_rpm: self.ride.cadence_rpm,
                heart_rate_bpm: self.ride.heart_rate_bpm,
                grade: self.ride.grade,
            });
        }

        match self.ride.mode {
            RideMode::Erg => trainer.set_target_power(self.target_power),
            RideMode::Sim => trainer.set_simulation(
                grade,
                self.physics.crr,
                self.physics.cda,
            ),
            RideMode::Free => {}
        }

        self.push_log(format!(
            "tick={} dist={:.1}m speed={:.2}m/s power={:.0}W",
            self.tick,
            self.ride.distance_m,
            self.ride.speed_mps,
            power.0
        ));
    }

    fn apply_sample(&mut self, sample: &TelemetrySample) {
        if let Some(p) = sample.power {
            self.ride.power_w = Some(p.0);
        }
        if let Some(c) = sample.cadence {
            self.ride.cadence_rpm = Some(c.0);
        }
        if let Some(hr) = sample.heart_rate {
            self.ride.heart_rate_bpm = Some(hr.0);
        }
        let _ = sample.elapsed;
    }

    pub fn recent_logs(&self, limit: usize) -> Vec<String> {
        let start = self.log.len().saturating_sub(limit);
        self.log[start..].to_vec()
    }

    fn push_log(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 256 {
            let drain = self.log.len() - 128;
            self.log.drain(0..drain);
        }
    }
}

impl Default for VeloApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use velo_platform::{MockSensorSource, RecordingTrainerControl};
    use velo_units::{Bpm, Rpm};

    #[test]
    fn toggle_increments_counter() {
        let mut app = VeloApp::new();
        assert_eq!(app.toggle(), 1);
        assert_eq!(app.toggle(), 2);
    }

    #[test]
    fn tick_drains_sensor_and_commands_trainer() {
        let mut app = VeloApp::new();
        app.set_ride_mode(RideMode::Erg);
        app.set_target_power(200.0);
        let mut sensors = MockSensorSource::default();
        sensors.push(TelemetrySample {
            elapsed: Duration::from_millis(0),
            power: Some(Watts::new(198.0)),
            cadence: Some(Rpm::new(90.0)),
            heart_rate: Some(Bpm::new(140.0)),
            wheel_speed: None,
        });
        let trainer = RecordingTrainerControl::default();
        app.tick(&mut sensors, &trainer);
        assert_eq!(app.ride.power_w, Some(198.0));
        assert_eq!(trainer.last_power(), Some(Watts::new(200.0)));
        assert!(app.ride.distance_m > 0.0);
    }

    #[test]
    fn ride_recording_pipeline() {
        let mut app = VeloApp::new();
        app.set_ride_mode(RideMode::Erg);
        app.set_target_power(200.0);
        app.start_ride();
        let mut sensors = MockSensorSource::default();
        let trainer = RecordingTrainerControl::default();
        for _ in 0..100 {
            sensors.push(TelemetrySample {
                elapsed: Duration::from_millis(0),
                power: Some(Watts::new(200.0)),
                cadence: Some(Rpm::new(90.0)),
                heart_rate: Some(Bpm::new(140.0)),
                wheel_speed: None,
            });
            app.tick(&mut sensors, &trainer);
        }
        let summary = app.stop_ride().unwrap();
        assert_eq!(summary.sample_count, 100);
        let fit = app.export_fit().unwrap();
        assert_eq!(&fit[8..12], b".FIT");

        let parsed = fitparser::from_bytes(&fit).unwrap();
        use fitparser::profile::MesgNum;
        assert!(parsed.iter().any(|m| m.kind() == MesgNum::Session));
        let records: Vec<_> = parsed.iter().filter(|m| m.kind() == MesgNum::Record).collect();
        assert_eq!(records.len(), 100);
    }
}
