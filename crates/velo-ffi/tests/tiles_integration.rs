use std::path::Path;

use velo_cesium::TilesSession;
use velo_core::{load_scenery_config, save_scenery_config, SceneryConfig};

#[test]
fn scenery_config_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let config = SceneryConfig {
        tiles_3d_enabled: true,
    };
    save_scenery_config(dir.path(), &config).unwrap();
    let loaded = load_scenery_config(dir.path());
    assert!(loaded.tiles_3d_enabled);
}

#[test]
fn tiles_session_synthetic_offline() {
    let mut session = TilesSession::synthetic();
    let meshes = session
        .tick(velo_cesium::ViewCorridor {
            lat: 37.77,
            lon: -122.42,
            radius_m: 300.0,
        })
        .unwrap();
    assert!(!meshes.is_empty());
}

#[test]
fn scenery_defaults_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let config = load_scenery_config(Path::new(dir.path()));
    assert!(!config.tiles_3d_enabled);
}
