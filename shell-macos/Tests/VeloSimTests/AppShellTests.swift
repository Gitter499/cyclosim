import XCTest
import VeloSimSupport

final class AppShellTests: XCTestCase {
    func testFourDestinations() {
        XCTAssertEqual(ShellDestination.allCases.count, 4)
        XCTAssertEqual(ShellDestination.allCases.map(\.title), ["Home", "Activities", "History", "Settings"])
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
}
