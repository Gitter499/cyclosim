import AppKit
import Foundation
import QuartzCore
import VeloFFI
import VeloSimSupport

@MainActor
final class VeloSimModel: ObservableObject {
    let handle: VeloHandle
    let mediaCapture = VeloMediaCapture()
    let activityPublisher = VeloActivityPublisher()
    let stravaAuth = StravaAuthCoordinator()

    private let fakeSensors = FakeSensorSource()
    private let replaySensors = ReplaySensorSource()
    private let ftmsBridge = FTMSBridge()
    private let loggingTrainer = LoggingTrainerControl()

    @Published var toggleCount: UInt32 = 0
    @Published var logs: [String] = []
    @Published var rideState: RideStateDto
    @Published var trainerLastPower: Double = 0
    @Published var trainerSimGrade: Double = 0
    @Published var rendererReady = false
    @Published var sensorMode: SensorInputMode = .replay
    @Published var rideMode: RideMode = .erg
    @Published var targetPower: Double = 180
    @Published var simGrade: Double = 0.0
    @Published var bleState: String = "idle"
    @Published var bleCapabilities: String = "—"
    @Published var bleTrainerStatus: String = "—"
    @Published var bleControlError: String?

    @Published var isRideRecording = false
    @Published var lastRideSummary: RideSummaryDto?
    @Published var lastPublishResult: PublishResultDto?
    @Published var rideFlowStatus: String = "idle"
    @Published var isFinishingRide = false
    @Published var rideHistory: [RideRecordDto] = []

    private var tickTimer: Timer?
    private var rideStore: LocalRideStoreHandle?

    init() {
        handle = VeloHandle()
        rideState = handle.rideState()
        toggleCount = handle.toggleCount()
        isRideRecording = handle.isRideRecording()
        handle.setRideMode(mode: .erg)
        handle.setTargetPower(watts: 180)

        if let library = try? LocalRideStore.defaultLibrary() {
            rideStore = LocalRideStore.open(library: library)
        }
        refreshRideHistory()

        ftmsBridge.onStateChange = { [weak self] state in
            Task { @MainActor in
                self?.bleState = state
            }
        }
        ftmsBridge.onCapabilitiesChange = { [weak self] caps in
            Task { @MainActor in
                self?.bleCapabilities = caps.description
            }
        }
        ftmsBridge.onTrainerStatusChange = { [weak self] status in
            Task { @MainActor in
                self?.bleTrainerStatus = status
            }
        }
        ftmsBridge.onControlErrorChange = { [weak self] error in
            Task { @MainActor in
                self?.bleControlError = error
            }
        }
    }

    func toggle() {
        toggleCount = handle.toggle()
    }

    func applyRideMode(_ mode: RideMode) {
        rideMode = mode
        handle.setRideMode(mode: mode)
    }

    func applyTargetPower(_ watts: Double) {
        targetPower = watts
        handle.setTargetPower(watts: watts)
    }

    func applySimGrade(_ grade: Double) {
        simGrade = grade
        handle.setGrade(grade: grade)
    }

    func setSensorMode(_ mode: SensorInputMode) {
        sensorMode = mode
        if mode == .bluetooth {
            ftmsBridge.startScanning()
        } else {
            ftmsBridge.stopScanning()
            ftmsBridge.disconnect()
        }
    }

    func startRide() {
        guard !isRideRecording else { return }
        handle.startRide()
        isRideRecording = handle.isRideRecording()
        lastPublishResult = nil
        rideFlowStatus = "recording"
    }

    func stopRideAndPublish() {
        guard isRideRecording, !isFinishingRide else { return }
        isFinishingRide = true
        rideFlowStatus = "exporting FIT + screenshot…"

        Task { @MainActor in
            defer { isFinishingRide = false }
            do {
                let result = try handle.finishRideAndPublish(
                    media: mediaCapture,
                    publisher: activityPublisher
                )
                lastPublishResult = result
                lastRideSummary = handle.lastRideSummary()
                isRideRecording = handle.isRideRecording()
                rideFlowStatus = result.savedLocally
                    ? "saved locally"
                    : "uploaded to Strava"
                refreshRideHistory()
                logs = handle.recentLogs(limit: 12)
            } catch {
                rideFlowStatus = "finish failed: \(error)"
            }
        }
    }

    func startSimLoop() {
        tickTimer?.invalidate()
        tickTimer = Timer.scheduledTimer(withTimeInterval: 1.0 / 30.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.simTick()
            }
        }
    }

    func stopSimLoop() {
        tickTimer?.invalidate()
        tickTimer = nil
    }

    func initRenderer(layer: CAMetalLayer, size: CGSize) {
        let ptr = UInt64(bitPattern: Int64(Int(bitPattern: Unmanaged.passUnretained(layer).toOpaque())))
        do {
            try handle.initRenderer(
                metalLayerPtr: ptr,
                width: UInt32(max(size.width, 1)),
                height: UInt32(max(size.height, 1))
            )
            rendererReady = true
        } catch {
            rendererReady = false
            logs.append("renderer init failed: \(error)")
        }
    }

    func resizeRenderer(size: CGSize) {
        guard rendererReady else { return }
        try? handle.resizeRenderer(
            width: UInt32(max(size.width, 1)),
            height: UInt32(max(size.height, 1))
        )
    }

    func renderFrame() {
        guard rendererReady else { return }
        try? handle.renderFrame()
    }

    func handleOAuthCallback(url: URL) {
        Task {
            await stravaAuth.handleCallback(url: url, publisher: activityPublisher)
        }
    }

    func openRide(_ record: RideRecordDto) {
        if let url = rideStore?.stravaURL(for: record) {
            NSWorkspace.shared.open(url)
        } else {
            rideStore?.openInFinder(record)
        }
    }

    func refreshRideHistory() {
        if let store = rideStore {
            rideHistory = (try? store.listRides()) ?? []
        } else {
            rideHistory = (try? handle.listRides()) ?? []
        }
    }

    private func simTick() {
        switch sensorMode {
        case .fake:
            handle.tick(sensors: fakeSensors, trainer: activeTrainer())
        case .replay:
            handle.tick(sensors: replaySensors, trainer: loggingTrainer)
        case .bluetooth:
            handle.tick(sensors: ftmsBridge, trainer: ftmsBridge)
        }

        toggleCount = handle.toggleCount()
        rideState = handle.rideState()
        isRideRecording = handle.isRideRecording()
        switch sensorMode {
        case .fake, .replay:
            trainerLastPower = loggingTrainer.lastTargetPower
            trainerSimGrade = loggingTrainer.lastGrade
        case .bluetooth:
            trainerLastPower = ftmsBridge.lastTargetPower
            trainerSimGrade = ftmsBridge.lastSimGrade
        }
        logs = handle.recentLogs(limit: 12)
        renderFrame()
    }

    private func activeTrainer() -> TrainerControlCallback {
        switch sensorMode {
        case .bluetooth: return ftmsBridge
        default: return loggingTrainer
        }
    }

    deinit {
        tickTimer?.invalidate()
        ftmsBridge.disconnect()
    }
}
