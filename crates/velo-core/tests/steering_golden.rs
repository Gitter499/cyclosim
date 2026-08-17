//! Golden values for steering deadzone, smoothing, recenter, and yaw integration.

use velo_core::SteeringController;
use velo_platform::MockSteeringInput;

#[test]
fn deadzone_suppresses_small_axis_values() {
    let mut ctrl = SteeringController::default();
    ctrl.set_enabled(true);
    let input = MockSteeringInput::with_axis(0.05);
    for _ in 0..50 {
        ctrl.poll(&input, 0.01, true);
    }
    assert_eq!(ctrl.filtered_axis(), 0.0);
    assert_eq!(ctrl.yaw_offset_rad(), 0.0);
}

#[test]
fn full_axis_integrates_to_bounded_yaw_golden() {
    let mut ctrl = SteeringController::default();
    ctrl.set_enabled(true);
    let input = MockSteeringInput::with_axis(1.0);

    for _ in 0..100 {
        ctrl.poll(&input, 0.01, true);
    }

    // After 1 s at full axis: smoothing settles near 1.0, yaw approaches cap.
    assert!((ctrl.filtered_axis() - 0.99).abs() < 0.02);
    let yaw = ctrl.yaw_offset_rad();
    assert!((yaw - 0.45).abs() < 0.01, "yaw should clamp near 0.45 rad, got {yaw}");
}

#[test]
fn recenter_resets_yaw_after_turning() {
    let mut ctrl = SteeringController::default();
    ctrl.set_enabled(true);
    let turn = MockSteeringInput::with_axis(1.0);
    for _ in 0..80 {
        ctrl.poll(&turn, 0.01, true);
    }
    assert!(ctrl.yaw_offset_rad() > 0.2);

    ctrl.poll(&MockSteeringInput::recenter(), 0.01, true);
    assert_eq!(ctrl.yaw_offset_rad(), 0.0);
    assert_eq!(ctrl.filtered_axis(), 0.0);
}

#[test]
fn steering_ignored_without_loaded_route() {
    let mut ctrl = SteeringController::default();
    ctrl.set_enabled(true);
    let input = MockSteeringInput::with_axis(1.0);
    for _ in 0..100 {
        ctrl.poll(&input, 0.01, false);
    }
    assert_eq!(ctrl.yaw_offset_rad(), 0.0);
}
