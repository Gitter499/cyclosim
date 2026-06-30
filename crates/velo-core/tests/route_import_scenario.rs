//! User story: imported GPX metadata drives core route state and grade lookup.

use std::path::PathBuf;

use velo_core::VeloApp;
use velo_route_import::{import_gpx, DEFAULT_GRADE_WINDOW_M, DEFAULT_SPACING_M};

fn fixture_gpx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../velo-route-import/tests/fixtures/simple_climb.gpx")
}

#[test]
fn user_story_gpx_import_metadata_used_by_core() {
    let data = std::fs::read(fixture_gpx()).unwrap();
    let route = import_gpx(
        &data,
        "meta-climb",
        "Metadata Climb",
        DEFAULT_SPACING_M,
        DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();

    assert_eq!(route.meta.route_id, "meta-climb");
    assert_eq!(route.meta.name, "Metadata Climb");
    assert!(route.meta.total_distance_m > 0.0);
    assert!(route.grade_at(100.0) > 0.0, "climb fixture should have positive grade");

    let mut app = VeloApp::new();
    app.load_route(route);
    assert_eq!(app.active_route_id(), Some("meta-climb"));
    assert!(app.ride.grade >= 0.0);

    let pos = app.route_position_enu().expect("route position");
    assert!(pos.0.is_finite() && pos.1.is_finite());
}
