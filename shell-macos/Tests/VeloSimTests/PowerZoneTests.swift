import XCTest
import VeloSimSupport

final class PowerZoneTests: XCTestCase {

    func testZoneBoundariesAt200Ftp() {
        XCTAssertEqual(PowerZone.of(watts: 0, ftp: 200), .z1)
        XCTAssertEqual(PowerZone.of(watts: 110, ftp: 200), .z1)
        XCTAssertEqual(PowerZone.of(watts: 111, ftp: 200), .z2)
        XCTAssertEqual(PowerZone.of(watts: 150, ftp: 200), .z2)
        XCTAssertEqual(PowerZone.of(watts: 151, ftp: 200), .z3)
        XCTAssertEqual(PowerZone.of(watts: 180, ftp: 200), .z3)
        XCTAssertEqual(PowerZone.of(watts: 181, ftp: 200), .z4)
        XCTAssertEqual(PowerZone.of(watts: 210, ftp: 200), .z4)
        XCTAssertEqual(PowerZone.of(watts: 211, ftp: 200), .z5)
        XCTAssertEqual(PowerZone.of(watts: 240, ftp: 200), .z5)
        XCTAssertEqual(PowerZone.of(watts: 241, ftp: 200), .z6)
        XCTAssertEqual(PowerZone.of(watts: 300, ftp: 200), .z6)
        XCTAssertEqual(PowerZone.of(watts: 301, ftp: 200), .z7)
    }

    func testZeroFtpFallsBackToZ1() {
        XCTAssertEqual(PowerZone.of(watts: 400, ftp: 0), .z1)
    }
}
