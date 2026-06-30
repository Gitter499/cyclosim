import Foundation
import VeloFFI

/// Fake sensor stream for dev — replays canned telemetry each tick.
public final class FakeSensorSource: SensorSourceCallback, @unchecked Sendable {
    private var elapsedMs: UInt64 = 0

    public init() {}

    public func pollSamples() -> [TelemetrySampleDto] {
        elapsedMs += 33
        let phase = Double(elapsedMs % 6000) / 6000.0
        let power = 160.0 + 40.0 * sin(phase * 2.0 * .pi)
        let cadence = 85.0 + 10.0 * cos(phase * 2.0 * .pi)
        let hr = 130.0 + 15.0 * sin(phase * .pi)

        return [
            TelemetrySampleDto(
                elapsedMs: elapsedMs,
                powerW: power,
                cadenceRpm: cadence,
                heartRateBpm: hr,
                wheelSpeedMps: nil
            ),
        ]
    }
}

public enum SensorInputMode: String, CaseIterable, Identifiable {
    case fake
    case replay
    case bluetooth

    public var id: String { rawValue }

    public var label: String {
        switch self {
        case .fake: return "Fake"
        case .replay: return "Replay"
        case .bluetooth: return "BLE (FTMS)"
        }
    }
}
