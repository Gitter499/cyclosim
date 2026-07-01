import Foundation
import VeloFFI

/// Drains pending BLE telemetry and falls back to the latest merged sample when needed.
public enum TelemetrySamplePoll {
    public static func hasTelemetry(_ sample: TelemetrySampleDto) -> Bool {
        sample.powerW != nil || sample.cadenceRpm != nil || sample.heartRateBpm != nil
    }

    public static func drain(
        latest: TelemetrySampleDto,
        pending: inout [TelemetrySampleDto],
        elapsedMs: UInt64
    ) -> [TelemetrySampleDto] {
        var out = pending
        pending.removeAll()
        if out.isEmpty, hasTelemetry(latest) {
            var sample = latest
            sample.elapsedMs = elapsedMs
            out = [sample]
        }
        return out
    }
}
