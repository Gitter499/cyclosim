//! Route pack directory helpers.

use std::fs;
use std::path::{Path, PathBuf};

use crate::route::{RouteError, RouteModel};

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
