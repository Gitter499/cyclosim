//! FFI callback wiring: sensor samples reach ride state; trainer commands echo ERG/SIM.

#[path = "common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use velo_ffi::{
    RideMode, SensorSourceCallback, TelemetrySampleDto, VeloHandle,
};

use common::{NoopSteering, NoopTrainer, RecordingTrainerCallback};

struct TickSensors {
    sample: TelemetrySampleDto,
}

impl SensorSourceCallback for TickSensors {
    fn poll_samples(&self) -> Vec<TelemetrySampleDto> {
        vec![self.sample.clone()]
    }
}

struct EmptySensors;

impl SensorSourceCallback for EmptySensors {
    fn poll_samples(&self) -> Vec<TelemetrySampleDto> {
        vec![TelemetrySampleDto::default()]
    }
}

#[test]
fn sensor_samples_update_ride_state() {
    let handle = VeloHandle::new();
    handle.set_target_power(200.0);

    handle.tick(
        Box::new(TickSensors {
            sample: TelemetrySampleDto {
                elapsed_ms: 100,
                power_w: Some(195.0),
                cadence_rpm: Some(88.0),
                heart_rate_bpm: Some(142.0),
                wheel_speed_mps: None,
            },
        }),
        Box::new(NoopTrainer),
        Box::new(NoopSteering),
    );

    let state = handle.ride_state();
    assert_eq!(state.power_w, Some(195.0));
    assert_eq!(state.cadence_rpm, Some(88.0));
    assert_eq!(state.heart_rate_bpm, Some(142.0));
    assert!(state.distance_m > 0.0);
}

#[test]
fn erg_mode_forwards_target_power_to_trainer() {
    let handle = VeloHandle::new();
    handle.set_ride_mode(RideMode::Erg);
    handle.set_target_power(225.0);

    let last_power = Arc::new(Mutex::new(None));
    let last_sim = Arc::new(Mutex::new(None));
    let trainer = RecordingTrainerCallback {
        last_power: Arc::clone(&last_power),
        last_sim: Arc::clone(&last_sim),
    };

    handle.tick(Box::new(EmptySensors), Box::new(trainer), Box::new(NoopSteering));

    assert_eq!(*last_power.lock().unwrap(), Some(225.0));
    assert!(last_sim.lock().unwrap().is_none());
}

#[test]
fn sim_mode_forwards_grade_to_trainer() {
    let handle = VeloHandle::new();
    handle.set_ride_mode(RideMode::Sim);
    handle.set_grade(0.05);

    let last_power = Arc::new(Mutex::new(None));
    let last_sim = Arc::new(Mutex::new(None));
    let trainer = RecordingTrainerCallback {
        last_power: Arc::clone(&last_power),
        last_sim: Arc::clone(&last_sim),
    };

    handle.tick(Box::new(EmptySensors), Box::new(trainer), Box::new(NoopSteering));

    let sim = last_sim.lock().unwrap().unwrap();
    assert!((sim.0 - 0.05).abs() < 1e-9);
}
