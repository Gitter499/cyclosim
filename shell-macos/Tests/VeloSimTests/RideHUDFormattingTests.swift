import XCTest
import VeloFFI
import VeloSimSupport

final class RideHUDFormattingTests: XCTestCase {

    func testFormatPowerMissing() {
        XCTAssertEqual(RideHUDFormatting.formatPower(nil), "—")
        XCTAssertEqual(RideHUDFormatting.formatPower(200.4), "200 W")
    }

    func testFormatElevationAndGrade() {
        XCTAssertEqual(RideHUDFormatting.formatElevation(nil), "—")
        XCTAssertEqual(RideHUDFormatting.formatElevation(842.6), "843 m")
        XCTAssertEqual(RideHUDFormatting.formatGradePercent(0.052), "5.2%")
    }

    func testIntervalFractionAndRemaining() {
        XCTAssertNil(RideHUDFormatting.intervalFraction(durationS: 0, elapsedS: 30))
        let fraction = RideHUDFormatting.intervalFraction(durationS: 120, elapsedS: 30)
        let remaining = RideHUDFormatting.intervalRemainingS(durationS: 120, elapsedS: 30)
        XCTAssertEqual(fraction, 0.25, accuracy: 0.0001)
        XCTAssertEqual(remaining, 90, accuracy: 0.0001)
    }

    func testIntervalBarActiveWorkout() {
        let live = WorkoutLiveDto(
            active: true,
            workoutName: "Threshold",
            intervalName: "Block 1",
            intervalElapsedS: 30,
            intervalDurationS: 120,
            workoutElapsedS: 300,
            targetWatts: 250,
            finished: false
        )
        let bar = RideHUDFormatting.intervalBar(live: live)
        XCTAssertNotNil(bar)
        XCTAssertEqual(bar?.fraction ?? 0, 0.25, accuracy: 0.0001)
        XCTAssertEqual(bar?.remainingS ?? 0, 90, accuracy: 0.0001)
        XCTAssertEqual(bar?.intervalName, "Block 1")
        XCTAssertEqual(bar?.targetLabel, "250 W")
    }

    func testWorkoutBannerActive() {
        let live = WorkoutLiveDto(
            active: true,
            workoutName: "Threshold",
            intervalName: "Block 1",
            intervalElapsedS: 60,
            intervalDurationS: 120,
            workoutElapsedS: 300,
            targetWatts: 250,
            finished: false
        )
        XCTAssertEqual(RideHUDFormatting.workoutBanner(live: live), "Block 1 · 250 W · 1:00")
    }

    func testSteeringHintKeyboardOnRoute() {
        let hint = RideHUDFormatting.steeringHint(mode: .keyboard, routeLoaded: true)
        XCTAssertNotNil(hint)
        XCTAssertTrue(hint?.contains("steer") ?? false)
    }
}
