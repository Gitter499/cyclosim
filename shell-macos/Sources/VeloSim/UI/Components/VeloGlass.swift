import SwiftUI
import VeloFFI

// Shared Liquid Glass helpers — see docs/VeloSim-UI-and-Zwift-Parity-Guide.md §4 and §8.
//
// Real `.glassEffect` on macOS 26+ only. Pre-26 / CI uses solid `.quaternary` — no fake glass.

extension View {
    /// Navigation-layer capsule chrome with Liquid Glass on macOS 26+, solid fallback otherwise.
    @ViewBuilder
    public func veloGlassCapsule(interactive: Bool = false) -> some View {
        #if VELO_LIQUID_GLASS
        if #available(macOS 26, *) {
            modifier(VeloGlassCapsuleModifier(interactive: interactive))
        } else {
            background(.quaternary, in: Capsule())
        }
        #else
        background(.quaternary, in: Capsule())
        #endif
    }

    /// Rounded rect chrome for section headers / action bars.
    @ViewBuilder
    public func veloGlassRoundedRect(cornerRadius: CGFloat = 12, interactive: Bool = false) -> some View {
        #if VELO_LIQUID_GLASS
        if #available(macOS 26, *) {
            modifier(VeloGlassRoundedRectModifier(cornerRadius: cornerRadius, interactive: interactive))
        } else {
            background(.quaternary, in: RoundedRectangle(cornerRadius: cornerRadius))
        }
        #else
        background(.quaternary, in: RoundedRectangle(cornerRadius: cornerRadius))
        #endif
    }

    /// Plain browse-screen header — no glass (guide §0).
    public func veloBrowseHeader(cornerRadius: CGFloat = 12) -> some View {
        background(.quaternary, in: RoundedRectangle(cornerRadius: cornerRadius))
    }
}

#if VELO_LIQUID_GLASS
@available(macOS 26, *)
private struct VeloGlassCapsuleModifier: ViewModifier {
    let interactive: Bool

    func body(content: Content) -> some View {
        if interactive {
            content.glassEffect(.regular.interactive(), in: Capsule())
        } else {
            content.glassEffect(.regular, in: Capsule())
        }
    }
}

@available(macOS 26, *)
private struct VeloGlassRoundedRectModifier: ViewModifier {
    let cornerRadius: CGFloat
    let interactive: Bool

    func body(content: Content) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius)
        if interactive {
            content.glassEffect(.regular.interactive(), in: shape)
        } else {
            content.glassEffect(.regular, in: shape)
        }
    }
}
#endif

/// Browse-mode section — plain `.quaternary` surfaces only (guide §0: no glass on content).
public struct VeloBrowseSection<Content: View>: View {
    let title: String
    @ViewBuilder private var content: () -> Content

    public init(_ title: String, @ViewBuilder content: @escaping () -> Content) {
        self.title = title
        self.content = content
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.subheadline.weight(.semibold))
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(.quaternary, in: RoundedRectangle(cornerRadius: 10))

            content()
                .padding(10)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(.quaternary, in: RoundedRectangle(cornerRadius: 10))
        }
    }
}

/// Section with glass header bar and solid `.quaternary` content body.
public struct VeloGlassSection<Content: View>: View {
    let title: String
    @ViewBuilder private var content: () -> Content

    public init(_ title: String, @ViewBuilder content: @escaping () -> Content) {
        self.title = title
        self.content = content
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.subheadline.weight(.semibold))
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity, alignment: .leading)
                .veloGlassRoundedRect(cornerRadius: 10)

            content()
                .padding(10)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(.quaternary, in: RoundedRectangle(cornerRadius: 10))
        }
    }
}

/// Capsule badge for publish status (solid background, not glass).
public struct VeloPublishBadge: View {
    let status: PublishStatus

    public init(status: PublishStatus) {
        self.status = status
    }

    public var body: some View {
        Text(RideSummaryFormatting.publishBadgeTitle(for: status))
            .font(.caption2.weight(.medium))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(badgeColor.opacity(0.22))
            .foregroundStyle(badgeColor)
            .clipShape(Capsule())
    }

    private var badgeColor: Color {
        switch status {
        case .local: return .secondary
        case .strava: return .green
        case .failed: return .red
        }
    }
}

public struct VeloGlassContainer<Content: View>: View {
    private let spacing: CGFloat
    @ViewBuilder private var content: () -> Content

    public init(spacing: CGFloat = 12, @ViewBuilder content: @escaping () -> Content) {
        self.spacing = spacing
        self.content = content
    }

    public var body: some View {
        #if VELO_LIQUID_GLASS
        if #available(macOS 26, *) {
            GlassEffectContainer(spacing: spacing, content: content)
        } else {
            content()
        }
        #else
        content()
        #endif
    }
}

/// HUD region glass grouping (alias per guide §4).
public typealias VeloHUDGlassContainer = VeloGlassContainer

@ViewBuilder
public func veloGlassProminentButton(_ title: String, systemImage: String, action: @escaping () -> Void) -> some View {
    #if VELO_LIQUID_GLASS
    if #available(macOS 26, *) {
        Button(title, systemImage: systemImage, action: action)
            .buttonStyle(.glassProminent)
    } else {
        Button(title, systemImage: systemImage, action: action)
            .buttonStyle(VeloGlassPrimaryButtonStyle())
    }
    #else
    Button(title, systemImage: systemImage, action: action)
        .buttonStyle(VeloGlassPrimaryButtonStyle())
    #endif
}

@ViewBuilder
public func veloGlassButton(_ title: String, systemImage: String, action: @escaping () -> Void) -> some View {
    #if VELO_LIQUID_GLASS
    if #available(macOS 26, *) {
        Button(title, systemImage: systemImage, action: action)
            .buttonStyle(.glass)
    } else {
        Button(title, systemImage: systemImage, action: action)
            .buttonStyle(VeloGlassSecondaryButtonStyle())
    }
    #else
    Button(title, systemImage: systemImage, action: action)
        .buttonStyle(VeloGlassSecondaryButtonStyle())
    #endif
}

public struct VeloGlassPrimaryButtonStyle: ButtonStyle {
    public init() {}

    public func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.body.weight(.semibold))
            .padding(.horizontal, 14)
            .padding(.vertical, 8)
            .foregroundStyle(.primary)
            .opacity(configuration.isPressed ? 0.75 : 1)
            .veloGlassCapsule(interactive: true)
    }
}

public struct VeloGlassSecondaryButtonStyle: ButtonStyle {
    public init() {}

    public func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.body)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .foregroundStyle(.secondary)
            .opacity(configuration.isPressed ? 0.75 : 1)
            .veloGlassCapsule(interactive: true)
    }
}
