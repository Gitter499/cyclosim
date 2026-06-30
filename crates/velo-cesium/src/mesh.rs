use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct TileVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMesh {
    pub vertices: Vec<TileVertex>,
    pub indices: Vec<u32>,
    pub tile_id: String,
}

impl TileMesh {
    pub fn triangle(id: impl Into<String>, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Self {
        Self {
            vertices: vec![
                TileVertex { position: a, uv: [0.0, 0.0] },
                TileVertex { position: b, uv: [1.0, 0.0] },
                TileVertex { position: c, uv: [0.5, 1.0] },
            ],
            indices: vec![0, 1, 2],
            tile_id: id.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_has_three_verts() {
        let m = TileMesh::triangle("t", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(m.vertices.len(), 3);
        assert_eq!(m.indices.len(), 3);
    }
}
