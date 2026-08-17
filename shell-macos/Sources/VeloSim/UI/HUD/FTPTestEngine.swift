import Foundation
import Observation

/// FTP ramp-test engine per guide §6.3 — steps ERG upward, detects failure, computes FTP.
@Observable
@MainActor
public final class RampTestEngine {
    public enum ProtocolKind: String, CaseIterable, Identifiable {
        case ramp = "Ramp Test"
        case rampLite = "Ramp Test Lite"
        case twentyMin = "FTP Test (20 min)"

        public var id: String { rawValue }

        public var subtitle: String {
            switch self {
            case .ramp: "100 W start · +20 W/min · 75% of best 1-min avg"
            case .rampLite: "50 W start · +10 W/min · 75% of best 1-min avg"
            case .twentyMin: "20-min max effort · 95% of average power"
            }
        }
    }

    /// Fixed effort length for the 20-min protocol.
    public static let twentyMinDurationS = 20 * 60

    public let kind: ProtocolKind
    public let startWatts: Int
    public let stepWatts: Int
    public private(set) var target: Int
    public private(set) var secs = 0
    private var window: [Double] = []
    private var effortSum = 0.0
    public private(set) var best1MinAvg = 0.0
    public private(set) var failed = false
    private let previousFTP: Int

    /// Seconds left in the fixed-length 20-min protocol (nil for open-ended ramps).
    public var remainingS: Int? {
        kind == .twentyMin ? max(0, Self.twentyMinDurationS - secs) : nil
    }

    public init(kind: ProtocolKind, previousFTP: Int) {
        self.kind = kind
        self.previousFTP = previousFTP
        switch kind {
        case .ramp:
            startWatts = 100
            stepWatts = 20
        case .rampLite:
            startWatts = 50
            stepWatts = 10
        case .twentyMin:
            startWatts = 0
            stepWatts = 0
        }
        target = startWatts
    }

    /// Call once per second with rider power; returns the ERG target (0 = rider-paced)
    /// and whether the test just ended (ramp failure, or the 20-min effort elapsing).
    public func tick(power: Double, cadence: Double) -> (target: Int, done: Bool) {
        secs += 1
        switch kind {
        case .ramp, .rampLite:
            window.append(power)
            if window.count > 60 { window.removeFirst() }
            if !window.isEmpty {
                best1MinAvg = max(best1MinAvg, window.reduce(0, +) / Double(window.count))
            }
            if secs % 60 == 0 { target += stepWatts }

            let failing = cadence < 50 || power < Double(target) * 0.70
            if failing { failed = true }
            return (target, failing)
        case .twentyMin:
            if secs <= Self.twentyMinDurationS { effortSum += power }
            return (0, secs >= Self.twentyMinDurationS)
        }
    }

    public func finish() -> (ftp: Int, changed: Bool) {
        let ftp: Int
        switch kind {
        case .ramp, .rampLite:
            ftp = Int((best1MinAvg * 0.75).rounded())
        case .twentyMin:
            let counted = min(secs, Self.twentyMinDurationS)
            let avg = counted > 0 ? effortSum / Double(counted) : 0
            // A zero-power "effort" is an abandoned test, not a 1 W FTP.
            ftp = avg > 0 ? Int((avg * 0.95).rounded()) : previousFTP
        }
        return (max(ftp, 1), ftp != previousFTP)
    }
}
