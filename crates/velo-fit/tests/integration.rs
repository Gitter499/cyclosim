//! Comprehensive FIT encoder tests — round-trip, golden, edge cases, property tests.

use fitparser::profile::MesgNum;
use fitparser::{FitDataRecord, Value};
use proptest::prelude::*;
use velo_fit::{
    encode_activity, FitRecordSample, FitRide, INDOOR_LAT_DEG, INDOOR_LON_DEG,
};

fn sample_ride(n: usize, dt: f64, with_hr: bool, with_cadence: bool) -> FitRide {
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 * dt;
        samples.push(FitRecordSample {
            elapsed_s: t,
            distance_m: t * 7.5,
            speed_mps: 7.5,
            power_w: Some(150.0 + (i % 5) as f64 * 10.0),
            cadence_rpm: if with_cadence { Some(85.0) } else { None },
            heart_rate_bpm: if with_hr { Some(130.0) } else { None },
            grade: 0.02,
        });
    }
    FitRide {
        started_at_unix: 1_700_000_000,
        samples,
    }
}

fn parse_fit(bytes: &[u8]) -> Vec<FitDataRecord> {
    fitparser::from_bytes(bytes).expect("parseable FIT")
}

fn field_u64(record: &FitDataRecord, name: &str) -> Option<u64> {
    record.fields().iter().find(|f| f.name() == name).and_then(|f| match f.value() {
        Value::UInt8(v) => Some(*v as u64),
        Value::UInt16(v) => Some(*v as u64),
        Value::UInt32(v) => Some(*v as u64),
        Value::Enum(v) => Some(*v as u64),
        Value::String(s) => match s.as_str() {
            "cycling" => Some(2),
            "activity" => Some(4),
            "indoor_cycling" => Some(6),
            _ => None,
        },
        _ => None,
    })
}

fn field_string<'a>(record: &'a FitDataRecord, name: &str) -> Option<&'a str> {
    record.fields().iter().find(|f| f.name() == name).and_then(|f| match f.value() {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    })
}

#[test]
fn round_trip_has_cycling_session() {
    let ride = sample_ride(30, 1.0, true, true);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);

    let sessions: Vec<_> = records.iter().filter(|m| m.kind() == MesgNum::Session).collect();
    assert_eq!(sessions.len(), 1);
    let session = sessions[0];
    assert!(
        field_u64(session, "sport") == Some(2)
            || field_string(session, "sport") == Some("cycling")
    );
}

#[test]
fn round_trip_record_power_and_cadence() {
    let ride = sample_ride(5, 1.0, true, true);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);

    let recs: Vec<_> = records.iter().filter(|m| m.kind() == MesgNum::Record).collect();
    assert_eq!(recs.len(), 5);
    let first = recs[0];
    assert!(first.fields().iter().any(|f| f.name() == "power"));
    assert!(first.fields().iter().any(|f| f.name() == "cadence"));
    assert!(first.fields().iter().any(|f| f.name() == "heart_rate"));
}

#[test]
fn round_trip_indoor_position() {
    let ride = sample_ride(3, 1.0, false, false);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);
    let record = records.iter().find(|m| m.kind() == MesgNum::Record).unwrap();
    let has_lat = record.fields().iter().any(|f| f.name() == "position_lat");
    let has_lon = record.fields().iter().any(|f| f.name() == "position_long");
    assert!(has_lat);
    assert!(has_lon);
    let _ = (INDOOR_LAT_DEG, INDOOR_LON_DEG);
}

#[test]
fn missing_hr_and_cadence_still_parseable() {
    let ride = sample_ride(10, 0.5, false, false);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);
    let recs: Vec<_> = records.iter().filter(|m| m.kind() == MesgNum::Record).collect();
    assert_eq!(recs.len(), 10);
}

