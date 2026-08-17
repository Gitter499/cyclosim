//! Cinematic replay camera for highlight clips (M5 finish).
//!
//! Given the recorded ride timeline and a highlight window, produce a
//! deterministic camera path the renderer replays instead of the live chase
//! camera. Pure math — the platform shell only samples poses and feeds frames
//! to its encoder.

use glam::DVec3;

use crate::highlight::HighlightClipRequest;
use crate::ride_session::RideSample;
use crate::route::RouteModel;

/// Camera movement vocabulary for clips. Styles are matched to the clip's
/// narrative label so a reel cuts between distinct shots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayCameraStyle {
    /// Rise from handlebar height to a high reveal behind the rider.
    DroneRise,
    /// Sweep around the rider (¾ orbit) at constant radius.
    OrbitRider,
    /// Static low trackside camera; the rider passes through frame.
    FlybyLow,
    /// Chase camera that pulls back and up as the clip ends.
    ChasePull,
}

/// One rider position on the replay track, in the route's local ENU frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiderTrackPoint {
    pub elapsed_s: f64,
    pub east: f64,
    pub up: f64,
    pub north: f64,
}

/// A camera pose in the same ENU frame as the rider track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPose {
    pub eye_east: f64,
    pub eye_up: f64,
    pub eye_north: f64,
    pub look_east: f64,
    pub look_up: f64,
    pub look_north: f64,
}

/// Deterministic camera path over one highlight clip.
#[derive(Debug, Clone)]
pub struct ReplayCamera {
    track: Vec<RiderTrackPoint>,
    style: ReplayCameraStyle,
    start_s: f64,
    duration_s: f64,
}

/// Map a highlight clip's label to a shot style (stable across runs).
pub fn style_for_label(label: &str) -> ReplayCameraStyle {
    match label {
        "Start" => ReplayCameraStyle::DroneRise,
        "Power surge" => ReplayCameraStyle::OrbitRider,
        "Mid-ride" => ReplayCameraStyle::FlybyLow,
        "Finish" => ReplayCameraStyle::ChasePull,
        _ => ReplayCameraStyle::ChasePull,
    }
}

/// Build the rider ENU track for a recorded ride over its route.
pub fn build_rider_track(route: &RouteModel, samples: &[RideSample]) -> Vec<RiderTrackPoint> {
    samples
        .iter()
        .map(|s| {
            let (east, up, north) = route.position_enu_at(s.distance_m);
            RiderTrackPoint {
                elapsed_s: s.elapsed_s,
                east,
                up,
                north,
            }
        })
        .collect()
}

impl ReplayCamera {
    /// Create a camera path for `clip` over the rider `track`.
    ///
    /// Returns `None` when the track is empty (nothing to film).
    pub fn new(
        track: Vec<RiderTrackPoint>,
        clip: &HighlightClipRequest,
        style: ReplayCameraStyle,
    ) -> Option<Self> {
        if track.is_empty() || clip.duration_s <= 0.0 {
            return None;
        }
        Some(Self {
            track,
            style,
            start_s: clip.start_elapsed_s,
            duration_s: clip.duration_s,
        })
    }

    /// Convenience: style chosen from the clip label.
    pub fn for_clip(track: Vec<RiderTrackPoint>, clip: &HighlightClipRequest) -> Option<Self> {
        Self::new(track, clip, style_for_label(&clip.label))
    }

    pub fn style(&self) -> ReplayCameraStyle {
        self.style
    }

    pub fn duration_s(&self) -> f64 {
        self.duration_s
    }

    /// Rider position at absolute ride time (clamped, linear interpolation).
    pub fn rider_at(&self, elapsed_s: f64) -> DVec3 {
        let track = &self.track;
        if elapsed_s <= track[0].elapsed_s {
            return point_vec(&track[0]);
        }
        let last = track.last().expect("track non-empty");
        if elapsed_s >= last.elapsed_s {
            return point_vec(last);
        }
        let idx = track
            .partition_point(|p| p.elapsed_s <= elapsed_s)
            .saturating_sub(1);
        let a = &track[idx];
        let b = &track[(idx + 1).min(track.len() - 1)];
        let span = (b.elapsed_s - a.elapsed_s).max(1e-9);
        let f = ((elapsed_s - a.elapsed_s) / span).clamp(0.0, 1.0);
        point_vec(a).lerp(point_vec(b), f)
    }

