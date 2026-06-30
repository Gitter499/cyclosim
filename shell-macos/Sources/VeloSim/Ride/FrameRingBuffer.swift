import Foundation
import VeloFFI

/// Rolling capture of RGBA frames sampled during an active ride (~2 fps, last 30 s).
public final class FrameRingBuffer: @unchecked Sendable {
  public struct Entry: Sendable {
    public let elapsedS: Double
    public let width: Int
    public let height: Int
    public let rgba: Data
  }

  private let lock = NSLock()
  private var entries: [Entry] = []
  private var lastSampleElapsedS: Double = -1
  private let maxDurationS: Double = 30
  private let sampleIntervalS: Double = 0.5

  public init() {}

  public func reset() {
    lock.lock()
    defer { lock.unlock() }
    entries.removeAll(keepingCapacity: true)
    lastSampleElapsedS = -1
  }

  public func maybeCapture(elapsedS: Double, width: Int, height: Int, rgba: Data) {
    guard width > 0, height > 0, rgba.count == width * height * 4 else { return }
    lock.lock()
    defer { lock.unlock() }
    if elapsedS - lastSampleElapsedS < sampleIntervalS {
      return
    }
    lastSampleElapsedS = elapsedS
    entries.append(Entry(elapsedS: elapsedS, width: width, height: height, rgba: rgba))
    trim(keepingAfter: elapsedS - maxDurationS)
  }

  /// Frames for one highlight window, sorted by time.
  public func frames(for clip: HighlightClipRequestDto) -> [Entry] {
    lock.lock()
    defer { lock.unlock() }
    let start = clip.startElapsedS
    let end = clip.startElapsedS + clip.durationS
    return entries.filter { $0.elapsedS >= start && $0.elapsedS <= end }
  }

  /// All frames across clip windows, in playback order.
  public func frames(for clips: [HighlightClipRequestDto]) -> [Entry] {
    clips.flatMap { frames(for: $0) }
  }

  private func trim(keepingAfter minElapsed: Double) {
    if let idx = entries.firstIndex(where: { $0.elapsedS >= minElapsed }) {
      if idx > 0 {
        entries.removeFirst(idx)
      }
    } else if !entries.isEmpty, let last = entries.last, last.elapsedS < minElapsed {
      entries.removeAll(keepingCapacity: true)
    }
  }
}
