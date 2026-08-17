import XCTest
import VeloSimSupport

final class SteeringInputTests: XCTestCase {
    func testNoopSteeringReturnsZeroAxis() {
        let input = NoopSteeringInput()
        let state = input.poll()
        XCTAssertEqual(state.axis, 0)
        XCTAssertFalse(state.recenter)
    }

    func testSteeringModeLabels() {
        XCTAssertEqual(SteeringInputMode.keyboard.label, "Keyboard")
        XCTAssertEqual(SteeringInputMode.airpods.label, "AirPods")
    }
}
