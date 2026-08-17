import Foundation
import VeloFFI

/// Pure formatting helpers for the in-ride Swift HUD overlay (unit-tested).
public enum RideHUDFormatting {
    public struct IntervalBar: Equatable {
        public let fraction: Double
        public let remainingS: Double
        public let intervalName: String
        public let targetLabel: String

        public init(fraction: Double, remainingS: Double, intervalName: String, targetLabel: String) {
            self.fraction = fraction
            self.remainingS = remainingS
            self.intervalName = intervalName
            self.targetLabel = targetLabel
        }
    }

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
        RideSummaryFormatting.formatDistance(meters)
    }

    public static func formatElapsed(_ seconds: Double) -> String {
        RideSummaryFormatting.formatElapsed(seconds)
    }

    public static func formatElevation(_ meters: Double?) -> String {
        guard let meters else { return "—" }
        return String(format: "%.0f m", meters)
    }

    public static func formatGradePercent(_ grade: Double) -> String {
        String(format: "%.1f%%", grade * 100.0)
    }

    public static func intervalFraction(durationS: Double, elapsedS: Double) -> Double? {
        guard durationS > 0 else { return nil }
        return (elapsedS / durationS).clamped(to: 0 ... 1)
    }

    public static func intervalRemainingS(durationS: Double, elapsedS: Double) -> Double? {
        guard durationS > 0 else { return nil }
        return max(0, durationS - elapsedS)
    }

    public static func intervalBar(live: WorkoutLiveDto) -> IntervalBar? {
        guard live.active, !live.finished, live.intervalDurationS > 0 else { return nil }
        guard
            let fraction = intervalFraction(durationS: live.intervalDurationS, elapsedS: live.intervalElapsedS),
            let remaining = intervalRemainingS(durationS: live.intervalDurationS, elapsedS: live.intervalElapsedS)
        else {
            return nil
        }
        let target = live.targetWatts.map { "\(Int($0.rounded())) W" } ?? "Free"
        return IntervalBar(
            fraction: fraction,
            remainingS: remaining,
            intervalName: live.intervalName,
            targetLabel: target
        )
    }

    public static func workoutBanner(live: WorkoutLiveDto) -> String? {
        guard live.active, !live.finished else { return nil }
        if let bar = intervalBar(live: live) {
            return "\(bar.intervalName) · \(bar.targetLabel) · \(formatElapsed(bar.remainingS))"
        }
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

private extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}
