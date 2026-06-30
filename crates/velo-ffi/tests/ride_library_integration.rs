#[path = "common/mod.rs"]
mod common;

use velo_ffi::{PublishStatus, VeloHandle};

use common::{MockMedia, MockPublisher, NoopTrainer, TickSensors};

#[test]
fn finish_ride_persists_to_library() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("rides.db");
    let artifacts = dir.path().join("artifacts");

    let handle = VeloHandle::new();
    handle
        .configure_ride_library(
            db.display().to_string(),
            artifacts.display().to_string(),
        )
        .unwrap();

    handle.start_ride();
    let mut elapsed_ms = 0u64;
    for _ in 0..8 {
        elapsed_ms += 33;
        handle.tick(
            Box::new(TickSensors { elapsed_ms }),
            Box::new(NoopTrainer),
        );
    }

    let result = handle
        .finish_ride_and_publish(
            Box::new(MockMedia),
            Box::new(MockPublisher {
                saved_locally: true,
                activity_url: "/tmp/ride-folder".into(),
            }),
        )
        .expect("finish ride");

    assert!(!result.ride_id.is_empty());
    let rides = handle.list_rides().unwrap();
    assert_eq!(rides.len(), 1);
    assert_eq!(rides[0].id, result.ride_id);
    assert_eq!(rides[0].publish_status, PublishStatus::Local);
}

#[test]
fn strava_publish_records_activity_id() {
    let dir = tempfile::TempDir::new().unwrap();
    let handle = VeloHandle::new();
    handle
        .configure_ride_library(
            dir.path().join("rides.db").display().to_string(),
            dir.path().join("artifacts").display().to_string(),
        )
        .unwrap();

    handle.start_ride();
    for ms in (33..=264).step_by(33) {
        handle.tick(
            Box::new(TickSensors { elapsed_ms: ms }),
            Box::new(NoopTrainer),
        );
    }

    let result = handle
        .finish_ride_and_publish(
            Box::new(MockMedia),
            Box::new(MockPublisher {
                saved_locally: false,
                activity_url: "https://www.strava.com/activities/998877".into(),
            }),
        )
        .unwrap();

    let ride = handle.get_ride(result.ride_id).unwrap().unwrap();
    assert_eq!(ride.publish_status, PublishStatus::Strava);
    assert_eq!(ride.strava_activity_id.as_deref(), Some("998877"));
}

#[test]
fn finish_ride_plans_and_encodes_highlight_clips() {
    let dir = tempfile::TempDir::new().unwrap();
    let handle = VeloHandle::new();
    handle
        .configure_ride_library(
            dir.path().join("rides.db").display().to_string(),
            dir.path().join("artifacts").display().to_string(),
        )
        .unwrap();

    handle.start_ride();
    for ms in (33..=330).step_by(33) {
        handle.tick(
            Box::new(TickSensors { elapsed_ms: ms }),
            Box::new(NoopTrainer),
        );
    }

    let result = handle
        .finish_ride_and_publish(
            Box::new(MockMedia),
            Box::new(MockPublisher {
                saved_locally: true,
                activity_url: "/tmp/ride-folder".into(),
            }),
        )
        .expect("finish ride");

    let summary = handle.last_ride_summary().unwrap();
    assert!(!summary.highlight_clips.is_empty());
    assert!(result.highlight_clip_path.is_some());
    let clip_path = result.highlight_clip_path.unwrap();
    assert!(std::path::Path::new(&clip_path).exists());

    let ride = handle.get_ride(result.ride_id).unwrap().unwrap();
    assert_eq!(ride.highlight_clip_path.as_deref(), Some(clip_path.as_str()));
}
