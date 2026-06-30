# VeloSim Rust — adopted community rules

Rules below are distilled from high-star community standards (star counts from 2026-06-30). VeloSim-specific layout patterns follow.

## Source repositories

| Source | Stars | URL |
|--------|------:|-----|
| rust-lang/rust (Style Guide) | 114,327 | https://github.com/rust-lang/rust/tree/master/src/doc/style-guide |
| rust-lang/book | 17,978 | https://github.com/rust-lang/book |
| rust-lang/rust-clippy | 13,327 | https://github.com/rust-lang/rust-clippy |
| rust-unofficial/patterns | 8,849 | https://github.com/rust-unofficial/patterns |
| rust-lang/rustfmt | 6,893 | https://github.com/rust-lang/rustfmt |
| rust-lang/api-guidelines | 1,334 | https://github.com/rust-lang/api-guidelines |

## Formatting & style (rustfmt + Rust Style Guide)

- Run `cargo fmt --all`; do not hand-format — defaults match the official style guide.
- Max line width 100 (rustfmt default); break long signatures across lines.
- Trailing commas in multi-line literals and match arms.
- Module order: inner items, then `mod` declarations; `use` grouped and sorted by rustfmt.

## Naming (API Guidelines + Style Guide)

- Crates/modules/functions/variables: `snake_case`.
- Types/traits/enums/variants: `UpperCamelCase`.
- Constants/statics: `SCREAMING_SNAKE_CASE`.
- Type parameters: short uppercase (`T`, `E`) or descriptive (`Item`, `Error`).
- Conversion methods: `as_` (cheap ref), `to_` (expensive), `into_` (consuming).
- Getters omit `get_` prefix unless ambiguity requires it.

## Documentation (API Guidelines)

- Every public item: `///` one-line summary; add examples for non-obvious APIs.
- Crate-level docs explain purpose and link to README.
- Error variants documented with when they occur.
- `# Errors`, `# Panics`, `# Safety` sections when applicable.

## Error handling (API Guidelines + Clippy)

- Fallible functions return `Result<T, E>`; use `?` for propagation.
- Public error enums: `thiserror` + `std::error::Error`.
- No `unwrap`/`expect` in library paths (`clippy::unwrap_used` / `expect_used`).
- `Option` for optional values; don't use `Result` with unit error type.

## Public API design (API Guidelines)

- Structs with clear invariants; prefer builder or constructor when setup is non-trivial.
- Accept `impl Into<T>` / `AsRef<T>` at boundaries; return owned types by default.
- Implement `From`/`Into` for type conversions at crate edges.
- Sealed traits or newtypes to prevent downstream impl breakage where needed.
- Features behind `Cargo.toml` features, not cfgs scattered in code.

## Idioms (rust-unofficial/patterns + book)

- **Newtype** for domain units (`PowerW`, not bare `f64`).
- **RAII** for resources; `Drop` cleans up, don't expose half-initialized state.
- **Strategy via traits** (`velo-platform`) instead of platform `cfg` in core.
- Prefer borrowing over cloning; use `Cow` only when truly needed.
- Integration tests in `tests/`; unit tests in same file under `#[cfg(test)]`.

## Clippy highlights (rust-clippy)

- Fix all warnings; avoid blanket `#[allow(clippy::…)]`.
- Prefer `is_empty()` over `len() == 0`.
- Use `matches!` for simple pattern checks.
- Collapse nested `if`/`match` where readability improves.
- `todo!` / `unimplemented!` only in WIP branches — never in merged prod paths.

## VeloSim-specific patterns

### Portable crate dependency graph

```
velo-units
velo-platform → velo-units
velo-core → velo-units, velo-platform, velo-fit
velo-render → velo-core, velo-units
velo-ffi → velo-core, velo-render, velo-rides, …
```

### Mock trainer (tests)

Use `RecordingTrainerControl` from `velo-platform` instead of local duplicate stubs:

```rust
use velo_platform::RecordingTrainerControl;
let trainer = RecordingTrainerControl::default();
// after tick: trainer.commands() captures set_target_power calls
```

### FFI integration test layout

```
crates/velo-ffi/tests/
├── common/mod.rs       # MockMedia, MockPublisher, TickSensors, …
├── workout_integration.rs
└── ride_library_integration.rs
```

Share mocks via `mod common;` — do not copy-paste callback impls.

### Golden files

Store expected output under `tests/fixtures/` or inline `include_str!`. Update goldens only when behavior intentionally changes; note in commit message.

### Commit prefixes

```
feat(core): …
fix(ffi): …
test(rides): …
refactor(platform): …
docs: …
chore(quality): …
```
