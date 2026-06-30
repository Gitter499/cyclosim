/// Seconds since FIT epoch (1989-12-31 00:00:00 UTC).
pub type FitTimestamp = u32;

/// FIT epoch offset from Unix epoch in seconds.
pub const FIT_EPOCH_UNIX_OFFSET: i64 = 631_065_600;

pub fn unix_to_fit_timestamp(unix_secs: u64) -> FitTimestamp {
    let fit = unix_secs as i64 - FIT_EPOCH_UNIX_OFFSET;
    fit.max(0) as u32
}

pub fn fit_timestamp_to_unix(ts: FitTimestamp) -> u64 {
    ts as u64 + FIT_EPOCH_UNIX_OFFSET as u64
}

/// Degrees → FIT semicircles (sint32).
pub fn degrees_to_semicircles(degrees: f64) -> i32 {
    (degrees * (i32::MAX as f64 / 180.0)).round() as i32
}

/// Meters per second → FIT speed field (uint16, scale 1000).
pub fn speed_mps_to_fit(speed_mps: f64) -> u16 {
    (speed_mps * 1000.0).round().clamp(0.0, u16::MAX as f64) as u16
}

/// Distance in meters → FIT distance field (uint32, scale 100 → centimeters stored as uint32/100).
pub fn distance_m_to_fit(distance_m: f64) -> u32 {
    (distance_m * 100.0).round().clamp(0.0, u32::MAX as f64) as u32
}

/// Elapsed seconds → FIT duration field (uint32, scale 1000 → milliseconds).
pub fn duration_s_to_fit(seconds: f64) -> u32 {
    (seconds * 1000.0).round().clamp(0.0, u32::MAX as f64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_round_trip_near_now() {
        let unix = 1_700_000_000_u64;
        let fit = unix_to_fit_timestamp(unix);
        assert_eq!(fit_timestamp_to_unix(fit), unix);
    }

    #[test]
    fn semicircles_sanity() {
        assert_eq!(degrees_to_semicircles(0.0), 0);
        assert!(degrees_to_semicircles(45.0).abs() > 0);
    }
}
