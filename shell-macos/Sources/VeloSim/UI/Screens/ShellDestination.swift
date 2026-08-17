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
    public enum Severity {
        case ready
        case warning
        case blocked
    }

    /// One row of the pre-ride readiness banner.
    public struct Check: Identifiable {
        public let id: String
        public let label: String
        public let detail: String
        public let severity: Severity

        public init(id: String, label: String, detail: String, severity: Severity) {
            self.id = id
            self.label = label
            self.detail = detail
            self.severity = severity
        }
    }

    /// Readiness checklist shown above the Start button: trainer, music, tiles.
    /// Warnings inform but never block; only a `.blocked` row disables the start.
    public static func checks(
        sensorIsBluetooth: Bool,
        trainerReady: Bool,
        segmentMusicEnabled: Bool,
        musicAuthorized: Bool,
        tiles3dEnabled: Bool,
        tilesKeysConfigured: Bool,
        tilesLastError: String?
    ) -> [Check] {
        var out: [Check] = []

        if sensorIsBluetooth {
            out.append(trainerReady
                ? Check(
                    id: "trainer", label: "Trainer",
                    detail: "Bluetooth trainer connected", severity: .ready)
                : Check(
                    id: "trainer", label: "Trainer",
                    detail: "Not connected — pair a trainer or switch input",
                    severity: .warning))
        } else {
            out.append(Check(
                id: "trainer", label: "Trainer",
                detail: "Simulated input active", severity: .ready))
        }

        if segmentMusicEnabled {
            out.append(musicAuthorized
                ? Check(
                    id: "music", label: "Music",
                    detail: "Segment music ready", severity: .ready)
                : Check(
                    id: "music", label: "Music",
                    detail: "Segment music on but Apple Music not authorized",
                    severity: .warning))
        }

        if tiles3dEnabled {
            if !tilesKeysConfigured {
                out.append(Check(
                    id: "tiles", label: "3D Tiles",
                    detail: "No API keys — add keys in Settings or disable tiles",
                    severity: .blocked))
            } else if let err = tilesLastError, !err.isEmpty {
                out.append(Check(
                    id: "tiles", label: "3D Tiles",
                    detail: "Provider error — synthetic terrain will be used",
                    severity: .warning))
            } else {
                out.append(Check(
                    id: "tiles", label: "3D Tiles",
                    detail: "Photorealistic tiles ready", severity: .ready))
            }
        }

        return out
    }

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
