//! Synthetic placeholder glTF generator for CI and offline use.

use std::io::Cursor;
use std::path::Path;

use thiserror::Error;

use crate::asset::AnchorTransform;

pub const PLACEHOLDER_GENERATOR: &str = "placeholder-v1";

/// Target wheelbase for normalized bike models (meters).
pub const TARGET_WHEELBASE_M: f32 = 1.05;

#[derive(Debug, Error)]
pub enum PlaceholderError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("png: {0}")]
    Png(#[from] png::DecodingError),
    #[error("no images provided")]
    NoImages,
}

/// Average RGB from the first decodable PNG/JPEG path; neutral gray on failure.
pub fn sample_image_color(paths: &[impl AsRef<Path>]) -> [f32; 3] {
    for path in paths {
        if let Some(rgb) = try_read_image_color(path.as_ref()) {
            return rgb;
        }
    }
    [0.55, 0.58, 0.62]
}

fn try_read_image_color(path: &Path) -> Option<[f32; 3]> {
    let data = std::fs::read(path).ok()?;
    let decoder = png::Decoder::new(Cursor::new(&data));
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).ok()?;
    let pixels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return None,
    };
    if buf.len() < pixels {
        return None;
    }
    let mut r = 0u64;
    let mut g = 0u64;
    let mut b = 0u64;
    let count = (buf.len() / pixels).max(1) as u64;
    for chunk in buf.chunks(pixels) {
        r += chunk[0] as u64;
        g += chunk[1] as u64;
        b += chunk[2] as u64;
    }
    Some([
        (r / count) as f32 / 255.0,
        (g / count) as f32 / 255.0,
        (b / count) as f32 / 255.0,
    ])
}

/// Build a simple bike-shaped placeholder GLB tinted from source image metadata.
pub fn generate_placeholder_glb(image_paths: &[impl AsRef<Path>]) -> Result<Vec<u8>, PlaceholderError> {
    if image_paths.is_empty() {
        return Err(PlaceholderError::NoImages);
    }
    let color = sample_image_color(image_paths);
    Ok(build_bike_placeholder_glb(color))
}

/// Normalize placeholder anchor: scale to target wheelbase, sit on ground at origin.
pub fn default_placeholder_anchor() -> AnchorTransform {
    AnchorTransform {
        translation: [0.0, 0.0, 0.0],
        rotation_y: 0.0,
        scale: 1.0,
    }
}

