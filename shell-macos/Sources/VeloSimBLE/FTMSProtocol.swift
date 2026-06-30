import CoreBluetooth
import Foundation

/// Bluetooth FTMS UUIDs (Fitness Machine Service, SIG spec v1.0).
public enum FTMS {
    public static let service = CBUUID(string: "1826")
    public static let fitnessMachineFeature = CBUUID(string: "2ACC")
    public static let trainingStatus = CBUUID(string: "2AD3")
    public static let supportedPowerRange = CBUUID(string: "2AD8")
    public static let indoorBikeData = CBUUID(string: "2AD2")
    public static let fitnessMachineControlPoint = CBUUID(string: "2AD9")
    public static let fitnessMachineStatus = CBUUID(string: "2ADA")

    public static let heartRateService = CBUUID(string: "180D")
    public static let heartRateMeasurement = CBUUID(string: "2A37")

    public static let controlResponseCode: UInt8 = 0x80
    public static let stateRestoreIdentifier = "com.velosim.ftms.central"

    public enum ControlOpcode: UInt8, CustomStringConvertible {
        case requestControl = 0x00
        case reset = 0x01
        case setTargetPower = 0x05
        case startOrResume = 0x07
        case stopOrPause = 0x08
        case setIndoorBikeSimulation = 0x11

        public var description: String {
            switch self {
            case .requestControl: return "Request Control"
            case .reset: return "Reset"
            case .setTargetPower: return "Set Target Power"
            case .startOrResume: return "Start or Resume"
            case .stopOrPause: return "Stop or Pause"
            case .setIndoorBikeSimulation: return "Set Indoor Bike Simulation"
            }
        }
    }

    public enum ControlResult: UInt8, CustomStringConvertible {
        case success = 0x01
        case opCodeNotSupported = 0x02
        case invalidParameter = 0x03
        case operationFailed = 0x04
        case controlNotPermitted = 0x05

        public var description: String {
            switch self {
            case .success: return "Success"
            case .opCodeNotSupported: return "Op Code Not Supported"
            case .invalidParameter: return "Invalid Parameter"
            case .operationFailed: return "Operation Failed"
            case .controlNotPermitted: return "Control Not Permitted"
            }
        }

        public init?(raw: UInt8) {
            self.init(rawValue: raw)
        }
    }

    public enum StatusOpcode: UInt8, CustomStringConvertible {
        case reset = 0x01
        case stoppedOrPausedByUser = 0x02
        case stoppedBySafetyKey = 0x03
        case startedOrResumedByUser = 0x04
        case targetSpeedChanged = 0x05
        case targetInclineChanged = 0x06
        case targetResistanceChanged = 0x07
        case targetPowerChanged = 0x08
        case targetHeartRateChanged = 0x09
        case targetedExpendedEnergyChanged = 0x0A
        case targetedStepsChanged = 0x0B
        case targetedStridesChanged = 0x0C
        case targetedDistanceChanged = 0x0D
        case targetedTrainingTimeChanged = 0x0E
        case targetedTimeTwoHRZonesChanged = 0x0F
        case targetedTimeThreeHRZonesChanged = 0x10
        case targetedTimeFiveHRZonesChanged = 0x11
        case indoorBikeSimulationChanged = 0x12
        case wheelCircumferenceChanged = 0x13
        case spinDownStatus = 0x14
        case controlPermissionLost = 0xFF

        public var description: String {
            switch self {
            case .reset: return "Reset"
            case .stoppedOrPausedByUser: return "Stopped or Paused"
            case .stoppedBySafetyKey: return "Stopped (Safety Key)"
            case .startedOrResumedByUser: return "Started or Resumed"
            case .targetSpeedChanged: return "Target Speed Changed"
            case .targetInclineChanged: return "Target Incline Changed"
            case .targetResistanceChanged: return "Target Resistance Changed"
            case .targetPowerChanged: return "Target Power Changed"
            case .targetHeartRateChanged: return "Target Heart Rate Changed"
            case .targetedExpendedEnergyChanged: return "Target Expended Energy Changed"
            case .targetedStepsChanged: return "Target Steps Changed"
            case .targetedStridesChanged: return "Target Strides Changed"
            case .targetedDistanceChanged: return "Target Distance Changed"
            case .targetedTrainingTimeChanged: return "Target Training Time Changed"
            case .targetedTimeTwoHRZonesChanged: return "Target Time (2 HR Zones) Changed"
            case .targetedTimeThreeHRZonesChanged: return "Target Time (3 HR Zones) Changed"
            case .targetedTimeFiveHRZonesChanged: return "Target Time (5 HR Zones) Changed"
            case .indoorBikeSimulationChanged: return "Simulation Parameters Changed"
            case .wheelCircumferenceChanged: return "Wheel Circumference Changed"
            case .spinDownStatus: return "Spin Down Status"
            case .controlPermissionLost: return "Control Permission Lost"
            }
        }
    }
}

