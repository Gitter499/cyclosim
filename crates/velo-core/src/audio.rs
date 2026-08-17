//! Workout interval → segment energy mapping for [`AudioDirector`] callbacks.

use velo_platform::{PlaybackIntent, SegmentEnergy};

use crate::workout::{WorkoutInterval, WorkoutTarget};

/// Map a workout interval name/target to a coarse energy bucket for segment playback.
pub fn segment_energy_for_interval(interval: &WorkoutInterval, ftp_w: f64) -> SegmentEnergy {
    let name = interval.name.to_lowercase();
    if name.contains("warm") {
        return SegmentEnergy::Warmup;
    }
    if name.contains("cool") {
        return SegmentEnergy::Cooldown;
    }
    if name.contains("recover") || name.contains("rest") {
        return SegmentEnergy::Recovery;
    }

    let ftp_pct = match interval.target {
        WorkoutTarget::FtpPercent(p) => p,
        WorkoutTarget::ErgWatts(w) if ftp_w > 0.0 => w / ftp_w * 100.0,
        WorkoutTarget::FreeRide => 55.0,
        WorkoutTarget::ErgWatts(_) => 70.0,
    };

    if name.contains("threshold") || ftp_pct >= 90.0 {
        SegmentEnergy::Threshold
    } else if name.contains("build") || ftp_pct >= 75.0 {
        SegmentEnergy::Build
    } else if ftp_pct >= 60.0 {
        SegmentEnergy::Build
    } else {
        SegmentEnergy::Recovery
    }
}

/// Intent for the first interval vs later transitions.
pub fn playback_intent_for_index(interval_index: usize) -> PlaybackIntent {
    if interval_index == 0 {
        PlaybackIntent::Start
    } else {
        PlaybackIntent::Transition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workout::{Workout, WorkoutEngine, WorkoutTarget};

    #[test]
    fn maps_warmup_by_name() {
        let interval = WorkoutInterval {
            name: "Warmup".into(),
            duration_s: 600.0,
            target: WorkoutTarget::FtpPercent(55.0),
        };
        assert_eq!(
            segment_energy_for_interval(&interval, 250.0),
            SegmentEnergy::Warmup
        );
    }

    #[test]
    fn maps_threshold_by_ftp_percent() {
        let interval = WorkoutInterval {
            name: "Block".into(),
            duration_s: 1200.0,
            target: WorkoutTarget::FtpPercent(95.0),
        };
        assert_eq!(
            segment_energy_for_interval(&interval, 250.0),
            SegmentEnergy::Threshold
        );
    }

    #[test]
    fn engine_boundary_changes_interval_index() {
        let mut engine = WorkoutEngine::new(Workout::sample_threshold(), 250.0);
        assert_eq!(engine.state().interval_index, 0);
        for _ in 0..60_001 {
            engine.tick(0.01);
        }
        assert_eq!(engine.state().interval_index, 1);
    }
}
