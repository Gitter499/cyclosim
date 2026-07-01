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

    func testHudMinimalModeRoundTrip() {
        let prior = AppSettingsStore.hudMinimalMode
        defer { AppSettingsStore.hudMinimalMode = prior }
        AppSettingsStore.hudMinimalMode = true
        XCTAssertTrue(AppSettingsStore.hudMinimalMode)
        AppSettingsStore.hudMinimalMode = false
        XCTAssertFalse(AppSettingsStore.hudMinimalMode)
    }

    func testPinnedRouteAndWorkoutRoundTrip() {
        let priorRoute = AppSettingsStore.pinnedRouteId
        let priorWorkout = AppSettingsStore.pinnedWorkoutName
        defer {
            AppSettingsStore.pinnedRouteId = priorRoute
            AppSettingsStore.pinnedWorkoutName = priorWorkout
        }

        AppSettingsStore.pinnedRouteId = "alpe-du-zwift"
        AppSettingsStore.pinnedWorkoutName = "2x20 Threshold"
        XCTAssertEqual(AppSettingsStore.pinnedRouteId, "alpe-du-zwift")
        XCTAssertEqual(AppSettingsStore.pinnedWorkoutName, "2x20 Threshold")

        AppSettingsStore.pinnedRouteId = nil
        AppSettingsStore.pinnedWorkoutName = nil
        XCTAssertNil(AppSettingsStore.pinnedRouteId)
        XCTAssertNil(AppSettingsStore.pinnedWorkoutName)
    }
}
