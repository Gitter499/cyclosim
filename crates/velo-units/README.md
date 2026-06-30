# velo-units

Strongly typed physical quantities for VeloSim — no bare `f64` in core APIs.

## Key types

`Watts`, `Meters`, `MetersPerSecond`, `Grade` (rise/run ratio), `Kilograms`, `Rpm`, `Bpm` — thin newtypes with `new()` constructors.

## Dependencies

None. Leaf crate; safe for any portable layer to depend on.

**Apple-symbol rule:** must stay free of CoreBluetooth, AppKit, Metal, MusicKit, and other OS frameworks (enforced by CI for core crates).

## Test

```bash
cargo test -p velo-units
```

## Milestone

**M0** (foundation) · used throughout **M1+**

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
