//! Evaluation tool registry shared by the MCP server and the one-shot CLI.

use serde_json::{json, Value};
use velo_render::HudSnapshot;

use crate::frames;
use crate::scenario::{self, ModeParam, ScenarioParams};

/// A produced image (PNG bytes + a suggested file stem).
pub struct ToolImage {
    pub name: String,
    pub png: Vec<u8>,
}

/// Result of a tool invocation: human/model-readable text plus images.
pub struct ToolOutput {
    pub text: String,
    pub images: Vec<ToolImage>,
}

impl ToolOutput {
    fn text(value: &Value) -> Self {
        Self {
            text: serde_json::to_string_pretty(value).unwrap_or_default(),
            images: Vec::new(),
        }
    }
}

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub run: fn(&Value) -> Result<ToolOutput, String>,
}

fn scenario_properties() -> Value {
    json!({
        "mode": {"type": "string", "enum": ["erg", "sim", "free"], "description": "Ride mode (default erg)"},
        "target_power_w": {"type": "number", "description": "ERG target watts (default 200)"},
        "rider_power_w": {"type": "number", "description": "Watts the simulated rider produces (default: ERG target, else 200)"},
        "cadence_rpm": {"type": "number"},
        "heart_rate_bpm": {"type": "number"},
        "duration_s": {"type": "number", "description": "Simulated seconds (default 120, max 14400)"},
        "route": {"type": "string", "enum": ["none", "flat", "rolling", "alpine_climb"], "description": "Synthetic route (default none)"},
        "grade": {"type": "number", "description": "Fixed grade when route=none (0.05 = 5%)"},
        "ftp_w": {"type": "number", "description": "Rider FTP for %FTP workout targets"},
        "workout": {"type": "object", "description": "velo-core Workout JSON: {name, intervals: [{name, duration_s, target: {ErgWatts: w}|{FtpPercent: p}|\"FreeRide\"}]}"},
        "zwo_xml": {"type": "string", "description": "Zwift .zwo workout XML (overrides workout)"},
        "record": {"type": "boolean", "description": "Record ride session (enables FIT export)"},
        "sample_every_s": {"type": "number", "description": "Timeline sample period (default 1s)"}
    })
}

fn parse_scenario(args: &Value) -> Result<ScenarioParams, String> {
    serde_json::from_value(args.clone()).map_err(|e| format!("invalid scenario params: {e}"))
}

fn mode_label(params: &ScenarioParams) -> &'static str {
    if params.workout.is_some() || params.zwo_xml.is_some() {
        return "ERG";
    }
    match params.mode {
        ModeParam::Erg => "ERG",
        ModeParam::Sim => "SIM",
        ModeParam::Free => "FREE",
    }
}

fn dim(args: &Value, key: &str, default: u32) -> u32 {
    args.get(key).and_then(Value::as_u64).unwrap_or(default as u64) as u32
}

// ---------------------------------------------------------------------------
// Tool handlers
// ---------------------------------------------------------------------------

fn feature_inventory(_args: &Value) -> Result<ToolOutput, String> {
    let value = json!({
        "product": "VeloSim (cyclosim) — native offline cycling simulator",
        "architecture": "Rust sim core + wgpu renderer + thin Swift macOS shell (UniFFI)",
        "milestones": {
            "M0 skeleton & FFI boundary": "done",
            "M1 physics core (deterministic fixed-step integrator, ERG/SIM)": "done",
            "M2a trainer (BLE FTMS) + HUD ride": "done",
            "M2b FIT export + Strava upload + screenshots": "done",
            "M2c ride library (SQLite)": "done",
            "M3 real route import + terrain substrate": "done",
            "M3b Google 3D Tiles streaming (Cesium)": "done",
            "M3c bike model import (image-to-3D glTF)": "done",
            "M5 workouts + Liquid Glass shell + highlight clips (.zwo import)": "done",
            "M5 remaining: cinematic replay camera": "in progress",
            "M6 Apple Music AudioDirector + AirPods steering": "in progress"
        },
        "crates": {
            "velo-units": "physical quantity newtypes",
            "velo-platform": "shell<->core trait contracts + mocks",
            "velo-core": "physics, ride loop, routes, workouts, highlights",
            "velo-render": "wgpu scene + HUD; headless offscreen mode",
            "velo-fit": "FIT activity encoder",
            "velo-rides": "SQLite ride library",
            "velo-route-import": "GPX/TCX -> RouteModel",
            "velo-terrain": "DEM -> terrain mesh + texture",
            "velo-cesium": "3D Tiles streaming + glTF decode",
            "velo-bikegen": "bike image-to-3D asset pipeline",
            "velo-ffi": "UniFFI surface for Swift",
            "velo-eval-mcp": "this MCP evaluation server"
        },
        "eval_tools": [
            "feature_inventory", "sim_scenario", "render_frame",
            "render_ride_sequence", "hud_probe", "workout_preview",
            "fit_export_check", "replay_camera_preview"
        ]
    });
    Ok(ToolOutput::text(&value))
}

