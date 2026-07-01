import Foundation

/// Root tabs — Home → Activities → Ride → Settings (Zwift/MyWhoosh parity v1).
enum AppTab: String, CaseIterable, Identifiable {
    case home
    case activities
    case ride
    case settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .home: return "Home"
        case .activities: return "Activities"
        case .ride: return "Ride"
        case .settings: return "Settings"
        }
    }

    var systemImage: String {
        switch self {
        case .home: return "house.fill"
        case .activities: return "figure.outdoor.cycle"
        case .ride: return "play.rectangle.fill"
        case .settings: return "gearshape.fill"
        }
    }
}

enum ActivitiesTab: String, CaseIterable, Identifiable {
    case routes
    case workouts
    case history

    var id: String { rawValue }

    var title: String {
        switch self {
        case .routes: return "Routes"
        case .workouts: return "Workouts"
        case .history: return "History"
        }
    }
}
