import Foundation
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers

/// Encodes raw RGBA8 pixels to PNG (shell-side media capture per §14).
public enum PngEncoder {
  public static let pngMagic: [UInt8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]

  public static func encode(width: Int, height: Int, rgba: Data) throws -> Data {
    guard width > 0, height > 0 else {
      throw PngEncoderError.invalidDimensions
    }
    let expected = width * height * 4
    guard rgba.count == expected else {
      throw PngEncoderError.byteCountMismatch(expected: expected, actual: rgba.count)
    }

    guard let provider = CGDataProvider(data: rgba as CFData) else {
      throw PngEncoderError.providerFailed
    }
    guard let cgImage = CGImage(
      width: width,
      height: height,
      bitsPerComponent: 8,
      bitsPerPixel: 32,
      bytesPerRow: width * 4,
      space: CGColorSpaceCreateDeviceRGB(),
      bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue),
      provider: provider,
      decode: nil,
      shouldInterpolate: false,
      intent: .defaultIntent
    ) else {
      throw PngEncoderError.imageCreationFailed
    }

    let data = NSMutableData()
    guard let dest = CGImageDestinationCreateWithData(
      data,
      UTType.png.identifier as CFString,
      1,
      nil
    ) else {
      throw PngEncoderError.destinationFailed
    }
    CGImageDestinationAddImage(dest, cgImage, nil)
    guard CGImageDestinationFinalize(dest) else {
      throw PngEncoderError.finalizeFailed
    }
    return data as Data
  }

  public static func decode(png: Data) throws -> (width: Int, height: Int, rgba: Data) {
    guard png.count >= 8, png.prefix(8).elementsEqual(pngMagic) else {
      throw PngEncoderError.invalidPng
    }
    guard let source = CGImageSourceCreateWithData(png as CFData, nil),
          let cgImage = CGImageSourceCreateImageAtIndex(source, 0, nil)
    else {
      throw PngEncoderError.invalidPng
    }
    let width = cgImage.width
    let height = cgImage.height
    let bytesPerRow = width * 4
    var rgba = Data(count: bytesPerRow * height)
    let ok = rgba.withUnsafeMutableBytes { ptr -> Bool in
      guard let ctx = CGContext(
        data: ptr.baseAddress,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: bytesPerRow,
        space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
      ) else { return false }
      ctx.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))
      return true
    }
    guard ok else { throw PngEncoderError.decodeFailed }
    return (width, height, rgba)
  }
}

public enum PngEncoderError: Error, Equatable {
  case invalidDimensions
  case byteCountMismatch(expected: Int, actual: Int)
  case providerFailed
  case imageCreationFailed
  case destinationFailed
  case finalizeFailed
  case invalidPng
  case decodeFailed
}
