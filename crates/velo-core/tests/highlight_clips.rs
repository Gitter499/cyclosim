//! Golden expectations for highlight clip window planning (ride stop → clip DTOs).

use velo_core::{plan_highlight_clips, ride_session::RideSample};

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
fn golden_120s_ride_with_power_peak() {
    let mut samples: Vec<_> = (1..=120).map(|i| sample(i as f64, 180.0)).collect();
    samples[59].power_w = Some(450.0);

    let clips = plan_highlight_clips(&samples, 120.0);

    assert_eq!(clips.len(), 4);
    assert_eq!(clips[0].label, "Start");
    assert!((clips[0].start_elapsed_s - 0.0).abs() < f64::EPSILON);
    assert!((clips[0].duration_s - 5.0).abs() < f64::EPSILON);

    let peak = clips.iter().find(|c| c.label == "Power surge").expect("peak clip");
    assert!((peak.start_elapsed_s - 57.5).abs() < 0.01);
    assert!((peak.duration_s - 5.0).abs() < f64::EPSILON);

    assert!(clips.iter().any(|c| c.label == "Mid-ride"));
    assert_eq!(clips.last().unwrap().label, "Finish");
    assert!((clips.last().unwrap().start_elapsed_s - 115.0).abs() < f64::EPSILON);
}

#[test]
fn golden_short_ride_single_window() {
    let samples: Vec<_> = (1..=5).map(|i| sample(i as f64 * 0.5, 200.0)).collect();
    let clips = plan_highlight_clips(&samples, 2.5);

    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].label, "Ride");
    assert!((clips[0].duration_s - 2.5).abs() < f64::EPSILON);
}
