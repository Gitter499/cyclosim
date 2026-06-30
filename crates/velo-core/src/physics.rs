use velo_units::{Grade, Kilograms, Meters, MetersPerSecond, Watts};

/// Rider + bike parameters for longitudinal dynamics.
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    pub rider_mass: Kilograms,
    pub bike_mass: Kilograms,
    pub crr: f32,
    pub cda: f32,
    pub drivetrain_efficiency: f32,
    pub v_min: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            rider_mass: Kilograms::new(75.0),
            bike_mass: Kilograms::new(8.0),
            crr: 0.004,
            cda: 0.32,
            drivetrain_efficiency: 0.97,
            v_min: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RideSnapshot {
    pub distance: Meters,
    pub speed: MetersPerSecond,
}

const G: f32 = 9.80665;
const RHO: f32 = 1.225;

/// One fixed-step integration of 1-D cycling dynamics.
pub fn integrate_step(
    config: &PhysicsConfig,
    grade: Grade,
    power: Watts,
    speed: MetersPerSecond,
    dt: f32,
) -> RideSnapshot {
    let m = (config.rider_mass.0 + config.bike_mass.0) as f32;
    let theta = (grade.0 as f32).atan();
    let v = speed.0 as f32;

    let f_gravity = m * G * theta.sin();
    let f_rolling = m * G * config.crr * theta.cos();
    let f_drag = 0.5 * RHO * config.cda * v * v;
    let f_resist = f_gravity + f_rolling + f_drag;

    let p_wheel = (power.0 as f32) * config.drivetrain_efficiency;
    let v_eff = v.max(config.v_min);
    let f_propel = p_wheel / v_eff;

    let a = (f_propel - f_resist) / m;
    let v_new = (v + a * dt).max(0.0);
    let distance_delta = ((v + v_new) * 0.5 * dt) as f64;

    RideSnapshot {
        distance: Meters::new(distance_delta),
        speed: MetersPerSecond::new(v_new as f64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_steady_state_reasonable_speed() {
        let cfg = PhysicsConfig::default();
        let mut speed = MetersPerSecond::new(0.0);
        let mut distance = Meters::new(0.0);
        let power = Watts::new(200.0);
        let grade = Grade::new(0.0);

        for _ in 0..5000 {
            let snap = integrate_step(&cfg, grade, power, speed, 0.01);
            distance = Meters::new(distance.0 + snap.distance.0);
            speed = snap.speed;
        }

        // ~200W on flat should converge near 9–11 m/s depending on CdA/Crr.
        assert!(speed.0 > 7.0 && speed.0 < 13.0, "speed={}", speed.0);
        assert!(distance.0 > 100.0);
    }

    #[test]
    fn climb_reduces_speed_vs_flat() {
        let cfg = PhysicsConfig::default();
        let power = Watts::new(250.0);

        let mut flat = MetersPerSecond::new(0.0);
        let mut climb = MetersPerSecond::new(0.0);
        for _ in 0..8000 {
            flat = integrate_step(&cfg, Grade::new(0.0), power, flat, 0.01).speed;
            climb = integrate_step(&cfg, Grade::new(0.06), power, climb, 0.01).speed;
        }
        assert!(climb.0 < flat.0);
    }
}
