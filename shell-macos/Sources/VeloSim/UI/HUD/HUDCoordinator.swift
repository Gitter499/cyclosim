import Foundation
import VeloFFI

/// Drains core telemetry into `HUDModel` at ~8 Hz (125 ms) per guide §5.3.
@MainActor
public final class HUDCoordinator {
    public let model: HUDModel

    private var lastUpdate: CFAbsoluteTime = 0
    private let minInterval: TimeInterval = 0.125
    private let onThrottledUpdate: (() -> Void)?

    public init(model: HUDModel, onThrottledUpdate: (() -> Void)? = nil) {
        self.model = model
        self.onThrottledUpdate = onThrottledUpdate
    }

    /// Call from the sim loop after each tick; writes the model only when the interval elapses.
    public func ingest(
        rideState: RideStateDto,
        workoutLive: WorkoutLiveDto,
        ftp: Double,
        riderWeightKg: Double,
        minimalMode: Bool
    ) {
        let now = CFAbsoluteTimeGetCurrent()
        guard now - lastUpdate >= minInterval else { return }
        lastUpdate = now

        model.minimalMode = minimalMode
        model.ftp = max(1, Int(ftp.rounded()))
        model.power = Int((rideState.powerW ?? 0).rounded())
        model.cadence = Int((rideState.cadenceRpm ?? 0).rounded())
        model.heartRate = Int((rideState.heartRateBpm ?? 0).rounded())
        model.speedMps = rideState.speedMps
        model.distanceM = rideState.distanceM
        model.gradient = rideState.grade
        model.elapsedS = rideState.elapsedS
        model.elevationM = rideState.elevationM

        if riderWeightKg > 0, let watts = rideState.powerW {
            model.wattsPerKg = ((watts / riderWeightKg) * 10).rounded() / 10
        } else {
            model.wattsPerKg = 0
        }

        model.workout = HUDModel.mapWorkoutHUD(live: workoutLive, actualWatts: model.power)
        onThrottledUpdate?()
    }

    public func reset() {
        lastUpdate = 0
        model.power = 0
        model.cadence = 0
        model.heartRate = 0
        model.speedMps = 0
        model.distanceM = 0
        model.gradient = 0
        model.wattsPerKg = 0
        model.elapsedS = 0
        model.elevationM = nil
        model.workout = nil
    }
}
