//! FFI round-trip for structured workouts: start, tick, ERG target, live state.

use std::sync::{Arc, Mutex};

use velo_ffi::{SensorSourceCallback, TrainerControlCallback, VeloHandle};

struct EmptySensors;

impl SensorSourceCallback for EmptySensors {
    fn poll_samples(&self) -> Vec<velo_ffi::TelemetrySampleDto> {
        vec![]
    }
}

struct RecordingTrainer {
    last_power: Arc<Mutex<Option<f64>>>,
}

impl TrainerControlCallback for RecordingTrainer {
    fn set_target_power(&self, watts: f64) {
        *self.last_power.lock().unwrap() = Some(watts);
    }

    fn set_simulation(&self, _: f64, _: f64, _: f64) {}
    fn stop(&self) {}
}

#[test]
fn workout_live_state_round_trip() {
    let handle = VeloHandle::new();
    assert!(!handle.workout_active());

    handle.set_ftp(200.0);
    assert!((handle.ftp() - 200.0).abs() < f64::EPSILON);

    handle.start_sample_workout();
    assert!(handle.workout_active());

    let live = handle.workout_live();
    assert!(live.active);
    assert_eq!(live.workout_name, "2x20 Threshold");
    assert_eq!(live.interval_name, "Warmup");
    // 55% of 200 W FTP
    assert_eq!(live.target_watts, Some(110.0));

    let last_power = Arc::new(Mutex::new(None));
    let trainer = RecordingTrainer {
        last_power: Arc::clone(&last_power),
    };
    handle.tick(Box::new(EmptySensors), Box::new(trainer));
    assert_eq!(*last_power.lock().unwrap(), Some(110.0));

    handle.clear_workout();
    assert!(!handle.workout_active());
    assert!(!handle.workout_live().active);
}
