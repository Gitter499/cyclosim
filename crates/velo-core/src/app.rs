use velo_platform::{PlaybackIntent, SegmentEnergy, SensorSource, TelemetrySample, TrainerControl};
use velo_units::{Grade, MetersPerSecond, Watts};

use crate::audio::{energy_for_interval, AudioEvent};
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
    audio_events: Vec<AudioEvent>,
    steer_axis: f64,
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
            audio_events: Vec::new(),
            steer_axis: 0.0,
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
        let count = workout.intervals.len();
        if let Some(first) = workout.intervals.first() {
            let energy = energy_for_interval(first, self.physics.ftp_w, 0, count);
            self.push_audio_event(energy, PlaybackIntent::Start);
        }
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
        let ftp = self.physics.ftp_w;
        let mut boundary_energy = None;
        let mut finished = false;
        {
            let Some(engine) = self.workout_engine.as_mut() else {
                return;
            };
            if engine.state().finished {
                finished = true;
            } else {
                if engine.is_free_ride_interval() {
                    self.ride.mode = RideMode::Sim;
                } else if let Some(w) = engine.target_watts() {
                    self.ride.mode = RideMode::Erg;
                    self.target_power = w;
                }
                let changed = engine.tick(DT as f64);
                if changed && !engine.state().finished {
                    let idx = engine.state().interval_index;
                    let count = engine.workout().intervals.len();
                    boundary_energy = engine
                        .current_interval()
                        .map(|i| energy_for_interval(i, ftp, idx, count));
                }
            }
        }
        if finished {
            self.workout_engine = None;
            self.push_log("workout finished".into());
            self.push_audio_event(SegmentEnergy::Cooldown, PlaybackIntent::Transition);
            return;
        }
        if let Some(energy) = boundary_energy {
            self.push_audio_event(energy, PlaybackIntent::Transition);
        }
    }

    fn push_audio_event(&mut self, energy: SegmentEnergy, intent: PlaybackIntent) {
        self.audio_events.push(AudioEvent { energy, intent });
        if self.audio_events.len() > 64 {
            let drain = self.audio_events.len() - 32;
            self.audio_events.drain(0..drain);
        }
    }

    /// Drain queued audio direction events (shell forwards to AudioDirector).
    pub fn drain_audio_events(&mut self) -> Vec<AudioEvent> {
        std::mem::take(&mut self.audio_events)
    }

    /// Update the steering axis from the shell's input source.
    ///
    /// `axis` in [-1, 1]; `recenter` snaps the lateral offset back to the
    /// route line (drift correction for head-tracked steering).
    pub fn set_steering(&mut self, axis: f64, recenter: bool) {
        self.steer_axis = axis.clamp(-1.0, 1.0);
        if recenter {
            self.ride.lateral_offset_m = 0.0;
        }
    }

    fn integrate_steering(&mut self) {
        const DEADZONE: f64 = 0.1;
        const RATE_MPS: f64 = 2.5;
        const MAX_OFFSET_M: f64 = 3.5;
        let axis = self.steer_axis;
        if axis.abs() < DEADZONE {
            return;
        }
        self.ride.lateral_offset_m = (self.ride.lateral_offset_m
            + axis * RATE_MPS * DT as f64)
            .clamp(-MAX_OFFSET_M, MAX_OFFSET_M);
    }

    /// Rider ENU position with the steering offset applied perpendicular to
    /// the direction of travel (falls back to the route line when centered).
    pub fn steered_position_enu(&self) -> Option<(f64, f64, f64)> {
        let route = self.route.as_ref()?;
        let d = self.ride.distance_m;
        let (east, up, north) = route.position_enu_at(d);
        let offset = self.ride.lateral_offset_m;
        if offset == 0.0 {
            return Some((east, up, north));
        }
        let (east_ahead, _, north_ahead) = route.position_enu_at(d + 5.0);
        let (dx, dz) = (east_ahead - east, north_ahead - north);
        let len = (dx * dx + dz * dz).sqrt();
        if len < 1e-9 {
            return Some((east, up, north));
        }
        // Right of travel = (forward.z, -forward.x) in the EN plane.
        let (side_e, side_n) = (dz / len, -dx / len);
        Some((east + side_e * offset, up, north + side_n * offset))
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
        self.integrate_steering();

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
    fn workout_emits_audio_events_at_boundaries() {
        use crate::workout::{Workout, WorkoutInterval, WorkoutTarget};
        use velo_platform::{PlaybackIntent, SegmentEnergy};

        let mut app = VeloApp::new();
        app.set_ftp(250.0);
        app.start_workout(Workout {
            name: "two-step".into(),
            intervals: vec![
                WorkoutInterval {
                    name: "Warmup".into(),
                    duration_s: 0.05,
                    target: WorkoutTarget::FtpPercent(55.0),
                },
                WorkoutInterval {
                    name: "On".into(),
                    duration_s: 0.05,
                    target: WorkoutTarget::FtpPercent(95.0),
                },
            ],
        });

        let start_events = app.drain_audio_events();
        assert_eq!(
            start_events,
            vec![crate::audio::AudioEvent {
                energy: SegmentEnergy::Warmup,
                intent: PlaybackIntent::Start,
            }]
        );

        let mut sensors = MockSensorSource::default();
        let trainer = RecordingTrainerControl::default();
        for _ in 0..30 {
            app.tick(&mut sensors, &trainer);
        }

        let events = app.drain_audio_events();
        assert!(events.contains(&crate::audio::AudioEvent {
            energy: SegmentEnergy::Threshold,
            intent: PlaybackIntent::Transition,
        }));
        assert_eq!(
            events.last(),
            Some(&crate::audio::AudioEvent {
                energy: SegmentEnergy::Cooldown,
                intent: PlaybackIntent::Transition,
            })
        );
        assert!(app.drain_audio_events().is_empty());
    }

    #[test]
    fn steering_integrates_and_recenters() {
        let mut app = VeloApp::new();
        let mut sensors = MockSensorSource::default();
        let trainer = RecordingTrainerControl::default();

        app.set_steering(1.0, false);
        for _ in 0..100 {
            app.tick(&mut sensors, &trainer);
        }
        let offset = app.ride.lateral_offset_m;
        assert!((offset - 2.5).abs() < 0.1, "1s full right ≈ 2.5 m, got {offset}");

        for _ in 0..100 {
            app.tick(&mut sensors, &trainer);
        }
        assert!((app.ride.lateral_offset_m - 3.5).abs() < 1e-9, "clamped at 3.5");

        app.set_steering(0.05, false); // inside deadzone: hold position
        for _ in 0..50 {
            app.tick(&mut sensors, &trainer);
        }
        assert!((app.ride.lateral_offset_m - 3.5).abs() < 1e-9);

        app.set_steering(0.0, true); // recenter
        assert_eq!(app.ride.lateral_offset_m, 0.0);
    }

    #[test]
    fn steered_position_offsets_perpendicular() {
        use crate::route::{RouteModel, RoutePoint};
        let points: Vec<RoutePoint> = (0..=100)
            .map(|i| RoutePoint {
                distance_m: i as f64 * 10.0,
                lat: 45.0 + (i as f64 * 10.0) / 111_320.0,
                lon: 7.0,
                elevation_m: 100.0,
                grade: 0.0,
            })
            .collect();
        let route = RouteModel::new("r", "r", points).unwrap();
        let mut app = VeloApp::new();
        app.load_route(route);
        app.ride.distance_m = 500.0;

        app.ride.lateral_offset_m = 0.0;
        let (e0, _, n0) = app.steered_position_enu().unwrap();
        app.ride.lateral_offset_m = 2.0;
        let (e1, up, n1) = app.steered_position_enu().unwrap();
        // Route runs north; +offset goes east (right of travel).
        assert!((e1 - e0 - 2.0).abs() < 1e-6, "east offset: {}", e1 - e0);
        assert!((n1 - n0).abs() < 1e-6);
        assert!((up - 0.0).abs() < 1e-9); // relative to origin elevation
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
