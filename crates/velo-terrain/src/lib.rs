//! Offline terrain pipeline: heightfield → triangulated mesh + texture atlas → route pack.

mod heightfield;
mod mesh;
mod pack;
mod synthetic;

pub use heightfield::Heightfield;
pub use mesh::{mesh_from_heightfield, TerrainMesh, TerrainVertex};
pub use pack::{TerrainPack, TerrainPackError, TERRAIN_MESH_FILE, TERRAIN_TEXTURE_FILE};
pub use synthetic::synthetic_heightfield_for_route;

use thiserror::Error;
use velo_core::RouteModel;

pub const DEFAULT_CORRIDOR_M: f64 = 200.0;
pub const DEFAULT_CELL_M: f64 = 10.0;

#[derive(Debug, Error)]
pub enum TerrainError {
    #[error("pack: {0}")]
    Pack(#[from] TerrainPackError),
    #[error("route: {0}")]
    Route(#[from] velo_core::RouteError),
    #[error("no route in pack")]
    NoRoute,
}

/// Bake terrain mesh + procedural satellite-style texture for a route pack.
pub fn bake_terrain_pack(
    pack_dir: &std::path::Path,
    corridor_m: f64,
    cell_m: f64,
) -> Result<TerrainMesh, TerrainError> {
    let route = RouteModel::load_pack(pack_dir)?;
    bake_terrain_for_route(&route, pack_dir, corridor_m, cell_m)
}

pub fn bake_terrain_for_route(
    route: &RouteModel,
    pack_dir: &std::path::Path,
    corridor_m: f64,
    cell_m: f64,
) -> Result<TerrainMesh, TerrainError> {
    let hf = synthetic_heightfield_for_route(route, corridor_m, cell_m);
    let mesh = mesh::mesh_from_heightfield(&hf, &route.meta.origin);
    let texture = synthetic::procedural_texture(hf.cols, hf.rows);
    let pack = TerrainPack {
        mesh: mesh.clone(),
        texture_rgba: texture,
        texture_width: hf.cols as u32,
        texture_height: hf.rows as u32,
    };
    pack.write_to_dir(pack_dir)?;
    Ok(mesh)
}
