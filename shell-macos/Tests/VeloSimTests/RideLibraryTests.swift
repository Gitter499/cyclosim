import XCTest
import VeloFFI
import VeloSimSupport

final class RideLibraryTests: XCTestCase {

    func testListRidesEmptyByDefault() throws {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }

        let handle = VeloHandle()
        try handle.configureRideLibrary(
            dbPath: dir.appendingPathComponent("rides.db").path,
            artifactsBase: dir.appendingPathComponent("artifacts").path
        )

        let rides = try handle.listRides()
        XCTAssertTrue(rides.isEmpty)
    }

    func testFinishRideAppearsInLibrary() throws {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }

        let handle = VeloHandle()
        try handle.configureRideLibrary(
            dbPath: dir.appendingPathComponent("rides.db").path,
            artifactsBase: dir.appendingPathComponent("artifacts").path
        )

        handle.startRide()
        let sensors = FakeSensorSource()
        let trainer = MockTrainerControl()
        for _ in 0..<6 {
            handle.tick(sensors: sensors, trainer: trainer, steering: NoopSteeringInput())
        }

        let publisher = LocalOnlyPublisher()
        let result = try handle.finishRideAndPublish(
            media: PassthroughMedia(),
            publisher: publisher
        )

        XCTAssertFalse(result.rideId.isEmpty)
        let rides = try handle.listRides()
        XCTAssertEqual(rides.count, 1)
        XCTAssertEqual(rides[0].id, result.rideId)
        XCTAssertEqual(rides[0].publishStatus, .local)

        let fetched = try handle.getRide(id: result.rideId)
        XCTAssertNotNil(fetched)
        XCTAssertGreaterThan(fetched!.distanceM, 0)
    }

    func testDeleteRideRemovesFromLibrary() throws {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }

        let handle = VeloHandle()
        try handle.configureRideLibrary(
            dbPath: dir.appendingPathComponent("rides.db").path,
            artifactsBase: dir.appendingPathComponent("artifacts").path
        )

        handle.startRide()
        let sensors = FakeSensorSource()
        let trainer = MockTrainerControl()
        for _ in 0..<4 {
            handle.tick(sensors: sensors, trainer: trainer, steering: NoopSteeringInput())
        }

        let result = try handle.finishRideAndPublish(
            media: PassthroughMedia(),
            publisher: LocalOnlyPublisher()
        )

        let deleted = try handle.deleteRide(id: result.rideId)
        XCTAssertTrue(deleted)
        XCTAssertNil(try handle.getRide(id: result.rideId))
    }
}

private final class LocalOnlyPublisher: ActivityPublisherCallback, @unchecked Sendable {
    func publishRide(
        fitBytes: Data,
        screenshotPng: Data?,
        summary: RideSummaryDto
    ) -> PublishResultDto {
        PublishResultDto(activityUrl: "", savedLocally: true, rideId: "", highlightClipPath: nil)
    }
}

private final class PassthroughMedia: MediaCaptureCallback, @unchecked Sendable {
    func encodePngRgba(width: UInt32, height: UInt32, rgbaPixels: Data) -> Data {
        Data([0x89, 0x50, 0x4E, 0x47])
    }

    func encodeHighlightClip(clips: [HighlightClipRequestDto], outputPath: String) -> Bool {
        false
    }
}
