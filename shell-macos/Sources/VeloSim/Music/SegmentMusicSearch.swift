import VeloFFI

/// Maps workout segment energy to Apple Music catalog search terms (no network).
public enum SegmentMusicSearch {
    public static func searchTerm(for energy: SegmentEnergyDto) -> String {
        switch energy {
        case .warmup: return "warm up cycling"
        case .build: return "workout build upbeat"
        case .threshold: return "high energy cycling intense"
        case .recovery: return "recovery chill ambient"
        case .cooldown: return "cool down ambient"
        }
    }

    public static func energyLabel(for energy: SegmentEnergyDto) -> String {
        switch energy {
        case .warmup: return "Warmup"
        case .build: return "Build"
        case .threshold: return "Threshold"
        case .recovery: return "Recovery"
        case .cooldown: return "Cooldown"
        }
    }
}
