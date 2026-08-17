//! Ride metrics for HUD parity (P2-B) and post-ride summaries (P2-C).
//!
//! Normalized Power®-style rolling-average metrics, a live rolling power
//! window for the HUD graph, and lap tracking. All pure over recorded
//! samples so they stay portable and testable headless.

use crate::ride_session::RideSample;

/// Coggan-style normalized power: 30 s rolling average of power, raised to
/// the 4th power, averaged, then the 4th root. `None` until at least 30 s
/// of samples exist.
pub fn normalized_power_w(samples: &[RideSample]) -> Option<f64> {
    const WINDOW_S: f64 = 30.0;
    if samples.is_empty() {
        return None;
    }
    let total = samples.last()?.elapsed_s - samples.first()?.elapsed_s;
    if total < WINDOW_S {
        return None;
    }

    // Rolling 30 s mean via a two-pointer pass (samples are time-ordered).
    let mut sum_p = 0.0_f64;
    let mut count = 0usize;
    let mut start = 0usize;
    let mut fourth_sum = 0.0_f64;
    let mut fourth_count = 0usize;

    for (i, s) in samples.iter().enumerate() {
        sum_p += s.power_w.unwrap_or(0.0);
        count += 1;
        while samples[start].elapsed_s < s.elapsed_s - WINDOW_S {
            sum_p -= samples[start].power_w.unwrap_or(0.0);
            count -= 1;
            start += 1;
        }
        // Only start contributing once the window is fully primed.
        if s.elapsed_s - samples[0].elapsed_s >= WINDOW_S {
            let mean = sum_p / count as f64;
            fourth_sum += mean.powi(4);
            fourth_count += 1;
        }
        let _ = i;
    }

    if fourth_count == 0 {
        return None;
    }
    Some((fourth_sum / fourth_count as f64).powf(0.25))
}

/// Intensity factor: NP / FTP.
pub fn intensity_factor(np_w: f64, ftp_w: f64) -> Option<f64> {
    if ftp_w > 0.0 {
        Some(np_w / ftp_w)
    } else {
        None
    }
}

/// Training stress score: (seconds × NP × IF) / (FTP × 3600) × 100.
pub fn training_stress_score(elapsed_s: f64, np_w: f64, ftp_w: f64) -> Option<f64> {
    let if_ = intensity_factor(np_w, ftp_w)?;
    Some(elapsed_s * np_w * if_ / (ftp_w * 3600.0) * 100.0)
}

/// Total climbing over the ride, from route grade × distance per sample.
pub fn elevation_gain_m(samples: &[RideSample]) -> f64 {
    samples
        .windows(2)
        .map(|w| {
            let dd = (w[1].distance_m - w[0].distance_m).max(0.0);
            let rise = w[0].grade * dd;
            if rise > 0.0 {
                rise
            } else {
                0.0
            }
        })
        .sum()
}

/// Post-ride metric block for the summary sheet.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RideMetrics {
    pub normalized_power_w: Option<f64>,
    pub intensity_factor: Option<f64>,
    pub tss: Option<f64>,
    pub elevation_gain_m: f64,
}

pub fn ride_metrics(samples: &[RideSample], ftp_w: f64) -> RideMetrics {
    let np = normalized_power_w(samples);
    let elapsed = samples
        .last()
        .map(|s| s.elapsed_s - samples.first().map(|f| f.elapsed_s).unwrap_or(0.0))
        .unwrap_or(0.0);
    RideMetrics {
        normalized_power_w: np,
        intensity_factor: np.and_then(|np| intensity_factor(np, ftp_w)),
        tss: np.and_then(|np| training_stress_score(elapsed, np, ftp_w)),
        elevation_gain_m: elevation_gain_m(samples),
    }
}

/// Live rolling power window for the HUD graph (default 60 s).
///
/// Push one value per sim tick; `series(n)` downsamples to `n` points for
/// rendering. O(1) push with a ring of (elapsed, watts).
#[derive(Debug, Clone)]
pub struct RollingPower {
    window_s: f64,
    buf: std::collections::VecDeque<(f64, f64)>,
}

impl Default for RollingPower {
    fn default() -> Self {
        Self::new(60.0)
    }
}

impl RollingPower {
    pub fn new(window_s: f64) -> Self {
        Self {
            window_s: window_s.max(1.0),
            buf: std::collections::VecDeque::new(),
        }
    }

