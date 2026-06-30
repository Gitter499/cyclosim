import VeloFFI

/// Minimal trainer mock for FFI ride flow tests.
final class MockTrainerControl: TrainerControlCallback, @unchecked Sendable {
    func setTargetPower(watts: Double) {}
    func setSimulation(grade: Double, crr: Double, cwa: Double) {}
    func stop() {}
}