    /// Horizontal travel direction at absolute ride time (unit, ENU).
    fn forward_at(&self, elapsed_s: f64) -> DVec3 {
        let ahead = self.rider_at(elapsed_s + 0.5);
        let here = self.rider_at(elapsed_s);
        let mut dir = ahead - here;
        dir.y = 0.0;
        if dir.length_squared() < 1e-9 {
            DVec3::Z
        } else {
            dir.normalize()
        }
    }

    /// Camera pose at `clip_t` seconds into the clip (clamped to the clip).
    pub fn pose_at(&self, clip_t: f64) -> CameraPose {
        let t = clip_t.clamp(0.0, self.duration_s);
        let t01 = if self.duration_s > 0.0 {
            t / self.duration_s
        } else {
            0.0
        };
        let ride_t = self.start_s + t;
        let rider = self.rider_at(ride_t);
        let fwd = self.forward_at(ride_t);

        let (eye, look) = match self.style {
            ReplayCameraStyle::DroneRise => {
                let height = lerp(1.5, 9.0, ease(t01));
                let behind = lerp(2.5, 12.0, ease(t01));
                (
                    rider - fwd * behind + DVec3::Y * height,
                    rider + DVec3::Y * 1.0,
                )
            }
            ReplayCameraStyle::OrbitRider => {
                // ¾ orbit starting behind-left, constant radius and height.
                let angle = (-120.0 + 240.0 * t01).to_radians();
                let radius = 8.0;
                let side = DVec3::new(fwd.z, 0.0, -fwd.x); // right of travel
                let offset = (-fwd * angle.cos() + side * angle.sin()) * radius;
                (
                    rider + offset + DVec3::Y * 3.0,
                    rider + DVec3::Y * 1.0,
                )
            }
            ReplayCameraStyle::FlybyLow => {
                // Static camera planted ahead of the mid-clip rider position,
                // offset to the side of travel, near the ground.
                let mid_t = self.start_s + self.duration_s * 0.5;
                let anchor = self.rider_at(mid_t);
                let mid_fwd = self.forward_at(mid_t);
                let side = DVec3::new(mid_fwd.z, 0.0, -mid_fwd.x);
                let eye = anchor + mid_fwd * 6.0 + side * 4.0 + DVec3::Y * 1.0;
                (eye, rider + DVec3::Y * 1.0)
            }
            ReplayCameraStyle::ChasePull => {
                let behind = lerp(5.0, 14.0, ease(t01));
                let height = lerp(2.0, 5.5, ease(t01));
                (
                    rider - fwd * behind + DVec3::Y * height,
                    rider + fwd * 6.0 + DVec3::Y * 0.8,
                )
            }
        };

        CameraPose {
            eye_east: eye.x,
            eye_up: eye.y,
            eye_north: eye.z,
            look_east: look.x,
            look_up: look.y,
            look_north: look.z,
        }
    }

    /// Sample the whole clip at a fixed frame rate (for shell-side encoding).
    pub fn sample_fps(&self, fps: f64) -> Vec<CameraPose> {
        let fps = fps.clamp(1.0, 120.0);
        let frames = (self.duration_s * fps).round().max(1.0) as usize;
        (0..=frames)
            .map(|i| self.pose_at(i as f64 / fps))
            .collect()
    }
}

fn point_vec(p: &RiderTrackPoint) -> DVec3 {
    DVec3::new(p.east, p.up, p.north)
}

fn lerp(a: f64, b: f64, f: f64) -> f64 {
    a + (b - a) * f
}

