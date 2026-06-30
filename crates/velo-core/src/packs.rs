//! Route pack directory helpers.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::route::{RouteError, RouteModel};

pub const SCENERY_FILE: &str = "scenery.json";

/// Per-route scenery options (Tier B 3D Tiles toggle).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneryConfig {
    #[serde(default)]
    pub tiles_3d_enabled: bool,
}

impl Default for SceneryConfig {
    fn default() -> Self {
        Self {
            tiles_3d_enabled: false,
        }
    }
}

pub fn load_scenery_config(pack_dir: &Path) -> SceneryConfig {
    let path = pack_dir.join(SCENERY_FILE);
    if !path.is_file() {
        return SceneryConfig::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_scenery_config(pack_dir: &Path, config: &SceneryConfig) -> Result<(), RouteError> {
    fs::create_dir_all(pack_dir)?;
    fs::write(
        pack_dir.join(SCENERY_FILE),
        serde_json::to_vec_pretty(config)?,
    )?;
    Ok(())
}

/// Default user packs directory: `~/Documents/VeloSim/packs/`.
pub fn default_packs_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|h| h.join("Documents").join("VeloSim").join("packs"))
        .unwrap_or_else(|| PathBuf::from("assets/packs"))
}

/// List route IDs that have a `route.json` in the packs directory.
pub fn list_route_packs(packs_dir: &Path) -> Result<Vec<String>, RouteError> {
    let mut ids = Vec::new();
    if !packs_dir.is_dir() {
        return Ok(ids);
    }
    for entry in fs::read_dir(packs_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let route_json = entry.path().join("route.json");
            if route_json.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    ids.push(name.to_string());
                }
            }
        }
    }
    ids.sort();
    Ok(ids)
}

pub fn load_route_pack(pack_dir: &Path) -> Result<RouteModel, RouteError> {
    RouteModel::load_pack(pack_dir)
}

pub fn pack_dir_for_id(packs_dir: &Path, route_id: &str) -> PathBuf {
    packs_dir.join(route_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenery_config_defaults() {
        let dir = std::env::temp_dir().join("velo-scenery-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(!load_scenery_config(&dir).tiles_3d_enabled);
        save_scenery_config(&dir, &SceneryConfig { tiles_3d_enabled: true }).unwrap();
        assert!(load_scenery_config(&dir).tiles_3d_enabled);
        let _ = fs::remove_dir_all(&dir);
    }
}
