---
name: quality-pass
description: >-
  Cross-cutting VeloSim codebase quality pass: simplifications, cohesion, doc sync,
  integration tests. Use after milestone merges or when parallel milestone work may have
  desynced contexts. Run on the dev branch before merging to main.
---

# VeloSim Quality Pass

Post-milestone cleanup. Consumes the **entire** codebase and restores cohesion after parallel milestone work.

**Companion skills** — apply all three on a full pass:

- [rust-best-practices](../rust-best-practices/SKILL.md) — portable crates, tests, lint
- [swift-best-practices](../swift-best-practices/SKILL.md) — shell UI, UniFFI, Package.swift

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
- [ ] Read quality-pass + rust-best-practices + swift-best-practices skills
- [ ] Read VeloSim-Technical-Plan.md + root README + every crate README
- [ ] Map crate boundaries vs actual deps (Cargo.toml, Package.swift)
- [ ] Find dead code, duplicate logic, inconsistent naming
- [ ] Align public APIs and FFI surface with docs
- [ ] Run cargo test && just lint (if available)
- [ ] Add/strengthen integration tests (prefer cross-crate)
- [ ] Sync documentation (READMEs, STRAVA.md, inline rustdoc where thin)
- [ ] Write docs/QUALITY_PASS.md report (findings + deferred items)
- [ ] Granular commits on dev (refactor / test / docs prefixes)
```

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

Append or replace `docs/QUALITY_PASS.md`:

```markdown
# Quality pass — YYYY-MM-DD

## Scope
Milestone / trigger: …

## Changes made
- …

## Findings (deferred)
- …

## Doc sync
- …

## Test coverage added
- …
```

## Output

Return: summary of commits, test command results, deferred items for next milestone agent.
