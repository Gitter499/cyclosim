//! User story: rider selects a climb route, starts a structured workout, rides 60 s
//! with replay sensors, then stops and exports a valid FIT activity.

use std::path::PathBuf;
use std::time::Duration;

use fitparser::profile::MesgNum;
use velo_core::{
    parse_zwo_xml, RideMode, VeloApp, Workout, WorkoutInterval, WorkoutTarget,
};
use velo_platform::{
    MockSensorSource, MockSteeringInput, RecordingAudioDirector, RecordingTrainerControl,
    SegmentEnergy, TelemetrySample,
};
use velo_route_import::{import_gpx, DEFAULT_GRADE_WINDOW_M, DEFAULT_SPACING_M};
use velo_units::Watts;

fn fixture_gpx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../velo-route-import/tests/fixtures/simple_climb.gpx")
}

const THRESHOLD_ZWO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<workout_file>
  <name>Scenario FTP</name>
  <workout>
    <Warmup Duration="30" PowerLow="0.50" PowerHigh="0.55" />
    <SteadyState Duration="30" Power="0.85" />
    <Cooldown Duration="30" PowerHigh="0.55" PowerLow="0.25" />
  </workout>
</workout_file>"#;

fn push_replay_sample(sensors: &mut MockSensorSource, power: f64) {
    sensors.push(TelemetrySample {
        elapsed: Duration::from_millis(0),
        power: Some(Watts::new(power)),
        cadence: Some(velo_units::Rpm::new(90.0)),
        heart_rate: Some(velo_units::Bpm::new(140.0)),
        wheel_speed: None,
    });
}

#[test]
fn user_story_route_workout_ride_stop_exports_fit() {
    let data = std::fs::read(fixture_gpx()).unwrap();
    let route = import_gpx(
        &data,
        "scenario-climb",
        "Scenario Climb",
        DEFAULT_SPACING_M,
        DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();
    assert_eq!(route.meta.route_id, "scenario-climb");
    assert!(route.meta.total_distance_m > 100.0);

    let workout = parse_zwo_xml(THRESHOLD_ZWO).expect("import zwo");
    workout.validate().expect("valid workout");

    let mut app = VeloApp::new();
    app.set_clock_unix(1_700_000_000);
    app.set_ftp(250.0);
    app.load_route(route);
    app.set_segment_music_enabled(true);
    app.start_workout(workout);
    app.start_ride();

    let mut sensors = MockSensorSource::default();
    let trainer = RecordingTrainerControl::default();
    let audio = RecordingAudioDirector::default();

    // 60 s @ 100 Hz with alternating power for highlight planning.
    for i in 0..6000 {
        let power = if (50..55).contains(&(i / 100)) {
            420.0
        } else {
            200.0
        };
        push_replay_sample(&mut sensors, power);
        app.tick(
            &mut sensors,
            &trainer,
            None::<&MockSteeringInput>,
            Some(&audio),
        );
    }

    assert!(app.ride.distance_m > 100.0);
    assert!(app.ride.grade > 0.0, "climb route should report positive grade");
    assert_eq!(app.ride.mode, RideMode::Erg);
    assert!(trainer.last_power().is_some());

    let audio_calls = audio.calls();
    assert!(!audio_calls.is_empty());
    assert_eq!(audio_calls[0].0, SegmentEnergy::Warmup);

    let summary = app.stop_ride().expect("ride summary");
    assert_eq!(summary.sample_count, 6000);
    assert!(summary.elapsed_s >= 59.0);

    let fit = app.export_fit().expect("fit bytes");
    assert_eq!(&fit[8..12], b".FIT");
    let parsed = fitparser::from_bytes(&fit).unwrap();
    let records: Vec<_> = parsed.iter().filter(|m| m.kind() == MesgNum::Record).collect();
    assert_eq!(records.len(), 6000);

    let clips = summary.highlight_clips;
    assert!(!clips.is_empty());
    assert!(clips.iter().any(|c| c.label == "Power surge" || c.label == "Start"));
}

#[test]
fn user_story_free_ride_interval_switches_to_sim_mode() {
    let mut app = VeloApp::new();
    app.set_ftp(200.0);
    app.start_workout(Workout {
        name: "free block".into(),
        intervals: vec![
            WorkoutInterval {
                name: "ERG".into(),
                duration_s: 1.0,
                target: WorkoutTarget::ErgWatts(150.0),
            },
            WorkoutInterval {
                name: "Free".into(),
                duration_s: 2.0,
                target: WorkoutTarget::FreeRide,
            },
        ],
    });

    let mut sensors = MockSensorSource::default();
    let trainer = RecordingTrainerControl::default();

    for _ in 0..80 {
        push_replay_sample(&mut sensors, 150.0);
        app.tick(
            &mut sensors,
            &trainer,
            None::<&MockSteeringInput>,
            None::<&velo_platform::MockAudioDirector>,
        );
    }
    assert_eq!(app.ride.mode, RideMode::Erg);

    for _ in 0..120 {
        push_replay_sample(&mut sensors, 180.0);
        app.tick(
            &mut sensors,
            &trainer,
            None::<&MockSteeringInput>,
            None::<&velo_platform::MockAudioDirector>,
        );
    }
    assert_eq!(app.ride.mode, RideMode::Sim);
    assert!(trainer.last_sim().is_some());
}
