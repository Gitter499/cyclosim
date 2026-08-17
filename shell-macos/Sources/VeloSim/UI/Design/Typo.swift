import SwiftUI

/// Typography helpers for HUD metrics per `VeloSim-Roadmap.md` Part II §5.
public enum Typo {
    public static func bigMetric() -> Font {
        .system(size: 64, weight: .bold, design: .rounded)
    }

    public static func metric() -> Font {
        .system(size: 30, weight: .semibold, design: .rounded)
    }

    public static func unit() -> Font {
        .system(size: 15, weight: .semibold)
    }

    public static func label() -> Font {
        .system(size: 11, weight: .bold).width(.expanded)
    }
}
