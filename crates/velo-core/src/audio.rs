//! Segment-aware audio direction (M6).
//!
//! The core cannot mix audio (the platform music API is playback-control only, §13), so it
//! emits *events* at workout boundaries; the shell maps them to playlist and
//! transport changes. Events are queued in [`crate::VeloApp`] and drained by
//! the shell each tick (same polling model as telemetry over FFI).

use velo_platform::{PlaybackIntent, SegmentEnergy};

use crate::workout::{WorkoutInterval, WorkoutTarget};

/// One audio direction event: "the ride entered a segment with this energy".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioEvent {
    pub energy: SegmentEnergy,
    pub intent: PlaybackIntent,
}

/// Classify an interval's energy from its ERG intensity relative to FTP.
///
/// Position hints (first/last interval) take precedence for warmup/cooldown
/// so a low-intensity opener reads as Warmup rather than Recovery.
pub fn energy_for_interval(
    interval: &WorkoutInterval,
    ftp_w: f64,
    index: usize,
    interval_count: usize,
) -> SegmentEnergy {
    let pct = match interval.target {
        WorkoutTarget::FtpPercent(p) => p,
        WorkoutTarget::ErgWatts(w) if ftp_w > 0.0 => w / ftp_w * 100.0,
        WorkoutTarget::ErgWatts(_) => 75.0,
        WorkoutTarget::FreeRide => return SegmentEnergy::Build,
    };

    let is_first = index == 0;
    let is_last = interval_count > 0 && index == interval_count - 1;
    if pct < 65.0 {
        if is_first {
            return SegmentEnergy::Warmup;
        }
        if is_last {
            return SegmentEnergy::Cooldown;
        }
        return SegmentEnergy::Recovery;
    }
    if pct < 88.0 {
        SegmentEnergy::Build
    } else {
        SegmentEnergy::Threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interval(target: WorkoutTarget) -> WorkoutInterval {
        WorkoutInterval {
            name: "t".into(),
            duration_s: 60.0,
            target,
        }
    }

    #[test]
    fn low_intensity_position_aware() {
        let i = interval(WorkoutTarget::FtpPercent(55.0));
        assert_eq!(energy_for_interval(&i, 250.0, 0, 5), SegmentEnergy::Warmup);
        assert_eq!(energy_for_interval(&i, 250.0, 4, 5), SegmentEnergy::Cooldown);
        assert_eq!(energy_for_interval(&i, 250.0, 2, 5), SegmentEnergy::Recovery);
    }

    #[test]
    fn intensity_bands() {
        let build = interval(WorkoutTarget::FtpPercent(75.0));
        assert_eq!(energy_for_interval(&build, 250.0, 1, 5), SegmentEnergy::Build);
        let threshold = interval(WorkoutTarget::FtpPercent(95.0));
        assert_eq!(
            energy_for_interval(&threshold, 250.0, 1, 5),
            SegmentEnergy::Threshold
        );
    }

    #[test]
    fn erg_watts_resolve_against_ftp() {
        let i = interval(WorkoutTarget::ErgWatts(250.0)); // 100% of 250
        assert_eq!(energy_for_interval(&i, 250.0, 1, 3), SegmentEnergy::Threshold);
        let easy = interval(WorkoutTarget::ErgWatts(125.0)); // 50%
        assert_eq!(energy_for_interval(&easy, 250.0, 1, 3), SegmentEnergy::Recovery);
    }

    #[test]
    fn free_ride_reads_as_build() {
        let i = interval(WorkoutTarget::FreeRide);
        assert_eq!(energy_for_interval(&i, 250.0, 1, 3), SegmentEnergy::Build);
    }
}
