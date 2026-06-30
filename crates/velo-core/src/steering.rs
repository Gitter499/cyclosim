//! Steering input mapping — deadzone, low-pass, and camera yaw offset.
//!
//! Raw axis values come from the shell (`SteeringInput` over FFI). Filtering and
//! integration live here so headless tests stay portable and Apple-free.

use velo_platform::SteeringInput;

const DEFAULT_DEADZONE: f32 = 0.08;
const DEFAULT_SMOOTHING: f32 = 0.15;
const DEFAULT_YAW_RATE: f32 = 0.9;
const MAX_YAW_RAD: f32 = 0.45;

/// Applies deadzone, exponential smoothing, and yaw integration from poll results.
#[derive(Debug, Clone)]
pub struct SteeringController {
    enabled: bool,
    deadzone: f32,
    smoothing: f32,
    yaw_rate: f32,
    filtered_axis: f32,
    yaw_offset_rad: f32,
}

impl Default for SteeringController {
    fn default() -> Self {
        Self {
            enabled: false,
            deadzone: DEFAULT_DEADZONE,
            smoothing: DEFAULT_SMOOTHING,
            yaw_rate: DEFAULT_YAW_RATE,
            filtered_axis: 0.0,
            yaw_offset_rad: 0.0,
        }
    }
}

impl SteeringController {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn reset(&mut self) {
        self.filtered_axis = 0.0;
        self.yaw_offset_rad = 0.0;
    }

    /// Poll steering input, update filter state, integrate yaw for the sim step `dt_s`.
    pub fn poll<S: SteeringInput>(&mut self, input: &S, dt_s: f32, route_loaded: bool) {
        if !self.enabled || !route_loaded {
            self.filtered_axis = 0.0;
            return;
        }

        let raw = input.poll();
        if raw.recenter {
            self.reset();
            return;
        }

        let axis = apply_deadzone(raw.axis, self.deadzone);
        self.filtered_axis += self.smoothing * (axis - self.filtered_axis);

        self.yaw_offset_rad =
            (self.yaw_offset_rad + self.filtered_axis * self.yaw_rate * dt_s).clamp(-MAX_YAW_RAD, MAX_YAW_RAD);
    }

    pub fn filtered_axis(&self) -> f32 {
        self.filtered_axis
    }

    pub fn yaw_offset_rad(&self) -> f32 {
        if self.enabled {
            self.yaw_offset_rad
        } else {
            0.0
        }
    }
}

fn apply_deadzone(axis: f32, deadzone: f32) -> f32 {
    if axis.abs() < deadzone {
        0.0
    } else {
        let sign = axis.signum();
        sign * ((axis.abs() - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use velo_platform::MockSteeringInput;

    #[test]
    fn deadzone_zeros_small_inputs() {
        assert_eq!(apply_deadzone(0.05, 0.08), 0.0);
        assert!(apply_deadzone(0.5, 0.08).abs() > 0.0);
    }

    #[test]
    fn axis_integrates_into_yaw_on_route() {
        let mut ctrl = SteeringController::default();
        ctrl.set_enabled(true);
        let input = MockSteeringInput::with_axis(1.0);

        for _ in 0..200 {
            ctrl.poll(&input, 0.01, true);
        }

        assert!(ctrl.yaw_offset_rad() > 0.1);
        assert!(ctrl.filtered_axis() > 0.5);
    }

    #[test]
    fn recenter_clears_yaw() {
        let mut ctrl = SteeringController::default();
        ctrl.set_enabled(true);
        let input = MockSteeringInput::with_axis(1.0);
        for _ in 0..100 {
            ctrl.poll(&input, 0.01, true);
        }
        assert!(ctrl.yaw_offset_rad() > 0.0);

        let recenter = MockSteeringInput::recenter();
        ctrl.poll(&recenter, 0.01, true);
        assert_eq!(ctrl.yaw_offset_rad(), 0.0);
    }

    #[test]
    fn disabled_without_route() {
        let mut ctrl = SteeringController::default();
        ctrl.set_enabled(true);
        let input = MockSteeringInput::with_axis(1.0);
        ctrl.poll(&input, 0.01, false);
        assert_eq!(ctrl.yaw_offset_rad(), 0.0);
    }
}
