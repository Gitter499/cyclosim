import Foundation
import VeloFFI

/// Pure formatting helpers for ride summary UI (testable without SwiftUI).
public enum RideSummaryFormatting {
    public static func formatDistance(_ meters: Double) -> String {
        if meters >= 1000 {
            return String(format: "%.2f km", meters / 1000)
        }
        return String(format: "%.0f m", meters)
    }

    public static func formatElapsed(_ seconds: Double) -> String {
        let total = max(0, Int(seconds.rounded()))
        let hours = total / 3600
        let minutes = (total % 3600) / 60
        let secs = total % 60
        if hours > 0 {
            return String(format: "%d:%02d:%02d", hours, minutes, secs)
        }
        return String(format: "%d:%02d", minutes, secs)
    }

    public static func formatPower(_ watts: Double?) -> String {
        guard let watts else { return "—" }
        return String(format: "%.0f W", watts)
    }

    public static func formatRideDate(_ unix: UInt64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(unix))
        return date.formatted(date: .abbreviated, time: .shortened)
    }

    public static func publishStatusLabel(for result: PublishResultDto) -> String {
        if result.activityUrl.hasPrefix("error:") {
            return "Publish failed"
        }
        return result.savedLocally ? "Saved locally" : "Published to Strava"
    }

    public static func publishBadgeTitle(for status: PublishStatus) -> String {
        switch status {
        case .local: return "Local"
        case .strava: return "Strava"
        case .failed: return "Failed"
        }
    }

    public static func activityLinkLabel(for result: PublishResultDto) -> String {
        if result.activityUrl.hasPrefix("error:") {
            return String(result.activityUrl.dropFirst("error:".count))
        }
        if result.activityUrl.hasPrefix("http") {
            return result.activityUrl
        }
        if result.activityUrl.isEmpty {
            return LocalRideStore.ridesDirectory.path
        }
        return result.activityUrl
    }

    public static func isWebActivityURL(_ urlString: String) -> Bool {
        urlString.hasPrefix("http://") || urlString.hasPrefix("https://")
    }
}
