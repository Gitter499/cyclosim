//! Workout engine drives ERG targets during structured rides.

use std::time::Duration;
use velo_core::{RideMode, VeloApp, Workout, WorkoutInterval, WorkoutTarget};
use velo_platform::{MockSensorSource, TrainerControl, TelemetrySample};
use velo_units::{Watts};

struct RecordingTrainer {
    last: std::sync::Mutex<Option<f64>>,
}

impl TrainerControl for RecordingTrainer {
    fn set_target_power(&self, watts: Watts) {
        *self.last.lock().unwrap() = Some(watts.0);
    }
    fn set_simulation(
        &self,
        _: velo_units::Grade,
        _: f32,
        _: f32,
    ) {
    }
    fn stop(&self) {}
    fn capabilities(&self) -> velo_platform::TrainerCaps {
        velo_platform::TrainerCaps {
            erg: true,
            sim: true,
            max_watts: 2000,
        }
    }
}

#[test]
fn workout_advances_erg_target_during_ride() {
    let mut app = VeloApp::new();
    app.set_ftp(200.0);
    app.start_workout(Workout {
        name: "erg steps".into(),
        intervals: vec![
            WorkoutInterval {
                name: "low".into(),
                duration_s: 1.0,
                target: WorkoutTarget::ErgWatts(100.0),
            },
            WorkoutInterval {
                name: "high".into(),
                duration_s: 1.0,
                target: WorkoutTarget::ErgWatts(250.0),
            },
        ],
    });

    let mut sensors = MockSensorSource::default();
    let trainer = RecordingTrainer {
        last: std::sync::Mutex::new(None),
    };

    for _ in 0..100 {
        sensors.push(TelemetrySample {
            elapsed: Duration::from_millis(0),
            power: Some(Watts::new(100.0)),
            cadence: None,
            heart_rate: None,
            wheel_speed: None,
        });
        app.tick(&mut sensors, &trainer);
    }
    assert_eq!(*trainer.last.lock().unwrap(), Some(100.0));

    for _ in 0..100 {
        app.tick(&mut sensors, &trainer);
    }
    assert_eq!(*trainer.last.lock().unwrap(), Some(250.0));
    assert_eq!(app.ride.mode, RideMode::Erg);
}
