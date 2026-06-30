//! High-level activity encoder.

use thiserror::Error;

use crate::types::{
    degrees_to_semicircles, distance_m_to_fit, duration_s_to_fit, speed_mps_to_fit,
    unix_to_fit_timestamp, FitTimestamp,
};
use crate::writer::{BaseType, FieldDef, FitWriter};

/// Indoor placeholder coordinates (VeloSim HQ — documented in STRAVA.md).
pub const INDOOR_LAT_DEG: f64 = 45.5017;
pub const INDOOR_LON_DEG: f64 = -73.5673;

/// Per-tick telemetry sample for FIT encoding.
#[derive(Debug, Clone, PartialEq)]
pub struct FitRecordSample {
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub speed_mps: f64,
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub grade: f64,
}

/// Completed ride ready for FIT export.
#[derive(Debug, Clone)]
pub struct FitRide {
    pub started_at_unix: u64,
    pub samples: Vec<FitRecordSample>,
}

impl FitRide {
    pub fn elapsed_s(&self) -> f64 {
        self.samples.last().map(|s| s.elapsed_s).unwrap_or(0.0)
    }

    pub fn distance_m(&self) -> f64 {
        self.samples.last().map(|s| s.distance_m).unwrap_or(0.0)
    }

    pub fn avg_power_w(&self) -> Option<f64> {
        let powers: Vec<f64> = self.samples.iter().filter_map(|s| s.power_w).collect();
        if powers.is_empty() {
            None
        } else {
            Some(powers.iter().sum::<f64>() / powers.len() as f64)
        }
    }
}

#[derive(Debug, Error)]
pub enum FitEncodeError {
    #[error("ride has no samples")]
    EmptyRide,
}

/// Encode a completed ride as a standard FIT activity file.
pub fn encode_activity(ride: &FitRide) -> Result<Vec<u8>, FitEncodeError> {
    if ride.samples.is_empty() {
        return Err(FitEncodeError::EmptyRide);
    }

    let start_ts = unix_to_fit_timestamp(ride.started_at_unix);
    let end_ts = start_ts.saturating_add(duration_s_to_fit(ride.elapsed_s()) / 1000);

    let mut w = FitWriter::new();

    // Local 0: file_id (global 0)
    w.write_definition(
        0,
        0,
        &[
            FieldDef {
                num: 0,
                size: 1,
                base_type: BaseType::Enum,
            },
            FieldDef {
                num: 1,
                size: 2,
                base_type: BaseType::Uint16,
            },
            FieldDef {
                num: 2,
                size: 2,
                base_type: BaseType::Uint16,
            },
            FieldDef {
                num: 4,
                size: 4,
                base_type: BaseType::Uint32,
            },
        ],
    );
    {
        let mut m = w.begin_data(0);
        m.write_u8(4); // activity
        m.write_u16(255); // development
        m.write_u16(0);
        m.write_u32(start_ts);
    }

    // Local 1: file_creator (global 49)
    w.write_definition(
        1,
        49,
        &[
            FieldDef {
                num: 0,
                size: 2,
                base_type: BaseType::Uint16,
            },
            FieldDef {
                num: 1,
                size: 2,
                base_type: BaseType::Uint16,
            },
        ],
    );
    {
        let mut m = w.begin_data(1);
        m.write_u16(255);
        m.write_u16(1);
    }

    // Local 2: event start (global 21)
    w.write_definition(
        2,
        21,
        &[
            FieldDef {
                num: 253,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 0,
                size: 1,
                base_type: BaseType::Enum,
            },
            FieldDef {
                num: 1,
                size: 1,
                base_type: BaseType::Enum,
            },
        ],
    );
    {
        let mut m = w.begin_data(2);
        m.write_u32(start_ts);
        m.write_u8(0); // timer
        m.write_u8(0); // start
    }

    // Local 3: record (global 20) — optional fields via invalid sentinels
    w.write_definition(
        3,
        20,
        &[
            FieldDef {
                num: 253,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 0,
                size: 4,
                base_type: BaseType::Sint32,
            },
            FieldDef {
                num: 1,
                size: 4,
                base_type: BaseType::Sint32,
            },
            FieldDef {
                num: 5,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 6,
                size: 2,
                base_type: BaseType::Uint16,
            },
            FieldDef {
                num: 7,
                size: 2,
                base_type: BaseType::Uint16,
            },
            FieldDef {
                num: 3,
                size: 1,
                base_type: BaseType::Uint8,
            },
            FieldDef {
                num: 4,
                size: 1,
                base_type: BaseType::Uint8,
            },
        ],
    );

    let lat = degrees_to_semicircles(INDOOR_LAT_DEG);
    let lon = degrees_to_semicircles(INDOOR_LON_DEG);

    for sample in &ride.samples {
        let ts = sample_timestamp(start_ts, sample.elapsed_s);
        let mut m = w.begin_data(3);
        m.write_u32(ts);
        m.write_i32(lat);
        m.write_i32(lon);
        m.write_u32(distance_m_to_fit(sample.distance_m));
        m.write_u16(speed_mps_to_fit(sample.speed_mps));
        m.write_u16(fit_power(sample.power_w));
        m.write_u8(fit_hr(sample.heart_rate_bpm));
        m.write_u8(fit_cadence(sample.cadence_rpm));
    }

    // Local 4: event stop
    {
        let mut m = w.begin_data(2);
        m.write_u32(end_ts);
        m.write_u8(0);
        m.write_u8(4); // stop_all
    }

    let elapsed_fit = duration_s_to_fit(ride.elapsed_s());
    let distance_fit = distance_m_to_fit(ride.distance_m());

    // Local 5: lap (global 19)
    w.write_definition(
        5,
        19,
        &[
            FieldDef {
                num: 253,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 2,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 7,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 8,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 9,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 5,
                size: 1,
                base_type: BaseType::Enum,
            },
        ],
    );
    {
        let mut m = w.begin_data(5);
        m.write_u32(end_ts);
        m.write_u32(start_ts);
        m.write_u32(elapsed_fit);
        m.write_u32(elapsed_fit);
        m.write_u32(distance_fit);
        m.write_u8(2); // cycling
    }

    // Local 6: session (global 18)
    w.write_definition(
        6,
        18,
        &[
            FieldDef {
                num: 253,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 2,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 7,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 8,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 9,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 5,
                size: 1,
                base_type: BaseType::Enum,
            },
            FieldDef {
                num: 6,
                size: 1,
                base_type: BaseType::Enum,
            },
        ],
    );
    {
        let mut m = w.begin_data(6);
        m.write_u32(end_ts);
        m.write_u32(start_ts);
        m.write_u32(elapsed_fit);
        m.write_u32(elapsed_fit);
        m.write_u32(distance_fit);
        m.write_u8(2); // cycling
        m.write_u8(6); // indoor_cycling
    }

    // Local 7: activity (global 34)
    w.write_definition(
        7,
        34,
        &[
            FieldDef {
                num: 253,
                size: 4,
                base_type: BaseType::Uint32,
            },
            FieldDef {
                num: 1,
                size: 2,
                base_type: BaseType::Uint16,
            },
            FieldDef {
                num: 2,
                size: 1,
                base_type: BaseType::Enum,
            },
            FieldDef {
                num: 5,
                size: 1,
                base_type: BaseType::Enum,
            },
        ],
    );
    {
        let mut m = w.begin_data(7);
        m.write_u32(end_ts);
        m.write_u16(0);
        m.write_u8(0); // manual
        m.write_u8(2); // cycling
    }

    Ok(w.finish())
}

