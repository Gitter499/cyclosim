import XCTest
import VeloFFI
import VeloSimSupport

final class TelemetrySamplePollTests: XCTestCase {

    func testHasTelemetryIncludesHeartRateOnly() {
        let sample = TelemetrySampleDto(
            elapsedMs: 0,
            powerW: nil,
            cadenceRpm: nil,
            heartRateBpm: 142,
            wheelSpeedMps: nil
        )
        XCTAssertTrue(TelemetrySamplePoll.hasTelemetry(sample))
    }

    func testDrainReturnsPendingFirst() {
        let pending = [
            TelemetrySampleDto(
                elapsedMs: 100,
                powerW: 200,
                cadenceRpm: 90,
                heartRateBpm: 130,
                wheelSpeedMps: nil
            ),
        ]
        var queue = pending
        let out = TelemetrySamplePoll.drain(
            latest: TelemetrySampleDto(elapsedMs: 0, powerW: nil, cadenceRpm: nil, heartRateBpm: nil, wheelSpeedMps: nil),
            pending: &queue,
            elapsedMs: 200
        )
        XCTAssertEqual(out.count, 1)
        XCTAssertEqual(out[0].elapsedMs, 100)
        XCTAssertTrue(queue.isEmpty)
    }

    func testDrainFallsBackToLatestHeartRateOnly() {
        var queue: [TelemetrySampleDto] = []
        let latest = TelemetrySampleDto(
            elapsedMs: 0,
            powerW: nil,
            cadenceRpm: nil,
            heartRateBpm: 155,
            wheelSpeedMps: nil
        )
        let out = TelemetrySamplePoll.drain(latest: latest, pending: &queue, elapsedMs: 500)
        XCTAssertEqual(out.count, 1)
        XCTAssertEqual(out[0].heartRateBpm, 155)
        XCTAssertEqual(out[0].elapsedMs, 500)
    }

    func testDrainEmptyWhenNoTelemetry() {
        var queue: [TelemetrySampleDto] = []
        let latest = TelemetrySampleDto(
            elapsedMs: 0,
            powerW: nil,
            cadenceRpm: nil,
            heartRateBpm: nil,
            wheelSpeedMps: nil
        )
        let out = TelemetrySamplePoll.drain(latest: latest, pending: &queue, elapsedMs: 500)
        XCTAssertTrue(out.isEmpty)
    }
}
