//! Headless sim scenarios: drive `VeloApp` with mock sensors over synthetic routes.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use velo_core::{
    RideMode, RouteModel, RoutePoint, VeloApp, Workout,
};
use velo_platform::{MockAudioDirector, MockSensorSource, MockSteeringInput, RecordingTrainerControl, TelemetrySample};
use velo_units::{Bpm, Rpm, Watts};

const DT_S: f64 = 0.01;
const MAX_DURATION_S: f64 = 4.0 * 3600.0;

/// Synthetic route shapes used by scenarios (no external files needed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RouteKind {
    /// No route loaded; fixed grade from `grade`.
    #[default]
    None,
    /// 10 km dead flat.
    Flat,
    /// 20 km of ±30 m sinusoidal rollers.
    Rolling,
    /// 8 km steady 8% alpine climb.
    AlpineClimb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeParam {
    Erg,
    Sim,
    Free,
}

fn default_duration() -> f64 {
    120.0
}

fn default_target_power() -> f64 {
    200.0
}

fn default_sample_every() -> f64 {
    1.0
}

/// Input parameters for a headless scenario run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioParams {
    /// Ride mode: erg | sim | free. Workouts force ERG per interval.
    #[serde(default = "default_mode")]
    pub mode: ModeParam,
    /// ERG target watts (erg mode without workout).
    #[serde(default = "default_target_power")]
    pub target_power_w: f64,
    /// Power the simulated rider actually produces. Defaults to the ERG
    /// target in erg mode, else 200 W.
    #[serde(default)]
    pub rider_power_w: Option<f64>,
    #[serde(default)]
    pub cadence_rpm: Option<f64>,
    #[serde(default)]
    pub heart_rate_bpm: Option<f64>,
    /// Simulated wall-clock seconds (clamped to 4 h).
    #[serde(default = "default_duration")]
    pub duration_s: f64,
    /// Synthetic route to load.
    #[serde(default)]
    pub route: RouteKind,
    /// Fixed grade when `route` is `none` (rise/run, e.g. 0.05 = 5%).
    #[serde(default)]
    pub grade: f64,
    /// Rider FTP for %FTP workout targets.
    #[serde(default)]
    pub ftp_w: Option<f64>,
    /// Structured workout (velo-core Workout JSON). Overrides mode/target.
    #[serde(default)]
    pub workout: Option<Workout>,
    /// Zwift .zwo XML; parsed into a workout. Overrides `workout` if both set.
    #[serde(default)]
    pub zwo_xml: Option<String>,
    /// Record the ride session (enables FIT export).
    #[serde(default)]
    pub record: bool,
    /// Timeline sampling period in seconds.
    #[serde(default = "default_sample_every")]
    pub sample_every_s: f64,
    /// Steering axis held for the whole scenario, [-1, 1]; yaws the camera (M6).
    #[serde(default)]
    pub steer_axis: f64,
}

fn default_mode() -> ModeParam {
    ModeParam::Erg
}

impl Default for ScenarioParams {
    fn default() -> Self {
        serde_json::from_value(serde_json::json!({})).expect("defaults")
    }
}

