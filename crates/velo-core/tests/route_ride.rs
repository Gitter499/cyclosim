//! Integration: ride along imported route; grade from route affects physics.

use std::path::PathBuf;
use std::time::Duration;

use velo_platform::{MockSensorSource, RecordingTrainerControl, TelemetrySample};
use velo_route_import::{import_gpx, DEFAULT_GRADE_WINDOW_M, DEFAULT_SPACING_M};
use velo_units::Watts;

use velo_core::{RideMode, VeloApp};

fn fixture_gpx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../velo-route-import/tests/fixtures/simple_climb.gpx")
}

#[test]
fn route_grade_drives_sim_and_physics() {
    let data = std::fs::read(fixture_gpx()).unwrap();
    let route = import_gpx(
        &data,
        "climb",
        "Climb",
        DEFAULT_SPACING_M,
        DEFAULT_GRADE_WINDOW_M,
    )
    .unwrap();

    let mut app = VeloApp::new();
    app.set_ride_mode(RideMode::Sim);
    app.load_route(route);

    let mut sensors = MockSensorSource::default();
    let trainer = RecordingTrainerControl::default();

    for _ in 0..2000 {
        sensors.push(TelemetrySample {
            elapsed: Duration::from_millis(0),
            power: Some(Watts::new(250.0)),
            cadence: None,
            heart_rate: None,
            wheel_speed: None,
        });
        app.tick(
            &mut sensors,
            &trainer,
            None::<&velo_platform::MockSteeringInput>,
            None::<&velo_platform::MockAudioDirector>,
        );
    }

    assert!(app.ride.distance_m > 50.0);
    assert!(
        app.ride.grade > 0.02,
        "expected positive grade on climb, got {}",
        app.ride.grade
    );
    let grades = trainer.sim_grades();
    assert!(grades.iter().any(|&g| g > 0.02));
}