fn build_bike_placeholder_glb(frame_color: [f32; 3]) -> Vec<u8> {
    let wheel_dark = [0.10, 0.10, 0.12];
    // Jersey reads brighter than the frame so the rider pops from the chase cam.
    let jersey = [
        (frame_color[0] * 1.25 + 0.10).min(1.0),
        (frame_color[1] * 1.25 + 0.10).min(1.0),
        (frame_color[2] * 1.25 + 0.10).min(1.0),
    ];
    // Lightest value on the model so the head silhouette reads at chase
    // distance (game-graphics skill §3 Rider) — a near-white shell with a
    // hint of the frame tint.
    let helmet = [
        (frame_color[0] * 0.20 + 0.74_f32).min(1.0),
        (frame_color[1] * 0.20 + 0.74_f32).min(1.0),
        (frame_color[2] * 0.20 + 0.76_f32).min(1.0),
    ];

    // Wheel centers at ±0.5 m on X; frame spans between. Total wheelbase ≈ 1.0 m before normalization.
    let wheel_r = 0.35_f32;
    let wheel_y = wheel_r;
    let positions: Vec<[f32; 3]> = vec![
        // Top tube (quad as two triangles)
        [-0.15, wheel_y + 0.35, 0.0],
        [0.35, wheel_y + 0.42, 0.0],
        [0.35, wheel_y + 0.38, 0.0],
        [-0.15, wheel_y + 0.31, 0.0],
        // Down tube
        [-0.45, wheel_y + 0.05, 0.0],
        [0.35, wheel_y + 0.40, 0.0],
        [0.32, wheel_y + 0.36, 0.0],
        [-0.42, wheel_y + 0.08, 0.0],
        // Seat tube
        [-0.12, wheel_y + 0.05, 0.0],
        [-0.08, wheel_y + 0.38, 0.0],
        [-0.12, wheel_y + 0.34, 0.0],
        [-0.16, wheel_y + 0.05, 0.0],
        // Rear wheel (octagon)
        wheel_vertex(-0.50, wheel_y, wheel_r, 0),
        wheel_vertex(-0.50, wheel_y, wheel_r, 1),
        wheel_vertex(-0.50, wheel_y, wheel_r, 2),
        wheel_vertex(-0.50, wheel_y, wheel_r, 3),
        wheel_vertex(-0.50, wheel_y, wheel_r, 4),
        wheel_vertex(-0.50, wheel_y, wheel_r, 5),
        wheel_vertex(-0.50, wheel_y, wheel_r, 6),
        wheel_vertex(-0.50, wheel_y, wheel_r, 7),
        // Front wheel (octagon)
        wheel_vertex(0.50, wheel_y, wheel_r, 0),
        wheel_vertex(0.50, wheel_y, wheel_r, 1),
        wheel_vertex(0.50, wheel_y, wheel_r, 2),
        wheel_vertex(0.50, wheel_y, wheel_r, 3),
        wheel_vertex(0.50, wheel_y, wheel_r, 4),
        wheel_vertex(0.50, wheel_y, wheel_r, 5),
        wheel_vertex(0.50, wheel_y, wheel_r, 6),
        wheel_vertex(0.50, wheel_y, wheel_r, 7),
    ];

    let mut indices: Vec<u16> = Vec::new();
    // Frame quads
    for base in [0u16, 4, 8] {
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    // Wheels (fan from center — duplicate center verts per wheel for simplicity)
    let rear_center = positions.len() as u16;
    let front_center = rear_center + 1;
    let rear_start = 12u16;
    let front_start = 20u16;
    for i in 0..8 {
        let next = (i + 1) % 8;
        indices.extend([
            rear_center,
            rear_start + i as u16,
            rear_start + next as u16,
        ]);
        indices.extend([
            front_center,
            front_start + i as u16,
            front_start + next as u16,
        ]);
    }

    let mut all_positions = positions;
    all_positions.push([-0.50, wheel_y, 0.0]);
    all_positions.push([0.50, wheel_y, 0.0]);

    // Per-vertex colors: frame tubes in the tint, wheels dark.
    let mut colors: Vec<[f32; 3]> = Vec::with_capacity(all_positions.len());
    colors.extend(std::iter::repeat(frame_color).take(12)); // frame quads
    colors.extend(std::iter::repeat(wheel_dark).take(16)); // wheel rims
    colors.extend(std::iter::repeat(wheel_dark).take(2)); // wheel centers

    let push_quad = |positions: &mut Vec<[f32; 3]>,
                         colors: &mut Vec<[f32; 3]>,
                         indices: &mut Vec<u16>,
                         quad: [[f32; 3]; 4],
                         color: [f32; 3]| {
        let base = positions.len() as u16;
        positions.extend_from_slice(&quad);
        colors.extend(std::iter::repeat(color).take(4));
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    };

    // The frame and wheels are planar (z = 0), which makes the bike invisible
    // dead-astern — exactly the chase camera's view. Everything below spans z
    // (or sits in a YZ plane) so the silhouette reads from behind too.
    let handlebar_y = wheel_y + 0.48;
    let saddle_y = wheel_y + 0.44;

    // Handlebar: narrow bar across the travel axis at the front.
    push_quad(
        &mut all_positions,
        &mut colors,
        &mut indices,
        [
            [0.33, handlebar_y, -0.22],
            [0.37, handlebar_y, -0.22],
            [0.37, handlebar_y, 0.22],
            [0.33, handlebar_y, 0.22],
        ],
        wheel_dark,
    );
    // Saddle: short and wider than the frame plane, at the rear.
    push_quad(
        &mut all_positions,
        &mut colors,
        &mut indices,
        [
            [-0.17, saddle_y, -0.08],
            [-0.05, saddle_y, -0.08],
            [-0.05, saddle_y, 0.08],
            [-0.17, saddle_y, 0.08],
        ],
        wheel_dark,
    );

    // Rider torso, astern-facing: a forward-leaning quad spanning z so the
    // chase camera sees shoulders instead of a paper edge. Two-tone: cool
    // shaded waist grading to sun-lit shoulders (game-graphics skill §3).
    let jersey_shade = [jersey[0] * 0.68, jersey[1] * 0.66, jersey[2] * 0.75];
    // Extra lift on the lit tone: the grayscale check (skill §1.2) showed
    // the jersey's value sitting too close to the road's midtone.
    let jersey_lit = [
        (jersey[0] * 1.22 + 0.10_f32).min(1.0),
        (jersey[1] * 1.16 + 0.08_f32).min(1.0),
        (jersey[2] * 1.08 + 0.05_f32).min(1.0),
    ];
    // Waist narrower than shoulders so the back reads as a person, not a
    // plank (silhouette first — game-graphics skill §1).
    {
        let base = all_positions.len() as u16;
        all_positions.extend_from_slice(&[
            [-0.10, saddle_y + 0.02, -0.11],
            [-0.10, saddle_y + 0.02, 0.11],
            [0.10, saddle_y + 0.55, 0.19],
            [0.10, saddle_y + 0.55, -0.19],
        ]);
        colors.extend_from_slice(&[jersey_shade, jersey_shade, jersey_lit, jersey_lit]);
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    // Arms: from the shoulders down-forward to the bar ends. Shaded — they
    // face away from the sun from the chase view.
    for zs in [-1.0_f32, 1.0] {
        let (sz, bz) = (0.16 * zs, 0.20 * zs);
        push_quad(
            &mut all_positions,
            &mut colors,
            &mut indices,
            [
                [0.08, saddle_y + 0.50, sz - 0.04 * zs],
                [0.08, saddle_y + 0.50, sz + 0.04 * zs],
                [0.34, handlebar_y + 0.02, bz + 0.035 * zs],
                [0.34, handlebar_y + 0.02, bz - 0.035 * zs],
            ],
            jersey_shade,
        );
    }
    // Rider torso, side profile: saddle to bars in the frame plane.
    push_quad(
        &mut all_positions,
        &mut colors,
        &mut indices,
        [
            [-0.12, saddle_y, 0.0],
            [0.33, handlebar_y, 0.0],
            [0.33, handlebar_y + 0.18, 0.0],
            [-0.12, saddle_y + 0.35, 0.0],
        ],
        jersey,
    );
    // Head: a round helmet disc in the YZ plane (billboard toward the chase
    // cam) — the old flat quad read as a floating chip, not a head.
    {
        let (hx, hy, hr) = (0.12_f32, saddle_y + 0.60, 0.095_f32);
        let center = all_positions.len() as u16;
        all_positions.push([hx, hy, 0.0]);
        colors.push(helmet);
        let rim_start = all_positions.len() as u16;
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::TAU / 8.0;
            all_positions.push([hx, hy + hr * angle.sin(), hr * angle.cos()]);
            colors.push(helmet);
        }
        for i in 0..8u16 {
            let next = (i + 1) % 8;
            indices.extend([center, rim_start + i, rim_start + next]);
        }
        // Side-profile helmet sliver so the head survives a profile view.
        push_quad(
            &mut all_positions,
            &mut colors,
            &mut indices,
            [
                [hx - hr, hy - hr * 0.4, 0.0],
                [hx + hr, hy - hr * 0.4, 0.0],
                [hx + hr * 0.8, hy + hr, 0.0],
                [hx - hr * 0.6, hy + hr, 0.0],
            ],
            helmet,
        );
    }
    // Astern wheel strips: thin vertical quads in the YZ plane at each hub, so
    // the wheels read as tire profiles from behind instead of vanishing.
    for cx in [-0.50_f32, 0.50] {
        push_quad(
            &mut all_positions,
            &mut colors,
            &mut indices,
            [
                [cx, 0.0, -0.03],
                [cx, 0.0, 0.03],
                [cx, 2.0 * wheel_r, 0.03],
                [cx, 2.0 * wheel_r, -0.03],
            ],
            wheel_dark,
        );
    }

    // Crossed wheel discs (YZ plane) at each hub — the tree-billboard trick.
    // At chase distance the strips above are sub-pixel; a full rotated disc
    // keeps the wheels visible from dead astern (r19 eval: bike read as a
    // torso slab on a stick).
    for cx in [-0.50_f32, 0.50] {
        let center = all_positions.len() as u16;
        all_positions.push([cx, wheel_y, 0.0]);
        colors.push(wheel_dark);
        let rim_start = all_positions.len() as u16;
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::TAU / 8.0;
            all_positions.push([cx, wheel_y + wheel_r * angle.sin(), wheel_r * angle.cos()]);
            colors.push(wheel_dark);
        }
        for i in 0..8u16 {
            let next = (i + 1) % 8;
            indices.extend([center, rim_start + i, rim_start + next]);
        }
    }

    // Contact blob shadow spanning both wheels: a flat dark capsule-ish
    // octagon strip at ground level grounds the rider (skill §3 — the
    // single biggest believability win at chase distance).
    {
        let shadow = [0.13_f32, 0.13, 0.14];
        let (x0, x1, half_w, yy) = (-0.62_f32, 0.62_f32, 0.20_f32, 0.012_f32);
        let base = all_positions.len() as u16;
        all_positions.extend_from_slice(&[
            [x0 + 0.15, yy, -half_w],
            [x1 - 0.15, yy, -half_w],
            [x1, yy, 0.0],
            [x1 - 0.15, yy, half_w],
            [x0 + 0.15, yy, half_w],
            [x0, yy, 0.0],
        ]);
        colors.extend(std::iter::repeat(shadow).take(6));
        indices.extend([
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
            base,
            base + 3,
            base + 4,
            base,
            base + 4,
            base + 5,
        ]);
    }

    // Rider legs: two dark vertical quads from saddle height down toward the
    // cranks, spanning z on either side of the frame plane.
    let shorts = [
        frame_color[0] * 0.22,
        frame_color[1] * 0.22,
        frame_color[2] * 0.22,
    ];
    for zc in [-0.10_f32, 0.10] {
        push_quad(
            &mut all_positions,
            &mut colors,
            &mut indices,
            [
                [-0.13, wheel_y + 0.05, zc - 0.05],
                [-0.13, wheel_y + 0.05, zc + 0.05],
                [-0.08, saddle_y + 0.04, zc + 0.05],
                [-0.08, saddle_y + 0.04, zc - 0.05],
            ],
            shorts,
        );
    }

    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; all_positions.len()];
    build_colored_glb(&all_positions, &uvs, &indices, &colors)
}

