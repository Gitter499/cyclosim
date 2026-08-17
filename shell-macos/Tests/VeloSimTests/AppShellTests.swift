import XCTest
import VeloSimSupport

final class AppShellTests: XCTestCase {
    func testFourDestinations() {
        XCTAssertEqual(ShellDestination.allCases.count, 4)
        XCTAssertEqual(ShellDestination.allCases.map(\.title), ["Home", "Activities", "History", "Settings"])
    }

    func testDestinationSystemImages() {
        XCTAssertEqual(ShellDestination.home.systemImage, "house")
        XCTAssertEqual(ShellDestination.activities.systemImage, "map")
        XCTAssertEqual(ShellDestination.history.systemImage, "clock")
        XCTAssertEqual(ShellDestination.settings.systemImage, "gearshape")
    }

    func testActivitiesTabExcludesHistory() {
        XCTAssertEqual(ActivitiesTab.allCases.count, 2)
        XCTAssertFalse(ActivitiesTab.allCases.contains(where: { $0.title == "History" }))
    }

    func testPreRideBlockReasonWhenTilesEnabledWithoutKeys() {
        let reason = PreRideValidation.blockReason(
            tiles3dEnabled: true,
            tilesKeysConfigured: false,
            tilesLastError: nil
        )
        XCTAssertNotNil(reason)
        XCTAssertFalse(reason?.isEmpty ?? true)
    }

    func testPreRideBlockReasonNilWhenTilesOff() {
        XCTAssertNil(PreRideValidation.blockReason(
            tiles3dEnabled: false,
            tilesKeysConfigured: false,
            tilesLastError: nil
        ))
    }

    func testPreRideChecksAllReadyOnDefaults() {
        let checks = PreRideValidation.checks(
            sensorIsBluetooth: false,
            trainerReady: false,
            segmentMusicEnabled: false,
            musicAuthorized: false,
            tiles3dEnabled: false,
            tilesKeysConfigured: false,
            tilesLastError: nil
        )
        // Simulated trainer input only; nothing optional enabled.
        XCTAssertEqual(checks.map(\.id), ["trainer"])
        XCTAssertTrue(checks.allSatisfy { $0.severity == .ready })
    }

    func testPreRideChecksWarnOnDisconnectedTrainerAndUnauthorizedMusic() {
        let checks = PreRideValidation.checks(
            sensorIsBluetooth: true,
            trainerReady: false,
            segmentMusicEnabled: true,
            musicAuthorized: false,
            tiles3dEnabled: false,
            tilesKeysConfigured: false,
            tilesLastError: nil
        )
        XCTAssertEqual(checks.map(\.id), ["trainer", "music"])
        XCTAssertTrue(checks.allSatisfy { $0.severity == .warning })
    }

    func testPreRideChecksBlockOnTilesWithoutKeys() {
        let checks = PreRideValidation.checks(
            sensorIsBluetooth: false,
            trainerReady: false,
            segmentMusicEnabled: false,
            musicAuthorized: false,
            tiles3dEnabled: true,
            tilesKeysConfigured: false,
            tilesLastError: nil
        )
        XCTAssertEqual(checks.first(where: { $0.id == "tiles" })?.severity, .blocked)
    }

    func testPreRideChecksTilesProviderErrorIsWarningNotBlock() {
        let checks = PreRideValidation.checks(
            sensorIsBluetooth: false,
            trainerReady: false,
            segmentMusicEnabled: false,
            musicAuthorized: false,
            tiles3dEnabled: true,
            tilesKeysConfigured: true,
            tilesLastError: "quota exceeded"
        )
        XCTAssertEqual(checks.first(where: { $0.id == "tiles" })?.severity, .warning)
    }
}
