//! Steering and segment-aware audio integration in the ride loop.

use velo_core::{VeloApp, Workout};
use velo_platform::{
    MockAudioDirector, MockSensorSource, MockSteeringInput, MockTrainerControl,
    PlaybackIntent, RecordingAudioDirector, SegmentEnergy,
};
use velo_core::workout::{WorkoutInterval, WorkoutTarget};

#[test]
fn steering_axis_affects_yaw_on_route() {
    let mut app = VeloApp::new();
    app.set_steering_enabled(true);
    app.load_route(sample_route());

    let steering = MockSteeringInput::with_axis(1.0);
    let mut sensors = MockSensorSource::default();
    let trainer = MockTrainerControl;

    for _ in 0..300 {
        app.tick(
            &mut sensors,
            &trainer,
            Some(&steering),
            None::<&MockAudioDirector>,
        );
    }

    assert!(app.steer_yaw_rad() > 0.05);
    assert!(app.steer_axis() > 0.0);
}

#[test]
fn workout_interval_boundary_notifies_audio_director() {
    let mut app = VeloApp::new();
    app.set_segment_music_enabled(true);
    app.start_workout(Workout {
        name: "short".into(),
        intervals: vec![
            WorkoutInterval {
                name: "Warmup".into(),
                duration_s: 0.5,
                target: WorkoutTarget::FtpPercent(55.0),
            },
            WorkoutInterval {
                name: "Threshold".into(),
                duration_s: 10.0,
                target: WorkoutTarget::FtpPercent(95.0),
            },
        ],
    });

    let audio = RecordingAudioDirector::default();
    let mut sensors = MockSensorSource::default();
    let trainer = MockTrainerControl;

    for _ in 0..120 {
        app.tick(
            &mut sensors,
            &trainer,
            None::<&MockSteeringInput>,
            Some(&audio),
        );
    }

    let calls = audio.calls();
    assert!(!calls.is_empty());
    assert_eq!(calls[0].0, SegmentEnergy::Warmup);
    assert_eq!(calls[0].1, PlaybackIntent::Start);
    assert!(
        calls.iter().any(|(e, i)| {
            *e == SegmentEnergy::Threshold && *i == PlaybackIntent::Transition
        }),
        "expected threshold transition, got {calls:?}"
    );
}

fn sample_route() -> velo_core::RouteModel {
    velo_core::RouteModel::new(
        "test",
        "Test",
        vec![
            velo_core::RoutePoint {
                distance_m: 0.0,
                lat: 37.0,
                lon: -122.0,
                elevation_m: 0.0,
                grade: 0.0,
            },
            velo_core::RoutePoint {
                distance_m: 500.0,
                lat: 37.001,
                lon: -122.0,
                elevation_m: 0.0,
                grade: 0.0,
            },
        ],
    )
    .expect("valid route")
}
