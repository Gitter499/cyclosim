//! FFI callback wiring: sensor samples reach ride state; trainer commands echo ERG/SIM.

use std::sync::{Arc, Mutex};

use velo_ffi::{
    RideMode, SensorSourceCallback, TelemetrySampleDto, TrainerControlCallback, VeloHandle,
};

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

struct RecordingTrainer {
    last_power: Arc<Mutex<Option<f64>>>,
    last_sim: Arc<Mutex<Option<(f64, f64, f64)>>>,
}

impl TrainerControlCallback for RecordingTrainer {
    fn set_target_power(&self, watts: f64) {
        *self.last_power.lock().unwrap() = Some(watts);
    }

    fn set_simulation(&self, grade: f64, crr: f64, cwa: f64) {
        *self.last_sim.lock().unwrap() = Some((grade, crr, cwa));
    }

    fn stop(&self) {}
}

struct NoopTrainer;

impl TrainerControlCallback for NoopTrainer {
    fn set_target_power(&self, _: f64) {}
    fn set_simulation(&self, _: f64, _: f64, _: f64) {}
    fn stop(&self) {}
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
    let trainer = RecordingTrainer {
        last_power: Arc::clone(&last_power),
        last_sim: Arc::clone(&last_sim),
    };

    handle.tick(Box::new(EmptySensors), Box::new(trainer));

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
    let trainer = RecordingTrainer {
        last_power: Arc::clone(&last_power),
        last_sim: Arc::clone(&last_sim),
    };

    handle.tick(Box::new(EmptySensors), Box::new(trainer));

    let sim = last_sim.lock().unwrap().unwrap();
    assert!((sim.0 - 0.05).abs() < 1e-9);
}
