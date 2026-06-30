#[path = "common/mod.rs"]
mod common;

use velo_rides::{NewRideRecord, PublishStatus};

#[test]
fn empty_library_lists_nothing() {
    let (lib, _dir) = common::temp_library();
    let rides = lib.list_rides().unwrap();
    assert!(rides.is_empty());
}

#[test]
fn insert_and_list_newest_first() {
    let (lib, _dir) = common::temp_library();
    let artifacts = lib.save_ride_artifacts(b"fit-a", None).unwrap();
    lib.insert_ride_with_id(
        &artifacts.ride_id,
        NewRideRecord {
            started_at_unix: 1_700_000_100,
            elapsed_s: 60.0,
            distance_m: 1000.0,
            avg_power_w: Some(150.0),
            max_power_w: Some(200.0),
            fit_path: artifacts.fit_path.display().to_string(),
            screenshot_path: None,
            strava_activity_id: None,
            publish_status: PublishStatus::Local,
            route_id: None,
        },
    )
    .unwrap();

    let artifacts2 = lib.save_ride_artifacts(b"fit-b", Some(b"png")).unwrap();
    lib.insert_ride_with_id(
        &artifacts2.ride_id,
        NewRideRecord {
            started_at_unix: 1_700_000_200,
            elapsed_s: 90.0,
            distance_m: 2000.0,
            avg_power_w: None,
            max_power_w: None,
            fit_path: artifacts2.fit_path.display().to_string(),
            screenshot_path: artifacts2
                .screenshot_path
                .as_ref()
                .map(|p| p.display().to_string()),
            strava_activity_id: Some("12345".into()),
            publish_status: PublishStatus::Strava,
            route_id: None,
        },
    )
    .unwrap();

    let rides = lib.list_rides().unwrap();
    assert_eq!(rides.len(), 2);
    assert_eq!(rides[0].started_at_unix, 1_700_000_200);
    assert_eq!(rides[1].started_at_unix, 1_700_000_100);
    assert_eq!(rides[0].publish_status, PublishStatus::Strava);
    assert_eq!(rides[0].strava_activity_id.as_deref(), Some("12345"));
}

#[test]
fn get_ride_by_id() {
    let (lib, _dir) = common::temp_library();
    let artifacts = lib.save_ride_artifacts(b"fit", None).unwrap();
    lib.insert_ride_with_id(
        &artifacts.ride_id,
        common::sample_record(&artifacts.fit_path.display().to_string()),
    )
    .unwrap();

    let ride = lib.get_ride(&artifacts.ride_id).unwrap().unwrap();
    assert_eq!(ride.id, artifacts.ride_id);
    assert_eq!(ride.distance_m, 3500.0);
}

#[test]
fn get_missing_id_returns_none() {
    let (lib, _dir) = common::temp_library();
    let missing = "00000000-0000-4000-8000-000000000001";
    assert!(lib.get_ride(missing).unwrap().is_none());
}

#[test]
fn delete_ride_removes_row_and_artifacts() {
    let (lib, _dir) = common::temp_library();
    let artifacts = lib.save_ride_artifacts(b"fit-data", Some(b"png-data")).unwrap();
    let ride_dir = artifacts.ride_dir.clone();
    lib.insert_ride_with_id(
        &artifacts.ride_id,
        common::sample_record(&artifacts.fit_path.display().to_string()),
    )
    .unwrap();

    assert!(ride_dir.exists());
    assert!(lib.delete_ride(&artifacts.ride_id).unwrap());
    assert!(lib.get_ride(&artifacts.ride_id).unwrap().is_none());
    assert!(!ride_dir.exists());
}

#[test]
fn delete_missing_returns_false() {
    let (lib, _dir) = common::temp_library();
    let missing = "00000000-0000-4000-8000-000000000002";
    assert!(!lib.delete_ride(missing).unwrap());
}
