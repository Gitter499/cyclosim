import XCTest
@testable import VeloSimSupport

final class AppSettingsStoreTests: XCTestCase {
    func testDefaultSteeringModeRoundTrip() {
        let prior = AppSettingsStore.defaultSteeringMode
        defer { AppSettingsStore.defaultSteeringMode = prior }
        AppSettingsStore.defaultSteeringMode = .keyboard
        XCTAssertEqual(AppSettingsStore.defaultSteeringMode, .keyboard)
    }

    func testSegmentMusicEnabledRoundTrip() {
        let prior = AppSettingsStore.segmentMusicEnabled
        defer { AppSettingsStore.segmentMusicEnabled = prior }
        AppSettingsStore.segmentMusicEnabled = true
        XCTAssertTrue(AppSettingsStore.segmentMusicEnabled)
    }
}