    pub fn push(&mut self, elapsed_s: f64, power_w: f64) {
        self.buf.push_back((elapsed_s, power_w));
        while let Some(&(t, _)) = self.buf.front() {
            if t < elapsed_s - self.window_s {
                self.buf.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn avg_w(&self) -> Option<f64> {
        if self.buf.is_empty() {
            return None;
        }
        Some(self.buf.iter().map(|&(_, p)| p).sum::<f64>() / self.buf.len() as f64)
    }

    /// Downsample the window to `n` evenly spaced points (bucket means),
    /// oldest first. Fewer points are returned while the window fills.
    pub fn series(&self, n: usize) -> Vec<f64> {
        if self.buf.is_empty() || n == 0 {
            return Vec::new();
        }
        let len = self.buf.len();
        if len <= n {
            return self.buf.iter().map(|&(_, p)| p).collect();
        }
        let mut out = Vec::with_capacity(n);
        for b in 0..n {
            let lo = b * len / n;
            let hi = ((b + 1) * len / n).max(lo + 1);
            let mean = self.buf.iter().skip(lo).take(hi - lo).map(|&(_, p)| p).sum::<f64>()
                / (hi - lo) as f64;
            out.push(mean);
        }
        out
    }
}

/// One completed lap (manual lap button).
#[derive(Debug, Clone, PartialEq)]
pub struct Lap {
    pub index: u32,
    pub start_elapsed_s: f64,
    pub end_elapsed_s: f64,
    pub distance_m: f64,
    pub avg_power_w: Option<f64>,
}

/// Manual lap tracking over the live ride.
#[derive(Debug, Clone, Default)]
pub struct LapTracker {
    laps: Vec<Lap>,
    lap_start_s: f64,
    lap_start_distance_m: f64,
    power_sum: f64,
    power_count: u64,
}

impl LapTracker {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Feed each sim tick so the current lap accumulates averages.
    pub fn tick(&mut self, elapsed_s: f64, power_w: Option<f64>) {
        let _ = elapsed_s;
        if let Some(p) = power_w {
            self.power_sum += p;
            self.power_count += 1;
        }
    }

    /// Close the current lap at the given ride position and start the next.
    pub fn mark(&mut self, elapsed_s: f64, distance_m: f64) -> Lap {
        let lap = Lap {
            index: self.laps.len() as u32 + 1,
            start_elapsed_s: self.lap_start_s,
            end_elapsed_s: elapsed_s,
            distance_m: distance_m - self.lap_start_distance_m,
            avg_power_w: if self.power_count > 0 {
                Some(self.power_sum / self.power_count as f64)
            } else {
                None
            },
        };
        self.laps.push(lap.clone());
        self.lap_start_s = elapsed_s;
        self.lap_start_distance_m = distance_m;
        self.power_sum = 0.0;
        self.power_count = 0;
        lap
    }

    pub fn laps(&self) -> &[Lap] {
        &self.laps
    }

    /// Elapsed time in the current (unclosed) lap.
    pub fn current_lap_elapsed_s(&self, elapsed_s: f64) -> f64 {
        (elapsed_s - self.lap_start_s).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steady(power: f64, seconds: u32) -> Vec<RideSample> {
        (0..seconds * 10)
            .map(|i| RideSample {
                elapsed_s: i as f64 * 0.1,
                distance_m: i as f64,
                speed_mps: 10.0,
                power_w: Some(power),
                cadence_rpm: None,
                heart_rate_bpm: None,
                grade: 0.02,
            })
            .collect()
    }

    #[test]
    fn np_of_steady_ride_equals_power() {
        let np = normalized_power_w(&steady(200.0, 120)).unwrap();
        assert!((np - 200.0).abs() < 0.5, "steady NP should equal power: {np}");
    }

    #[test]
    fn np_none_for_short_rides() {
        assert!(normalized_power_w(&steady(200.0, 20)).is_none());
    }

    #[test]
    fn np_weighs_surges_above_average() {
        // 100 W with a 60 s surge to 400 W: NP must exceed the plain mean.
        let mut samples = steady(100.0, 300);
        for s in samples.iter_mut() {
            if s.elapsed_s >= 120.0 && s.elapsed_s < 180.0 {
                s.power_w = Some(400.0);
            }
        }
        let mean = samples.iter().filter_map(|s| s.power_w).sum::<f64>() / samples.len() as f64;
        let np = normalized_power_w(&samples).unwrap();
        assert!(np > mean + 20.0, "np {np} should be well above mean {mean}");
    }

    #[test]
    fn tss_one_hour_at_ftp_is_100() {
        let tss = training_stress_score(3600.0, 250.0, 250.0).unwrap();
        assert!((tss - 100.0).abs() < 1e-9);
        let if_ = intensity_factor(250.0, 250.0).unwrap();
        assert!((if_ - 1.0).abs() < 1e-9);
    }

    #[test]
    fn elevation_gain_counts_only_climbing() {
        // 2% grade over 1200 m of samples → 24 m gain.
        let gain = elevation_gain_m(&steady(200.0, 120));
        assert!((gain - 23.98).abs() < 0.5, "gain {gain}");
        // Descent contributes nothing.
        let mut down = steady(200.0, 60);
        for s in down.iter_mut() {
            s.grade = -0.05;
        }
        assert_eq!(elevation_gain_m(&down), 0.0);
    }

    #[test]
    fn rolling_power_window_and_series() {
        let mut rp = RollingPower::new(60.0);
        for i in 0..1200 {
            let t = i as f64 * 0.1;
            rp.push(t, if t < 60.0 { 100.0 } else { 300.0 });
        }
        // Window (last 60 s) should be all 300s.
        assert!((rp.avg_w().unwrap() - 300.0).abs() < 1.0);
        let series = rp.series(30);
        assert_eq!(series.len(), 30);
        // First bucket may contain the single boundary sample at exactly t-60.
        assert!(series.iter().skip(1).all(|&p| (p - 300.0).abs() < 1.0));
        assert!(series[0] > 280.0);
    }

    #[test]
    fn lap_tracker_marks_and_averages() {
        let mut laps = LapTracker::default();
        for i in 0..100 {
            laps.tick(i as f64 * 0.1, Some(200.0));
        }
        let lap1 = laps.mark(10.0, 80.0);
        assert_eq!(lap1.index, 1);
        assert!((lap1.distance_m - 80.0).abs() < 1e-9);
        assert!((lap1.avg_power_w.unwrap() - 200.0).abs() < 1e-9);

        for i in 0..50 {
            laps.tick(10.0 + i as f64 * 0.1, Some(300.0));
        }
        let lap2 = laps.mark(15.0, 120.0);
        assert_eq!(lap2.index, 2);
        assert!((lap2.distance_m - 40.0).abs() < 1e-9);
        assert!((lap2.avg_power_w.unwrap() - 300.0).abs() < 1e-9);
        assert_eq!(laps.laps().len(), 2);
        assert!((laps.current_lap_elapsed_s(18.0) - 3.0).abs() < 1e-9);
    }
}
