# VeloSim (cyclosim)

[![CI](https://github.com/Gitter499/cyclosim/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/Gitter499/cyclosim/actions/workflows/ci.yml)

Native offline cycling simulator for macOS. Rust simulation core, Swift shell, smart trainer support.

## Requirements

- macOS 26+ recommended (Liquid Glass UI); macOS 14+ with solid fallbacks
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Xcode](https://developer.apple.com/xcode/) (Swift, Metal)

## Build and run

```bash
cargo test
./scripts/build.sh
open shell-macos/.build/release/VeloSim
```

Optional: [just](https://github.com/casey/just) wraps common tasks (`just build`, `just run`, `just lint`).

## Documentation

| Doc | Description |
|-----|-------------|
| [VeloSim-Technical-Plan.md](VeloSim-Technical-Plan.md) | Architecture, milestones, testing, quality log |
| [VeloSim-Roadmap.md](VeloSim-Roadmap.md) | Product parity vs Zwift/MyWhoosh, UI spec, issue tracker |
| [AGENTS.md](AGENTS.md) | Agent and contributor guide |
| [STRAVA.md](STRAVA.md) | Strava OAuth setup |

## Crates

Rust workspace under [`crates/`](crates/). macOS app under [`shell-macos/`](shell-macos/).

## Development

Work happens on [`dev`](https://github.com/Gitter499/cyclosim/tree/dev) via [GitHub issues](https://github.com/Gitter499/cyclosim/issues) and pull requests. See [AGENTS.md](AGENTS.md) for workflow.

## License

MIT