fn sim_scenario(args: &Value) -> Result<ToolOutput, String> {
    let params = parse_scenario(args)?;
    let run = scenario::run_scenario(&params)?;
    let value = json!({
        "summary": run.summary,
        "timeline": run.timeline,
    });
    Ok(ToolOutput::text(&value))
}

fn render_frame(args: &Value) -> Result<ToolOutput, String> {
    let params = parse_scenario(args)?;
    let width = dim(args, "width", 960);
    let height = dim(args, "height", 540);

    let run = scenario::run_scenario(&params)?;
    let mut renderer = frames::headless_renderer(width, height)?;
    if args.get("show_bike").and_then(Value::as_bool).unwrap_or(false) {
        frames::load_placeholder_bike(&mut renderer)?;
    }
    let hud = frames::hud_from_app(&run.app, mode_label(&params));
    let follow = frames::follow_from_app(&run.app);
    let png = frames::capture_png(&mut renderer, &hud, run.app.ride.distance_m, follow)?;

    Ok(ToolOutput {
        text: serde_json::to_string_pretty(&json!({
            "summary": run.summary,
            "hud_lines": hud.lines(),
        }))
        .unwrap_or_default(),
        images: vec![ToolImage {
            name: "frame".into(),
            png,
        }],
    })
}

fn render_ride_sequence(args: &Value) -> Result<ToolOutput, String> {
    let params = parse_scenario(args)?;
    let width = dim(args, "width", 800);
    let height = dim(args, "height", 450);
    let frame_count = dim(args, "frames", 4).clamp(1, 12) as usize;

    // Re-run the scenario once per capture point so each frame reflects the
    // exact sim state at that time (the sim is deterministic).
    let mut images = Vec::with_capacity(frame_count);
    let mut checkpoints = Vec::with_capacity(frame_count);
    let mut renderer = frames::headless_renderer(width, height)?;
    for i in 0..frame_count {
        let frac = (i + 1) as f64 / frame_count as f64;
        let mut p = params.clone();
        p.duration_s = params.duration_s * frac;
        let run = scenario::run_scenario(&p)?;
        let hud = frames::hud_from_app(&run.app, mode_label(&params));
        let follow = frames::follow_from_app(&run.app);
        let png = frames::capture_png(&mut renderer, &hud, run.app.ride.distance_m, follow)?;
        checkpoints.push(json!({
            "t_s": run.app.ride.elapsed_s,
            "distance_m": run.app.ride.distance_m,
            "speed_kmh": run.app.ride.speed_mps * 3.6,
            "grade": run.app.ride.grade,
        }));
        images.push(ToolImage {
            name: format!("frame-{:02}", i + 1),
            png,
        });
    }

    Ok(ToolOutput {
        text: serde_json::to_string_pretty(&json!({ "checkpoints": checkpoints }))
            .unwrap_or_default(),
        images,
    })
}

