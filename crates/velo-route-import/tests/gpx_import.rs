use std::path::PathBuf;

use velo_route_import::{import_file, import_gpx, DEFAULT_GRADE_WINDOW_M, DEFAULT_SPACING_M};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn imports_simple_climb_gpx() {
    let data = std::fs::read(fixture("simple_climb.gpx")).unwrap();
    let model = import_gpx(
        &data,
        "simple-climb",
        "Simple Climb",
        DEFAULT_SPACING_M,
        DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();

    assert_eq!(model.meta.route_id, "simple-climb");
    assert!(model.points.len() > 5);
    assert!(model.total_distance_m() > 300.0);
    assert!(model.points.iter().any(|p| p.grade > 0.03));
}

#[test]
fn import_file_round_trip_pack() {
    let dir = std::env::temp_dir().join("velo-import-test-pack");
    let _ = std::fs::remove_dir_all(&dir);
    let model = import_file(
        &fixture("simple_climb.gpx"),
        "climb",
        None,
        DEFAULT_SPACING_M,
        DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();
    model.save_pack(&dir).unwrap();
    let loaded = velo_core::RouteModel::load_pack(&dir).unwrap();
    assert_eq!(loaded.points.len(), model.points.len());
    let _ = std::fs::remove_dir_all(&dir);
}
