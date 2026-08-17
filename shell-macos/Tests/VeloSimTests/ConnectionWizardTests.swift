import XCTest
import VeloSimSupport

final class ConnectionWizardTests: XCTestCase {
    func testStepAdvanceAndRetreat() {
        XCTAssertEqual(ConnectionWizardStep.advance(from: .intro), .action)
        XCTAssertEqual(ConnectionWizardStep.advance(from: .done), .done)
        XCTAssertEqual(ConnectionWizardStep.retreat(from: .test), .action)
        XCTAssertEqual(ConnectionWizardStep.retreat(from: .intro), .intro)
    }

    func testAllStepsHaveTitles() {
        for step in ConnectionWizardStep.allCases {
            XCTAssertFalse(step.title.isEmpty)
        }
    }
}
