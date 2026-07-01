---
name: quality-pass
description: >-
  Cross-cutting VeloSim codebase quality pass: simplifications, cohesion, doc sync,
  integration tests. Use after milestone merges or when parallel milestone work may have
  desynced contexts. Run on the dev branch before merging to main.
---

# VeloSim Quality Pass

Post-milestone cleanup. Consumes the **entire** codebase and restores cohesion after parallel milestone work.

**Companion skills** — apply all four on a full pass:

- [rust-best-practices](../rust-best-practices/SKILL.md) — portable crates, tests, lint
- [swift-best-practices](../swift-best-practices/SKILL.md) — shell UI, UniFFI, Package.swift
- [velo-ui-parity](../velo-ui-parity/SKILL.md) — multi-page shell IA, dashboard, ride HUD layout (required for shell chrome work)

## When to run

- After merging a milestone (M3, M4, …) into `dev`
- When crate READMEs, plan doc, or APIs drift apart
- Before opening a `dev` → `main` PR

## Branch workflow

```bash
git checkout dev
git pull origin dev
git checkout -b feat/issue-N-short-name
# … work …
# Open PR to dev; use "Closes #N" in body
```

After milestone merges to `dev`, run a quality pass before opening `dev` → `main`.

## Pass checklist

Copy and track:

```
Quality pass progress:
- [ ] Read quality-pass + rust-best-practices + swift-best-practices (+ velo-ui-parity if shell changed)
- [ ] Read VeloSim-Technical-Plan.md + root README + every crate README
- [ ] Map crate boundaries vs actual deps (Cargo.toml, Package.swift)
- [ ] Find dead code, duplicate logic, inconsistent naming
- [ ] Align public APIs and FFI surface with docs
- [ ] Run cargo test && just lint (if available)
- [ ] Add/strengthen integration tests (prefer cross-crate)
- [ ] Sync documentation (READMEs, STRAVA.md, inline rustdoc where thin)
- [ ] Append quality log entry to [VeloSim-Technical-Plan.md §22](../../../VeloSim-Technical-Plan.md)
- [ ] Granular commits on dev (refactor / test / docs prefixes)
```

Also read [rust-best-practices](../rust-best-practices/SKILL.md) and [swift-best-practices](../swift-best-practices/SKILL.md) when touching those languages. For shell navigation or ride layout changes, read [velo-ui-parity](../velo-ui-parity/SKILL.md) and [VeloSim-Roadmap.md](../../../VeloSim-Roadmap.md) Part II.

## Priorities (high → low)

1. **Correctness** — fix bugs or test gaps found during review
2. **Cohesion** — one obvious way to do things; shared types live in the right crate
3. **Reduction** — delete unused code; collapse over-abstraction from rushed milestone slices
4. **Integration tests** — ride loop, FFI round-trip, FIT encode, ride DB lifecycle
5. **Docs** — README tables match reality; milestone status current

## Constraints

- **Minimize scope** — no feature work; defer to milestone agents
- **Portable crates** (`velo-units`, `velo-platform`, `velo-core`) must stay Apple-free (`just lint`)
- **No history rewrite** on `main`
- **No breaking FFI** without updating Swift bindings and tests
- Prefer existing patterns over new frameworks

## Integration test targets

| Area | Suggested location | What to assert |
|------|-------------------|----------------|
| Physics + session | `velo-core/tests/` | Golden integrator, save/load session |
| FIT pipeline | `velo-fit/tests/` or workspace test | Encode → parse round-trip |
| Ride library | `velo-rides/tests/` | Insert, list, delete, paths |
| FFI | `velo-ffi/tests/` | Callback registration, handle lifecycle |
| Shell | `shell-macos/Tests/` | BLE codec, Strava token store (existing) |

## Commit style

```
refactor(core): unify ride tick helpers
test(ffi): integration test for sensor callback round-trip
docs: sync velo-render README with capture API
chore(quality): quality pass report for M3 merge
```

## Report template

Append to [VeloSim-Technical-Plan.md §22](../../../VeloSim-Technical-Plan.md):

```markdown
| YYYY-MM-DD | Trigger | Summary |
| … | … | … |
```

Include deferred findings and test coverage in the commit message or PR body.

## Output

Return: summary of commits, test command results, deferred items for next milestone agent.
