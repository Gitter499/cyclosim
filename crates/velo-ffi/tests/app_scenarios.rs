//! End-to-end app scenarios without BLE hardware — mirrors rider flows via FFI mocks.

#[path = "common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use fitparser::profile::MesgNum;
use velo_ffi::{
    PlaybackIntentDto, RideMode, SegmentEnergyDto, VeloHandle, WorkoutDto, WorkoutIntervalDto,
    WorkoutTargetDto,
};
use velo_route_import::import_gpx;

use common::{
    fixture_gpx_path, MockMedia, MockPublisher, MockSteering, NoopSteering, NoopTrainer,
    RecordingAudioDirectorCallback, RecordingTrainerCallback, ReplaySensors,
};

fn route_handle_with_library(dir: &tempfile::TempDir) -> VeloHandle {
    let packs_dir = dir.path().join("packs");
    std::fs::create_dir_all(&packs_dir).unwrap();
    let pack_dir = packs_dir.join("app-scenario-climb");

    let data = std::fs::read(fixture_gpx_path()).unwrap();
    let model = import_gpx(
        &data,
        "app-scenario-climb",
        "App Scenario Climb",
        velo_route_import::DEFAULT_SPACING_M,
        velo_route_import::DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();
    model.save_pack(&pack_dir).unwrap();
    velo_terrain::bake_terrain_for_route(
        &model,
        &pack_dir,
        velo_terrain::DEFAULT_CORRIDOR_M,
        velo_terrain::DEFAULT_CELL_M,
    )
    .unwrap();

    let handle = VeloHandle::with_packs_dir_for_tests(packs_dir);
    handle
        .configure_ride_library(
            dir.path().join("rides.db").display().to_string(),
            dir.path().join("artifacts").display().to_string(),
        )
        .unwrap();
    handle
        .set_active_route("app-scenario-climb".into())
        .expect("set route");
    handle
}

#[test]
fn user_story_import_route_sim_ride_publish_to_library() {
    let dir = tempfile::tempdir().unwrap();
    let handle = route_handle_with_library(&dir);
    handle.set_ride_mode(RideMode::Sim);

    let publisher = MockPublisher::local("/tmp/scenario-ride");
    let publish_count = Arc::clone(&publisher.publish_count);
    let media = MockMedia::default();

    handle.start_ride();
    let tick = Arc::new(Mutex::new(0u64));
    for _ in 0..2000 {
        handle.tick(
            Box::new(ReplaySensors {
                tick: Arc::clone(&tick),
                power_w: 180.0,
                step_ms: 10,
            }),
            Box::new(NoopTrainer),
            Box::new(NoopSteering),
        );
    }

    let result = handle
        .finish_ride_and_publish(Box::new(media), Box::new(publisher))
        .expect("finish ride");

    assert!(!result.ride_id.is_empty());
    assert_eq!(*publish_count.lock().unwrap(), 1);

    let summary = handle.last_ride_summary().unwrap();
    assert!(summary.sample_count >= 1500);
    assert!(summary.distance_m > 10.0);
    assert!(!summary.highlight_clips.is_empty());

    let fit = handle.export_fit().expect("fit export");
    assert_eq!(&fit[8..12], b".FIT");
    let parsed = fitparser::from_bytes(&fit).unwrap();
    assert!(parsed.iter().any(|m| m.kind() == MesgNum::Session));

    let rides = handle.list_rides().unwrap();
    assert_eq!(rides.len(), 1);
    assert_eq!(rides[0].id, result.ride_id);

    let ride = handle.get_ride(result.ride_id).unwrap().unwrap();
    assert!(ride.highlight_clip_path.is_some());
    assert!(std::path::Path::new(ride.highlight_clip_path.as_ref().unwrap()).exists());
}

#[test]
fn user_story_workout_ride_erg_transitions_and_audio_at_boundaries() {
    let handle = VeloHandle::new();
    handle.set_ftp(200.0);
    handle.set_segment_music_enabled(true);

    let calls = Arc::new(Mutex::new(Vec::new()));
    handle.set_audio_director(Box::new(RecordingAudioDirectorCallback {
        calls: Arc::clone(&calls),
    }));

    handle
        .start_workout(WorkoutDto {
            name: "Interval steps".into(),
            intervals: vec![
                WorkoutIntervalDto {
                    name: "Warmup".into(),
                    duration_s: 0.5,
                    target: WorkoutTargetDto::FtpPercent { percent: 55.0 },
                },
                WorkoutIntervalDto {
                    name: "Threshold".into(),
                    duration_s: 0.5,
                    target: WorkoutTargetDto::FtpPercent { percent: 95.0 },
                },
            ],
        })
        .expect("start workout");

    let last_power = Arc::new(Mutex::new(None));
    let last_sim = Arc::new(Mutex::new(None));

    handle.start_ride();
    for _ in 0..40 {
        handle.tick(
            Box::new(ReplaySensors::at_180w()),
            Box::new(RecordingTrainerCallback {
                last_power: Arc::clone(&last_power),
                last_sim: Arc::clone(&last_sim),
            }),
            Box::new(NoopSteering),
        );
    }
    assert_eq!(*last_power.lock().unwrap(), Some(110.0));

    for _ in 0..60 {
        handle.tick(
            Box::new(ReplaySensors::at_180w()),
            Box::new(RecordingTrainerCallback {
                last_power: Arc::clone(&last_power),
                last_sim: Arc::clone(&last_sim),
            }),
            Box::new(NoopSteering),
        );
    }
    assert_eq!(*last_power.lock().unwrap(), Some(190.0));

    let recorded = calls.lock().unwrap();
    assert_eq!(recorded[0].0, SegmentEnergyDto::Warmup);
    assert_eq!(recorded[0].1, PlaybackIntentDto::Start);
    assert!(
        recorded.iter().any(|(e, i)| {
            *e == SegmentEnergyDto::Threshold && *i == PlaybackIntentDto::Transition
        }),
        "expected threshold transition, got {recorded:?}"
    );
}

#[test]
fn user_story_steering_axis_updates_ride_state_dto() {
    let dir = tempfile::tempdir().unwrap();
    let handle = route_handle_with_library(&dir);
    handle.set_steering_enabled(true);

    for _ in 0..100 {
        handle.tick(
            Box::new(ReplaySensors::at_180w()),
            Box::new(NoopTrainer),
            Box::new(MockSteering { axis: 0.75 }),
        );
    }

    let state = handle.ride_state();
    assert!(
        state.steer_axis > 0.3,
        "filtered axis should reflect input, got {}",
        state.steer_axis
    );
    assert!(
        state.steer_yaw_rad > 0.05,
        "yaw should integrate, got {}",
        state.steer_yaw_rad
    );
}