fn sample_timestamp(start_ts: FitTimestamp, elapsed_s: f64) -> FitTimestamp {
    start_ts.saturating_add((elapsed_s * 1000.0).round() as u32 / 1000)
}

fn fit_power(power_w: Option<f64>) -> u16 {
    match power_w {
        Some(p) if p >= 0.0 => p.round().clamp(0.0, u16::MAX as f64) as u16,
        _ => 0xFFFF,
    }
}

fn fit_hr(hr: Option<f64>) -> u8 {
    match hr {
        Some(h) if h > 0.0 => h.round().clamp(0.0, 255.0) as u8,
        _ => 0xFF,
    }
}

fn fit_cadence(cadence: Option<f64>) -> u8 {
    match cadence {
        Some(c) if c > 0.0 => c.round().clamp(0.0, 255.0) as u8,
        _ => 0xFF,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ride(n: usize, dt: f64) -> FitRide {
        let mut samples = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 * dt;
            samples.push(FitRecordSample {
                elapsed_s: t,
                distance_m: t * 8.0,
                speed_mps: 8.0,
                power_w: Some(180.0),
                cadence_rpm: Some(90.0),
                heart_rate_bpm: Some(140.0),
                grade: 0.0,
            });
        }
        FitRide {
            started_at_unix: 1_700_000_000,
            samples,
        }
    }

    #[test]
    fn encodes_non_empty_ride() {
        let ride = sample_ride(10, 1.0);
        let bytes = encode_activity(&ride).unwrap();
        assert!(bytes.len() > 64);
        assert_eq!(&bytes[8..12], b".FIT");
    }

    #[test]
    fn empty_ride_errors() {
        let ride = FitRide {
            started_at_unix: 1_700_000_000,
            samples: vec![],
        };
        assert!(encode_activity(&ride).is_err());
    }
}
