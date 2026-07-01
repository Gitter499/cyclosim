// AppScenarioTests.swift
//
// Headless shell logic tests — no Metal viewport, no CoreBluetooth hardware.
//
// Requirements:
// - Full suite: Xcode or `swift test` with built `target/release/libvelo_ffi.dylib`
//   and DEVELOPER_DIR pointing at an Xcode app (Metal link for VeloFFIBridge).
// - Pure DTO / enum tests run under SwiftPM from the command line when the dylib is present.
// - MusicKit types are not queried here; use NoopAudioDirector for callback wiring tests.

import XCTest
import VeloFFI
import VeloSimSupport

final class AppScenarioTests: XCTestCase {

    func testWorkoutBuilderDtoRoundTrip() {
        let original = WorkoutBuilderInterval(
            name: "Threshold",
            durationMinutes: 5,
            durationSeconds: 30,
            targetKind: .ftpPercent,
            ergWatts: 200,
            ftpPercent: 95
        )
        let dto = original.toDto()
        XCTAssertEqual(dto.name, "Threshold")
        XCTAssertEqual(dto.durationS, 330, accuracy: 0.01)
        if case .ftpPercent(let pct) = dto.target {
            XCTAssertEqual(pct, 95, accuracy: 0.01)
        } else {
            XCTFail("expected ftp percent target")
        }

        let restored = WorkoutBuilderInterval.fromDto(dto)
        XCTAssertEqual(restored.name, original.name)
        XCTAssertEqual(restored.durationMinutes, original.durationMinutes)
        XCTAssertEqual(restored.durationSeconds, original.durationSeconds)
        XCTAssertEqual(restored.targetKind, original.targetKind)
        XCTAssertEqual(restored.ftpPercent, original.ftpPercent, accuracy: 0.01)
    }

    func testSteeringModeCoversOffKeyboardAirPods() {
        XCTAssertEqual(SteeringInputMode.allCases.count, 3)
        XCTAssertEqual(SteeringInputMode.off.label, "Off")
        XCTAssertEqual(SteeringInputMode.keyboard.label, "Keyboard")
        XCTAssertEqual(SteeringInputMode.airpods.label, "AirPods")
    }

    func testNoopAudioDirectorAcceptsSegmentCallbacks() {
        let director = NoopAudioDirector()
        director.onSegment(energy: .warmup, intent: .start)
        director.onSegment(energy: .threshold, intent: .transition)
    }

    @MainActor
    func testMusicDirectorEnableToggle() {
        let director = VeloMusicDirector()
        XCTAssertEqual(director.status, "Music off")
        director.setEnabled(true)
        XCTAssertEqual(director.status, "Ready — segment music on")
        director.onSegment(energy: .build, intent: .transition)
        director.setEnabled(false)
        XCTAssertEqual(director.status, "Music off")
    }

    func testHeadlessRideStartStopViaFfi() {
        let handle = VeloHandle()
        handle.setFtp(ftpW: 200)
        handle.startRide()
        let sensors = FakeSensorSource()
        let trainer = MockTrainerControl()
        let steering = NoopSteeringInput()
        for _ in 0..<20 {
            handle.tick(sensors: sensors, trainer: trainer, steering: steering)
        }
        let summary = handle.stopRide()
        XCTAssertNotNil(summary)
        XCTAssertGreaterThanOrEqual(summary?.sampleCount ?? 0, 20)
    }
}
