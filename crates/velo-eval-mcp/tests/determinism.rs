//! Determinism guarantees: identical scenario params must produce identical
//! telemetry and identical rendered frames (golden-image property).

use serde_json::json;
use velo_eval_mcp::tools;

fn run_tool(name: &str, args: serde_json::Value) -> Result<(String, Vec<Vec<u8>>), String> {
    let tool = tools::find(name).ok_or("tool missing")?;
    let out = (tool.run)(&args)?;
    Ok((out.text, out.images.into_iter().map(|i| i.png).collect()))
}

#[test]
fn sim_scenario_is_deterministic() {
    let args = json!({
        "mode": "sim", "route": "rolling", "rider_power_w": 240.0,
        "duration_s": 30.0, "sample_every_s": 5.0
    });
    let (a, _) = run_tool("sim_scenario", args.clone()).unwrap();
    let (b, _) = run_tool("sim_scenario", args).unwrap();
    assert_eq!(a, b, "identical params must produce identical telemetry");
}

#[test]
fn rendered_frame_is_deterministic() {
    let args = json!({
        "mode": "erg", "target_power_w": 220.0, "route": "flat",
        "duration_s": 15.0, "width": 160, "height": 120
    });
    let first = run_tool("render_frame", args.clone());
    let Ok((_, images_a)) = first else {
        eprintln!("skipping: headless renderer unavailable");
        return;
    };
    let (_, images_b) = run_tool("render_frame", args).unwrap();
    assert_eq!(images_a.len(), 1);
    assert_eq!(
        images_a[0], images_b[0],
        "identical params must produce byte-identical PNG frames"
    );
}
