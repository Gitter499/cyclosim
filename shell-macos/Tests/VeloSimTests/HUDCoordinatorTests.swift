import XCTest
import VeloFFI
import VeloSimSupport

final class HUDCoordinatorTests: XCTestCase {

    private func rideState(powerW: Double?) -> RideStateDto {
        RideStateDto(
            mode: .free,
            distanceM: 1000,
            speedMps: 8,
            elapsedS: 60,
            grade: 0.02,
            elevationM: 100,
            powerW: powerW,
            cadenceRpm: 90,
            heartRateBpm: 140,
            steerAxis: 0,
            steerYawRad: 0
        )
    }

    private func idleWorkout() -> WorkoutLiveDto {
        WorkoutLiveDto(
            active: false,
            workoutName: "",
            intervalName: "",
            intervalElapsedS: 0,
            intervalDurationS: 0,
            workoutElapsedS: 0,
            targetWatts: nil,
            nextIntervalName: nil,
            finished: false
        )
    }

    /// The HUD shows a 3 s rolling average, never raw watts (hud-design §3).
    @MainActor
    func testPowerIsSmoothedAcrossRecentSamples() {
        let model = HUDModel()
        let coordinator = HUDCoordinator(model: model)

        coordinator.ingest(
            rideState: rideState(powerW: 100),
            workoutLive: idleWorkout(),
            ftp: 250,
            riderWeightKg: 0,
            minimalMode: false
        )
        // Cross the 125 ms throttle so the second ingest writes the model;
        // both samples are inside the 3 s smoothing window.
        Thread.sleep(forTimeInterval: 0.15)
        coordinator.ingest(
            rideState: rideState(powerW: 300),
            workoutLive: idleWorkout(),
            ftp: 250,
            riderWeightKg: 0,
            minimalMode: false
        )

        XCTAssertEqual(model.power, 200, "expected mean of [100, 300], not raw 300")
    }

    /// Interval changes raise a transient banner event; the first sighting
    /// of an interval (ride start) does not (hud-design §3b).
    @MainActor
    func testIntervalChangeRaisesTransientEvent() {
        let model = HUDModel()
        let coordinator = HUDCoordinator(model: model)
        func live(_ name: String) -> WorkoutLiveDto {
            WorkoutLiveDto(
                active: true,
                workoutName: "2x20",
                intervalName: name,
                intervalElapsedS: 0,
                intervalDurationS: 300,
                workoutElapsedS: 0,
                targetWatts: 250,
                nextIntervalName: nil,
                finished: false
            )
        }

        coordinator.ingest(
            rideState: rideState(powerW: 200), workoutLive: live("Warmup"),
            ftp: 250, riderWeightKg: 0, minimalMode: false
        )
        XCTAssertNil(model.transientEvent, "first interval sighting is not a change")

        Thread.sleep(forTimeInterval: 0.15)
        coordinator.ingest(
            rideState: rideState(powerW: 200), workoutLive: live("Threshold 1"),
            ftp: 250, riderWeightKg: 0, minimalMode: false
        )
        XCTAssertEqual(model.transientEvent?.title, "Threshold 1")
        XCTAssertEqual(model.transientEvent?.detail, "250 W")
    }

    @MainActor
    func testWorkoutHUDProgressFraction() {
        let live = WorkoutLiveDto(
            active: true,
            workoutName: "2x20",
            intervalName: "Threshold 1",
            intervalElapsedS: 30,
            intervalDurationS: 120,
            workoutElapsedS: 30,
            targetWatts: 250,
            nextIntervalName: nil,
            finished: false
        )
        let hud = HUDModel.mapWorkoutHUD(live: live, actualWatts: 245)
        XCTAssertEqual(hud?.intervalProgress ?? -1, 0.25, accuracy: 1e-9)
        XCTAssertEqual(hud?.intervalRemainingS ?? -1, 90, accuracy: 1e-9)
    }

    @MainActor
    func testThrottlesUpdatesToRoughly8Hz() {
        let model = HUDModel()
        var updateCount = 0
        let coordinator = HUDCoordinator(model: model) {
            updateCount += 1
        }

        let ride = RideStateDto(
            mode: .free,
            distanceM: 1000,
            speedMps: 8,
            elapsedS: 60,
            grade: 0.02,
            elevationM: 100,
            powerW: 200,
            cadenceRpm: 90,
            heartRateBpm: 140,
            steerAxis: 0,
            steerYawRad: 0
        )
        let live = WorkoutLiveDto(
            active: false,
            workoutName: "",
            intervalName: "",
            intervalElapsedS: 0,
            intervalDurationS: 0,
            workoutElapsedS: 0,
            targetWatts: nil,
            nextIntervalName: nil,
            finished: false
        )

        for _ in 0 ..< 30 {
            coordinator.ingest(
                rideState: ride,
                workoutLive: live,
                ftp: 250,
                riderWeightKg: 75,
                minimalMode: false
            )
        }

        XCTAssertEqual(updateCount, 1)
        XCTAssertEqual(model.power, 200)
    }
}
