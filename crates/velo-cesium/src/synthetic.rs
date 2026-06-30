//! Minimal glTF/GLB fixture for offline tests.

use crate::mesh::TileMesh;

/// Single-triangle GLB (2×2×2 m on XZ plane at y=0).
pub fn synthetic_triangle_glb() -> Vec<u8> {
    build_minimal_glb(
        &[[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 0.0, 2.0]],
        &[[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]],
        &[0_u16, 1, 2],
    )
}

/// In-memory mesh equivalent (no glTF parse needed).
pub fn synthetic_triangle_mesh() -> TileMesh {
    TileMesh::triangle(
        "synthetic",
        [0.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [1.0, 0.0, 2.0],
    )
}

fn build_minimal_glb(positions: &[[f32; 3]], uvs: &[[f32; 2]], indices: &[u16]) -> Vec<u8> {
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
    let idx_offset = bin.len();
    for i in indices {
        bin.extend_from_slice(&i.to_le_bytes());
    }

    let pos_byte_len = positions.len() * 12;
    let uv_byte_len = uvs.len() * 8;
    let idx_byte_len = indices.len() * 2;

    let json = format!(
        r#"{{
  "asset": {{"version": "2.0"}},
  "buffers": [{{"byteLength": {bin_len}}}],
  "bufferViews": [
    {{"buffer": 0, "byteOffset": {pos_off}, "byteLength": {pos_len}, "target": 34962}},
    {{"buffer": 0, "byteOffset": {uv_off}, "byteLength": {uv_len}, "target": 34962}},
    {{"buffer": 0, "byteOffset": {idx_off}, "byteLength": {idx_len}, "target": 34963}}
  ],
  "accessors": [
    {{"bufferView": 0, "componentType": 5126, "count": {vcount}, "type": "VEC3",
      "max": [2.0, 0.0, 2.0], "min": [0.0, 0.0, 0.0]}},
    {{"bufferView": 1, "componentType": 5126, "count": {vcount}, "type": "VEC2"}},
    {{"bufferView": 2, "componentType": 5123, "count": {icount}, "type": "SCALAR"}}
  ],
  "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0, "TEXCOORD_0": 1}}, "indices": 2}}]}}]
}}"#,
        bin_len = bin.len(),
        pos_off = pos_offset,
        pos_len = pos_byte_len,
        uv_off = uv_offset,
        uv_len = uv_byte_len,
        idx_off = idx_offset,
        idx_len = idx_byte_len,
        vcount = positions.len(),
        icount = indices.len(),
    );

    // Pad JSON chunk to 4-byte boundary with spaces.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gltf::decode_gltf_bytes;

    #[test]
    fn glb_round_trip() {
        let glb = synthetic_triangle_glb();
        assert!(glb.starts_with(b"glTF"));
        let mesh = decode_gltf_bytes(&glb, "synthetic").unwrap();
        assert_eq!(mesh.vertices.len(), 3);
    }
}
