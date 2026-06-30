# cyclosim

Native offline cycling simulator for macOS. Rust simulation core, Swift shell, smart trainer support.

## Requirements

- macOS (Apple Silicon)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Xcode](https://developer.apple.com/xcode/) (Swift, Metal)

## Build and run

```bash
cargo test
just build && just run
```

Optional: [just](https://github.com/casey/just) wraps common tasks (`just test`, `just lint`).

## Documentation

| Doc | Description |
|-----|-------------|
| [VeloSim-Technical-Plan.md](VeloSim-Technical-Plan.md) | Architecture and milestones |
| [AGENTS.md](AGENTS.md) | Agent and contributor guide |
| [STRAVA.md](STRAVA.md) | Strava OAuth setup |
| [docs/QUALITY_PASS.md](docs/QUALITY_PASS.md) | Quality pass log |

## Crates

Rust workspace under [`crates/`](crates/). macOS app under [`shell-macos/`](shell-macos/).

## Development

Work happens on [`dev`](https://github.com/Gitter499/cyclosim/tree/dev) via [GitHub issues](https://github.com/Gitter499/cyclosim/issues) and pull requests. See [AGENTS.md](AGENTS.md) for workflow.

## License

MIT
