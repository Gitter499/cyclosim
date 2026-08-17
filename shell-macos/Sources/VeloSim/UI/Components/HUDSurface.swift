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
    @ViewBuilder
    public func hudPowerSurface(
        zone: PowerZone,
        reduceTransparency: Bool,
        reduceMotion: Bool
    ) -> some View {
        let shape = RoundedRectangle(cornerRadius: Tok.rCard)
        if reduceTransparency {
            background(zone.color.opacity(0.35), in: shape)
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
                background(zone.color.opacity(0.35), in: shape)
            }
            #else
            background(zone.color.opacity(0.35), in: shape)
            #endif
        }
    }
}
