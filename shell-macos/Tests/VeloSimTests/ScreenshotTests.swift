import XCTest
import VeloSimSupport

final class ScreenshotTests: XCTestCase {

    func testPngRoundTripPreservesDimensions() throws {
        let width = 4
        let height = 2
        var rgba = Data()
        for y in 0..<height {
            for x in 0..<width {
                rgba.append(UInt8(x * 60))
                rgba.append(UInt8(y * 60))
                rgba.append(128)
                rgba.append(255)
            }
        }
        let png = try PngEncoder.encode(width: width, height: height, rgba: rgba)
        XCTAssertEqual(png.prefix(8), Data(PngEncoder.pngMagic))
        let decoded = try PngEncoder.decode(png: png)
        XCTAssertEqual(decoded.width, width)
        XCTAssertEqual(decoded.height, height)
        XCTAssertEqual(decoded.rgba.count, width * height * 4)
    }

    func testInvalidByteCountThrows() {
        XCTAssertThrowsError(try PngEncoder.encode(width: 2, height: 2, rgba: Data([1, 2, 3]))) { error in
            XCTAssertEqual(error as? PngEncoderError, .byteCountMismatch(expected: 16, actual: 3))
        }
    }

    func testMediaCaptureCallbackProducesPngMagic() {
        let capture = VeloMediaCapture()
        var rgba = Data(repeating: 0, count: 16)
        rgba[0] = 255
        rgba[3] = 255
        rgba[7] = 255
        rgba[11] = 255
        rgba[15] = 255
        let png = capture.encodePngRgba(width: 2, height: 2, rgbaPixels: rgba)
        XCTAssertGreaterThan(png.count, 8)
        XCTAssertEqual(png.prefix(8), Data(PngEncoder.pngMagic))
    }
}
