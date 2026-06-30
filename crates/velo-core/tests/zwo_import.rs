//! Integration tests for Zwift `.zwo` import.

use velo_core::{parse_zwo_xml, WorkoutTarget, ZwoError};

const THRESHOLD_ZWO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<workout_file>
  <author>Test</author>
  <name>2x20 Threshold</name>
  <sportType>bike</sportType>
  <workout>
    <Warmup Duration="600" PowerLow="0.25" PowerHigh="0.55" />
    <SteadyState Duration="1200" Power="0.95" />
    <SteadyState Duration="600" Power="0.50" />
    <SteadyState Duration="1200" Power="0.95" />
    <Cooldown Duration="600" PowerHigh="0.55" PowerLow="0.25" />
  </workout>
</workout_file>"#;

#[test]
fn zwo_threshold_import_validates() {
    let workout = parse_zwo_xml(THRESHOLD_ZWO).expect("parse threshold zwo");
    assert_eq!(workout.name, "2x20 Threshold");
    assert!(workout.intervals.len() > 5);
    workout.validate().expect("valid workout");
    assert!(
        workout
            .intervals
            .iter()
            .any(|i| matches!(i.target, WorkoutTarget::FtpPercent(p) if (p - 95.0).abs() < 0.1))
    );
}

#[test]
fn zwo_skips_text_steps() {
    let xml = r#"<workout_file>
  <name>With text</name>
  <workout>
    <Text Duration="10" message="Push!" />
    <SteadyState Duration="60" Power="0.75" />
  </workout>
</workout_file>"#;
    let workout = parse_zwo_xml(xml).expect("parse");
    assert_eq!(workout.intervals.len(), 1);
    workout.validate().expect("valid");
}

#[test]
fn zwo_invalid_xml_returns_error() {
    let err = parse_zwo_xml("<not xml").unwrap_err();
    assert!(matches!(err, ZwoError::Xml(_)));
}
