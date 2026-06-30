---
name: rust-best-practices
description: >-
  VeloSim Rust coding standards: portable crate boundaries, error handling, newtypes,
  platform traits, integration tests, and lint/commit workflow. Use when writing or
  reviewing Rust in crates/, running quality passes, or adding cross-crate tests.
---

# VeloSim Rust Best Practices

Checklist for agents working in `crates/`. Pair with [quality-pass](../quality-pass/SKILL.md) on milestone cleanup.

## Crate boundaries

| Crate | Rule |
|-------|------|
| `velo-units` | Physical quantity newtypes only; no I/O, no platform |
| `velo-platform` | **Traits only** — `TrainerControl`, `SensorSource`, `MediaCapture`, … |
| `velo-core` | Physics, ride loop, workouts, routes; depends on units + platform only |
| `velo-ffi` | UniFFI surface; wires core, render, rides — **only** crate Swift links |
| Other crates | Feature-specific (`velo-fit`, `velo-rides`, `velo-render`, …) |

**Portable rule:** `velo-units`, `velo-platform`, `velo-core` must stay Apple-free. Run `./scripts/lint-apple-symbols.sh` or `just lint`.

Do not add Apple, wgpu, SQLite, or UniFFI deps to portable crates.

## Error handling

- Libraries return `Result<T, E>` — use `thiserror` for public errors.
- No `unwrap()` / `expect()` in library code paths; tests and `main`-style binaries may use them sparingly.
- Propagate with `?`; map foreign errors at crate boundaries.

## Types

- Put physical quantities in `velo-units` newtypes (`PowerW`, `DistanceM`, …) — avoid raw `f64` in public APIs.
- Shell↔core contracts live in `velo-platform` traits; implementations stay in shell or test mocks.
- DTOs for FFI live in `velo-ffi` (generated + hand-written); keep core types separate.

## Tests

| Pattern | Where |
|---------|-------|
| Golden replay | `velo-core/tests/golden_replay.rs` |
| Cross-crate integration | `velo-ffi/tests/*`, `velo-rides/tests/*` |
| Headless mocks | `velo-ffi/tests/common/mod.rs` — reuse, don't duplicate |
| Workout / highlight / zwo | `velo-core/tests/workout_erg.rs`, `highlight_clips.rs`, `zwo_import.rs` |

Prefer integration tests that assert real behavior over trivial unit asserts.

## Before commit

```bash
cargo test --workspace
./scripts/lint-apple-symbols.sh
```

Commit via `./scripts/git-commit-clean.sh "message"` (not plain `git commit`).

## Checklist

```
Rust pass:
- [ ] Change lives in the correct crate; no boundary violations
- [ ] Portable crates Apple-free (lint passed)
- [ ] Result + thiserror in libs; no unwrap in prod paths
- [ ] New physical types in velo-units; traits in velo-platform
- [ ] Integration test if behavior crosses crates or FFI
- [ ] README / rustdoc updated if public API changed
- [ ] cargo test --workspace green
```

## Additional resources

- Detailed patterns: [reference.md](reference.md)
- Milestone architecture: [VeloSim-Technical-Plan.md](../../../VeloSim-Technical-Plan.md)
- Contributor workflow: [AGENTS.md](../../../AGENTS.md)
