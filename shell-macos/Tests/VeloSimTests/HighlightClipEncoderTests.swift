import XCTest
import VeloFFI
import VeloSimSupport

final class HighlightClipEncoderTests: XCTestCase {

    func testEncodesSyntheticFramesToMp4() throws {
        let width = 8
        let height = 8
        var rgba = Data()
        for i in 0..<(width * height) {
            rgba.append(UInt8((i * 17) % 256))
            rgba.append(64)
            rgba.append(128)
            rgba.append(255)
        }
        let frames = [
            HighlightClipEncoder.Frame(width: width, height: height, rgba: rgba),
            HighlightClipEncoder.Frame(width: width, height: height, rgba: rgba),
        ]
        let out = FileManager.default.temporaryDirectory
            .appendingPathComponent("velosim-highlight-\(UUID().uuidString).mp4")
        defer { try? FileManager.default.removeItem(at: out) }

        try HighlightClipEncoder.encode(frames: frames, fps: 2, outputURL: out)
        let attrs = try FileManager.default.attributesOfItem(atPath: out.path)
        let size = attrs[.size] as? Int ?? 0
        XCTAssertGreaterThan(size, 100)
    }

    func testFrameRingBufferSamplesAtInterval() {
        let buffer = FrameRingBuffer()
        var rgba = Data(repeating: 0, count: 16)
        rgba[0] = 255
        rgba[3] = 255
        buffer.maybeCapture(elapsedS: 0.0, width: 2, height: 2, rgba: rgba)
        buffer.maybeCapture(elapsedS: 0.2, width: 2, height: 2, rgba: rgba)
        buffer.maybeCapture(elapsedS: 0.6, width: 2, height: 2, rgba: rgba)
        let clip = HighlightClipRequestDto(startElapsedS: 0, durationS: 1, label: "Start")
        XCTAssertEqual(buffer.frames(for: clip).count, 2)
    }
}
