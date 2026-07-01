import Foundation
import VeloFFI

/// Pure formatting helpers for the in-ride Swift HUD overlay (unit-tested).
public enum RideHUDFormatting {
    public static func formatPower(_ watts: Double?) -> String {
        guard let watts else { return "—" }
        return "\(Int(watts.rounded())) W"
    }

    public static func formatCadence(_ rpm: Double?) -> String {
        guard let rpm else { return "—" }
        return "\(Int(rpm.rounded())) rpm"
    }

    public static func formatHeartRate(_ bpm: Double?) -> String {
        guard let bpm else { return "—" }
        return "\(Int(bpm.rounded())) bpm"
    }

    public static func formatSpeedKmh(_ mps: Double) -> String {
        String(format: "%.1f km/h", mps * 3.6)
    }

    public static func formatDistance(_ meters: Double) -> String {
        if meters >= 1000 {
            return String(format: "%.2f km", meters / 1000)
        }
        return String(format: "%.0f m", meters)
    }

    public static func formatElapsed(_ seconds: Double) -> String {
        RideSummaryFormatting.formatElapsed(seconds)
    }

    public static func workoutBanner(live: WorkoutLiveDto) -> String? {
        guard live.active, !live.finished else { return nil }
        let target: String
        if let watts = live.targetWatts {
            target = "\(Int(watts.rounded())) W"
        } else {
            target = "Free"
        }
        return "\(live.intervalName) · \(target)"
    }

    public static func steeringHint(mode: SteeringInputMode, routeLoaded: Bool) -> String? {
        guard mode != .off, routeLoaded else { return nil }
        switch mode {
        case .keyboard: return "← → steer · Space recenter"
        case .airpods: return "Head yaw steer"
        case .off: return nil
        }
    }
}
