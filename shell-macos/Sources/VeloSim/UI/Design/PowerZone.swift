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

    public var color: Color {
        switch self {
        case .z1: .gray
        case .z2: .blue
        case .z3: .green
        case .z4: .yellow
        case .z5: .orange
        case .z6: .red
        case .z7: .purple
        }
    }
}