/// Smoothstep easing — gentle in/out so shots don't start or stop abruptly.
fn ease(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(label: &str) -> HighlightClipRequest {
        HighlightClipRequest {
            start_elapsed_s: 10.0,
            duration_s: 4.0,
            label: label.into(),
        }
    }

    fn straight_track() -> Vec<RiderTrackPoint> {
        // North at 10 m/s from t=0..60.
        (0..=60)
            .map(|i| RiderTrackPoint {
                elapsed_s: i as f64,
                east: 0.0,
                up: 0.0,
                north: i as f64 * 10.0,
            })
            .collect()
    }

    #[test]
    fn styles_map_from_labels() {
        assert_eq!(style_for_label("Start"), ReplayCameraStyle::DroneRise);
        assert_eq!(style_for_label("Power surge"), ReplayCameraStyle::OrbitRider);
        assert_eq!(style_for_label("Mid-ride"), ReplayCameraStyle::FlybyLow);
        assert_eq!(style_for_label("Finish"), ReplayCameraStyle::ChasePull);
        assert_eq!(style_for_label("anything"), ReplayCameraStyle::ChasePull);
    }

    #[test]
    fn empty_track_yields_none() {
        assert!(ReplayCamera::for_clip(Vec::new(), &clip("Start")).is_none());
    }

    #[test]
    fn drone_rise_gains_height_and_distance() {
        let cam = ReplayCamera::for_clip(straight_track(), &clip("Start")).unwrap();
        let first = cam.pose_at(0.0);
        let last = cam.pose_at(4.0);
        assert!(last.eye_up > first.eye_up + 5.0);
        let rider_first = cam.rider_at(10.0);
        let rider_last = cam.rider_at(14.0);
        let d0 = (rider_first.z - first.eye_north).abs();
        let d1 = (rider_last.z - last.eye_north).abs();
        assert!(d1 > d0, "camera should fall further behind ({d0} -> {d1})");
    }

    #[test]
    fn orbit_crosses_sides() {
        let cam = ReplayCamera::for_clip(straight_track(), &clip("Power surge")).unwrap();
        let sides: Vec<f64> = (0..=8)
            .map(|i| {
                let pose = cam.pose_at(i as f64 * 0.5);
                let rider = cam.rider_at(10.0 + i as f64 * 0.5);
                pose.eye_east - rider.x // travel is +Z, so east = side offset
            })
            .collect();
        let min = sides.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = sides.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(min < -2.0 && max > 2.0, "orbit should cross both sides: {sides:?}");
    }

    #[test]
    fn flyby_camera_is_static_while_look_tracks_rider() {
        let cam = ReplayCamera::for_clip(straight_track(), &clip("Mid-ride")).unwrap();
        let a = cam.pose_at(0.5);
        let b = cam.pose_at(3.5);
        assert!((a.eye_east - b.eye_east).abs() < 1e-9);
        assert!((a.eye_north - b.eye_north).abs() < 1e-9);
        assert!(b.look_north > a.look_north + 10.0);
    }

    #[test]
    fn chase_pull_retreats() {
        let cam = ReplayCamera::for_clip(straight_track(), &clip("Finish")).unwrap();
        let d = |t: f64| {
            let pose = cam.pose_at(t);
            let rider = cam.rider_at(10.0 + t);
            ((pose.eye_north - rider.z).powi(2) + (pose.eye_up - rider.y).powi(2)).sqrt()
        };
        assert!(d(4.0) > d(0.0) + 4.0);
    }

    #[test]
    fn sample_fps_covers_clip() {
        let cam = ReplayCamera::for_clip(straight_track(), &clip("Finish")).unwrap();
        let poses = cam.sample_fps(30.0);
        assert_eq!(poses.len(), 121); // 4 s * 30 fps + inclusive end
        assert_eq!(poses[0], cam.pose_at(0.0));
        assert_eq!(poses[120], cam.pose_at(4.0));
    }

    #[test]
    fn track_interpolation_clamps_ends() {
        let cam = ReplayCamera::for_clip(straight_track(), &clip("Start")).unwrap();
        assert_eq!(cam.rider_at(-5.0), DVec3::new(0.0, 0.0, 0.0));
        assert_eq!(cam.rider_at(1000.0), DVec3::new(0.0, 0.0, 600.0));
        let mid = cam.rider_at(30.5);
        assert!((mid.z - 305.0).abs() < 1e-9);
    }
}
