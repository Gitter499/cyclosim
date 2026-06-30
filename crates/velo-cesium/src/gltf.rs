use std::io::Cursor;

use gltf::Gltf;
use thiserror::Error;

use crate::mesh::{TileMesh, TileVertex};

#[derive(Debug, Error)]
pub enum GltfDecodeError {
    #[error("gltf: {0}")]
    Gltf(#[from] gltf::Error),
    #[error("no mesh in glTF")]
    NoMesh,
    #[error("unsupported accessor")]
    UnsupportedAccessor,
}

/// Decode the first mesh primitive from glTF/GLB bytes into a [`TileMesh`].
pub fn decode_gltf_bytes(bytes: &[u8], tile_id: impl Into<String>) -> Result<TileMesh, GltfDecodeError> {
    let gltf = Gltf::from_reader_without_validation(Cursor::new(bytes))?;
    let blob = gltf.blob.as_deref();

    let mesh = gltf
        .meshes()
        .next()
        .ok_or(GltfDecodeError::NoMesh)?;

    let primitive = mesh
        .primitives()
        .next()
        .ok_or(GltfDecodeError::NoMesh)?;

    let reader = primitive.reader(|buffer| blob.filter(|_| buffer.index() == 0));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or(GltfDecodeError::UnsupportedAccessor)?
        .collect();

    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    let indices: Vec<u32> = reader
        .read_indices()
        .map(|iter| iter.into_u32().collect())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());

    let vertices: Vec<TileVertex> = positions
        .into_iter()
        .zip(uvs)
        .map(|(position, uv)| TileVertex { position, uv })
        .collect();

    Ok(TileMesh {
        vertices,
        indices,
        tile_id: tile_id.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::synthetic_triangle_glb;

    #[test]
    fn decode_synthetic_glb() {
        let glb = synthetic_triangle_glb();
        let mesh = decode_gltf_bytes(&glb, "test").unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }
}
