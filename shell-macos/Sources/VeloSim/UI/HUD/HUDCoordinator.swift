import Foundation
import VeloFFI

/// Drains core telemetry into `HUDModel` at ~8 Hz (125 ms) per guide §5.3.
@MainActor
public final class HUDCoordinator {
    public let model: HUDModel

    private var lastUpdate: CFAbsoluteTime = 0
    private let minInterval: TimeInterval = 0.125
    private let onThrottledUpdate: (() -> Void)?

    /// Raw power samples from the last ~3 s. The HUD never shows instantaneous
    /// watts (hud-design skill §3); smoothing lives here so every view agrees.
    private var powerSamples: [(t: CFAbsoluteTime, w: Double)] = []
    private let smoothingWindowS: TimeInterval = 3.0

    /// Transient-event triggers are model-side (hud-design §3b).
    private var lastIntervalName: String?
    private var lastLapCount: Int = 0
    private let transientHoldS: TimeInterval = 2.0

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
        minimalMode: Bool,
        hudMetrics: HudMetricsDto? = nil,
        ergBiasPct: Double = 100.0
    ) {
        let now = CFAbsoluteTimeGetCurrent()

        // Accumulate every tick (pre-throttle) so the 3 s average is dense.
        if let watts = rideState.powerW {
            powerSamples.append((now, watts))
        }
        powerSamples.removeAll { now - $0.t > smoothingWindowS }

        guard now - lastUpdate >= minInterval else { return }
        lastUpdate = now

        let smoothedW = powerSamples.isEmpty
            ? (rideState.powerW ?? 0)
            : powerSamples.reduce(0) { $0 + $1.w } / Double(powerSamples.count)

        model.minimalMode = minimalMode
        model.ftp = max(1, Int(ftp.rounded()))
        model.power = Int(smoothedW.rounded())
        model.cadence = Int((rideState.cadenceRpm ?? 0).rounded())
        model.heartRate = Int((rideState.heartRateBpm ?? 0).rounded())
        model.speedMps = rideState.speedMps
        model.distanceM = rideState.distanceM
        model.gradient = rideState.grade
        model.elapsedS = rideState.elapsedS
        model.elevationM = rideState.elevationM

        if riderWeightKg > 0, rideState.powerW != nil {
            model.wattsPerKg = ((smoothedW / riderWeightKg) * 10).rounded() / 10
        } else {
            model.wattsPerKg = 0
        }

        model.workout = HUDModel.mapWorkoutHUD(live: workoutLive, actualWatts: model.power)
        if let metrics = hudMetrics {
            model.rollingPower = metrics.rollingPowerSeries
            model.lapCount = Int(metrics.lapCount)
            model.currentLapElapsedS = metrics.currentLapElapsedS
        }
        model.ergBiasPct = ergBiasPct

        updateTransientEvents(workoutLive: workoutLive, now: now)
        onThrottledUpdate?()
    }

    /// Raise interval-change/lap announcements and expire the current one
    /// after its hold window (hud-design §3b — no view-side timers).
    private func updateTransientEvents(workoutLive: WorkoutLiveDto, now: CFAbsoluteTime) {
        let intervalName: String? =
            workoutLive.active && !workoutLive.finished ? workoutLive.intervalName : nil
        if let name = intervalName, lastIntervalName != nil, name != lastIntervalName {
            model.transientEvent = TransientHUDEvent(
                title: name,
                detail: workoutLive.targetWatts.map { "\(Int($0.rounded())) W" },
                raisedAt: now
            )
        }
        lastIntervalName = intervalName

        if model.lapCount > lastLapCount {
            model.transientEvent = TransientHUDEvent(
                title: "Lap \(model.lapCount + 1)",
                detail: nil,
                raisedAt: now
            )
        }
        lastLapCount = model.lapCount

        if let event = model.transientEvent, now - event.raisedAt > transientHoldS {
            model.transientEvent = nil
        }
    }

    public func reset() {
        lastUpdate = 0
        powerSamples = []
        lastIntervalName = nil
        lastLapCount = 0
        model.transientEvent = nil
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
        model.rollingPower = []
        model.lapCount = 0
        model.currentLapElapsedS = 0
        model.ergBiasPct = 100.0
        // Route context too — a later route-less ride must not show the
        // previous route's elevation profile.
        model.elevationProfile = []
        model.routeTotalM = 0
    }
}
