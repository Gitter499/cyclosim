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

    let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; all_positions.len()];
    build_colored_glb(&all_positions, &uvs, &indices, frame_color)
}

fn wheel_vertex(cx: f32, cy: f32, r: f32, i: usize) -> [f32; 3] {
    let angle = (i as f32) * std::f32::consts::TAU / 8.0;
    [cx + r * angle.cos(), cy + r * angle.sin(), 0.0]
}

fn build_colored_glb(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u16],
    color: [f32; 3],
) -> Vec<u8> {
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
    for _ in positions {
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
