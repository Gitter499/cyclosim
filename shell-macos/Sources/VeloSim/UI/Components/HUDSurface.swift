import SwiftUI

/// HUD legibility surfaces — real `.glassEffect` on macOS 26+; solid fallback otherwise (guide §8).
extension View {
    @ViewBuilder
    public func hudSurface<S: Shape>(_ shape: S, reduceTransparency: Bool) -> some View {
        #if VELO_LIQUID_GLASS
        if #available(macOS 26, *), !reduceTransparency {
            glassEffect(.regular, in: shape)
        } else {
            background(Color.black.opacity(0.65), in: shape)
        }
        #else
        background(Color.black.opacity(0.65), in: shape)
        #endif
    }

    /// Zone-tinted power card — the only tinted HUD element (§5).
    ///
    /// The zone hue always sits on a dark base so white text keeps contrast
    /// over bright scenes (hud-design skill §5: tint, don't flood).
    @ViewBuilder
    public func hudPowerSurface(
        zone: PowerZone,
        reduceTransparency: Bool,
        reduceMotion: Bool
    ) -> some View {
        let shape = RoundedRectangle(cornerRadius: Tok.rCard)
        if reduceTransparency {
            hudZoneTintedFill(zone: zone, shape: shape)
        } else {
            #if VELO_LIQUID_GLASS
            if #available(macOS 26, *) {
                let tint = zone.color.opacity(0.30)
                if reduceMotion {
                    glassEffect(.regular.tint(tint), in: shape)
                } else {
                    glassEffect(.regular.tint(tint), in: shape)
                        .animation(.snappy, value: zone)
                }
            } else {
                hudZoneTintedFill(zone: zone, shape: shape)
            }
            #else
            hudZoneTintedFill(zone: zone, shape: shape)
            #endif
        }
    }

    /// Zone tint layered over a dark scrim: later `background` renders behind,
    /// so the order is tint-over-dark, both clipped to the card shape.
    private func hudZoneTintedFill(zone: PowerZone, shape: RoundedRectangle) -> some View {
        background(zone.color.opacity(0.35), in: shape)
            .background(Color.black.opacity(0.55), in: shape)
    }
}
