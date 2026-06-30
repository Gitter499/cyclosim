//! FFI integration: load route pack and list routes.

use std::path::PathBuf;

use velo_ffi::VeloHandle;
use velo_route_import::import_gpx;

fn fixture_gpx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../velo-route-import/tests/fixtures/simple_climb.gpx")
}

#[test]
fn set_active_route_and_list() {
    let temp = tempfile::tempdir().unwrap();
    let packs_dir = temp.path().join("packs");
    std::fs::create_dir_all(&packs_dir).unwrap();
    let pack_dir = packs_dir.join("ffi-test-climb");

    let data = std::fs::read(fixture_gpx()).unwrap();
    let model = import_gpx(
        &data,
        "ffi-test-climb",
        "FFI Climb",
        velo_route_import::DEFAULT_SPACING_M,
        velo_route_import::DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();
    model.save_pack(&pack_dir).unwrap();
    velo_terrain::bake_terrain_for_route(
        &model,
        &pack_dir,
        velo_terrain::DEFAULT_CORRIDOR_M,
        velo_terrain::DEFAULT_CELL_M,
    )
    .unwrap();

    let handle = VeloHandle::with_packs_dir_for_tests(packs_dir);
    handle
        .set_active_route("ffi-test-climb".into())
        .expect("set route");
    assert_eq!(
        handle.active_route_id().as_deref(),
        Some("ffi-test-climb")
    );
    let routes = handle.list_routes().unwrap();
    assert!(routes.iter().any(|r| r.route_id == "ffi-test-climb"));

    handle.clear_active_route();
    assert!(handle.active_route_id().is_none());
}
