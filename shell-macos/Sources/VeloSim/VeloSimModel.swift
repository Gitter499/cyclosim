import AppKit
import Foundation
import QuartzCore
import UniformTypeIdentifiers
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

    @Published var availableRoutes: [RouteInfoDto] = []
    @Published var activeRouteId: String?
    @Published var routeImportStatus: String = ""
    @Published var tiles3dEnabled: Bool = false
    @Published var tilesAttribution: String = ""

    @Published var availableBikes: [BikeInfoDto] = []
    @Published var activeBikeId: String?
    @Published var bikeImportStatus: String = ""

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
        refreshRoutes()
        refreshBikes()

        ftmsBridge.onStateChange = { [weak self] state in
            Task { @MainActor [weak self] in
                self?.bleState = state
            }
        }
        ftmsBridge.onCapabilitiesChange = { [weak self] caps in
            Task { @MainActor [weak self] in
                self?.bleCapabilities = caps.description
            }
        }
        ftmsBridge.onTrainerStatusChange = { [weak self] status in
            Task { @MainActor [weak self] in
                self?.bleTrainerStatus = status
            }
        }
        ftmsBridge.onControlErrorChange = { [weak self] error in
            Task { @MainActor [weak self] in
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
        if activeRouteId == nil {
            handle.setGrade(grade: grade)
        }
    }

    func refreshRoutes() {
        availableRoutes = (try? handle.listRoutes()) ?? []
        activeRouteId = handle.activeRouteId()
    }

    func refreshBikes() {
        availableBikes = (try? handle.listBikes()) ?? []
        activeBikeId = handle.activeBikeId()
    }

    func selectBike(_ bikeId: String) {
        do {
            try handle.setActiveBike(bikeId: bikeId)
            activeBikeId = bikeId
            bikeImportStatus = "Riding: \(bikeId)"
        } catch {
            bikeImportStatus = "Failed to load bike: \(error)"
        }
    }

    func clearBike() {
        handle.clearActiveBike()
        activeBikeId = nil
        bikeImportStatus = "No custom bike"
    }

    func importBikePhotos() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.png, .jpeg]
        panel.allowsMultipleSelection = true
        panel.canChooseDirectories = false
        panel.message = "Select 1–4 bike photos"
        guard panel.runModal() == .OK else { return }

        let urls = panel.urls
        guard (1...4).contains(urls.count) else {
            bikeImportStatus = "Need 1–4 images"
            return
        }

        let stem = urls[0].deletingPathExtension().lastPathComponent
            .lowercased()
            .replacingOccurrences(of: " ", with: "-")
        let bikeId = stem.isEmpty ? "my-bike" : stem
        bikeImportStatus = "Importing \(bikeId)…"

        do {
            try handle.importBikeFromImages(
                imagePaths: urls.map(\.path),
                bikeId: bikeId,
                name: urls[0].deletingPathExtension().lastPathComponent
            )
            activeBikeId = bikeId
            bikeImportStatus = "Imported \(bikeId)"
            refreshBikes()
        } catch {
            bikeImportStatus = "Import failed: \(error)"
        }
    }

    func selectRoute(_ routeId: String) {
        do {
            try handle.setActiveRoute(routeId: routeId)
            activeRouteId = routeId
            tiles3dEnabled = handle.routeTiles3dEnabled()
            tilesAttribution = handle.tilesAttribution()
            routeImportStatus = "Riding: \(routeId)"
            rideState = handle.rideState()
            simGrade = rideState.grade
        } catch {
            routeImportStatus = "Failed to load route: \(error)"
        }
    }

    func setTiles3d(_ enabled: Bool) {
        do {
            try handle.setRouteTiles3d(enabled: enabled)
            tiles3dEnabled = enabled
            tilesAttribution = handle.tilesAttribution()
        } catch {
            routeImportStatus = "3D Tiles toggle failed: \(error)"
        }
    }

    func clearRoute() {
        handle.clearActiveRoute()
        activeRouteId = nil
        tiles3dEnabled = false
        tilesAttribution = ""
        routeImportStatus = "Flat plane (no route)"
        refreshRoutes()
    }

    func importGpxFile() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [UTType(filenameExtension: "gpx") ?? .xml]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.message = "Select a GPX route file"
        guard panel.runModal() == .OK, let url = panel.url else { return }

        let routeId = url.deletingPathExtension().lastPathComponent
            .lowercased()
            .replacingOccurrences(of: " ", with: "-")
        routeImportStatus = "Importing \(routeId)…"

        do {
            try handle.importGpxRoute(
                gpxPath: url.path,
                routeId: routeId,
                name: url.deletingPathExtension().lastPathComponent
            )
            activeRouteId = routeId
            routeImportStatus = "Imported \(routeId)"
            refreshRoutes()
            rideState = handle.rideState()
            simGrade = rideState.grade
        } catch {
            routeImportStatus = "Import failed: \(error)"
        }
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
            Task { @MainActor [weak self] in
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
