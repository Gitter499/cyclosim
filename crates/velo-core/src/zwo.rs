//! Zwift `.zwo` workout XML → [`Workout`].
//!
//! Supported step types: `Warmup`, `SteadyState`, `Cooldown`, `FreeRide`, `Ramp`, `MaxEffort`,
//! and basic `IntervalsT` (repeat/on/off).
//!
//! Unsupported elements (`Text`, `Intervals`, `Rest`, `SpinUp`, …) are skipped.
//! Ramps (`Warmup`, `Cooldown`, `Ramp`) expand into ~30 s ERG slices with linear power interpolation.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::workout::{Workout, WorkoutInterval, WorkoutTarget};

const RAMP_SLICE_S: f64 = 30.0;

/// Errors while parsing Zwift workout XML.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ZwoError {
    #[error("invalid XML: {0}")]
    Xml(String),
    #[error("missing workout name")]
    MissingName,
    #[error("no supported workout steps found")]
    EmptyWorkout,
    #[error("invalid step: {0}")]
    InvalidStep(String),
}

/// Parse Zwift `.zwo` XML into a [`Workout`]. Call [`Workout::validate`] before playback.
pub fn parse_zwo_xml(xml: &str) -> Result<Workout, ZwoError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut name = String::new();
    let mut in_workout = false;
    let mut in_name = false;
    let mut intervals = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = local_name(&e);
                match local.as_str() {
                    "name" if !in_workout => in_name = true,
                    "workout" => in_workout = true,
                    step if in_workout => {
                        if let Some(step_intervals) = parse_step(step, &e)? {
                            intervals.extend(step_intervals);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let local = local_name(&e);
                if in_workout {
                    if let Some(step_intervals) = parse_step(&local, &e)? {
                        intervals.extend(step_intervals);
                    }
                }
            }
            Ok(Event::Text(t)) if in_name => {
                name = t.unescape().unwrap_or_default().into_owned();
                in_name = false;
            }
            Ok(Event::End(e)) => {
                let local = local_name_end(&e);
                if local == "workout" {
                    in_workout = false;
                } else if local == "name" {
                    in_name = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ZwoError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    if name.trim().is_empty() {
        return Err(ZwoError::MissingName);
    }
    if intervals.is_empty() {
        return Err(ZwoError::EmptyWorkout);
    }

    Ok(Workout {
        name: name.trim().to_string(),
        intervals,
    })
}

fn parse_step(
    step: &str,
    element: &quick_xml::events::BytesStart,
) -> Result<Option<Vec<WorkoutInterval>>, ZwoError> {
    match step {
        "Warmup" | "Cooldown" | "Ramp" => {
            let duration = required_attr(element, "Duration")?;
            let low = optional_attr(element, "PowerLow")
                .or_else(|| optional_attr(element, "Power"))
                .unwrap_or(0.5);
            let high = optional_attr(element, "PowerHigh")
                .or_else(|| optional_attr(element, "Power"))
                .unwrap_or(low);
            let label = match step {
                "Warmup" => "Warmup",
                "Cooldown" => "Cooldown",
                _ => "Ramp",
            };
            Ok(Some(expand_ramp(label, duration, low, high)))
        }
        "SteadyState" => {
            let duration = required_attr(element, "Duration")?;
            let power = required_attr(element, "Power")?;
            Ok(Some(vec![WorkoutInterval {
                name: "Steady".into(),
                duration_s: duration,
                target: parse_power(power)?,
            }]))
        }
        "FreeRide" => {
            let duration = required_attr(element, "Duration")?;
            Ok(Some(vec![WorkoutInterval {
                name: "Free ride".into(),
                duration_s: duration,
                target: WorkoutTarget::FreeRide,
            }]))
        }
        "MaxEffort" => {
            let duration = required_attr(element, "Duration")?;
            let target = optional_attr(element, "Power")
                .map(parse_power)
                .transpose()?
                .unwrap_or(WorkoutTarget::FtpPercent(150.0));
            Ok(Some(vec![WorkoutInterval {
                name: "Max effort".into(),
                duration_s: duration,
                target,
            }]))
        }
        "IntervalsT" => Ok(Some(parse_intervals_t(element)?)),
        // Instructional or unsupported — skip silently.
        "Text" | "Intervals" | "Rest" | "SpinUp" | "Cadence" => Ok(None),
        _ => Ok(None),
    }
}

fn parse_intervals_t(element: &quick_xml::events::BytesStart) -> Result<Vec<WorkoutInterval>, ZwoError> {
    let repeat = required_attr(element, "Repeat")? as u32;
    let on_duration = required_attr(element, "OnDuration")?;
    let off_duration = required_attr(element, "OffDuration")?;
    let on_power = required_attr(element, "OnPower")?;
    let off_power = required_attr(element, "OffPower")?;
    let on_target = parse_power(on_power)?;
    let off_target = parse_power(off_power)?;

    let mut intervals = Vec::with_capacity(repeat as usize * 2);
    for i in 0..repeat {
        intervals.push(WorkoutInterval {
            name: format!("Interval {}", i + 1),
            duration_s: on_duration,
            target: on_target,
        });
        intervals.push(WorkoutInterval {
            name: format!("Recovery {}", i + 1),
            duration_s: off_duration,
            target: off_target,
        });
    }
    Ok(intervals)
}

fn expand_ramp(name: &str, duration_s: f64, low: f64, high: f64) -> Vec<WorkoutInterval> {
    if duration_s <= 0.0 {
        return Vec::new();
    }
    let low_pct = power_to_ftp_percent(low);
    let high_pct = power_to_ftp_percent(high);
    let mut remaining = duration_s;
    let mut elapsed = 0.0;
    let mut slices = Vec::new();
    let mut index = 1;

    while remaining > 0.0 {
        let slice = remaining.min(RAMP_SLICE_S);
        let mid = elapsed + slice * 0.5;
        let t = (mid / duration_s).clamp(0.0, 1.0);
        let pct = low_pct + (high_pct - low_pct) * t;
        slices.push(WorkoutInterval {
            name: format!("{name} {index}"),
            duration_s: slice,
            target: WorkoutTarget::FtpPercent(pct),
        });
        elapsed += slice;
        remaining -= slice;
        index += 1;
    }
    slices
}

/// Zwift encodes power as FTP fraction (≤5) or absolute watts (>5).
fn parse_power(value: f64) -> Result<WorkoutTarget, ZwoError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(ZwoError::InvalidStep(format!("invalid power {value}")));
    }
    if value <= 5.0 {
        Ok(WorkoutTarget::FtpPercent(power_to_ftp_percent(value)))
    } else {
        Ok(WorkoutTarget::ErgWatts(value))
    }
}

fn power_to_ftp_percent(fraction: f64) -> f64 {
    fraction * 100.0
}

fn required_attr(element: &quick_xml::events::BytesStart, key: &str) -> Result<f64, ZwoError> {
    optional_attr(element, key).ok_or_else(|| ZwoError::InvalidStep(format!("missing {key}")))
}

fn optional_attr(element: &quick_xml::events::BytesStart, key: &str) -> Option<f64> {
    element
        .attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == key.as_bytes())
        .and_then(|a| String::from_utf8_lossy(&a.value).parse().ok())
}

