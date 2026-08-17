import XCTest
import VeloFFI
import VeloSimSupport

final class HUDCoordinatorTests: XCTestCase {

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
