//! Headless frame rendering + PNG encoding for evaluation tools.

use velo_core::VeloApp;
use velo_render::{forward_from_enu, FramebufferRgba, HudSnapshot, Renderer, RouteFollow};

use crate::scenario::workout_hud_fields;

/// Create a headless renderer, mapping adapter absence to a friendly error.
pub fn headless_renderer(width: u32, height: u32) -> Result<Renderer, String> {
    Renderer::headless(width.clamp(64, 3840), height.clamp(64, 2160)).map_err(|e| {
        format!("headless renderer unavailable ({e}); install a Vulkan driver (e.g. mesa lavapipe) or run on a GPU host")
    })
}

/// Build the HUD snapshot the shell would show for the app's current state.
pub fn hud_from_app(app: &VeloApp, mode_label: &'static str) -> HudSnapshot {
    let (interval, target_w) = workout_hud_fields(app);
    HudSnapshot {
        power_w: app.ride.power_w,
        cadence_rpm: app.ride.cadence_rpm,
        heart_rate_bpm: app.ride.heart_rate_bpm,
        speed_mps: app.ride.speed_mps,
        distance_m: app.ride.distance_m,
        elapsed_s: app.ride.elapsed_s,
        grade: app.ride.grade,
        mode: mode_label,
        workout_interval: interval,
        workout_target_w: target_w,
        attribution: None,
    }
}

/// Chase-camera follow state for the app's route position (None off-route).
pub fn follow_from_app(app: &VeloApp) -> Option<RouteFollow> {
    let route = app.route.as_ref()?;
    let d = app.ride.distance_m;
    let (east, up, north) = route.position_enu_at(d);
    let (east_ahead, _, north_ahead) = route.position_enu_at(d + 5.0);
    Some(RouteFollow {
        east,
        up,
        north,
        forward: forward_from_enu(east, up, north, east_ahead, north_ahead),
    })
}

/// Render one frame and return encoded PNG bytes.
pub fn capture_png(
    renderer: &mut Renderer,
    hud: &HudSnapshot,
    distance_m: f64,
    follow: Option<RouteFollow>,
) -> Result<Vec<u8>, String> {
    let frame = renderer
        .capture_framebuffer_rgba(hud, distance_m, follow)
        .map_err(|e| e.to_string())?;
    encode_png(&frame)
}

/// Encode an RGBA framebuffer as PNG.
pub fn encode_png(frame: &FramebufferRgba) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, frame.width, frame.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&frame.pixels)
            .map_err(|e| e.to_string())?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_png_emits_magic() {
        let frame = FramebufferRgba {
            width: 2,
            height: 2,
            pixels: vec![128; 16],
        };
        let png = encode_png(&frame).unwrap();
        assert_eq!(&png[..8], &velo_render::PNG_MAGIC);
    }
}