fn wheel_vertex(cx: f32, cy: f32, r: f32, i: usize) -> [f32; 3] {
    let angle = (i as f32) * std::f32::consts::TAU / 8.0;
    [cx + r * angle.cos(), cy + r * angle.sin(), 0.0]
}

fn build_colored_glb(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u16],
    colors: &[[f32; 3]],
) -> Vec<u8> {
    debug_assert_eq!(positions.len(), colors.len());
    let mut bin = Vec::new();
    let pos_offset = 0usize;
    for p in positions {
        for v in p {
            bin.extend_from_slice(&v.to_le_bytes());
        }
    }
    let uv_offset = bin.len();
    for uv in uvs {
        for v in uv {
            bin.extend_from_slice(&v.to_le_bytes());
        }
    }
    let color_offset = bin.len();
    for color in colors {
        for c in color {
            bin.extend_from_slice(&c.to_le_bytes());
        }
    }
    let idx_offset = bin.len();
    for i in indices {
        bin.extend_from_slice(&i.to_le_bytes());
    }

    let pos_byte_len = positions.len() * 12;
    let uv_byte_len = uvs.len() * 8;
    let color_byte_len = positions.len() * 12;
    let idx_byte_len = indices.len() * 2;

    let (min, max) = bbox(positions);

    let json = format!(
        r#"{{
  "asset": {{"version": "2.0", "generator": "{gen}"}},
  "buffers": [{{"byteLength": {bin_len}}}],
  "bufferViews": [
    {{"buffer": 0, "byteOffset": {pos_off}, "byteLength": {pos_len}, "target": 34962}},
    {{"buffer": 0, "byteOffset": {uv_off}, "byteLength": {uv_len}, "target": 34962}},
    {{"buffer": 0, "byteOffset": {color_off}, "byteLength": {color_len}, "target": 34962}},
    {{"buffer": 0, "byteOffset": {idx_off}, "byteLength": {idx_len}, "target": 34963}}
  ],
  "accessors": [
    {{"bufferView": 0, "componentType": 5126, "count": {vcount}, "type": "VEC3",
      "max": [{max_x}, {max_y}, {max_z}], "min": [{min_x}, {min_y}, {min_z}]}},
    {{"bufferView": 1, "componentType": 5126, "count": {vcount}, "type": "VEC2"}},
    {{"bufferView": 2, "componentType": 5126, "count": {vcount}, "type": "VEC3"}},
    {{"bufferView": 3, "componentType": 5123, "count": {icount}, "type": "SCALAR"}}
  ],
  "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0, "TEXCOORD_0": 1, "COLOR_0": 2}}, "indices": 3}}]}}]
}}"#,
        gen = PLACEHOLDER_GENERATOR,
        bin_len = bin.len(),
        pos_off = pos_offset,
        pos_len = pos_byte_len,
        uv_off = uv_offset,
        uv_len = uv_byte_len,
        color_off = color_offset,
        color_len = color_byte_len,
        idx_off = idx_offset,
        idx_len = idx_byte_len,
        vcount = positions.len(),
        icount = indices.len(),
        min_x = min[0],
        min_y = min[1],
        min_z = min[2],
        max_x = max[0],
        max_y = max[1],
        max_z = max[2],
    );

    let mut json_bytes = json.into_bytes();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    let mut glb = Vec::new();
    let total_len = 12 + 8 + json_bytes.len() + 8 + bin.len();
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\x00");
    glb.extend_from_slice(&bin);
    glb
}

fn bbox(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for (i, v) in p.iter().enumerate() {
            min[i] = min[i].min(*v);
            max[i] = max[i].max(*v);
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use velo_cesium::decode_gltf_bytes;

    #[test]
    fn placeholder_glb_decodes() {
        let glb = build_bike_placeholder_glb([0.2, 0.5, 0.8]);
        assert!(glb.starts_with(b"glTF"));
        let mesh = decode_gltf_bytes(&glb, "placeholder").unwrap();
        assert!(mesh.vertices.len() > 3);
        assert!(!mesh.indices.is_empty());
    }
}
