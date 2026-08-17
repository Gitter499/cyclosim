import XCTest
import VeloFFI
import VeloSimSupport

final class SegmentMusicSearchTests: XCTestCase {

    func testSearchTermsCoverAllEnergies() {
        XCTAssertEqual(SegmentMusicSearch.searchTerm(for: .warmup), "warm up cycling")
        XCTAssertEqual(SegmentMusicSearch.searchTerm(for: .build), "workout build upbeat")
        XCTAssertEqual(SegmentMusicSearch.searchTerm(for: .threshold), "high energy cycling intense")
        XCTAssertEqual(SegmentMusicSearch.searchTerm(for: .recovery), "recovery chill ambient")
        XCTAssertEqual(SegmentMusicSearch.searchTerm(for: .cooldown), "cool down ambient")
    }

    func testEnergyLabelsAreHumanReadable() {
        XCTAssertEqual(SegmentMusicSearch.energyLabel(for: .warmup), "Warmup")
        XCTAssertEqual(SegmentMusicSearch.energyLabel(for: .threshold), "Threshold")
    }
}