fn local_name(e: &quick_xml::events::BytesStart) -> String {
    String::from_utf8_lossy(e.name().local_name().as_ref()).into_owned()
}

fn local_name_end(e: &quick_xml::events::BytesEnd) -> String {
    String::from_utf8_lossy(e.name().local_name().as_ref()).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<workout_file>
  <name>Threshold</name>
  <workout>
    <Warmup Duration="60" PowerLow="0.5" PowerHigh="0.75" />
    <SteadyState Duration="120" Power="0.95" />
    <Cooldown Duration="60" PowerHigh="0.5" PowerLow="0.25" />
  </workout>
</workout_file>"#;

    #[test]
    fn parses_basic_workout_and_validates() {
        let workout = parse_zwo_xml(SIMPLE).expect("parse");
        assert_eq!(workout.name, "Threshold");
        assert!(workout.intervals.len() >= 3);
        workout.validate().expect("valid");
        assert!(
            workout
                .intervals
                .iter()
                .any(|i| matches!(i.target, WorkoutTarget::FtpPercent(p) if (p - 95.0).abs() < 0.1))
        );
    }

    #[test]
    fn parses_intervals_t() {
        let xml = r#"<workout_file>
  <name>VO2</name>
  <workout>
    <IntervalsT Repeat="2" OnDuration="60" OffDuration="30" OnPower="1.1" OffPower="0.5" />
  </workout>
</workout_file>"#;
        let workout = parse_zwo_xml(xml).expect("parse");
        assert_eq!(workout.intervals.len(), 4);
        workout.validate().expect("valid");
    }

    #[test]
    fn parses_free_ride_and_absolute_watts() {
        let xml = r#"<workout_file>
  <name>Mixed</name>
  <workout>
    <FreeRide Duration="300" />
    <SteadyState Duration="60" Power="200" />
  </workout>
</workout_file>"#;
        let workout = parse_zwo_xml(xml).expect("parse");
        assert_eq!(workout.intervals.len(), 2);
        assert!(matches!(
            workout.intervals[0].target,
            WorkoutTarget::FreeRide
        ));
        assert!(matches!(
            workout.intervals[1].target,
            WorkoutTarget::ErgWatts(w) if (w - 200.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn rejects_missing_name() {
        let xml = r#"<workout_file><workout><SteadyState Duration="60" Power="0.5" /></workout></workout_file>"#;
        assert_eq!(parse_zwo_xml(xml), Err(ZwoError::MissingName));
    }

    #[test]
    fn rejects_empty_workout() {
        let xml = r#"<workout_file><name>Empty</name><workout></workout></workout_file>"#;
        assert_eq!(parse_zwo_xml(xml), Err(ZwoError::EmptyWorkout));
    }
}
