import AppKit
import VeloFFI

public enum WorkoutTargetKind: String, CaseIterable, Identifiable, Sendable {
    case ergWatts = "ERG (W)"
    case ftpPercent = "FTP %"
    case freeRide = "Free ride"

    public var id: String { rawValue }
}

/// Editable workout interval for the in-app builder (shared by UI and unit tests).
public struct WorkoutBuilderInterval: Identifiable, Equatable, Sendable {
    public let id = UUID()
    public var name: String
    public var durationMinutes: Int
    public var durationSeconds: Int
    public var targetKind: WorkoutTargetKind
    public var ergWatts: Double
    public var ftpPercent: Double

    public var durationS: Double {
        Double(durationMinutes * 60 + durationSeconds)
    }

    public init(
        name: String,
        durationMinutes: Int,
        durationSeconds: Int,
        targetKind: WorkoutTargetKind,
        ergWatts: Double,
        ftpPercent: Double
    ) {
        self.name = name
        self.durationMinutes = durationMinutes
        self.durationSeconds = durationSeconds
        self.targetKind = targetKind
        self.ergWatts = ergWatts
        self.ftpPercent = ftpPercent
    }

    public static func warmupDefault() -> Self {
        Self(
            name: "Warmup",
            durationMinutes: 10,
            durationSeconds: 0,
            targetKind: .ftpPercent,
            ergWatts: 150,
            ftpPercent: 55
        )
    }

    public static func fromDto(_ dto: WorkoutIntervalDto) -> Self {
        let totalSeconds = max(0, Int(dto.durationS.rounded()))
        let minutes = totalSeconds / 60
        let seconds = totalSeconds % 60
        switch dto.target {
        case .ergWatts(let watts):
            return Self(
                name: dto.name,
                durationMinutes: minutes,
                durationSeconds: seconds,
                targetKind: .ergWatts,
                ergWatts: watts,
                ftpPercent: 55
            )
        case .ftpPercent(let percent):
            return Self(
                name: dto.name,
                durationMinutes: minutes,
                durationSeconds: seconds,
                targetKind: .ftpPercent,
                ergWatts: 150,
                ftpPercent: percent
            )
        case .freeRide:
            return Self(
                name: dto.name,
                durationMinutes: minutes,
                durationSeconds: seconds,
                targetKind: .freeRide,
                ergWatts: 150,
                ftpPercent: 55
            )
        }
    }

    public func toDto() -> WorkoutIntervalDto {
        let target: WorkoutTargetDto
        switch targetKind {
        case .ergWatts:
            target = .ergWatts(watts: ergWatts)
        case .ftpPercent:
            target = .ftpPercent(percent: ftpPercent)
        case .freeRide:
            target = .freeRide
        }
        return WorkoutIntervalDto(name: name, durationS: durationS, target: target)
    }
}
