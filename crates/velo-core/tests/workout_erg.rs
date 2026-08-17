//! Workout engine drives ERG targets during structured rides.

use std::time::Duration;
use velo_core::{RideMode, VeloApp, Workout, WorkoutInterval, WorkoutTarget};
use velo_platform::{MockSensorSource, RecordingTrainerControl, TelemetrySample};
use velo_units::Watts;

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
    let trainer = RecordingTrainerControl::default();

    for _ in 0..100 {
        sensors.push(TelemetrySample {
            elapsed: Duration::from_millis(0),
            power: Some(Watts::new(100.0)),
            cadence: None,
            heart_rate: None,
            wheel_speed: None,
        });
        app.tick(
            &mut sensors,
            &trainer,
            None::<&velo_platform::MockSteeringInput>,
            None::<&velo_platform::MockAudioDirector>,
        );
    }
    assert_eq!(trainer.last_power(), Some(Watts::new(100.0)));

    for _ in 0..100 {
        app.tick(
            &mut sensors,
            &trainer,
            None::<&velo_platform::MockSteeringInput>,
            None::<&velo_platform::MockAudioDirector>,
        );
    }
    assert_eq!(trainer.last_power(), Some(Watts::new(250.0)));
    assert_eq!(app.ride.mode, RideMode::Erg);
}
