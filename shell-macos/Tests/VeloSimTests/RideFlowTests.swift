import XCTest
import VeloFFI
import VeloSimSupport

final class RideFlowTests: XCTestCase {

    func testStartStopRideState() {
        let handle = VeloHandle()
        XCTAssertFalse(handle.isRideRecording())
        handle.startRide()
        XCTAssertTrue(handle.isRideRecording())
        let sensors = FakeSensorSource()
        let trainer = MockTrainerControl()
        let steering = NoopSteeringInput()
        for _ in 0..<10 {
            handle.tick(sensors: sensors, trainer: trainer, steering: steering)
        }
        let summary = handle.stopRide()
        XCTAssertNotNil(summary)
        XCTAssertEqual(summary?.sampleCount, 10)
        XCTAssertFalse(handle.isRideRecording())
    }

    func testExportFitAfterRide() throws {
        let handle = VeloHandle()
        handle.startRide()
        let sensors = FakeSensorSource()
        let trainer = MockTrainerControl()
        let steering = NoopSteeringInput()
        for _ in 0..<5 {
            handle.tick(sensors: sensors, trainer: trainer, steering: steering)
        }
        _ = handle.stopRide()
        let fit = try handle.exportFit()
        XCTAssertGreaterThan(fit.count, 50)
        XCTAssertEqual(fit.prefix(4), Data([0x0E, 0x10, 0x5E, 0x01]))
    }

    func testDoubleStartIsIdempotent() {
        let handle = VeloHandle()
        handle.startRide()
        handle.startRide()
        XCTAssertTrue(handle.isRideRecording())
    }

    func testPublishFallsBackLocallyWithoutStrava() {
        let publisher = VeloActivityPublisher(config: StravaConfig(clientId: "", clientSecret: "", redirectURI: ""))
        let summary = RideSummaryDto(
            elapsedS: 10,
            distanceM: 80,
            sampleCount: 5,
            avgPowerW: 180,
            maxPowerW: 200,
            startedAtUnix: 1_700_000_000,
            highlightClips: []
        )
        let result = publisher.publishRide(
            fitBytes: Data([0x0E, 0x10, 0x5E, 0x01]),
            screenshotPng: nil,
            summary: summary
        )
        XCTAssertTrue(result.savedLocally)
    }
}