// MARK: - Capabilities

public struct FitnessMachineCapabilities: Equatable, CustomStringConvertible {
    public var supportsErg: Bool
    public var supportsSimulation: Bool
    public var supportsResistanceTarget: Bool
    public var supportsSpeedTarget: Bool
    public var powerRange: SupportedPowerRange?

    public init(
        supportsErg: Bool = false,
        supportsSimulation: Bool = false,
        supportsResistanceTarget: Bool = false,
        supportsSpeedTarget: Bool = false,
        powerRange: SupportedPowerRange? = nil
    ) {
        self.supportsErg = supportsErg
        self.supportsSimulation = supportsSimulation
        self.supportsResistanceTarget = supportsResistanceTarget
        self.supportsSpeedTarget = supportsSpeedTarget
        self.powerRange = powerRange
    }

    public var description: String {
        var parts: [String] = []
        parts.append("ERG: \(supportsErg ? "yes" : "no")")
        parts.append("SIM: \(supportsSimulation ? "yes" : "no")")
        if let range = powerRange {
            parts.append("Power: \(range.minWatts)–\(range.maxWatts) W (±\(range.incrementWatts) W)")
        }
        return parts.joined(separator: ", ")
    }
}

public struct SupportedPowerRange: Equatable {
    public let minWatts: Int
    public let maxWatts: Int
    public let incrementWatts: Int

    public init(minWatts: Int, maxWatts: Int, incrementWatts: Int) {
        self.minWatts = minWatts
        self.maxWatts = maxWatts
        self.incrementWatts = incrementWatts
    }

    public func clamp(_ watts: Double) -> Int16 {
        let w = Int(watts.rounded())
        let clamped = min(max(w, minWatts), maxWatts)
        if incrementWatts > 0 {
            let steps = Int((Double(clamped - minWatts) / Double(incrementWatts)).rounded())
            let snapped = minWatts + steps * incrementWatts
            return Int16(min(max(snapped, minWatts), maxWatts))
        }
        return Int16(clamped)
    }
}

// MARK: - Parsed telemetry

public struct ParsedIndoorBikeData: Equatable {
    public var instantaneousSpeedKmh: Double?
    public var averageSpeedKmh: Double?
    public var instantaneousCadenceRpm: Double?
    public var averageCadenceRpm: Double?
    public var totalDistanceM: Double?
    public var resistanceLevel: Int?
    public var instantaneousPowerW: Double?
    public var averagePowerW: Double?
    public var totalEnergyKcal: Int?
    public var energyPerHourKcal: Int?
    public var energyPerMinuteKcal: Int?
    public var heartRateBpm: Int?
    public var elapsedTimeSec: Int?
    public var remainingTimeSec: Int?

    public init() {}
}

public struct ParsedFitnessMachineStatus: Equatable {
    public let opcode: FTMS.StatusOpcode
    public let description: String
}

public struct ControlPointResponse: Equatable {
    public let requestOpcode: FTMS.ControlOpcode
    public let result: FTMS.ControlResult
    public let raw: Data
}

// MARK: - Parsers

public enum FTMSParser {
    /// Target Setting Features — bit 3 = power target (ERG), bit 13 = indoor bike simulation.
    public static func parseFitnessMachineFeature(_ data: Data) -> FitnessMachineCapabilities {
        guard data.count >= 8 else { return FitnessMachineCapabilities() }
        let targetFlags = readUInt32LE(data, at: 4)
        return FitnessMachineCapabilities(
            supportsErg: targetFlags & (1 << 3) != 0,
            supportsSimulation: targetFlags & (1 << 13) != 0,
            supportsResistanceTarget: targetFlags & (1 << 2) != 0,
            supportsSpeedTarget: targetFlags & (1 << 0) != 0
        )
    }

    public static func parseSupportedPowerRange(_ data: Data) -> SupportedPowerRange? {
        guard data.count >= 6 else { return nil }
        let minW = Int(readInt16LE(data, at: 0))
        let maxW = Int(readInt16LE(data, at: 2))
        let inc = Int(readUInt16LE(data, at: 4))
        guard minW <= maxW, maxW > 0 else { return nil }
        return SupportedPowerRange(minWatts: minW, maxWatts: maxW, incrementWatts: max(inc, 1))
    }

