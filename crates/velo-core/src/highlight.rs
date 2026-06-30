//! Highlight clip moment selection — core owns the ride timeline (§14).

use crate::ride_session::RideSample;

/// A window of ride time to include in the post-ride highlight reel.
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightClipRequest {
    pub start_elapsed_s: f64,
    pub duration_s: f64,
    pub label: String,
}

const MIN_CLIP_S: f64 = 3.0;
const MAX_CLIP_S: f64 = 5.0;
const MAX_CLIPS: usize = 4;

/// Pick 2–4 simple highlight windows from recorded samples.
pub fn plan_highlight_clips(samples: &[RideSample], total_elapsed_s: f64) -> Vec<HighlightClipRequest> {
    if samples.is_empty() || total_elapsed_s <= 0.0 {
        return Vec::new();
    }

    let clip_len = (total_elapsed_s / 6.0).clamp(MIN_CLIP_S, MAX_CLIP_S);

    if total_elapsed_s < clip_len * 1.5 {
        return vec![HighlightClipRequest {
            start_elapsed_s: 0.0,
            duration_s: total_elapsed_s.min(MAX_CLIP_S),
            label: "Ride".into(),
        }];
    }

    let mut clips = Vec::new();

    clips.push(HighlightClipRequest {
        start_elapsed_s: 0.0,
        duration_s: clip_len,
        label: "Start".into(),
    });

    if let Some(peak) = max_power_sample(samples) {
        let start = (peak.elapsed_s - clip_len / 2.0).clamp(0.0, (total_elapsed_s - clip_len).max(0.0));
        if !overlaps_existing(&clips, start, clip_len) {
            clips.push(HighlightClipRequest {
                start_elapsed_s: start,
                duration_s: clip_len,
                label: "Power surge".into(),
            });
        }
    }

    if total_elapsed_s >= clip_len * 3.0 {
        let third = total_elapsed_s / 3.0;
        let start = (third - clip_len / 2.0).clamp(0.0, total_elapsed_s - clip_len);
        if !overlaps_existing(&clips, start, clip_len) {
            clips.push(HighlightClipRequest {
                start_elapsed_s: start,
                duration_s: clip_len,
                label: "Mid-ride".into(),
            });
        }
    }

    let finish_start = (total_elapsed_s - clip_len).max(0.0);
    if !overlaps_existing(&clips, finish_start, clip_len) {
        clips.push(HighlightClipRequest {
            start_elapsed_s: finish_start,
            duration_s: clip_len,
            label: "Finish".into(),
        });
    }

    clips.truncate(MAX_CLIPS);
    clips
}

fn max_power_sample(samples: &[RideSample]) -> Option<&RideSample> {
    samples
        .iter()
        .filter(|s| s.power_w.is_some())
        .max_by(|a, b| {
            a.power_w
                .unwrap_or(0.0)
                .partial_cmp(&b.power_w.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn overlaps_existing(clips: &[HighlightClipRequest], start: f64, duration: f64) -> bool {
    let end = start + duration;
    clips.iter().any(|c| {
        let c_end = c.start_elapsed_s + c.duration_s;
        start < c_end && end > c.start_elapsed_s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(elapsed: f64, power: f64) -> RideSample {
        RideSample {
            elapsed_s: elapsed,
            distance_m: elapsed * 8.0,
            speed_mps: 8.0,
            power_w: Some(power),
            cadence_rpm: Some(90.0),
            heart_rate_bpm: Some(140.0),
            grade: 0.0,
        }
    }

    #[test]
    fn empty_samples_yield_no_clips() {
        assert!(plan_highlight_clips(&[], 60.0).is_empty());
    }

    #[test]
    fn short_ride_single_clip() {
        let samples: Vec<_> = (1..=5).map(|i| sample(i as f64 * 0.5, 180.0)).collect();
        let clips = plan_highlight_clips(&samples, 2.5);
        assert_eq!(clips.len(), 1);
        assert_eq!(clips[0].label, "Ride");
    }

    #[test]
    fn long_ride_includes_start_peak_and_finish() {
        let mut samples: Vec<_> = (1..=120).map(|i| sample(i as f64, 180.0)).collect();
        samples[60].power_w = Some(450.0);
        let clips = plan_highlight_clips(&samples, 120.0);
        assert!(clips.len() >= 2);
        assert_eq!(clips[0].label, "Start");
        assert!(clips.iter().any(|c| c.label == "Power surge"));
        assert!(clips.last().unwrap().label == "Finish");
    }

    #[test]
    fn clips_do_not_overlap() {
        let samples: Vec<_> = (1..=30).map(|i| sample(i as f64, 200.0)).collect();
        let clips = plan_highlight_clips(&samples, 30.0);
        for i in 0..clips.len() {
            for j in (i + 1)..clips.len() {
                let a = &clips[i];
                let b = &clips[j];
                let overlap = a.start_elapsed_s < b.start_elapsed_s + b.duration_s
                    && a.start_elapsed_s + a.duration_s > b.start_elapsed_s;
                assert!(!overlap, "clips {:?} and {:?} overlap", a.label, b.label);
            }
        }
    }
}