fn hud_probe(args: &Value) -> Result<ToolOutput, String> {
    let width = dim(args, "width", 960);
    let height = dim(args, "height", 540);
    let f = |k: &str| args.get(k).and_then(Value::as_f64);

    let hud = HudSnapshot {
        power_w: f("power_w"),
        cadence_rpm: f("cadence_rpm"),
        heart_rate_bpm: f("heart_rate_bpm"),
        speed_mps: f("speed_mps").unwrap_or(0.0),
        distance_m: f("distance_m").unwrap_or(0.0),
        elapsed_s: f("elapsed_s").unwrap_or(0.0),
        grade: f("grade").unwrap_or(0.0),
        mode: match args.get("mode").and_then(Value::as_str) {
            Some("sim") | Some("SIM") => "SIM",
            Some("free") | Some("FREE") => "FREE",
            _ => "ERG",
        },
        workout_interval: args
            .get("interval")
            .and_then(Value::as_str)
            .map(String::from),
        workout_target_w: f("target_w"),
        attribution: args
            .get("attribution")
            .and_then(Value::as_str)
            .map(String::from),
    };

    let mut renderer = frames::headless_renderer(width, height)?;
    let png = frames::capture_png(&mut renderer, &hud, hud.distance_m, None)?;
    Ok(ToolOutput {
        text: serde_json::to_string_pretty(&json!({ "hud_lines": hud.lines() }))
            .unwrap_or_default(),
        images: vec![ToolImage {
            name: "hud".into(),
            png,
        }],
    })
}

fn workout_preview(args: &Value) -> Result<ToolOutput, String> {
    let ftp = args.get("ftp_w").and_then(Value::as_f64).unwrap_or(250.0);
    let workout = if let Some(xml) = args.get("zwo_xml").and_then(Value::as_str) {
        velo_core::parse_zwo_xml(xml).map_err(|e| e.to_string())?
    } else if let Some(w) = args.get("workout") {
        serde_json::from_value(w.clone()).map_err(|e| format!("invalid workout: {e}"))?
    } else if args
        .get("sample")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        velo_core::Workout::sample_threshold()
    } else {
        return Err("provide workout JSON, zwo_xml, or sample=true".into());
    };
    workout.validate()?;

    let mut t = 0.0;
    let intervals: Vec<Value> = workout
        .intervals
        .iter()
        .map(|i| {
            let engine_target = match i.target {
                velo_core::WorkoutTarget::ErgWatts(w) => Some(w),
                velo_core::WorkoutTarget::FtpPercent(p) => Some(ftp * p / 100.0),
                velo_core::WorkoutTarget::FreeRide => None,
            };
            let start = t;
            t += i.duration_s;
            json!({
                "name": i.name,
                "start_s": start,
                "duration_s": i.duration_s,
                "target": i.target,
                "resolved_watts": engine_target,
            })
        })
        .collect();

    Ok(ToolOutput::text(&json!({
        "name": workout.name,
        "ftp_w": ftp,
        "total_duration_s": workout.total_duration_s(),
        "intervals": intervals,
    })))
}

fn fit_export_check(args: &Value) -> Result<ToolOutput, String> {
    let mut params = parse_scenario(args)?;
    params.record = true;
    let run = scenario::run_scenario(&params)?;
    let fit = run.app.export_fit().map_err(|e| e.to_string())?;

    let parsed = fitparser::from_bytes(&fit).map_err(|e| format!("FIT parse failed: {e}"))?;
    use fitparser::profile::MesgNum;
    let records = parsed.iter().filter(|m| m.kind() == MesgNum::Record).count();
    let sessions = parsed.iter().filter(|m| m.kind() == MesgNum::Session).count();
    let laps = parsed.iter().filter(|m| m.kind() == MesgNum::Lap).count();

    Ok(ToolOutput::text(&json!({
        "fit_bytes": fit.len(),
        "valid_header": &fit[8..12] == b".FIT",
        "record_messages": records,
        "session_messages": sessions,
        "lap_messages": laps,
        "scenario_summary": run.summary,
    })))
}

