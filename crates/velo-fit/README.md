# velo-fit

Minimal FIT activity file encoder — indoor rides with power, cadence, HR, speed, and distance records.

## Key modules

| Module | Role |
|--------|------|
| `encode` | `encode_activity`, `FitRide`, `FitRecordSample`, `FitEncodeError` |
| `writer` | Low-level `FitWriter`, field definitions |
| `types` | Timestamp/distance/speed conversions, semicircles |
| `crc` | FIT CRC-16 |

Indoor rides use fixed lat/lon constants (`INDOOR_LAT_DEG`, `INDOOR_LON_DEG`).

## Dependencies

`thiserror` only. Portable — no OS or network.

**Apple-symbol rule:** keep this crate Apple-free so FIT export stays cross-platform.

## Test

```bash
cargo test -p velo-fit    # round-trip via fitparser, proptest on timestamps
```

## Milestone

**M2b** (Strava upload input)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
