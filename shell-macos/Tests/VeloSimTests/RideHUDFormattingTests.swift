import XCTest
import VeloFFI
import VeloSimSupport

final class RideHUDFormattingTests: XCTestCase {

    func testFormatPowerMissing() {
        XCTAssertEqual(RideHUDFormatting.formatPower(nil), "—")
        XCTAssertEqual(RideHUDFormatting.formatPower(200.4), "200 W")
    }

    func testWorkoutBannerActive() {
        let live = WorkoutLiveDto(
            active: true,
            workoutName: "Threshold",
            intervalName: "Block 1",
            intervalElapsedS: 60,
            workoutElapsedS: 300,
            targetWatts: 250,
            finished: false
        )
        XCTAssertEqual(RideHUDFormatting.workoutBanner(live: live), "Block 1 · 250 W")
    }

    func testSteeringHintKeyboardOnRoute() {
        let hint = RideHUDFormatting.steeringHint(mode: .keyboard, routeLoaded: true)
        XCTAssertNotNil(hint)
        XCTAssertTrue(hint?.contains("steer") ?? false)
    }
}
