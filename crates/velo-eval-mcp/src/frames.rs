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
    let (interval_duration_s, interval_elapsed_s) = app
        .workout_engine
        .as_ref()
        .and_then(|e| {
            let st = e.state();
            e.current_interval()
                .map(|i| (Some(i.duration_s), Some(st.interval_elapsed_s)))
        })
        .unwrap_or((None, None));
    let elevation_m = app
        .route
        .as_ref()
        .map(|r| r.lat_lon_elev_at(app.ride.distance_m).2);
    let elevation_profile: Vec<f32> = app
        .route
        .as_ref()
        .map(|r| {
            r.elevation_profile(48)
                .into_iter()
                .map(|(_, e)| e as f32)
                .collect()
        })
        .unwrap_or_default();
    let route_total_m = app.route.as_ref().map(|r| r.total_distance_m());
    HudSnapshot {
        ftp_w: Some(app.ftp()),
        power_w: app.ride.power_w,
        cadence_rpm: app.ride.cadence_rpm,
        heart_rate_bpm: app.ride.heart_rate_bpm,
        speed_mps: app.ride.speed_mps,
        distance_m: app.ride.distance_m,
        elapsed_s: app.ride.elapsed_s,
        grade: app.ride.grade,
        elevation_m,
        mode: mode_label,
        workout_interval: interval,
        workout_target_w: target_w,
        interval_duration_s,
        interval_elapsed_s,
        attribution: None,
        elevation_profile,
        route_total_m,
        workout_next_interval: app
            .workout_engine
            .as_ref()
            .and_then(|e| e.next_interval())
            .map(|i| i.name.clone()),
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
    steer_yaw_rad: f32,
) -> Result<Vec<u8>, String> {
    let frame = renderer
        .capture_framebuffer_rgba(hud, distance_m, follow, steer_yaw_rad)
        .map_err(|e| e.to_string())?;
    encode_png(&frame)
}

/// Bake a synthetic Tier A terrain pack for the route and load it, so frames
/// exercise the textured-terrain pass instead of the fallback grid.
pub fn bake_and_load_terrain(
    renderer: &mut Renderer,
    route: &velo_core::RouteModel,
) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!(
        "velo-eval-terrain-{}-{}",
        std::process::id(),
        route.meta.route_id
    ));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Finer than the app defaults so the road band renders crisply in evals:
    // cell 3 m × the 4x upsample = 0.75 m texels (markings on) up to ~4 km
    // routes; longer routes degrade gracefully under the texture cap.
    let bake = velo_terrain::bake_terrain_for_route(route, &dir, 120.0, 3.0)
    .map(|_| ())
    .map_err(|e| e.to_string());
    let load = bake.and_then(|()| {
        renderer
            .load_terrain_pack(&dir)
            .map_err(|e| e.to_string())
    });
    if load.is_ok() {
        renderer.load_scenery_for_route(route);
    }
    let _ = std::fs::remove_dir_all(&dir);
    load
}

/// Load the procedural placeholder bike so frames have a visible rider.
pub fn load_placeholder_bike(renderer: &mut Renderer) -> Result<(), String> {
    let tmp = std::env::temp_dir();
    let pid = std::process::id();

    // The generator tints from source images; synthesize a small red one.
    let tint_path = tmp.join(format!("velo-eval-bike-tint-{pid}.png"));
    let tint = FramebufferRgba {
        width: 2,
        height: 2,
        pixels: vec![
            200, 40, 30, 255, 200, 40, 30, 255, //
            200, 40, 30, 255, 200, 40, 30, 255,
        ],
    };
    std::fs::write(&tint_path, encode_png(&tint)?).map_err(|e| e.to_string())?;

    let glb = velo_bikegen::placeholder::generate_placeholder_glb(&[&tint_path])
        .map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&tint_path);

    let glb_path = tmp.join(format!("velo-eval-bike-{pid}.glb"));
    std::fs::write(&glb_path, glb?).map_err(|e| e.to_string())?;
    let result = renderer
        .load_bike_gltf(
            &glb_path,
            velo_bikegen::placeholder::default_placeholder_anchor(),
        )
        .map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&glb_path);
    result
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
