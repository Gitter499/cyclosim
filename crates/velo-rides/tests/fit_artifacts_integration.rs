//! Cross-crate: velo-fit encoded activity stored via velo-rides artifact paths.

#[path = "common/mod.rs"]
mod common;

use fitparser::profile::MesgNum;
use velo_fit::{encode_activity, FitRecordSample, FitRide};
use velo_rides::{NewRideRecord, PublishStatus};

fn short_ride() -> FitRide {
    FitRide {
        started_at_unix: 1_700_000_000,
        samples: vec![
            FitRecordSample {
                elapsed_s: 0.0,
                distance_m: 0.0,
                speed_mps: 7.0,
                power_w: Some(180.0),
                cadence_rpm: Some(90.0),
                heart_rate_bpm: Some(140.0),
                grade: 0.0,
            },
            FitRecordSample {
                elapsed_s: 1.0,
                distance_m: 7.0,
                speed_mps: 7.0,
                power_w: Some(182.0),
                cadence_rpm: Some(91.0),
                heart_rate_bpm: Some(141.0),
                grade: 0.0,
            },
        ],
    }
}

#[test]
fn encoded_fit_round_trips_through_library_artifacts() {
    let (lib, dir) = common::temp_library();
    let fit_bytes = encode_activity(&short_ride()).unwrap();
    assert_eq!(&fit_bytes[8..12], b".FIT");

    let artifacts = lib.save_ride_artifacts(&fit_bytes, None).unwrap();
    lib.insert_ride_with_id(
        &artifacts.ride_id,
        NewRideRecord {
            started_at_unix: 1_700_000_000,
            elapsed_s: 1.0,
            distance_m: 7.0,
            avg_power_w: Some(181.0),
            max_power_w: Some(182.0),
            fit_path: artifacts.fit_path.display().to_string(),
            screenshot_path: None,
            highlight_clip_path: None,
            strava_activity_id: None,
            publish_status: PublishStatus::Local,
            route_id: None,
        },
    )
    .unwrap();

    let on_disk = std::fs::read(&artifacts.fit_path).unwrap();
    assert_eq!(on_disk, fit_bytes);

    let parsed = fitparser::from_bytes(&on_disk).unwrap();
    let records: Vec<_> = parsed.iter().filter(|m| m.kind() == MesgNum::Record).collect();
    assert_eq!(records.len(), 2);

    let ride = lib.get_ride(&artifacts.ride_id).unwrap().unwrap();
    assert_eq!(ride.fit_path, artifacts.fit_path.display().to_string());
    assert!(dir.path().join("artifacts").exists());
}
