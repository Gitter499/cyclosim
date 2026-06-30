import XCTest
import VeloSimBLE

final class FTMSParserTests: XCTestCase {

    // MARK: - Indoor Bike Data

    func testIndoorBikeDataMandatoryFieldsOnly() {
        // Flags=0x0000, speed=2500 -> 25.00 km/h
        let data = FTMSParser.dataFromHex("00 00 C4 09")
        let parsed = FTMSParser.parseIndoorBikeData(data)
        XCTAssertEqual(parsed.instantaneousSpeedKmh ?? 0, 25.0, accuracy: 0.01)
        XCTAssertNil(parsed.instantaneousCadenceRpm)
        XCTAssertNil(parsed.instantaneousPowerW)
    }

    func testIndoorBikeDataCadenceAndPower() {
        // Flags: cadence(2) + power(6) = 0x0044
        // speed=1000 (10 km/h), cadence=170 (->85 rpm), power=200W
        let data = FTMSParser.dataFromHex("44 00 E8 03 AA 00 C8 00")
        let parsed = FTMSParser.parseIndoorBikeData(data)
        XCTAssertEqual(parsed.instantaneousSpeedKmh ?? 0, 10.0, accuracy: 0.01)
        XCTAssertEqual(parsed.instantaneousCadenceRpm ?? 0, 85.0, accuracy: 0.01)
        XCTAssertEqual(parsed.instantaneousPowerW ?? 0, 200.0, accuracy: 0.01)
    }

    func testIndoorBikeDataDistanceUint24LittleEndian() {
        // Flags: distance(4) = 0x0010, speed=0, distance=0x563412 -> 0x123456 m
        let data = FTMSParser.dataFromHex("10 00 00 00 56 34 12")
        let parsed = FTMSParser.parseIndoorBikeData(data)
        XCTAssertEqual(parsed.totalDistanceM ?? 0, 1_193_046, accuracy: 0.5)
    }

    func testIndoorBikeDataAllCommonFields() {
        // bits 1,2,3,4,5,6,7,9,11 = 0x0AFE
        let hex = """
            FE 0A \
            2C 01 \
            2C 01 \
            B4 00 \
            96 00 \
            40 42 0F \
            0A 00 \
            2C 01 \
            2C 01 \
            78 \
            10 27
            """
        let parsed = FTMSParser.parseIndoorBikeData(FTMSParser.dataFromHex(hex))
        XCTAssertEqual(parsed.instantaneousSpeedKmh ?? 0, 3.0, accuracy: 0.01)
        XCTAssertEqual(parsed.averageSpeedKmh ?? 0, 3.0, accuracy: 0.01)
        XCTAssertEqual(parsed.instantaneousCadenceRpm ?? 0, 90.0, accuracy: 0.01)
        XCTAssertEqual(parsed.averageCadenceRpm ?? 0, 75.0, accuracy: 0.01)
        XCTAssertEqual(parsed.totalDistanceM ?? 0, 1_000_000, accuracy: 0.5)
        XCTAssertEqual(parsed.resistanceLevel, 10)
        XCTAssertEqual(parsed.instantaneousPowerW ?? 0, 300.0, accuracy: 0.01)
        XCTAssertEqual(parsed.averagePowerW ?? 0, 300.0, accuracy: 0.01)
        XCTAssertEqual(parsed.heartRateBpm, 120)
        XCTAssertEqual(parsed.elapsedTimeSec, 10_000)
    }

    func testIndoorBikeDataExpendedEnergy() {
        // energy(8) = 0x0100, speed=0, total=100kcal, per hour=500, per min=10
        let data = FTMSParser.dataFromHex("00 01 00 00 64 00 F4 01 0A")
        let parsed = FTMSParser.parseIndoorBikeData(data)
        XCTAssertEqual(parsed.totalEnergyKcal, 100)
        XCTAssertEqual(parsed.energyPerHourKcal, 500)
        XCTAssertEqual(parsed.energyPerMinuteKcal, 10)
    }

    func testIndoorBikeDataRemainingTime() {
        // remaining time(12) = 0x1000, speed=0, remaining=600s
        let data = FTMSParser.dataFromHex("00 10 00 00 58 02")
        let parsed = FTMSParser.parseIndoorBikeData(data)
        XCTAssertEqual(parsed.remainingTimeSec, 600)
    }

    func testIndoorBikeDataTruncatedPayload() {
        let data = FTMSParser.dataFromHex("44 00 E8 03")
        let parsed = FTMSParser.parseIndoorBikeData(data)
        XCTAssertEqual(parsed.instantaneousSpeedKmh ?? 0, 10.0, accuracy: 0.01)
        XCTAssertNil(parsed.instantaneousCadenceRpm)
    }

    // MARK: - Feature flags

    func testFitnessMachineFeatureErgAndSim() {
        // Target flags at offset 4: ERG (bit 3) + SIM (bit 13) = 0x00002008
        let data = FTMSParser.dataFromHex("00 00 00 00 08 20 00 00")
        let caps = FTMSParser.parseFitnessMachineFeature(data)
        XCTAssertTrue(caps.supportsErg)
        XCTAssertTrue(caps.supportsSimulation)
    }

    func testFitnessMachineFeatureNoErg() {
        let data = FTMSParser.dataFromHex("00 00 00 00 00 00 00 00")
        let caps = FTMSParser.parseFitnessMachineFeature(data)
        XCTAssertFalse(caps.supportsErg)
        XCTAssertFalse(caps.supportsSimulation)
    }

    // MARK: - Supported Power Range

    func testSupportedPowerRange() {
        let data = FTMSParser.dataFromHex("64 00 C8 00 05 00")
        let range = FTMSParser.parseSupportedPowerRange(data)!
        XCTAssertEqual(range.minWatts, 100)
        XCTAssertEqual(range.maxWatts, 200)
        XCTAssertEqual(range.incrementWatts, 5)
        XCTAssertEqual(range.clamp(103), 105)
        XCTAssertEqual(range.clamp(50), 100)
        XCTAssertEqual(range.clamp(250), 200)
    }

    // MARK: - Control point response

    func testControlPointResponseSuccess() {
        let data = FTMSParser.dataFromHex("80 00 01")
        let resp = FTMSParser.parseControlPointResponse(data)!
        XCTAssertEqual(resp.requestOpcode, .requestControl)
        XCTAssertEqual(resp.result, .success)
    }

    func testControlPointResponseFailure() {
        let data = FTMSParser.dataFromHex("80 05 03")
        let resp = FTMSParser.parseControlPointResponse(data)!
        XCTAssertEqual(resp.requestOpcode, .setTargetPower)
        XCTAssertEqual(resp.result, .invalidParameter)
    }

    // MARK: - Fitness Machine Status

    func testFitnessMachineStatusStarted() {
        let status = FTMSParser.parseFitnessMachineStatus(FTMSParser.dataFromHex("04"))!
        XCTAssertEqual(status.opcode, .startedOrResumedByUser)
        XCTAssertEqual(status.description, "Started or Resumed")
    }

    func testFitnessMachineStatusControlLost() {
        let status = FTMSParser.parseFitnessMachineStatus(FTMSParser.dataFromHex("FF"))!
        XCTAssertEqual(status.opcode, .controlPermissionLost)
    }

    // MARK: - Heart rate

    func testHeartRateUInt8() {
        XCTAssertEqual(FTMSParser.parseHeartRate(FTMSParser.dataFromHex("00 4B")), 75)
    }

    func testHeartRateUInt16() {
        XCTAssertEqual(FTMSParser.parseHeartRate(FTMSParser.dataFromHex("01 00 4B")), 75)
    }
}
