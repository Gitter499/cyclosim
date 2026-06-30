//! Bike asset metadata and anchor transform.

use std::path::{Path, PathBuf};

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

pub const META_FILE: &str = "bike.json";
pub const GLTF_FILE: &str = "bike.glb";
pub const SOURCES_DIR: &str = "sources";

/// Normalized glTF path plus rig anchor for the foreground-object pass.
#[derive(Debug, Clone, PartialEq)]
pub struct BikeAsset {
    pub bike_id: String,
    pub gltf_path: PathBuf,
    pub anchor: AnchorTransform,
}

/// Serializable anchor: translation + Y rotation + uniform scale (bike rig).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnchorTransform {
    #[serde(default)]
    pub translation: [f32; 3],
    /// Y-axis rotation in radians (forward alignment).
    #[serde(default)]
    pub rotation_y: f32,
    #[serde(default = "default_scale")]
    pub scale: f32,
}

fn default_scale() -> f32 {
    1.0
}

impl Default for AnchorTransform {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation_y: 0.0,
            scale: 1.0,
        }
    }
}

impl AnchorTransform {
    pub fn to_mat4(self) -> Mat4 {
        let t = Vec3::from_array(self.translation);
        let r = Quat::from_rotation_y(self.rotation_y);
        let s = Vec3::splat(self.scale);
        Mat4::from_scale_rotation_translation(s, r, t)
    }
}

/// On-disk bike library entry (`bike.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BikeMeta {
    pub bike_id: String,
    pub name: String,
    #[serde(default = "default_gltf_file")]
    pub gltf_file: String,
    #[serde(default)]
    pub anchor: AnchorTransform,
    #[serde(default)]
    pub source_images: Vec<String>,
    #[serde(default)]
    pub generator: String,
}

fn default_gltf_file() -> String {
    GLTF_FILE.to_string()
}

impl BikeMeta {
    pub fn gltf_path(&self, bike_dir: &Path) -> PathBuf {
        bike_dir.join(&self.gltf_file)
    }

    pub fn to_asset(&self, bike_dir: &Path) -> BikeAsset {
        BikeAsset {
            bike_id: self.bike_id.clone(),
            gltf_path: self.gltf_path(bike_dir),
            anchor: self.anchor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_default_is_identity() {
        let m = AnchorTransform::default().to_mat4();
        assert!((m.w_axis - glam::Vec4::new(0.0, 0.0, 0.0, 1.0)).length() < 1e-5);
    }
}
