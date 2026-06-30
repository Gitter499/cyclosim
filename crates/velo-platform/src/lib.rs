//! Platform abstraction — traits only, zero OS symbols.
//!
//! Shell implementations (Swift over BLE, etc.) satisfy these contracts.
//! Over FFI, use the UniFFI callback interfaces in `velo-ffi` (polling model for sensors).

use std::sync::Mutex;
use std::time::Duration;
use velo_units::{Bpm, Grade, MetersPerSecond, Rpm, Watts};

/// Commands the app sends TO the trainer.
pub trait TrainerControl: Send + Sync {
    fn set_target_power(&self, watts: Watts);
    fn set_simulation(&self, grade: Grade, crr: f32, cw_a: f32);
    fn stop(&self);
    fn capabilities(&self) -> TrainerCaps;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainerCaps {
    pub erg: bool,
    pub sim: bool,
    pub max_watts: u16,
}

/// Telemetry coming FROM trainer + sensors.
pub trait SensorSource: Send + Sync {
    /// Drain pending samples pushed since the last tick.
    fn drain_samples(&mut self) -> Vec<TelemetrySample>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySample {
    pub elapsed: Duration,
    pub power: Option<Watts>,
    pub cadence: Option<Rpm>,
    pub heart_rate: Option<Bpm>,
    pub wheel_speed: Option<MetersPerSecond>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentEnergy {
    Warmup,
    Build,
    Threshold,
    Recovery,
    Cooldown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackIntent {
    Start,
    Transition,
    Duck,
}

pub trait AudioDirector: Send + Sync {
    fn on_segment(&self, energy: SegmentEnergy, intent: PlaybackIntent);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SteerState {
    pub axis: f32,
    pub recenter: bool,
}

pub trait SteeringInput: Send + Sync {
    fn poll(&self) -> SteerState;
}

pub trait Clock: Send + Sync {
    fn elapsed(&self) -> Duration;
}

/// In-memory sensor buffer for tests and headless replay.
#[derive(Default)]
pub struct MockSensorSource {
    pending: Vec<TelemetrySample>,
}

impl MockSensorSource {
    pub fn push(&mut self, sample: TelemetrySample) {
        self.pending.push(sample);
    }
}

impl SensorSource for MockSensorSource {
    fn drain_samples(&mut self) -> Vec<TelemetrySample> {
        std::mem::take(&mut self.pending)
    }
}

/// No-op trainer for tests that only need a valid [`TrainerControl`] impl.
#[derive(Default, Debug, Clone, Copy)]
pub struct MockTrainerControl;

impl TrainerControl for MockTrainerControl {
    fn set_target_power(&self, _: Watts) {}

    fn set_simulation(&self, _: Grade, _: f32, _: f32) {}

    fn stop(&self) {}

    fn capabilities(&self) -> TrainerCaps {
        TrainerCaps {
            erg: true,
            sim: true,
            max_watts: 2000,
        }
    }
}

/// Records trainer commands for assertion in headless tests.
#[derive(Default, Debug)]
pub struct RecordingTrainerControl {
    last_power: Mutex<Option<Watts>>,
    sim_grades: Mutex<Vec<f64>>,
    last_sim: Mutex<Option<(Grade, f32, f32)>>,
    stopped: Mutex<bool>,
}

impl RecordingTrainerControl {
    pub fn last_power(&self) -> Option<Watts> {
        *self.last_power.lock().unwrap()
    }

    pub fn sim_grades(&self) -> Vec<f64> {
        self.sim_grades.lock().unwrap().clone()
    }

    pub fn last_sim(&self) -> Option<(Grade, f32, f32)> {
        *self.last_sim.lock().unwrap()
    }

    pub fn stopped(&self) -> bool {
        *self.stopped.lock().unwrap()
    }
}

impl TrainerControl for RecordingTrainerControl {
    fn set_target_power(&self, watts: Watts) {
        *self.last_power.lock().unwrap() = Some(watts);
    }

    fn set_simulation(&self, grade: Grade, crr: f32, cw_a: f32) {
        self.sim_grades.lock().unwrap().push(grade.0);
        *self.last_sim.lock().unwrap() = Some((grade, crr, cw_a));
    }

    fn stop(&self) {
        *self.stopped.lock().unwrap() = true;
    }

    fn capabilities(&self) -> TrainerCaps {
        TrainerCaps {
            erg: true,
            sim: true,
            max_watts: 2000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_trainer_captures_erg_and_sim() {
        let trainer = RecordingTrainerControl::default();
        trainer.set_target_power(Watts::new(200.0));
        trainer.set_simulation(Grade::new(0.05), 0.004, 0.32);
        trainer.stop();

        assert_eq!(trainer.last_power(), Some(Watts::new(200.0)));
        assert_eq!(trainer.sim_grades(), vec![0.05]);
        assert!(trainer.last_sim().is_some());
        assert!(trainer.stopped());
    }
}
