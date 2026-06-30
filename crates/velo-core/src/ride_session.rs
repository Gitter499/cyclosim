//! Ride session recording — accumulates per-tick telemetry for FIT export.

use crate::highlight::{plan_highlight_clips, HighlightClipRequest};
use velo_fit::{encode_activity, FitEncodeError, FitRecordSample, FitRide};

/// Snapshot of one sim tick while a ride is active.
#[derive(Debug, Clone, PartialEq)]
pub struct RideSample {
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub speed_mps: f64,
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub grade: f64,
}

/// Summary returned when a ride stops.
#[derive(Debug, Clone, PartialEq)]
pub struct RideSummary {
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub sample_count: u32,
    pub avg_power_w: Option<f64>,
    pub max_power_w: Option<f64>,
    pub started_at_unix: u64,
    pub highlight_clips: Vec<HighlightClipRequest>,
}

/// In-memory ride recorder.
#[derive(Debug, Clone)]
pub struct RideSession {
    active: bool,
    started_at_unix: u64,
    samples: Vec<RideSample>,
    last_completed: Option<CompletedRide>,
}

#[derive(Debug, Clone)]
struct CompletedRide {
    started_at_unix: u64,
    samples: Vec<RideSample>,
    summary: RideSummary,
}

impl Default for RideSession {
    fn default() -> Self {
        Self::new()
    }
}

impl RideSession {
    pub fn new() -> Self {
        Self {
            active: false,
            started_at_unix: 0,
            samples: Vec::new(),
            last_completed: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Start recording. Idempotent if already active.
    pub fn start(&mut self, started_at_unix: u64) {
        if self.active {
            return;
        }
        self.active = true;
        self.started_at_unix = started_at_unix;
        self.samples.clear();
    }

    /// Stop recording and return summary. Safe to call when already stopped.
    pub fn stop(&mut self) -> Option<RideSummary> {
        if !self.active {
            return self.last_completed.as_ref().map(|c| c.summary.clone());
        }
        self.active = false;
        let summary = compute_summary(self.started_at_unix, &self.samples);
        let completed = CompletedRide {
            started_at_unix: self.started_at_unix,
            samples: self.samples.clone(),
            summary: summary.clone(),
        };
        self.last_completed = Some(completed);
        Some(summary)
    }

    pub fn record_tick(&mut self, sample: RideSample) {
        if self.active {
            self.samples.push(sample);
        }
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn export_fit(&self) -> Result<Vec<u8>, FitEncodeError> {
        let ride = self.fit_ride()?;
        encode_activity(&ride)
    }

    pub fn last_summary(&self) -> Option<RideSummary> {
        self.last_completed.as_ref().map(|c| c.summary.clone())
    }

    fn fit_ride(&self) -> Result<FitRide, FitEncodeError> {
        let completed = if self.active {
            // Export in-progress ride.
            FitRide {
                started_at_unix: self.started_at_unix,
                samples: self.samples.iter().map(map_sample).collect(),
            }
        } else if let Some(c) = &self.last_completed {
            FitRide {
                started_at_unix: c.started_at_unix,
                samples: c.samples.iter().map(map_sample).collect(),
            }
        } else {
            return Err(FitEncodeError::EmptyRide);
        };
        if completed.samples.is_empty() {
            return Err(FitEncodeError::EmptyRide);
        }
        Ok(completed)
    }
}

fn map_sample(s: &RideSample) -> FitRecordSample {
    FitRecordSample {
        elapsed_s: s.elapsed_s,
        distance_m: s.distance_m,
        speed_mps: s.speed_mps,
        power_w: s.power_w,
        cadence_rpm: s.cadence_rpm,
        heart_rate_bpm: s.heart_rate_bpm,
        grade: s.grade,
    }
}

fn compute_summary(started_at_unix: u64, samples: &[RideSample]) -> RideSummary {
    let elapsed_s = samples.last().map(|s| s.elapsed_s).unwrap_or(0.0);
    let distance_m = samples.last().map(|s| s.distance_m).unwrap_or(0.0);
    let powers: Vec<f64> = samples.iter().filter_map(|s| s.power_w).collect();
    let avg_power_w = if powers.is_empty() {
        None
    } else {
        Some(powers.iter().sum::<f64>() / powers.len() as f64)
    };
    let max_power_w = powers.iter().copied().reduce(f64::max);
    RideSummary {
        elapsed_s,
        distance_m,
        sample_count: samples.len() as u32,
        avg_power_w,
        max_power_w,
        started_at_unix,
        highlight_clips: plan_highlight_clips(samples, elapsed_s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(elapsed: f64, dist: f64) -> RideSample {
        RideSample {
            elapsed_s: elapsed,
            distance_m: dist,
            speed_mps: 8.0,
            power_w: Some(180.0),
            cadence_rpm: Some(90.0),
            heart_rate_bpm: Some(140.0),
            grade: 0.0,
        }
    }

    #[test]
    fn accumulates_samples_across_ticks() {
        let mut session = RideSession::new();
        session.start(1_700_000_000);
        session.record_tick(tick(0.01, 0.08));
        session.record_tick(tick(0.02, 0.16));
        assert_eq!(session.sample_count(), 2);
    }

    #[test]
    fn start_is_idempotent() {
        let mut session = RideSession::new();
        session.start(100);
        session.record_tick(tick(1.0, 8.0));
        session.start(200);
        assert_eq!(session.sample_count(), 1);
        assert_eq!(session.started_at_unix, 100);
    }

    #[test]
    fn double_stop_returns_last_summary() {
        let mut session = RideSession::new();
        session.start(1_700_000_000);
        session.record_tick(tick(1.0, 8.0));
        let s1 = session.stop().unwrap();
        let s2 = session.stop().unwrap();
        assert_eq!(s1, s2);
        assert!(!session.is_active());
    }

    #[test]
    fn stop_without_start_returns_none() {
        let mut session = RideSession::new();
        assert!(session.stop().is_none());
    }

    #[test]
    fn summary_matches_integrated_distance() {
        let mut session = RideSession::new();
        session.start(1_700_000_000);
        for i in 1..=10 {
            let t = i as f64 * 0.1;
            session.record_tick(tick(t, t * 8.0));
        }
        let summary = session.stop().unwrap();
        assert!((summary.distance_m - 8.0).abs() < 0.01);
        assert!((summary.elapsed_s - 1.0).abs() < 0.01);
        assert_eq!(summary.sample_count, 10);
        assert_eq!(summary.avg_power_w, Some(180.0));
        assert_eq!(summary.max_power_w, Some(180.0));
    }

    #[test]
    fn export_fit_after_stop() {
        let mut session = RideSession::new();
        session.start(1_700_000_000);
        session.record_tick(tick(1.0, 8.0));
        session.stop();
        let fit = session.export_fit().unwrap();
        assert!(fit.len() > 50);
        assert_eq!(&fit[8..12], b".FIT");
    }

    #[test]
    fn no_record_while_inactive() {
        let mut session = RideSession::new();
        session.record_tick(tick(1.0, 8.0));
        assert_eq!(session.sample_count(), 0);
    }
}
