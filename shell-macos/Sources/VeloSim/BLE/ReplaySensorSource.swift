import Foundation
import VeloFFI

/// Replays a recorded FTMS telemetry trace for headless/CI dev without hardware.
final class ReplaySensorSource: SensorSourceCallback, @unchecked Sendable {
    struct TraceSample: Decodable {
        let elapsedMs: UInt64
        let powerW: Double?
        let cadenceRpm: Double?
        let heartRateBpm: Double?
        let wheelSpeedMps: Double?

        enum CodingKeys: String, CodingKey {
            case elapsedMs = "elapsed_ms"
            case powerW = "power_w"
            case cadenceRpm = "cadence_rpm"
            case heartRateBpm = "heart_rate_bpm"
            case wheelSpeedMps = "wheel_speed_mps"
        }
    }

    private let samples: [TraceSample]
    private var index = 0
    private var loopStart = Date()

    init(samples: [TraceSample] = ReplaySensorSource.defaultTrace) {
        self.samples = samples
    }

    init(json: String) {
        if let data = json.data(using: .utf8),
           let decoded = try? JSONDecoder().decode([TraceSample].self, from: data) {
            samples = decoded
        } else {
            samples = Self.defaultTrace
        }
    }

    func pollSamples() -> [TelemetrySampleDto] {
        guard !samples.isEmpty else { return [] }
        let elapsed = UInt64(Date().timeIntervalSince(loopStart) * 1000)
        while index + 1 < samples.count, samples[index + 1].elapsedMs <= elapsed {
            index += 1
        }
        if index + 1 >= samples.count, elapsed > samples.last!.elapsedMs + 500 {
            index = 0
            loopStart = Date()
        }
        let s = samples[index]
        return [
            TelemetrySampleDto(
                elapsedMs: elapsed,
                powerW: s.powerW,
                cadenceRpm: s.cadenceRpm,
                heartRateBpm: s.heartRateBpm,
                wheelSpeedMps: s.wheelSpeedMps
            ),
        ]
    }

    /// ~60 s canned ride trace (power/cadence/HR vary like a steady interval).
    static let defaultTrace: [TraceSample] = {
        var out: [TraceSample] = []
        for i in 0..<1800 {
            let t = Double(i) / 30.0
            let phase = t.truncatingRemainder(dividingBy: 60.0) / 60.0
            let power = 170.0 + 50.0 * sin(phase * 2.0 * .pi)
            let cadence = 88.0 + 8.0 * cos(phase * 2.0 * .pi)
            let hr = 135.0 + 12.0 * sin(phase * .pi)
            out.append(
                TraceSample(
                    elapsedMs: UInt64(i) * 33,
                    powerW: power,
                    cadenceRpm: cadence,
                    heartRateBpm: hr,
                    wheelSpeedMps: 8.5 + 1.5 * sin(phase * .pi)
                )
            )
        }
        return out
    }()
}

/// Trainer stub that logs commands — used with replay/fake sensors.
final class LoggingTrainerControl: TrainerControlCallback, @unchecked Sendable {
    private(set) var lastTargetPower: Double = 0
    private(set) var lastGrade: Double = 0
    private(set) var stopped = false

    func setTargetPower(watts: Double) {
        lastTargetPower = watts
    }

    func setSimulation(grade: Double, crr: Double, cwa: Double) {
        lastGrade = grade
        _ = (crr, cwa)
    }

    func stop() {
        stopped = true
    }
}
