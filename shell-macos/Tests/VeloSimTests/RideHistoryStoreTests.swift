import XCTest
import VeloFFI
import VeloSimSupport

final class RideHistoryStoreTests: XCTestCase {

    func testLocalRideStoreListsFromLibrary() throws {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }

        let library = try RideLibraryHandle.open(
            dbPath: dir.appendingPathComponent("rides.db").path,
            artifactsBase: dir.appendingPathComponent("artifacts").path
        )
        let store = LocalRideStore.open(library: library)

        XCTAssertTrue(try store.listRides().isEmpty)
    }

    func testRideFolderDerivedFromFitPath() throws {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        let artifacts = dir.appendingPathComponent("artifacts", isDirectory: true)
        try FileManager.default.createDirectory(at: artifacts, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: dir) }

        let rideId = UUID().uuidString
        let rideDir = artifacts.appendingPathComponent(rideId, isDirectory: true)
        try FileManager.default.createDirectory(at: rideDir, withIntermediateDirectories: true)
        let fitPath = rideDir.appendingPathComponent("ride.fit")
        try Data("fit".utf8).write(to: fitPath)

        let handle = try RideLibraryHandle.open(
            dbPath: dir.appendingPathComponent("rides.db").path,
            artifactsBase: artifacts.path
        )

        let record = RideRecordDto(
            id: rideId,
            startedAtUnix: 1_700_000_000,
            elapsedS: 60,
            distanceM: 1000,
            avgPowerW: 180,
            maxPowerW: 220,
            fitPath: fitPath.path,
            screenshotPath: nil,
            stravaActivityId: nil,
            publishStatus: .local,
            routeId: nil
        )

        let store = LocalRideStore.open(library: handle)
        let folder = store.rideFolder(for: record)
        XCTAssertEqual(folder.path, rideDir.path)
    }

    func testStravaURLWhenActivityIdPresent() throws {
        let library = try RideLibraryHandle.open(
            dbPath: FileManager.default.temporaryDirectory
                .appendingPathComponent(UUID().uuidString)
                .appendingPathComponent("rides.db").path,
            artifactsBase: FileManager.default.temporaryDirectory
                .appendingPathComponent(UUID().uuidString)
                .appendingPathComponent("artifacts").path
        )
        let store = LocalRideStore.open(library: library)

        let record = RideRecordDto(
            id: UUID().uuidString,
            startedAtUnix: 1,
            elapsedS: 1,
            distanceM: 1,
            avgPowerW: nil,
            maxPowerW: nil,
            fitPath: "/tmp/x/ride.fit",
            screenshotPath: nil,
            stravaActivityId: "424242",
            publishStatus: .strava,
            routeId: nil
        )

        let url = store.stravaURL(for: record)
        XCTAssertEqual(url?.absoluteString, "https://www.strava.com/activities/424242")
    }

    func testIntegrationWithVeloHandleHistory() throws {
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
        for _ in 0..<3 {
            handle.tick(sensors: FakeSensorSource(), trainer: MockTrainerControl())
        }
        _ = try handle.finishRideAndPublish(
            media: PassthroughMediaCapture(),
            publisher: LocalOnlyPublisherForStore()
        )

        let library = try RideLibraryHandle.open(
            dbPath: dir.appendingPathComponent("rides.db").path,
            artifactsBase: dir.appendingPathComponent("artifacts").path
        )
        let store = LocalRideStore.open(library: library)
        XCTAssertEqual(try store.listRides().count, 1)
    }
}

private final class LocalOnlyPublisherForStore: ActivityPublisherCallback, @unchecked Sendable {
    func publishRide(
        fitBytes: Data,
        screenshotPng: Data?,
        summary: RideSummaryDto
    ) -> PublishResultDto {
        PublishResultDto(activityUrl: "", savedLocally: true, rideId: "")
    }
}

private final class PassthroughMediaCapture: MediaCaptureCallback, @unchecked Sendable {
    func encodePngRgba(width: UInt32, height: UInt32, rgbaPixels: Data) -> Data {
        Data()
    }
}
