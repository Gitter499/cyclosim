use velo_core::RouteOrigin;

use crate::heightfield::Heightfield;

#[derive(Debug, Clone)]
pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

/// Triangulate heightfield in local ENU: X=east, Y=up, Z=north (renderer +Z forward).
pub fn mesh_from_heightfield(hf: &Heightfield, origin: &RouteOrigin) -> TerrainMesh {
    let mut vertices = Vec::with_capacity(hf.cols * hf.rows);
    for row in 0..hf.rows {
        for col in 0..hf.cols {
            let east = hf.east_at(col) as f32;
            let north = hf.north_at(row) as f32;
            let elev = hf.elevation_at(col, row) - origin.elevation_m as f32;
            let u = col as f32 / (hf.cols - 1).max(1) as f32;
            let v = row as f32 / (hf.rows - 1).max(1) as f32;
            vertices.push(TerrainVertex {
                position: [east, elev, north],
                uv: [u, v],
            });
        }
    }

    let mut indices = Vec::new();
    for row in 0..hf.rows - 1 {
        for col in 0..hf.cols - 1 {
            let i0 = (row * hf.cols + col) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + hf.cols as u32;
            let i3 = i2 + 1;
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    TerrainMesh { vertices, indices }
}
