//! 3D Tiles streaming for VeloSim M3b (Tier B scenery).
//!
//! Default build: Rust glTF decode + in-memory tile fetch (CI-friendly).
//! Optional `cesium-native` feature links a cxx stub for future Cesium Native integration.

pub mod attribution;
pub mod gltf;
pub mod mesh;
pub mod policy;
pub mod session;
pub mod synthetic;
pub mod tileset;

#[cfg(feature = "cesium-native")]
mod bridge;

pub use attribution::{attribution_for_provider, TileAttribution, TileProvider};
pub use gltf::{decode_gltf_bytes, GltfDecodeError};
pub use mesh::{TileMesh, TileVertex};
pub use policy::{OnlineOnlyPolicy, PolicyError};
pub use session::{SessionError, TilesSession, ViewCorridor};
pub use synthetic::synthetic_triangle_glb;
pub use tileset::{TilesetDocument, TilesetError};

/// Pinned Cesium Native release (see README).
pub const CESIUM_NATIVE_VERSION: &str = "0.44.0";

/// Default Cesium ion asset for dev (OSM Buildings sample).
pub const DEV_ION_ASSET_ID: u32 = 96188;

/// Returns the pinned Cesium Native version string.
pub fn cesium_native_version() -> &'static str {
    CESIUM_NATIVE_VERSION
}

#[cfg(feature = "cesium-native")]
pub fn native_bridge_ok() -> bool {
    bridge::native_build_ok()
}

#[cfg(not(feature = "cesium-native"))]
pub fn native_bridge_ok() -> bool {
    false
}
