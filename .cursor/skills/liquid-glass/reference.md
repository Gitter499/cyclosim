# Liquid Glass — extended reference

## Modifier cheat sheet

```swift
// Single floating control
Button("Start") { }
    .veloGlassCapsule(interactive: true)

// Toolbar group
GlassEffectContainer(spacing: 12) {
    HStack {
        Button("ERG") { }.veloGlassCapsule(interactive: true)
        Button("SIM") { }.veloGlassCapsule(interactive: true)
    }
}

// Morphing related chrome
@Namespace private var chromeNS
GlassEffectContainer {
    if showSummary {
        summaryBar.glassEffectID("summary", in: chromeNS)
    } else {
        setupBar.glassEffectID("summary", in: chromeNS)
    }
}
```

## Fallback matrix

| macOS | Material |
|-------|----------|
| 26+ | `.glassEffect(.regular)` / `.interactive()` |
| 14–25 | `.ultraThinMaterial`, `.bar` toolbar style |

## macOS Tahoe specifics

- Window uses concentric corners — avoid hard rectangular chrome flush to window edge; use padding.
- Sidebars: prefer native `NavigationSplitView` glass sidebar when restructuring; VeloSim currently uses `HSplitView` — apply glass to the **trailing chrome column**, not the split divider.

## Testing accessibility

Manual checklist (Simulator or device):

1. System Settings → Accessibility → Display → Reduce Transparency ON → chrome still readable.
2. Reduce Motion ON → no broken layout when morphing disabled.
3. Light and dark appearance — stats text contrast on `.quaternary` section bodies.
