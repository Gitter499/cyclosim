import Foundation
import Observation
import VeloFFI

/// Workout-mode HUD slice (§6.2).
public struct WorkoutHUD: Equatable {
    public let targetWatts: Int
    public let actualWatts: Int
    public let intervalRemainingS: Double
    public let blockName: String
    public let nextBlockName: String?

    public init(
        targetWatts: Int,
        actualWatts: Int,
        intervalRemainingS: Double,
        blockName: String,
        nextBlockName: String?
    ) {
        self.targetWatts = targetWatts
        self.actualWatts = actualWatts
        self.intervalRemainingS = intervalRemainingS
        self.blockName = blockName
        self.nextBlockName = nextBlockName
    }
}

/// Throttled in-ride readout model (~8 Hz) per guide §5.3.
@Observable
@MainActor
public final class HUDModel {
    public var power = 0
    public var cadence = 0
    public var heartRate = 0
    public var ftp = 200
    public var speedMps = 0.0
    public var distanceM = 0.0
    public var gradient = 0.0
    public var wattsPerKg = 0.0
    public var elapsedS = 0.0
    public var elevationM: Double?
    public var workout: WorkoutHUD?
    public var minimalMode = false

    public var distanceKm: Double { distanceM / 1000.0 }
    public var speedKph: Double { speedMps * 3.6 }
    public var gradientPercent: Double { gradient * 100.0 }

    public init() {}

    public static func mapWorkoutHUD(live: WorkoutLiveDto, actualWatts: Int) -> WorkoutHUD? {
        guard live.active, !live.finished else { return nil }
        let target = Int((live.targetWatts ?? 0).rounded())
        let remainingS = max(0, live.intervalDurationS - live.intervalElapsedS)
        return WorkoutHUD(
            targetWatts: target,
            actualWatts: actualWatts,
            intervalRemainingS: remainingS,
            blockName: live.intervalName,
            nextBlockName: nil
        )
    }
}

public enum HUDDurationFormat {
    public static func hms(seconds: Double) -> String {
        RideSummaryFormatting.formatElapsed(seconds)
    }

    public static func mmss(seconds: Double) -> String {
        let total = max(0, Int(seconds.rounded()))
        let m = total / 60
        let s = total % 60
        return String(format: "%d:%02d", m, s)
    }
}
