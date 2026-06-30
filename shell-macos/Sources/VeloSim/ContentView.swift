import AppKit
import SwiftUI
import QuartzCore
import Metal
import VeloFFI
import VeloSimSupport

struct ContentView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        HSplitView {
            MetalRideView(model: model)
                .frame(minWidth: 640)

            VStack(alignment: .leading, spacing: 12) {
                Text("VeloSim M3c")
                    .font(.title2.bold())

                Text("core v\(version())")
                    .foregroundStyle(.secondary)

                GroupBox("Route (M3)") {
                    HStack {
                        Button("Import GPX…") {
                            model.importGpxFile()
                        }
                        if model.activeRouteId != nil {
                            Button("Clear route") {
                                model.clearRoute()
                            }
                        }
                    }

                    Text(model.routeImportStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)

                    if !model.availableRoutes.isEmpty {
                        Picker("Active route", selection: Binding(
                            get: { model.activeRouteId ?? "" },
                            set: { id in
                                if id.isEmpty {
                                    model.clearRoute()
                                } else {
                                    model.selectRoute(id)
                                }
                            }
                        )) {
                            Text("None (flat)").tag("")
                            ForEach(model.availableRoutes, id: \.routeId) { route in
                                Text("\(route.name) (\(Int(route.totalDistanceM)) m)")
                                    .tag(route.routeId)
                            }
                        }
                    }

                    if model.activeRouteId != nil {
                        Toggle("3D Tiles mode (online)", isOn: Binding(
                            get: { model.tiles3dEnabled },
                            set: { model.setTiles3d($0) }
                        ))
                        .help("Streams photorealistic tiles during the ride. Online-only; attribution shown in HUD.")

                        if model.tiles3dEnabled, !model.tilesAttribution.isEmpty {
                            Text(model.tilesAttribution)
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                GroupBox("Bike (M3c)") {
                    HStack {
                        Button("Import photos…") {
                            model.importBikePhotos()
                        }
                        if model.activeBikeId != nil {
                            Button("Clear bike") {
                                model.clearBike()
                            }
                        }
                    }

                    Text(model.bikeImportStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)

                    if !model.availableBikes.isEmpty {
                        Picker("Active bike", selection: Binding(
                            get: { model.activeBikeId ?? "" },
                            set: { id in
                                if id.isEmpty {
                                    model.clearBike()
                                } else {
                                    model.selectBike(id)
                                }
                            }
                        )) {
                            Text("None (default)").tag("")
                            ForEach(model.availableBikes, id: \.bikeId) { bike in
                                Text(bike.name).tag(bike.bikeId)
                            }
                        }
                    }
                }

                GroupBox("Input") {
                    Picker("Sensors", selection: Binding(
                        get: { model.sensorMode },
                        set: { model.setSensorMode($0) }
                    )) {
                        ForEach(SensorInputMode.allCases) { mode in
                            Text(mode.label).tag(mode)
                        }
                    }
                    .pickerStyle(.segmented)

                    if model.sensorMode == .bluetooth {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("BLE: \(model.bleState)")
                            Text("Capabilities: \(model.bleCapabilities)")
                            Text("Trainer: \(model.bleTrainerStatus)")
                            if let err = model.bleControlError {
                                Text("CP error: \(err)")
                                    .foregroundStyle(.red)
                            }
                        }
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    }
                }

                GroupBox("Ride mode") {
                    Picker("Mode", selection: Binding(
                        get: { model.rideMode },
                        set: { model.applyRideMode($0) }
                    )) {
                        Text("ERG").tag(RideMode.erg)
                        Text("SIM").tag(RideMode.sim)
                        Text("Free").tag(RideMode.free)
                    }
                    .pickerStyle(.segmented)

                    if model.rideMode == .erg {
                        HStack {
                            Text("Target")
                            Slider(value: Binding(
                                get: { model.targetPower },
                                set: { model.applyTargetPower($0) }
                            ), in: 80...400, step: 5)
                            Text("\(Int(model.targetPower)) W")
                                .monospacedDigit()
                                .frame(width: 56, alignment: .trailing)
                        }
                    }

                    if model.rideMode == .sim {
                        if model.activeRouteId != nil {
                            Text(String(format: "Route grade: %.1f%%", model.rideState.grade * 100))
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        } else {
                            HStack {
                                Text("Grade")
                                Slider(value: Binding(
                                    get: { model.simGrade },
                                    set: { model.applySimGrade($0) }
                                ), in: -0.08...0.12, step: 0.005)
                                Text(String(format: "%.1f%%", model.simGrade * 100))
                                    .monospacedDigit()
                                    .frame(width: 56, alignment: .trailing)
                            }
                        }
                    }

                    if model.rideMode == .erg {
                        Text("Trainer target: \(Int(model.trainerLastPower)) W")
                            .font(.caption)
                    }
                    if model.rideMode == .sim {
                        Text(String(format: "SIM grade sent: %.1f%%", model.trainerSimGrade * 100))
                            .font(.caption)
                    }
                }

                GroupBox("Ride recording (M2b)") {
                    HStack {
                        if model.isRideRecording {
                            Label("Recording", systemImage: "record.circle.fill")
                                .foregroundStyle(.red)
                            Button("Stop & publish") {
                                model.stopRideAndPublish()
                            }
                            .disabled(model.isFinishingRide)
                        } else {
                            Button("Start ride") {
                                model.startRide()
                            }
                            .disabled(model.isFinishingRide)
                        }
                    }

                    Text(model.rideFlowStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    if let summary = model.lastRideSummary {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(String(format: "Last: %.0f m · %.0f s", summary.distanceM, summary.elapsedS))
                            if let avg = summary.avgPowerW {
                                Text(String(format: "Avg power: %.0f W", avg))
                            }
                        }
                        .font(.caption)
                    }

                    if let pub = model.lastPublishResult {
                        if pub.savedLocally {
                            Text("Saved: \(pub.activityUrl)")
                                .font(.caption2)
                                .lineLimit(2)
                        } else {
                            Text("Strava: \(pub.activityUrl)")
                                .font(.caption2)
                                .lineLimit(2)
                        }
                    }

                    if model.activityPublisher.isStravaConfigured {
                        HStack {
                            Text("Strava: \(model.stravaAuth.status)")
                                .font(.caption)
                            Button("Connect") {
                                model.stravaAuth.beginAuth()
                            }
                            .disabled(!model.activityPublisher.isStravaConfigured)
                        }
                    } else {
                        Text("Strava not configured — rides save to ~/Documents/VeloSim/rides/")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                GroupBox("Ride history") {
                    if model.rideHistory.isEmpty {
                        Text("No rides yet")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    } else {
                        ScrollView {
                            LazyVStack(alignment: .leading, spacing: 6) {
                                ForEach(model.rideHistory, id: \.id) { ride in
                                    Button {
                                        model.openRide(ride)
                                    } label: {
                                        HStack {
                                            VStack(alignment: .leading, spacing: 2) {
                                                Text(rideDate(ride.startedAtUnix))
                                                    .font(.caption.bold())
                                                Text(String(format: "%.0f m · %.0f s", ride.distanceM, ride.elapsedS))
                                                    .font(.caption2)
                                                if let avg = ride.avgPowerW {
                                                    Text(String(format: "Avg %.0f W", avg))
                                                        .font(.caption2)
                                                        .foregroundStyle(.secondary)
                                                }
                                            }
                                            Spacer()
                                            publishBadge(ride.publishStatus)
                                        }
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                        .frame(maxHeight: 140)
                    }
                }

                GroupBox("RideState") {
                    VStack(alignment: .leading, spacing: 4) {
                        stat("Power", value: model.rideState.powerW, suffix: "W")
                        stat("Cadence", value: model.rideState.cadenceRpm, suffix: "rpm")
                        stat("HR", value: model.rideState.heartRateBpm, suffix: "bpm")
                        Text(String(format: "Speed: %.1f km/h", model.rideState.speedMps * 3.6))
                        Text(String(format: "Distance: %.0f m", model.rideState.distanceM))
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                GroupBox("Rust log") {
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 2) {
                            ForEach(Array(model.logs.enumerated()), id: \.offset) { _, line in
                                Text(line)
                                    .font(.caption.monospaced())
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }
                    }
                    .frame(maxHeight: 160)
                }

                Spacer()
            }
            .padding()
            .frame(minWidth: 300, maxWidth: 380)
        }
        .onAppear {
            model.startSimLoop()
            model.refreshRideHistory()
        }
        .onDisappear { model.stopSimLoop() }
    }

    @ViewBuilder
    private func publishBadge(_ status: PublishStatus) -> some View {
        switch status {
        case .local:
            Text("Local")
                .font(.caption2)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(.quaternary)
                .clipShape(Capsule())
        case .strava:
            Text("Strava")
                .font(.caption2)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(.green.opacity(0.2))
                .clipShape(Capsule())
        case .failed:
            Text("Failed")
                .font(.caption2)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(.red.opacity(0.2))
                .clipShape(Capsule())
        }
    }

    private func rideDate(_ unix: UInt64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(unix))
        return date.formatted(date: .abbreviated, time: .shortened)
    }

    @ViewBuilder
    private func stat(_ label: String, value: Double?, suffix: String) -> some View {
        if let value {
            Text("\(label): \(Int(value)) \(suffix)")
        } else {
            Text("\(label): —")
                .foregroundStyle(.secondary)
        }
    }
}

struct MetalRideView: NSViewRepresentable {
    @ObservedObject var model: VeloSimModel

    func makeNSView(context: Context) -> MetalHostView {
        let view = MetalHostView()
        view.onLayerReady = { layer, size in
            model.initRenderer(layer: layer, size: size)
        }
        view.onResize = { size in
            model.resizeRenderer(size: size)
        }
        return view
    }

    func updateNSView(_ nsView: MetalHostView, context: Context) {}
}

final class MetalHostView: NSView {
    var onLayerReady: ((CAMetalLayer, CGSize) -> Void)?
    var onResize: ((CGSize) -> Void)?

    override func makeBackingLayer() -> CALayer {
        let layer = CAMetalLayer()
        layer.device = MTLCreateSystemDefaultDevice()
        layer.pixelFormat = .bgra8Unorm
        layer.framebufferOnly = false
        self.layer = layer
        self.wantsLayer = true
        return layer
    }

    override func layout() {
        super.layout()
        guard let metalLayer = layer as? CAMetalLayer else { return }
        let scale = window?.backingScaleFactor ?? 2.0
        let size = bounds.size
        metalLayer.drawableSize = CGSize(width: size.width * scale, height: size.height * scale)
        onResize?(metalLayer.drawableSize)
        onLayerReady?(metalLayer, metalLayer.drawableSize)
    }
}
