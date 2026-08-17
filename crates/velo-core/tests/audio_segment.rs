//! Workout interval boundaries map to segment energy for Apple Music (M6).

use velo_core::audio::{playback_intent_for_index, segment_energy_for_interval};
use velo_core::workout::{Workout, WorkoutEngine, WorkoutInterval, WorkoutTarget};
use velo_platform::{PlaybackIntent, RecordingAudioDirector, SegmentEnergy, MockSensorSource, MockSteeringInput, MockTrainerControl};
use velo_core::VeloApp;

#[test]
fn interval_names_map_to_energy_buckets() {
    let cases = [
        ("Warmup", WorkoutTarget::FtpPercent(55.0), SegmentEnergy::Warmup),
        ("Recovery", WorkoutTarget::FtpPercent(50.0), SegmentEnergy::Recovery),
        ("Cooldown", WorkoutTarget::FtpPercent(40.0), SegmentEnergy::Cooldown),
        ("Build", WorkoutTarget::FtpPercent(80.0), SegmentEnergy::Build),
        ("Threshold", WorkoutTarget::FtpPercent(95.0), SegmentEnergy::Threshold),
        ("Block", WorkoutTarget::FtpPercent(95.0), SegmentEnergy::Threshold),
        ("Rest", WorkoutTarget::FtpPercent(45.0), SegmentEnergy::Recovery),
    ];
    for (name, target, expected) in cases {
        let interval = WorkoutInterval {
            name: name.into(),
            duration_s: 60.0,
            target,
        };
        assert_eq!(
            segment_energy_for_interval(&interval, 250.0),
            expected,
            "name={name}"
        );
    }
}

#[test]
fn erg_watts_derives_ftp_percent_for_energy() {
    let interval = WorkoutInterval {
        name: "Hard".into(),
        duration_s: 60.0,
        target: WorkoutTarget::ErgWatts(237.5),
    };
    assert_eq!(
        segment_energy_for_interval(&interval, 250.0),
        SegmentEnergy::Threshold
    );
}

#[test]
fn playback_intent_first_vs_transition() {
    assert_eq!(playback_intent_for_index(0), PlaybackIntent::Start);
    assert_eq!(playback_intent_for_index(1), PlaybackIntent::Transition);
    assert_eq!(playback_intent_for_index(3), PlaybackIntent::Transition);
}

#[test]
fn user_story_workout_boundary_fires_audio_at_each_interval() {
    let mut app = VeloApp::new();
    app.set_ftp(200.0);
    app.set_segment_music_enabled(true);
    app.start_workout(Workout {
        name: "3-step".into(),
        intervals: vec![
            WorkoutInterval {
                name: "Warmup".into(),
                duration_s: 0.5,
                target: WorkoutTarget::FtpPercent(55.0),
            },
            WorkoutInterval {
                name: "Build".into(),
                duration_s: 0.5,
                target: WorkoutTarget::FtpPercent(80.0),
            },
            WorkoutInterval {
                name: "Cooldown".into(),
                duration_s: 0.5,
                target: WorkoutTarget::FtpPercent(45.0),
            },
        ],
    });

    let audio = RecordingAudioDirector::default();
    let mut sensors = MockSensorSource::default();
    let trainer = MockTrainerControl;

    for _ in 0..200 {
        app.tick(
            &mut sensors,
            &trainer,
            None::<&MockSteeringInput>,
            Some(&audio),
        );
    }

    let calls = audio.calls();
    assert!(calls.len() >= 3, "expected at least 3 segment notifications, got {calls:?}");
    assert_eq!(calls[0], (SegmentEnergy::Warmup, PlaybackIntent::Start));
    assert!(
        calls.iter().any(|c| c.0 == SegmentEnergy::Build && c.1 == PlaybackIntent::Transition)
    );
    assert!(
        calls.iter().any(|c| c.0 == SegmentEnergy::Cooldown && c.1 == PlaybackIntent::Transition)
    );
}

#[test]
fn resync_segment_music_refires_current_interval() {
    let mut app = VeloApp::new();
    app.set_ftp(200.0);
    app.set_segment_music_enabled(true);
    app.start_workout(Workout {
        name: "resync".into(),
        intervals: vec![WorkoutInterval {
            name: "Warmup".into(),
            duration_s: 10.0,
            target: WorkoutTarget::FtpPercent(55.0),
        }],
    });
    let audio = RecordingAudioDirector::default();
    let mut sensors = MockSensorSource::default();
    let trainer = MockTrainerControl;
    app.tick(&mut sensors, &trainer, None::<&MockSteeringInput>, Some(&audio));
    assert_eq!(audio.calls().len(), 1);
    app.resync_segment_music();
    app.tick(&mut sensors, &trainer, None::<&MockSteeringInput>, Some(&audio));
    assert_eq!(audio.calls().len(), 2);
}

#[test]
fn enabling_segment_music_resyncs_pending_interval() {
    let mut app = VeloApp::new();
    app.set_ftp(200.0);
    app.start_workout(Workout {
        name: "enable".into(),
        intervals: vec![WorkoutInterval {
            name: "Build".into(),
            duration_s: 10.0,
            target: WorkoutTarget::FtpPercent(80.0),
        }],
    });
    let audio = RecordingAudioDirector::default();
    let mut sensors = MockSensorSource::default();
    let trainer = MockTrainerControl;
    for _ in 0..5 {
        app.tick(&mut sensors, &trainer, None::<&MockSteeringInput>, None::<&RecordingAudioDirector>);
    }
    app.set_segment_music_enabled(true);
    app.tick(&mut sensors, &trainer, None::<&MockSteeringInput>, Some(&audio));
    assert_eq!(audio.calls().len(), 1);
}

#[test]
fn workout_engine_advances_interval_index_at_boundary() {
    let mut engine = WorkoutEngine::new(
        Workout {
            name: "ftp".into(),
            intervals: vec![
                WorkoutInterval {
                    name: "A".into(),
                    duration_s: 1.0,
                    target: WorkoutTarget::FtpPercent(70.0),
                },
                WorkoutInterval {
                    name: "B".into(),
                    duration_s: 1.0,
                    target: WorkoutTarget::FtpPercent(95.0),
                },
            ],
        },
        250.0,
    );
    assert_eq!(engine.target_watts(), Some(velo_units::Watts::new(175.0)));
    for _ in 0..101 {
        engine.tick(0.01);
    }
    assert_eq!(engine.state().interval_index, 1);
    assert_eq!(engine.target_watts(), Some(velo_units::Watts::new(237.5)));
}
