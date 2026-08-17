import Foundation

/// Browse-mode shell destinations (no dedicated Ride tab — ride is a phase).
public enum ShellDestination: String, CaseIterable, Identifiable, Hashable {
    case home
    case activities
    case history
    case settings

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .home: return "Home"
        case .activities: return "Activities"
        case .history: return "History"
        case .settings: return "Settings"
        }
    }

    public var systemImage: String {
        switch self {
        case .home: return "house"
        case .activities: return "map"
        case .history: return "clock"
        case .settings: return "gearshape"
        }
    }
}

public enum ShellPhase: String, Equatable {
    case browse
    case riding
}

/// Routes vs workouts within Activities (history is its own destination).
public enum ActivitiesTab: String, CaseIterable, Identifiable {
    case routes
    case workouts

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .routes: return "Routes"
        case .workouts: return "Workouts"
        }
    }
}

public enum PreRideValidation {
    public static func blockReason(
        tiles3dEnabled: Bool,
        tilesKeysConfigured: Bool,
        tilesLastError: String?
    ) -> String? {
        guard tiles3dEnabled else { return nil }
        if !tilesKeysConfigured {
            return "3D Tiles enabled but no API keys — add keys in Settings or disable tiles."
        }
        return nil
    }
}
