import AVFoundation
import CoreMedia
import Foundation

/// H.264 MP4 encoder for highlight reels (VideoToolbox via AVAssetWriter).
public enum HighlightClipEncoder {
  public struct Frame {
    public let width: Int
    public let height: Int
    public let rgba: Data

    public init(width: Int, height: Int, rgba: Data) {
      self.width = width
      self.height = height
      self.rgba = rgba
    }
  }

  public static let defaultFps: Double = 2.0

  public static func encode(frames: [Frame], fps: Double, outputURL: URL) throws {
    guard !frames.isEmpty else {
      throw HighlightClipEncoderError.noFrames
    }
    let width = frames[0].width
    let height = frames[0].height
    guard width > 0, height > 0 else {
      throw HighlightClipEncoderError.invalidDimensions
    }

    if FileManager.default.fileExists(atPath: outputURL.path) {
      try FileManager.default.removeItem(at: outputURL)
    }

    let writer = try AVAssetWriter(outputURL: outputURL, fileType: .mp4)
    let settings: [String: Any] = [
      AVVideoCodecKey: AVVideoCodecType.h264,
      AVVideoWidthKey: width,
      AVVideoHeightKey: height,
      AVVideoCompressionPropertiesKey: [
        AVVideoAverageBitRateKey: width * height * 4,
        AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
      ],
    ]
    let input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
    input.expectsMediaDataInRealTime = false

    let attrs: [String: Any] = [
      kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
      kCVPixelBufferWidthKey as String: width,
      kCVPixelBufferHeightKey as String: height,
    ]
    let adaptor = AVAssetWriterInputPixelBufferAdaptor(
      assetWriterInput: input,
      sourcePixelBufferAttributes: attrs
    )

    guard writer.canAdd(input) else {
      throw HighlightClipEncoderError.writerSetupFailed
    }
    writer.add(input)
    guard writer.startWriting() else {
      throw HighlightClipEncoderError.writerStartFailed(writer.error)
    }
    writer.startSession(atSourceTime: .zero)

    let frameDuration = CMTime(value: 1, timescale: CMTimeScale(max(Int32(fps.rounded()), 1)))
    var frameIndex: Int64 = 0

    for frame in frames {
      guard frame.width == width, frame.height == height else { continue }
      while !input.isReadyForMoreMediaData {
        Thread.sleep(forTimeInterval: 0.001)
      }
      guard let pool = adaptor.pixelBufferPool else {
        throw HighlightClipEncoderError.pixelBufferPoolMissing
      }
      var pixelBuffer: CVPixelBuffer?
      let status = CVPixelBufferPoolCreatePixelBuffer(nil, pool, &pixelBuffer)
      guard status == kCVReturnSuccess, let buffer = pixelBuffer else {
        throw HighlightClipEncoderError.pixelBufferAllocationFailed
      }
      try copyRGBA(frame.rgba, width: width, height: height, into: buffer)
      let pts = CMTimeMultiply(frameDuration, multiplier: Int32(frameIndex))
      if !adaptor.append(buffer, withPresentationTime: pts) {
        throw HighlightClipEncoderError.appendFailed
      }
      frameIndex += 1
    }

    input.markAsFinished()
    let sem = DispatchSemaphore(value: 0)
    writer.finishWriting {
      sem.signal()
    }
    sem.wait()
    guard writer.status == .completed else {
      throw HighlightClipEncoderError.writerFinishFailed(writer.error)
    }
  }

  private static func copyRGBA(_ rgba: Data, width: Int, height: Int, into buffer: CVPixelBuffer) throws {
    CVPixelBufferLockBaseAddress(buffer, [])
    defer { CVPixelBufferUnlockBaseAddress(buffer, []) }
    guard let base = CVPixelBufferGetBaseAddress(buffer) else {
      throw HighlightClipEncoderError.pixelBufferBaseMissing
    }
    let bytesPerRow = CVPixelBufferGetBytesPerRow(buffer)
    rgba.withUnsafeBytes { raw in
      guard let src = raw.baseAddress?.assumingMemoryBound(to: UInt8.self) else { return }
      for y in 0..<height {
        let dstRow = base.advanced(by: y * bytesPerRow).assumingMemoryBound(to: UInt8.self)
        let srcRow = src.advanced(by: y * width * 4)
        for x in 0..<width {
          let si = x * 4
          let r = srcRow[si]
          let g = srcRow[si + 1]
          let b = srcRow[si + 2]
          let di = x * 4
          dstRow[di] = b
          dstRow[di + 1] = g
          dstRow[di + 2] = r
          dstRow[di + 3] = 255
        }
      }
    }
  }
}

public enum HighlightClipEncoderError: Error, Equatable {
  case noFrames
  case invalidDimensions
  case writerSetupFailed
  case writerStartFailed(Error?)
  case pixelBufferPoolMissing
  case pixelBufferAllocationFailed
  case pixelBufferBaseMissing
  case appendFailed
  case writerFinishFailed(Error?)

  public static func == (lhs: HighlightClipEncoderError, rhs: HighlightClipEncoderError) -> Bool {
    switch (lhs, rhs) {
    case (.noFrames, .noFrames),
      (.invalidDimensions, .invalidDimensions),
      (.writerSetupFailed, .writerSetupFailed),
      (.pixelBufferPoolMissing, .pixelBufferPoolMissing),
      (.pixelBufferAllocationFailed, .pixelBufferAllocationFailed),
      (.pixelBufferBaseMissing, .pixelBufferBaseMissing),
      (.appendFailed, .appendFailed):
      return true
    case (.writerStartFailed, .writerStartFailed),
      (.writerFinishFailed, .writerFinishFailed):
      return true
    default:
      return false
    }
  }
}
