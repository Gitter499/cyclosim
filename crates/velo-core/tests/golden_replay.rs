//! Deterministic replay of constant-power samples over a flat route.

use velo_core::physics::{integrate_step, PhysicsConfig};
use velo_units::{Grade, MetersPerSecond, Watts};

#[test]
fn replay_constant_200w_flat_50s() {
    let cfg = PhysicsConfig::default();
    let grade = Grade::new(0.0);
    let power = Watts::new(200.0);
    let mut speed = MetersPerSecond::new(0.0);
    let mut distance = 0.0_f64;

    let steps = 50 * 100; // 50 s @ 100 Hz
    for _ in 0..steps {
        let snap = integrate_step(&cfg, grade, power, speed, 0.01);
        speed = snap.speed;
        distance += snap.distance.0;
    }

    // Golden expectations — tuned to default PhysicsConfig.
    assert!(distance > 350.0 && distance < 550.0, "distance={distance}");
    assert!(speed.0 > 8.0 && speed.0 < 12.0, "speed={}", speed.0);
}

#[test]
fn replay_constant_250w_climb_6pct_30s() {
    let cfg = PhysicsConfig::default();
    let grade = Grade::new(0.06);
    let power = Watts::new(250.0);
    let mut speed = MetersPerSecond::new(0.0);
    let mut distance = 0.0_f64;

    for _ in 0..3000 {
        let snap = integrate_step(&cfg, grade, power, speed, 0.01);
        speed = snap.speed;
        distance += snap.distance.0;
    }

    assert!(distance > 80.0 && distance < 200.0, "distance={distance}");
    assert!(speed.0 > 4.0 && speed.0 < 9.0, "speed={}", speed.0);
}
