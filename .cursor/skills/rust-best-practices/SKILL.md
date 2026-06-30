---
name: rust-best-practices
description: >-
  VeloSim Rust coding standards: portable crate boundaries, error handling, newtypes,
  platform traits, integration tests, and lint/commit workflow. Use when writing or
  reviewing Rust in crates/, running quality passes, or adding cross-crate tests.
---

# VeloSim Rust Best Practices

Checklist for agents working in `crates/`. Community standards summarized below; full citations in [reference.md](reference.md). Pair with [quality-pass](../quality-pass/SKILL.md) on milestone cleanup.

## Community baseline

VeloSim adopts (does not duplicate) these established guides:

| Area | Standard |
|------|----------|
| Formatting | `rustfmt` defaults — run `cargo fmt` before commit |
| Linting | `clippy` with workspace lints; fix warnings, don't `allow` without reason |
| Public APIs | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) checklist |
| Idioms | [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) where applicable |

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

## Style & naming

- Follow default `rustfmt` (official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)).
- `snake_case` functions/modules/vars; `UpperCamelCase` types; `SCREAMING_SNAKE_CASE` constants.
- Prefer `Self` and `use` imports over fully qualified paths in function bodies.
- Keep functions focused; extract helpers rather than deep nesting.

## Error handling

- Libraries return `Result<T, E>` — use `thiserror` for public errors implementing `std::error::Error`.
- No `unwrap()` / `expect()` in library code paths; tests and `main`-style binaries may use them sparingly.
- Propagate with `?`; map foreign errors at crate boundaries (`From` / `map_err`).
- Use `Option` only when absence is expected; prefer `Result` for failures.

## Types & public API

- Put physical quantities in `velo-units` newtypes (`PowerW`, `DistanceM`, …) — avoid raw `f64` in public APIs.
- Shell↔core contracts live in `velo-platform` traits; implementations stay in shell or test mocks.
- DTOs for FFI live in `velo-ffi` (generated + hand-written); keep core types separate.
- Public items need `///` docs with a one-line summary; crate root needs `#![doc = …]` or `//!` module docs.
- Prefer constructor methods with semantic names over bare `new()` when meaning isn't obvious.
- Accept `impl Into<T>` at boundaries; return concrete owned types unless trait objects are required.

## Idioms (from patterns + clippy)

- **Newtype** for units and IDs (`velo-units` is the canonical place).
- **Trait objects** (`dyn TrainerControl`) only in `velo-platform` / shell wiring — prefer generics in core hot paths.
- **RAII** for locks, file handles, GPU resources — no manual drop ordering hacks.
- Avoid `clone()` when borrowing suffices; clippy `redundant_clone` is a signal.
- Minimize `unsafe`; document invariants when unavoidable (render/FFI only).

## Tests

| Pattern | Where |
|---------|-------|
| Golden replay | `velo-core/tests/golden_replay.rs` |
| Cross-crate integration | `velo-ffi/tests/*`, `velo-rides/tests/*` |
| Headless mocks | `velo-ffi/tests/common/mod.rs` — reuse, don't duplicate |
| Workout / highlight / zwo | `velo-core/tests/workout_erg.rs`, `highlight_clips.rs`, `zwo_import.rs` |

Prefer integration tests in `tests/` that assert real behavior over trivial unit asserts.

## Before commit

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings   # if workspace enables it
cargo test --workspace
./scripts/lint-apple-symbols.sh
```

Commit via `./scripts/git-commit-clean.sh "message"` (not plain `git commit`).

## Checklist

```
Rust pass:
- [ ] Change lives in the correct crate; no boundary violations
- [ ] Portable crates Apple-free (lint passed)
- [ ] rustfmt + clippy clean
- [ ] Result + thiserror in libs; no unwrap in prod paths
- [ ] New physical types in velo-units; traits in velo-platform
- [ ] Public API items documented; API Guidelines respected
- [ ] Integration test if behavior crosses crates or FFI
- [ ] README / rustdoc updated if public API changed
- [ ] cargo test --workspace green
```

## Additional resources

- Adopted rules + source links: [reference.md](reference.md)
- Milestone architecture: [VeloSim-Technical-Plan.md](../../../VeloSim-Technical-Plan.md)
- Contributor workflow: [AGENTS.md](../../../AGENTS.md)
