//! FFI round-trip for structured workouts: start, tick, ERG target, live state.

#[path = "common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use velo_ffi::{
    SensorSourceCallback, VeloError, VeloHandle, WorkoutDto, WorkoutIntervalDto, WorkoutTargetDto,
};

use common::{RecordingTrainerCallback};

struct EmptySensors;

impl SensorSourceCallback for EmptySensors {
    fn poll_samples(&self) -> Vec<velo_ffi::TelemetrySampleDto> {
        vec![]
    }
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
    let trainer = RecordingTrainerCallback {
        last_power: Arc::clone(&last_power),
        last_sim: Arc::new(Mutex::new(None)),
    };
    handle.tick(Box::new(EmptySensors), Box::new(trainer));
    assert_eq!(*last_power.lock().unwrap(), Some(110.0));

    handle.clear_workout();
    assert!(!handle.workout_active());
    assert!(!handle.workout_live().active);
}

#[test]
fn custom_workout_start_and_erg_tick() {
    let handle = VeloHandle::new();
    handle.set_ftp(200.0);

    let workout = WorkoutDto {
        name: "Custom ERG steps".into(),
        intervals: vec![
            WorkoutIntervalDto {
                name: "Step 1".into(),
                duration_s: 60.0,
                target: WorkoutTargetDto::ErgWatts { watts: 150.0 },
            },
            WorkoutIntervalDto {
                name: "Step 2".into(),
                duration_s: 60.0,
                target: WorkoutTargetDto::ErgWatts { watts: 220.0 },
            },
        ],
    };

    handle.start_workout(workout).expect("valid workout");
    assert!(handle.workout_active());

    let live = handle.workout_live();
    assert_eq!(live.workout_name, "Custom ERG steps");
    assert_eq!(live.interval_name, "Step 1");
    assert_eq!(live.target_watts, Some(150.0));

    let last_power = Arc::new(Mutex::new(None));
    let trainer = RecordingTrainerCallback {
        last_power: Arc::clone(&last_power),
        last_sim: Arc::new(Mutex::new(None)),
    };
    handle.tick(Box::new(EmptySensors), Box::new(trainer));
    assert_eq!(*last_power.lock().unwrap(), Some(150.0));
}

#[test]
fn start_workout_rejects_empty_intervals() {
    let handle = VeloHandle::new();
    let err = handle
        .start_workout(WorkoutDto {
            name: "Empty".into(),
            intervals: vec![],
        })
        .expect_err("empty intervals");
    assert!(matches!(err, VeloError::RideError { .. }));
}
