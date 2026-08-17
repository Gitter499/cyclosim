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

    public let kind: ProtocolKind
    public let startWatts: Int
    public let stepWatts: Int
    public private(set) var target: Int
    public private(set) var secs = 0
    private var window: [Double] = []
    public private(set) var best1MinAvg = 0.0
    public private(set) var failed = false
    private let previousFTP: Int

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

    /// Call once per second with rider power; returns ERG target and whether the ramp failed.
    public func tick(power: Double, cadence: Double) -> (target: Int, failed: Bool) {
        guard kind != .twentyMin else { return (0, false) }

        secs += 1
        window.append(power)
        if window.count > 60 { window.removeFirst() }
        if !window.isEmpty {
            best1MinAvg = max(best1MinAvg, window.reduce(0, +) / Double(window.count))
        }
        if secs % 60 == 0, secs > 0 { target += stepWatts }

        let failing = cadence < 50 || power < Double(target) * 0.70
        if failing { failed = true }
        return (target, failing)
    }

    public func finish() -> (ftp: Int, changed: Bool) {
        let ftp: Int
        switch kind {
        case .ramp, .rampLite:
            ftp = Int((best1MinAvg * 0.75).rounded())
        case .twentyMin:
            ftp = previousFTP
        }
        return (max(ftp, 1), ftp != previousFTP)
    }
}
