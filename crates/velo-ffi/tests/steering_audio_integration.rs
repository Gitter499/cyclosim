//! M6 FFI: steering axis affects ride state; workout intervals notify audio director.

#[path = "common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use velo_ffi::{
    PlaybackIntentDto, SegmentEnergyDto, VeloHandle, WorkoutDto, WorkoutIntervalDto,
    WorkoutTargetDto,
};
use velo_route_import::import_gpx;

use common::{MockSteering, NoopSteering, NoopTrainer, RecordingAudioDirectorCallback, TickSensors};

fn fixture_gpx() -> PathBuf {
    common::fixture_gpx_path()
}

fn route_handle() -> VeloHandle {
    let temp = tempfile::tempdir().unwrap();
    let packs_dir = temp.path().join("packs");
    std::fs::create_dir_all(&packs_dir).unwrap();
    let pack_dir = packs_dir.join("steer-climb");

    let data = std::fs::read(fixture_gpx()).unwrap();
    let model = import_gpx(
        &data,
        "steer-climb",
        "Steer Climb",
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
        .set_active_route("steer-climb".into())
        .expect("set route");
    handle
}

#[test]
fn steering_callback_updates_ride_state_dto() {
    let handle = route_handle();
    handle.set_steering_enabled(true);

    for _ in 0..50 {
        handle.tick(
            Box::new(TickSensors { elapsed_ms: 33 }),
            Box::new(NoopTrainer),
            Box::new(MockSteering { axis: 1.0 }),
        );
    }

    let state = handle.ride_state();
    assert!(
        state.steer_axis > 0.0 || state.steer_yaw_rad > 0.0,
        "expected steering to affect state, got axis={} yaw={}",
        state.steer_axis,
        state.steer_yaw_rad
    );
}

#[test]
fn workout_interval_fires_audio_director_callback() {
    let handle = VeloHandle::with_packs_dir_for_tests(
        std::env::temp_dir().join(format!("velo-audio-test-{}", std::process::id())),
    );
    handle.set_segment_music_enabled(true);

    let calls = Arc::new(Mutex::new(Vec::new()));
    handle.set_audio_director(Box::new(RecordingAudioDirectorCallback {
        calls: Arc::clone(&calls),
    }));

    handle
        .start_workout(WorkoutDto {
            name: "audio-test".into(),
            intervals: vec![
                WorkoutIntervalDto {
                    name: "Warmup".into(),
                    duration_s: 0.5,
                    target: WorkoutTargetDto::FtpPercent { percent: 55.0 },
                },
                WorkoutIntervalDto {
                    name: "Threshold".into(),
                    duration_s: 10.0,
                    target: WorkoutTargetDto::FtpPercent { percent: 95.0 },
                },
            ],
        })
        .expect("start workout");

    for ms in (0..1200).step_by(10) {
        handle.tick(
            Box::new(TickSensors { elapsed_ms: ms }),
            Box::new(NoopTrainer),
            Box::new(NoopSteering),
        );
    }

    let recorded = calls.lock().unwrap();
    assert!(!recorded.is_empty());
    assert_eq!(recorded[0].0, SegmentEnergyDto::Warmup);
    assert_eq!(recorded[0].1, PlaybackIntentDto::Start);
    assert!(
        recorded.iter().any(|(e, i)| {
            *e == SegmentEnergyDto::Threshold && *i == PlaybackIntentDto::Transition
        }),
        "expected threshold transition, got {recorded:?}"
    );
}