#[test]
fn zero_distance_ride() {
    let ride = FitRide {
        started_at_unix: 1_700_000_000,
        samples: vec![FitRecordSample {
            elapsed_s: 0.0,
            distance_m: 0.0,
            speed_mps: 0.0,
            power_w: Some(0.0),
            cadence_rpm: None,
            heart_rate_bpm: None,
            grade: 0.0,
        }],
    };
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);
    assert!(records.iter().any(|m| m.kind() == MesgNum::Session));
}

#[test]
fn very_short_sub_second_ride() {
    let ride = FitRide {
        started_at_unix: 1_700_000_000,
        samples: vec![
            FitRecordSample {
                elapsed_s: 0.0,
                distance_m: 0.0,
                speed_mps: 5.0,
                power_w: Some(200.0),
                cadence_rpm: Some(95.0),
                heart_rate_bpm: Some(150.0),
                grade: 0.0,
            },
            FitRecordSample {
                elapsed_s: 0.5,
                distance_m: 2.5,
                speed_mps: 5.0,
                power_w: Some(205.0),
                cadence_rpm: Some(96.0),
                heart_rate_bpm: Some(151.0),
                grade: 0.0,
            },
        ],
    };
    let bytes = encode_activity(&ride).unwrap();
    parse_fit(&bytes);
}

#[test]
fn long_ride_stress() {
    let ride = sample_ride(3600, 1.0, true, true);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);
    let recs: Vec<_> = records.iter().filter(|m| m.kind() == MesgNum::Record).collect();
    assert_eq!(recs.len(), 3600);
}

#[test]
fn golden_fixture_byte_stable_header() {
    let ride = sample_ride(3, 1.0, true, true);
    let bytes = encode_activity(&ride).unwrap();
    assert_eq!(&bytes[0..1], &[14]);
    assert_eq!(&bytes[8..12], b".FIT");
    let fixture_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden_short.fit");
    if fixture_path.exists() {
        let golden = std::fs::read(&fixture_path).unwrap();
        assert_eq!(bytes.len(), golden.len(), "regenerate with UPDATE_GOLDEN=1");
        assert_eq!(bytes, golden);
    } else if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(fixture_path.parent().unwrap()).ok();
        std::fs::write(&fixture_path, &bytes).unwrap();
    }
}

#[test]
fn file_id_message_present() {
    let ride = sample_ride(2, 1.0, false, false);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);
    let file_id = records.iter().find(|m| m.kind() == MesgNum::FileId).unwrap();
    assert!(
        field_u64(file_id, "type") == Some(4)
            || field_string(file_id, "type") == Some("activity")
    );
}

#[test]
fn lap_and_activity_messages_present() {
    let ride = sample_ride(4, 1.0, true, true);
    let bytes = encode_activity(&ride).unwrap();
    let records = parse_fit(&bytes);
    assert_eq!(
        records.iter().filter(|m| m.kind() == MesgNum::Lap).count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|m| m.kind() == MesgNum::Activity)
            .count(),
        1
    );
}

proptest! {
    #[test]
    fn random_valid_telemetry_produces_parseable_fit(
        n in 1usize..50,
        power in 50.0f64..400.0,
    ) {
        let mut samples = Vec::with_capacity(n);
        for i in 0..n {
            samples.push(FitRecordSample {
                elapsed_s: i as f64,
                distance_m: i as f64 * 5.0,
                speed_mps: 5.0,
                power_w: Some(power),
                cadence_rpm: Some(80.0 + (i % 10) as f64),
                heart_rate_bpm: Some(120.0 + (i % 20) as f64),
                grade: 0.0,
            });
        }
        let ride = FitRide { started_at_unix: 1_700_000_000, samples };
        let bytes = encode_activity(&ride).unwrap();
        let records = parse_fit(&bytes);
        prop_assert!(records.iter().any(|m| m.kind() == MesgNum::Session));
        prop_assert_eq!(records.iter().filter(|m| m.kind() == MesgNum::Record).count(), n);
    }
}
