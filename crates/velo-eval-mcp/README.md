# velo-eval-mcp

MCP server + CLI that lets an AI agent (or a human) evaluate VeloSim's UI and
feature set headlessly: run deterministic ride scenarios, render real
wgpu scene + HUD screenshots for multimodal review, validate workouts and FIT
exports — no macOS shell, window, or GPU required (works on a software Vulkan
driver such as Mesa lavapipe).

## Modes

```bash
# MCP stdio server (newline-delimited JSON-RPC 2.0) — registered in /.mcp.json
cargo run -p velo-eval-mcp -- serve

# One-shot tool call; PNG frames are written to --save-dir
cargo run -p velo-eval-mcp -- call render_frame \
  --args '{"mode":"sim","route":"alpine_climb","rider_power_w":280,"duration_s":120}' \
  --save-dir /tmp

cargo run -p velo-eval-mcp -- list-tools
```

## Tools

| Tool | What it evaluates |
|------|-------------------|
| `feature_inventory` | Product/milestone/crate map |
| `sim_scenario` | Physics, trainer commands, workout engine (JSON timeline) |
| `render_frame` | Scene + HUD screenshot at the end of a scenario (PNG) |
| `render_ride_sequence` | N screenshots through a ride (PNG series) |
| `hud_probe` | HUD layout/formatting with explicit values (PNG) |
| `workout_preview` | Workout JSON / `.zwo` interval timeline + resolved watts |
| `fit_export_check` | End-to-end ride-recording → FIT encode → parse validation |

Scenarios support ERG/SIM/free modes, synthetic routes (`flat`, `rolling`,
`alpine_climb`), structured workouts (velo-core JSON or Zwift `.zwo` XML),
and ride recording. Simulation is deterministic — identical params produce
identical frames, which makes the screenshots usable as golden images.

## Why

The macOS shell can only be exercised on a Mac. Everything below the shell —
physics, ride loop, workout engine, HUD composition, renderer output — is
portable, and this server makes it observable to a multimodal agent so UI and
feature regressions can be caught from any host, including CI.
