use std::path::PathBuf;

use velo_route_import::{import_gpx, DEFAULT_GRADE_WINDOW_M, DEFAULT_SPACING_M};
use velo_terrain::bake_terrain_for_route;

fn fixture_gpx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../velo-route-import/tests/fixtures/simple_climb.gpx")
}

#[test]
fn bake_synthetic_terrain_pack() {
    let data = std::fs::read(fixture_gpx()).unwrap();
    let route = import_gpx(
        &data,
        "terrain-test",
        "Terrain Test",
        DEFAULT_SPACING_M,
        DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();

    let dir = std::env::temp_dir().join("velo-terrain-pack-test");
    let _ = std::fs::remove_dir_all(&dir);
    route.save_pack(&dir).unwrap();
    let mesh = bake_terrain_for_route(&route, &dir, 200.0, 10.0).unwrap();
    assert!(mesh.vertices.len() > 100);
    assert!(mesh.indices.len() > mesh.vertices.len());

    let loaded = velo_terrain::TerrainPack::load_from_dir(&dir).unwrap();
    assert_eq!(loaded.mesh.vertices.len(), mesh.vertices.len());
    assert!(!loaded.texture_rgba.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mesh_has_valid_uvs() {
    let data = std::fs::read(fixture_gpx()).unwrap();
    let route = import_gpx(&data, "uv", "UV", 8.0, 50.0).unwrap();
    let hf = velo_terrain::synthetic_heightfield_for_route(&route, 200.0, 10.0);
    let mesh = velo_terrain::mesh_from_heightfield(&hf, &route.meta.origin);
    for v in &mesh.vertices {
        assert!(v.uv[0] >= 0.0 && v.uv[0] <= 1.0);
        assert!(v.uv[1] >= 0.0 && v.uv[1] <= 1.0);
    }
}
