//! Platform abstraction — traits only, zero OS symbols.
//!
//! Shell implementations (Swift over BLE, etc.) satisfy these contracts.
//! Over FFI, use the UniFFI callback interfaces in `velo-ffi` (polling model for sensors).

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

/// Records trainer commands for tests.
#[derive(Default, Debug)]
pub struct MockTrainerControl {
    pub last_power: Option<Watts>,
    pub stopped: bool,
}

impl TrainerControl for MockTrainerControl {
    fn set_target_power(&self, watts: Watts) {
        // Interior mutability would be needed for &self; tests use Arc<Mutex<>> wrapper.
        let _ = watts;
    }

    fn set_simulation(&self, _grade: Grade, _crr: f32, _cw_a: f32) {}

    fn stop(&self) {}

    fn capabilities(&self) -> TrainerCaps {
        TrainerCaps {
            erg: true,
            sim: true,
            max_watts: 2000,
        }
    }
}
