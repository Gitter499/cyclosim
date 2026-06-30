import XCTest
import VeloFFI
import VeloSimSupport

final class RideSummaryFormattingTests: XCTestCase {

    func testFormatDistanceMeters() {
        XCTAssertEqual(RideSummaryFormatting.formatDistance(850), "850 m")
    }

    func testFormatDistanceKilometers() {
        XCTAssertEqual(RideSummaryFormatting.formatDistance(42195), "42.20 km")
    }

    func testFormatElapsedUnderOneHour() {
        XCTAssertEqual(RideSummaryFormatting.formatElapsed(372), "6:12")
    }

    func testFormatElapsedWithHours() {
        XCTAssertEqual(RideSummaryFormatting.formatElapsed(3723), "1:02:03")
    }

    func testFormatPowerOptional() {
        XCTAssertEqual(RideSummaryFormatting.formatPower(210), "210 W")
        XCTAssertEqual(RideSummaryFormatting.formatPower(nil), "—")
    }

    func testPublishStatusLabelLocal() {
        let result = PublishResultDto(activityUrl: "/tmp/ride", savedLocally: true, rideId: "abc", highlightClipPath: nil)
        XCTAssertEqual(RideSummaryFormatting.publishStatusLabel(for: result), "Saved locally")
    }

    func testPublishStatusLabelStrava() {
        let result = PublishResultDto(
            activityUrl: "https://www.strava.com/activities/123",
            savedLocally: false,
            rideId: "",
            highlightClipPath: nil
        )
        XCTAssertEqual(RideSummaryFormatting.publishStatusLabel(for: result), "Published to Strava")
    }

    func testPublishStatusLabelError() {
        let result = PublishResultDto(activityUrl: "error:timeout", savedLocally: true, rideId: "", highlightClipPath: nil)
        XCTAssertEqual(RideSummaryFormatting.publishStatusLabel(for: result), "Publish failed")
    }

    func testPublishBadgeTitles() {
        XCTAssertEqual(RideSummaryFormatting.publishBadgeTitle(for: .local), "Local")
        XCTAssertEqual(RideSummaryFormatting.publishBadgeTitle(for: .strava), "Strava")
        XCTAssertEqual(RideSummaryFormatting.publishBadgeTitle(for: .failed), "Failed")
    }

    func testActivityLinkLabelWeb() {
        let result = PublishResultDto(
            activityUrl: "https://www.strava.com/activities/42",
            savedLocally: false,
            rideId: "",
            highlightClipPath: nil
        )
        XCTAssertEqual(
            RideSummaryFormatting.activityLinkLabel(for: result),
            "https://www.strava.com/activities/42"
        )
    }

    func testIsWebActivityURL() {
        XCTAssertTrue(RideSummaryFormatting.isWebActivityURL("https://strava.com/a/1"))
        XCTAssertFalse(RideSummaryFormatting.isWebActivityURL("/Users/me/ride"))
    }
}