fn replay_camera_preview(args: &Value) -> Result<ToolOutput, String> {
    let mut params = parse_scenario(args)?;
    params.record = true;
    if matches!(params.route, crate::scenario::RouteKind::None) {
        params.route = crate::scenario::RouteKind::Rolling;
    }
    let width = dim(args, "width", 800);
    let height = dim(args, "height", 450);
    let frame_count = dim(args, "frames", 4).clamp(1, 12) as usize;

    let run = scenario::run_scenario(&params)?;
    let samples = run.app.ride_session.samples();
    if samples.is_empty() {
        return Err("scenario recorded no samples".into());
    }
    let route = run.app.route.as_ref().ok_or("no route loaded")?;

    let clips = velo_core::plan_highlight_clips(samples, run.app.ride.elapsed_s);
    if clips.is_empty() {
        return Err("no highlight clips planned (ride too short?)".into());
    }
    let clip = match args.get("clip_label").and_then(Value::as_str) {
        Some(label) => clips
            .iter()
            .find(|c| c.label == label)
            .ok_or_else(|| {
                let known: Vec<_> = clips.iter().map(|c| c.label.as_str()).collect();
                format!("no clip labeled {label:?}; available: {known:?}")
            })?
            .clone(),
        None => clips[0].clone(),
    };

    let track = velo_core::build_rider_track(route, samples);
    let camera = velo_core::ReplayCamera::for_clip(track, &clip)
        .ok_or("could not build replay camera")?;

    let mut renderer = frames::headless_renderer(width, height)?;
    if args.get("show_bike").and_then(Value::as_bool).unwrap_or(true) {
        frames::load_placeholder_bike(&mut renderer)?;
    }
    let hud = frames::hud_from_app(&run.app, mode_label(&params));
    let mut images = Vec::with_capacity(frame_count);
    let mut poses = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let clip_t = if frame_count == 1 {
            0.0
        } else {
            clip.duration_s * i as f64 / (frame_count - 1) as f64
        };
        let pose = camera.pose_at(clip_t);
        // The grid still needs the rider position to anchor itself.
        let ride_t = clip.start_elapsed_s + clip_t;
        let rider = camera.rider_at(ride_t);
        let (e2, _, n2) = route.position_enu_at(distance_at(samples, ride_t) + 5.0);
        let follow = velo_render::RouteFollow {
            east: rider.x,
            up: rider.y,
            north: rider.z,
            forward: velo_render::forward_from_enu(rider.x, rider.y, rider.z, e2, n2),
        };
        renderer.set_replay_camera(Some(pose));
        let png = frames::capture_png(&mut renderer, &hud, 0.0, Some(follow))?;
        poses.push(json!({
            "clip_t": clip_t,
            "eye": [pose.eye_east, pose.eye_up, pose.eye_north],
            "look": [pose.look_east, pose.look_up, pose.look_north],
        }));
        images.push(ToolImage {
            name: format!("clip-{}-{:02}", clip.label.to_lowercase().replace(' ', "-"), i + 1),
            png,
        });
    }

    Ok(ToolOutput {
        text: serde_json::to_string_pretty(&json!({
            "clip": { "label": clip.label, "start_s": clip.start_elapsed_s, "duration_s": clip.duration_s },
            "style": format!("{:?}", camera.style()),
            "available_clips": clips.iter().map(|c| c.label.clone()).collect::<Vec<_>>(),
            "poses": poses,
        }))
        .unwrap_or_default(),
        images,
    })
}