    /// Parse FTMS Indoor Bike Data characteristic payload (flags + fields per spec).
    public static func parseIndoorBikeData(_ data: Data) -> ParsedIndoorBikeData {
        guard data.count >= 4 else { return ParsedIndoorBikeData() }
        var offset = 0
        let flags = Int(readUInt16LE(data, at: offset))
        offset += 2

        var result = ParsedIndoorBikeData()

        result.instantaneousSpeedKmh = Double(readUInt16LE(data, at: offset)) / 100.0
        offset += 2

        if flags & (1 << 1) != 0, let v = readFieldUInt16(&offset, data) {
            result.averageSpeedKmh = Double(v) / 100.0
        }
        if flags & (1 << 2) != 0, let v = readFieldUInt16(&offset, data) {
            result.instantaneousCadenceRpm = Double(v) / 2.0
        }
        if flags & (1 << 3) != 0, let v = readFieldUInt16(&offset, data) {
            result.averageCadenceRpm = Double(v) / 2.0
        }
        if flags & (1 << 4) != 0, let v = readFieldUInt24(&offset, data) {
            result.totalDistanceM = Double(v)
        }
        if flags & (1 << 5) != 0, let v = readFieldInt16(&offset, data) {
            result.resistanceLevel = Int(v)
        }
        if flags & (1 << 6) != 0, let v = readFieldInt16(&offset, data) {
            result.instantaneousPowerW = Double(v)
        }
        if flags & (1 << 7) != 0, let v = readFieldInt16(&offset, data) {
            result.averagePowerW = Double(v)
        }
        if flags & (1 << 8) != 0 {
            if let total = readFieldUInt16(&offset, data) {
                result.totalEnergyKcal = Int(total)
            }
            if let perHour = readFieldUInt16(&offset, data) {
                result.energyPerHourKcal = Int(perHour)
            }
            if offset < data.count {
                result.energyPerMinuteKcal = Int(data[offset])
                offset += 1
            }
        }
        if flags & (1 << 9) != 0, offset < data.count {
            result.heartRateBpm = Int(data[offset])
            offset += 1
        }
        if flags & (1 << 10) != 0, offset < data.count {
            offset += 1
        }
        if flags & (1 << 11) != 0, let v = readFieldUInt16(&offset, data) {
            result.elapsedTimeSec = Int(v)
        }
        if flags & (1 << 12) != 0, let v = readFieldUInt16(&offset, data) {
            result.remainingTimeSec = Int(v)
        }

        return result
    }

    public static func parseFitnessMachineStatus(_ data: Data) -> ParsedFitnessMachineStatus? {
        guard let opcodeByte = data.first,
              let opcode = FTMS.StatusOpcode(rawValue: opcodeByte) else { return nil }
        return ParsedFitnessMachineStatus(opcode: opcode, description: opcode.description)
    }

    public static func parseControlPointResponse(_ data: Data) -> ControlPointResponse? {
        guard data.count >= 3,
              data[0] == FTMS.controlResponseCode,
              let opcode = FTMS.ControlOpcode(rawValue: data[1]),
              let result = FTMS.ControlResult(raw: data[2]) else { return nil }
        return ControlPointResponse(requestOpcode: opcode, result: result, raw: data)
    }

    public static func parseHeartRate(_ data: Data) -> Double? {
        guard data.count >= 2 else { return nil }
        let flags = data[0]
        if flags & 0x01 != 0, data.count >= 3 {
            return Double(data[2])
        }
        return Double(data[1])
    }

    public static func hexString(_ data: Data) -> String {
        data.map { String(format: "%02X", $0) }.joined(separator: " ")
    }

    public static func dataFromHex(_ hex: String) -> Data {
        let bytes = hex.split(whereSeparator: \.isWhitespace).compactMap { UInt8($0, radix: 16) }
        return Data(bytes)
    }

    private static func readUInt16LE(_ data: Data, at offset: Int) -> UInt16 {
        guard offset + 2 <= data.count else { return 0 }
        return UInt16(data[offset]) | (UInt16(data[offset + 1]) << 8)
    }

    private static func readInt16LE(_ data: Data, at offset: Int) -> Int16 {
        Int16(bitPattern: readUInt16LE(data, at: offset))
    }

    private static func readUInt32LE(_ data: Data, at offset: Int) -> UInt32 {
        guard offset + 4 <= data.count else { return 0 }
        return UInt32(data[offset])
            | (UInt32(data[offset + 1]) << 8)
            | (UInt32(data[offset + 2]) << 16)
            | (UInt32(data[offset + 3]) << 24)
    }

    private static func readFieldUInt16(_ offset: inout Int, _ data: Data) -> UInt16? {
        guard offset + 2 <= data.count else { return nil }
        let v = readUInt16LE(data, at: offset)
        offset += 2
        return v
    }

    private static func readFieldInt16(_ offset: inout Int, _ data: Data) -> Int16? {
        guard let raw = readFieldUInt16(&offset, data) else { return nil }
        return Int16(bitPattern: raw)
    }

    private static func readFieldUInt24(_ offset: inout Int, _ data: Data) -> UInt32? {
        guard offset + 3 <= data.count else { return nil }
        let v = UInt32(data[offset])
            | (UInt32(data[offset + 1]) << 8)
            | (UInt32(data[offset + 2]) << 16)
        offset += 3
        return v
    }
}
