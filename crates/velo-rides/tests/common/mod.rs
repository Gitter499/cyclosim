#![allow(dead_code)]

use tempfile::TempDir;
use velo_rides::{NewRideRecord, PublishStatus, RideLibrary};

pub fn temp_library() -> (RideLibrary, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("rides.db");
    let artifacts = dir.path().join("artifacts");
    let lib = RideLibrary::open(&db, &artifacts).unwrap();
    (lib, dir)
}

pub fn sample_record(fit_path: &str) -> NewRideRecord {
    NewRideRecord {
        started_at_unix: 1_700_000_000,
        elapsed_s: 120.0,
        distance_m: 3500.0,
        avg_power_w: Some(180.0),
        max_power_w: Some(250.0),
        fit_path: fit_path.to_string(),
        screenshot_path: None,
        strava_activity_id: None,
        publish_status: PublishStatus::Local,
        route_id: None,
    }
}