/// One sampled point on the scenario timeline.
#[derive(Debug, Clone, Serialize)]
pub struct TimelinePoint {
    pub t_s: f64,
    pub distance_m: f64,
    pub speed_mps: f64,
    pub grade: f64,
    pub power_w: Option<f64>,
    pub interval: Option<String>,
    pub target_w: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSummary {
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub avg_speed_kmh: f64,
    pub max_speed_kmh: f64,
    pub final_grade: f64,
    pub route: RouteKind,
    pub workout_finished: Option<bool>,
    pub trainer_last_erg_w: Option<f64>,
    pub trainer_last_sim_grade: Option<f64>,
    pub recorded_samples: Option<u32>,
    pub steer_yaw_rad: f64,
}

/// A completed scenario: the app (for rendering/FIT export) plus telemetry.
pub struct ScenarioRun {
    pub app: VeloApp,
    pub timeline: Vec<TimelinePoint>,
    pub summary: ScenarioSummary,
}

/// Build one of the synthetic evaluation routes.
pub fn build_route(kind: RouteKind) -> Option<RouteModel> {
    let (id, name, length_m, grade_fn): (_, _, f64, Box<dyn Fn(f64) -> f64>) = match kind {
        RouteKind::None => return None,
        RouteKind::Flat => ("eval-flat", "Eval Flat 10k", 10_000.0, Box::new(|_| 0.0)),
        RouteKind::Rolling => (
            "eval-rolling",
            "Eval Rollers 20k",
            20_000.0,
            // elevation = 30·sin(x/400) → grade = 0.075·cos(x/400), peaks ±7.5%.
            Box::new(|x: f64| 0.075 * (x / 400.0).cos()),
        ),
        RouteKind::AlpineClimb => (
            "eval-climb",
            "Eval Alpine 8k @ 8%",
            8_000.0,
            Box::new(|_| 0.08),
        ),
    };

    const SPACING_M: f64 = 10.0;
    let base_lat = 45.0_f64;
    let base_lon = 7.0_f64;
    let m_per_deg_lat = 111_320.0;

    let n = (length_m / SPACING_M) as usize + 1;
    let mut points = Vec::with_capacity(n);
    let mut elevation = 500.0;
    for i in 0..n {
        let d = i as f64 * SPACING_M;
        let g = grade_fn(d);
        if i > 0 {
            elevation += g * SPACING_M;
        }
        points.push(RoutePoint {
            distance_m: d,
            lat: base_lat + d / m_per_deg_lat,
            lon: base_lon,
            elevation_m: elevation,
            grade: g,
        });
    }
    Some(RouteModel::new(id, name, points).expect("synthetic route is non-empty"))
}

/// Run a scenario to completion and return the driven app + telemetry.
pub fn run_scenario(params: &ScenarioParams) -> Result<ScenarioRun, String> {
    let mut app = VeloApp::new();
    let mut sensors = MockSensorSource::default();
    let trainer = RecordingTrainerControl::default();

    if let Some(ftp) = params.ftp_w {
        app.set_ftp(ftp);
    }

    if let Some(route) = build_route(params.route) {
        app.load_route(route);
    }

    match params.mode {
        ModeParam::Erg => {
            app.set_ride_mode(RideMode::Erg);
            app.set_target_power(params.target_power_w);
        }
        ModeParam::Sim => {
            app.set_ride_mode(RideMode::Sim);
            app.set_grade(params.grade);
        }
        ModeParam::Free => app.set_ride_mode(RideMode::Free),
    }

    let workout = match (&params.zwo_xml, &params.workout) {
        (Some(xml), _) => Some(velo_core::parse_zwo_xml(xml).map_err(|e| e.to_string())?),
        (None, Some(w)) => Some(w.clone()),
        (None, None) => None,
    };
    if let Some(w) = workout {
        w.validate()?;
        app.start_workout(w);
    }

    if params.record {
        app.start_ride();
    }

    let steering = MockSteeringInput::with_axis(params.steer_axis as f32);
    if params.steer_axis != 0.0 {
        app.set_steering_enabled(true);
    }

    let has_workout = app.workout_active();
    let fixed_rider_power = params.rider_power_w.unwrap_or(match params.mode {
        ModeParam::Erg => params.target_power_w,
        _ => 200.0,
    });

    let duration = params.duration_s.clamp(DT_S, MAX_DURATION_S);
    let ticks = (duration / DT_S).round() as u64;
    let sample_every_ticks =
        ((params.sample_every_s.max(DT_S) / DT_S).round() as u64).max(1);

    let mut timeline = Vec::new();
    let mut max_speed = 0.0_f64;

    for tick in 0..ticks {
        // During a workout the simulated rider tracks the live ERG target
        // (unless an explicit rider power was given), so speed responds to
        // interval changes the way a real ERG ride would.
        let rider_power = match (params.rider_power_w, has_workout) {
            (None, true) => app.target_power(),
            _ => fixed_rider_power,
        };
        sensors.push(TelemetrySample {
            elapsed: Duration::from_millis((tick as f64 * DT_S * 1000.0) as u64),
            power: Some(Watts::new(rider_power)),
            cadence: params.cadence_rpm.map(Rpm::new),
            heart_rate: params.heart_rate_bpm.map(Bpm::new),
            wheel_speed: None,
        });
        app.tick(
            &mut sensors,
            &trainer,
            Some(&steering),
            None::<&MockAudioDirector>,
        );
        max_speed = max_speed.max(app.ride.speed_mps);

        if tick % sample_every_ticks == 0 || tick + 1 == ticks {
            timeline.push(sample_point(&app));
        }
    }

    let recorded = if params.record {
        let summary = app.stop_ride();
        summary.map(|s| s.sample_count)
    } else {
        None
    };

    let elapsed = app.ride.elapsed_s;
    let summary = ScenarioSummary {
        elapsed_s: elapsed,
        distance_m: app.ride.distance_m,
        avg_speed_kmh: if elapsed > 0.0 {
            app.ride.distance_m / elapsed * 3.6
        } else {
            0.0
        },
        max_speed_kmh: max_speed * 3.6,
        final_grade: app.ride.grade,
        route: params.route,
        workout_finished: match (params.zwo_xml.is_some() || params.workout.is_some(), app.workout_active()) {
            (false, _) => None,
            (true, active) => Some(!active),
        },
        trainer_last_erg_w: trainer.last_power().map(|w| w.0),
        trainer_last_sim_grade: trainer.last_sim().map(|(g, _, _)| g.0),
        recorded_samples: recorded,
        steer_yaw_rad: app.steer_yaw_rad() as f64,
    };

    Ok(ScenarioRun {
        app,
        timeline,
        summary,
    })
}

fn sample_point(app: &VeloApp) -> TimelinePoint {
    let (interval, target_w) = workout_hud_fields(app);
    TimelinePoint {
        t_s: app.ride.elapsed_s,
        distance_m: app.ride.distance_m,
        speed_mps: app.ride.speed_mps,
        grade: app.ride.grade,
        power_w: app.ride.power_w,
        interval,
        target_w,
    }
}

/// Active interval name + resolved ERG target, for HUD display and timelines.
pub fn workout_hud_fields(app: &VeloApp) -> (Option<String>, Option<f64>) {
    let Some(engine) = app.workout_engine.as_ref() else {
        return (None, None);
    };
    let interval = engine.current_interval().map(|i| i.name.clone());
    let target = engine.target_watts().map(|w| w.0);
    (interval, target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erg_scenario_advances_distance() {
        let params = ScenarioParams {
            duration_s: 30.0,
            ..Default::default()
        };
        let run = run_scenario(&params).expect("scenario");
        assert!(run.summary.distance_m > 50.0);
        assert_eq!(run.summary.trainer_last_erg_w, Some(200.0));
        assert!(!run.timeline.is_empty());
    }

    #[test]
    fn climb_route_reports_grade() {
        let params = ScenarioParams {
            mode: ModeParam::Sim,
            route: RouteKind::AlpineClimb,
            rider_power_w: Some(280.0),
            duration_s: 60.0,
            ..Default::default()
        };
        let run = run_scenario(&params).expect("scenario");
        assert!((run.summary.final_grade - 0.08).abs() < 1e-6);
        assert_eq!(run.summary.trainer_last_sim_grade, Some(0.08));
    }

    #[test]
    fn recorded_scenario_supports_fit_export() {
        let params = ScenarioParams {
            duration_s: 5.0,
            record: true,
            ..Default::default()
        };
        let run = run_scenario(&params).expect("scenario");
        assert!(run.summary.recorded_samples.unwrap_or(0) > 0);
        let fit = run.app.export_fit().expect("fit");
        assert_eq!(&fit[8..12], b".FIT");
    }

    #[test]
    fn synthetic_routes_have_expected_length() {
        let flat = build_route(RouteKind::Flat).unwrap();
        assert!((flat.total_distance_m() - 10_000.0).abs() < 1.0);
        let climb = build_route(RouteKind::AlpineClimb).unwrap();
        assert!((climb.total_distance_m() - 8_000.0).abs() < 1.0);
    }
}
