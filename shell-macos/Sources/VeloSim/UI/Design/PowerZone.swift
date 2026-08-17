import SwiftUI

/// Coggan 7-zone palette (% of FTP) — the only HUD color-coding per guide §3.
public enum PowerZone: Int, CaseIterable {
    case z1 = 1, z2, z3, z4, z5, z6, z7

    public static func of(watts: Int, ftp: Int) -> PowerZone {
        guard ftp > 0 else { return .z1 }
        switch Double(watts) / Double(ftp) {
        case ..<0.56: return .z1
        case ..<0.76: return .z2
        case ..<0.91: return .z3
        case ..<1.06: return .z4
        case ..<1.21: return .z5
        case ..<1.51: return .z6
        default: return .z7
        }
    }

    /// Canonical VeloSim zone hexes (hud-design skill §4) — shared with the
    /// Rust glyphon HUD's `zone_color` so both HUDs read identically.
    /// Z1 #9AA5B1 · Z2 #3D9BE9 · Z3 #3FBE58 · Z4 #F5C542 · Z5 #F07F2E ·
    /// Z6 #E43F4F · Z7 #B05CE0.
    public var color: Color {
        switch self {
        case .z1: Self.rgb(0x9A, 0xA5, 0xB1)
        case .z2: Self.rgb(0x3D, 0x9B, 0xE9)
        case .z3: Self.rgb(0x3F, 0xBE, 0x58)
        case .z4: Self.rgb(0xF5, 0xC5, 0x42)
        case .z5: Self.rgb(0xF0, 0x7F, 0x2E)
        case .z6: Self.rgb(0xE4, 0x3F, 0x4F)
        case .z7: Self.rgb(0xB0, 0x5C, 0xE0)
        }
    }

    private static func rgb(_ r: Int, _ g: Int, _ b: Int) -> Color {
        Color(red: Double(r) / 255, green: Double(g) / 255, blue: Double(b) / 255)
    }
}
