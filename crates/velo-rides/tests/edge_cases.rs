#[path = "common/mod.rs"]
mod common;

use velo_rides::{NewRideRecord, PublishStatus, RideStoreError};

#[test]
fn invalid_id_rejected_on_get() {
    let (lib, _dir) = common::temp_library();
    let err = lib.get_ride("not-a-uuid").unwrap_err();
    assert!(matches!(err, RideStoreError::InvalidId(_)));
}

#[test]
fn null_optional_fields_round_trip() {
    let (lib, _dir) = common::temp_library();
    let artifacts = lib.save_ride_artifacts(b"fit", None).unwrap();
    lib.insert_ride_with_id(
        &artifacts.ride_id,
        NewRideRecord {
            started_at_unix: 1,
            elapsed_s: 1.0,
            distance_m: 1.0,
            avg_power_w: None,
            max_power_w: None,
            fit_path: artifacts.fit_path.display().to_string(),
            screenshot_path: None,
            strava_activity_id: None,
            publish_status: PublishStatus::Failed,
            route_id: None,
        },
    )
    .unwrap();

    let ride = lib.get_ride(&artifacts.ride_id).unwrap().unwrap();
    assert!(ride.avg_power_w.is_none());
    assert!(ride.screenshot_path.is_none());
    assert!(ride.route_id.is_none());
    assert_eq!(ride.publish_status, PublishStatus::Failed);
}

#[test]
fn long_paths_stored_and_retrieved() {
    let (lib, _dir) = common::temp_library();
    let long_segment = "a".repeat(200);
    let long_path = format!("/tmp/{long_segment}/ride.fit");
    let id = lib.insert_ride(common::sample_record(&long_path)).unwrap();
    let ride = lib.get_ride(&id).unwrap().unwrap();
    assert_eq!(ride.fit_path, long_path);
}

#[test]
fn duplicate_insert_same_id_fails() {
    let (lib, _dir) = common::temp_library();
    let artifacts = lib.save_ride_artifacts(b"fit", None).unwrap();
    let record = common::sample_record(&artifacts.fit_path.display().to_string());
    lib.insert_ride_with_id(&artifacts.ride_id, record.clone())
        .unwrap();
    let err = lib.insert_ride_with_id(&artifacts.ride_id, record).unwrap_err();
    assert!(matches!(err, RideStoreError::Sqlite(_)));
}

#[test]
fn concurrent_reads_from_shared_library() {
    use std::sync::Arc;
    use std::thread;

    let (lib, _dir) = common::temp_library();
    let artifacts = lib.save_ride_artifacts(b"fit", None).unwrap();
    lib.insert_ride_with_id(
        &artifacts.ride_id,
        common::sample_record(&artifacts.fit_path.display().to_string()),
    )
    .unwrap();

    let shared = Arc::new(lib);
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let lib = Arc::clone(&shared);
            let id = artifacts.ride_id.clone();
            thread::spawn(move || lib.get_ride(&id).unwrap().is_some())
        })
        .collect();

    for h in handles {
        assert!(h.join().unwrap());
    }
}

#[test]
fn save_artifacts_writes_files() {
    let (lib, _dir) = common::temp_library();
    let artifacts = lib
        .save_ride_artifacts(b"fit-bytes", Some(b"png-bytes"))
        .unwrap();
    assert!(artifacts.fit_path.exists());
    assert!(artifacts.screenshot_path.as_ref().unwrap().exists());
    let fit = std::fs::read(&artifacts.fit_path).unwrap();
    assert_eq!(fit, b"fit-bytes");
}