/// Rider distance along the route at ride time `t` (linear over samples).
fn distance_at(samples: &[velo_core::RideSample], t: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    if t <= samples[0].elapsed_s {
        return samples[0].distance_m;
    }
    let last = samples.last().expect("non-empty");
    if t >= last.elapsed_s {
        return last.distance_m;
    }
    let idx = samples
        .partition_point(|s| s.elapsed_s <= t)
        .saturating_sub(1);
    let a = &samples[idx];
    let b = &samples[(idx + 1).min(samples.len() - 1)];
    let span = (b.elapsed_s - a.elapsed_s).max(1e-9);
    a.distance_m + (b.distance_m - a.distance_m) * ((t - a.elapsed_s) / span)
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub fn registry() -> Vec<ToolDef> {
    let scenario_schema = |extra: Value| -> Value {
        let mut props = scenario_properties();
        if let (Some(obj), Some(extra)) = (props.as_object_mut(), extra.as_object()) {
            for (k, v) in extra {
                obj.insert(k.clone(), v.clone());
            }
        }
        json!({"type": "object", "properties": props})
    };

    vec![
        ToolDef {
            name: "feature_inventory",
            description: "Product/milestone/crate inventory of the VeloSim cycling simulator, plus available evaluation tools.",
            input_schema: json!({"type": "object", "properties": {}}),
            run: feature_inventory,
        },
        ToolDef {
            name: "sim_scenario",
            description: "Run a deterministic headless ride scenario (ERG/SIM/free, synthetic routes, workouts) and return a telemetry timeline + summary JSON. Evaluates physics, trainer command, and workout engine behavior.",
            input_schema: scenario_schema(json!({})),
            run: sim_scenario,
        },
        ToolDef {
            name: "render_frame",
            description: "Run a scenario, then render the actual wgpu scene + HUD overlay at its final state and return a PNG screenshot for visual/multimodal evaluation.",
            input_schema: scenario_schema(json!({
                "width": {"type": "integer", "description": "default 960"},
                "height": {"type": "integer", "description": "default 540"}
            })),
            run: render_frame,
        },
        ToolDef {
            name: "render_ride_sequence",
            description: "Render N evenly spaced screenshots through a scenario (deterministic re-simulation per checkpoint). Use to evaluate HUD/scene evolution over a ride.",
            input_schema: scenario_schema(json!({
                "frames": {"type": "integer", "description": "number of frames, 1-12 (default 4)"},
                "width": {"type": "integer"},
                "height": {"type": "integer"}
            })),
            run: render_ride_sequence,
        },
        ToolDef {
            name: "hud_probe",
            description: "Render the HUD overlay with explicit values (power, cadence, HR, speed, grade, interval, attribution...) and return a PNG. Fast UI-only evaluation of formatting and layout.",
            input_schema: json!({"type": "object", "properties": {
                "power_w": {"type": "number"}, "cadence_rpm": {"type": "number"},
                "heart_rate_bpm": {"type": "number"}, "speed_mps": {"type": "number"},
                "distance_m": {"type": "number"}, "elapsed_s": {"type": "number"},
                "grade": {"type": "number"}, "mode": {"type": "string", "enum": ["erg", "sim", "free"]},
                "interval": {"type": "string"}, "target_w": {"type": "number"},
                "attribution": {"type": "string"},
                "width": {"type": "integer"}, "height": {"type": "integer"}
            }}),
            run: hud_probe,
        },
        ToolDef {
            name: "replay_camera_preview",
            description: "Run a recorded scenario, plan highlight clips, and render frames along the cinematic replay camera path (drone rise / orbit / flyby / chase pull) for a chosen clip. Visual evaluation of the M5 replay camera.",
            input_schema: scenario_schema(json!({
                "clip_label": {"type": "string", "description": "Start | Power surge | Mid-ride | Finish (default: first planned clip)"},
                "show_bike": {"type": "boolean", "description": "draw the placeholder bike as the subject (default true)"},
                "frames": {"type": "integer", "description": "frames across the clip, 1-12 (default 4)"},
                "width": {"type": "integer"},
                "height": {"type": "integer"}
            })),
            run: replay_camera_preview,
        },
        ToolDef {
            name: "workout_preview",
            description: "Validate a workout (velo-core JSON or Zwift .zwo XML, or sample=true for the built-in 2x20) and return its interval timeline with resolved ERG watts.",
            input_schema: json!({"type": "object", "properties": {
                "workout": {"type": "object"},
                "zwo_xml": {"type": "string"},
                "sample": {"type": "boolean"},
                "ftp_w": {"type": "number", "description": "default 250"}
            }}),
            run: workout_preview,
        },
        ToolDef {
            name: "fit_export_check",
            description: "Run a recorded scenario, export the FIT activity file, and validate it with a real FIT parser (message counts, header). End-to-end check of the ride-recording pipeline.",
            input_schema: scenario_schema(json!({})),
            run: fit_export_check,
        },
    ]
}

pub fn find(name: &str) -> Option<ToolDef> {
    registry().into_iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_are_unique() {
        let names: Vec<_> = registry().iter().map(|t| t.name).collect();
        let mut dedup = names.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(names.len(), dedup.len());
    }

    #[test]
    fn feature_inventory_mentions_crates() {
        let out = feature_inventory(&json!({})).unwrap();
        assert!(out.text.contains("velo-core"));
        assert!(out.text.contains("velo-eval-mcp"));
    }

    #[test]
    fn sim_scenario_tool_returns_timeline() {
        let out = sim_scenario(&json!({"duration_s": 10.0})).unwrap();
        assert!(out.text.contains("timeline"));
        assert!(out.text.contains("summary"));
    }

    #[test]
    fn workout_preview_sample_resolves_watts() {
        let out = workout_preview(&json!({"sample": true, "ftp_w": 200.0})).unwrap();
        assert!(out.text.contains("2x20 Threshold"));
        assert!(out.text.contains("190")); // 95% of 200 W
    }

    #[test]
    fn fit_export_check_validates() {
        let out = fit_export_check(&json!({"duration_s": 3.0})).unwrap();
        assert!(out.text.contains("\"valid_header\": true"));
    }
}
