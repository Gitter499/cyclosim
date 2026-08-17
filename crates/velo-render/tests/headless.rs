//! Headless offscreen rendering — the substrate for CI snapshots and velo-eval-mcp.
//!
//! Skips (rather than fails) when the host has no wgpu adapter at all.

use velo_render::{FramebufferRgba, HudSnapshot, Renderer};

fn headless_or_skip(width: u32, height: u32) -> Option<Renderer> {
    match Renderer::headless(width, height) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("skipping headless render test: {e}");
            None
        }
    }
}

#[test]
fn headless_capture_produces_full_frame() {
    let Some(mut renderer) = headless_or_skip(320, 200) else {
        return;
    };
    let hud = HudSnapshot {
        power_w: Some(250.0),
        speed_mps: 10.0,
        distance_m: 1234.0,
        elapsed_s: 65.0,
        grade: 0.04,
        mode: "SIM",
        ..Default::default()
    };
    let frame = renderer
        .capture_framebuffer_rgba(&hud, 1234.0, None, 0.0)
        .expect("capture");
    assert_eq!(frame.width, 320);
    assert_eq!(frame.height, 200);
    assert_eq!(frame.byte_len(), FramebufferRgba::expected_len(320, 200));

    // The clear color is a sky blue — the frame must not be all-black.
    let non_black = frame
        .pixels
        .chunks_exact(4)
        .filter(|px| px[0] > 8 || px[1] > 8 || px[2] > 8)
        .count();
    assert!(
        non_black > (320 * 200) / 2,
        "expected mostly non-black frame, got {non_black} non-black pixels"
    );
}

#[test]
fn headless_resize_changes_capture_dimensions() {
    let Some(mut renderer) = headless_or_skip(160, 120) else {
        return;
    };
    renderer.resize(200, 150);
    let hud = HudSnapshot::default();
    let frame = renderer
        .capture_framebuffer_rgba(&hud, 0.0, None, 0.0)
        .expect("capture after resize");
    assert_eq!((frame.width, frame.height), (200, 150));
}

#[test]
fn headless_render_frame_without_capture_ok() {
    let Some(mut renderer) = headless_or_skip(64, 64) else {
        return;
    };
    let hud = HudSnapshot::default();
    renderer.render_frame(&hud, 0.0, None, 0.0).expect("render");
}
