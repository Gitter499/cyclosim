# VeloSim Rust — reference patterns

## Portable crate dependency graph

```
velo-units
velo-platform → velo-units
velo-core → velo-units, velo-platform, velo-fit
velo-render → velo-core, velo-units
velo-ffi → velo-core, velo-render, velo-rides, …
```

## Mock trainer (tests)

Use `RecordingTrainerControl` from `velo-platform` instead of local duplicate stubs:

```rust
use velo_platform::RecordingTrainerControl;
let trainer = RecordingTrainerControl::default();
// after tick: trainer.commands() captures set_target_power calls
```

## FFI integration test layout

```
crates/velo-ffi/tests/
├── common/mod.rs       # MockMedia, MockPublisher, TickSensors, …
├── workout_integration.rs
└── ride_library_integration.rs
```

Share mocks via `mod common;` — do not copy-paste callback impls.

## Golden files

Store expected output under `tests/fixtures/` or inline `include_str!`. Update goldens only when behavior intentionally changes; note in commit message.

## Commit prefixes

```
feat(core): …
fix(ffi): …
test(rides): …
refactor(platform): …
docs: …
chore(quality): …
```
