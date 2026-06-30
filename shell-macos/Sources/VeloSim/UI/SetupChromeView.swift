import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct SetupChromeView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            headerChrome
            rideActionBar

            Text(model.rideFlowStatus)
                .font(.caption)
                .foregroundStyle(.secondary)

            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    inputSection
                    steeringSection
                    musicSection
                    routeSection
                    bikeSection
                    workoutSection
                    rideModeSection
                    rideHistorySection
                    rideStateSection
                    rustLogSection
                }
            }
        }
        .padding()
        .frame(minWidth: 300, maxWidth: 380)
    }

    private var headerChrome: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("VeloSim M6")
                .font(.title2.bold())
            Text("core v\(version())")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloGlassRoundedRect(cornerRadius: 14)
    }

    private var rideActionBar: some View {
        VStack(alignment: .leading, spacing: 6) {
            VeloGlassContainer(spacing: 10) {
                HStack(spacing: 10) {
                    if model.isRideRecording {
                        Label("Recording", systemImage: "record.circle.fill")
                            .foregroundStyle(.red)
                            .font(.caption)
                        Button("Stop & publish") {
                            model.stopRideAndPublish()
                        }
                        .disabled(model.isFinishingRide)
                        .buttonStyle(VeloGlassPrimaryButtonStyle())
                    } else {
                        Button("Start ride") {
                            model.startRide()
                        }
                        .disabled(model.isFinishingRide)
                        .buttonStyle(VeloGlassPrimaryButtonStyle())
                    }

                    if model.activityPublisher.isStravaConfigured {
                        Button("Connect Strava") {
                            model.stravaAuth.beginAuth()
                        }
                        .buttonStyle(VeloGlassSecondaryButtonStyle())
                    }
                }
            }

            if model.activityPublisher.isStravaConfigured {
                Text("Strava: \(model.stravaAuth.status)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                Text("Strava not configured — rides save to ~/Documents/VeloSim/rides/")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var inputSection: some View {
        VeloGlassSection("Input") {
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
    }

    private var steeringSection: some View {
        VeloGlassSection("Steering (M6)") {
            Picker("Steering", selection: Binding(
                get: { model.steeringMode },
                set: { model.setSteeringMode($0) }
            )) {
                ForEach(SteeringInputMode.allCases) { mode in
                    Text(mode.label).tag(mode)
                }
            }
            .pickerStyle(.segmented)

            Text("Keyboard: ← → or A/D, Space to recenter. AirPods: head yaw on loaded routes.")
                .font(.caption)
                .foregroundStyle(.secondary)

            if model.steeringMode == .airpods {
                Button("Recenter heading") {
                    model.recenterSteering()
                }
                .buttonStyle(VeloGlassSecondaryButtonStyle())
            }

            if model.activeRouteId != nil, model.steeringMode != .off {
                HStack {
                    Text("Axis")
                    Spacer()
                    Text(String(format: "%.2f", model.rideState.steerAxis))
                        .monospacedDigit()
                    Text("Yaw")
                    Text(String(format: "%.2f°", model.rideState.steerYawRad * 180 / .pi))
                        .monospacedDigit()
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            } else if model.steeringMode != .off {
                Text("Load a route to enable steering offset.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var musicSection: some View {
        VeloGlassSection("Segment music (M6)") {
            Toggle("Shift music at workout intervals", isOn: Binding(
                get: { model.segmentMusicEnabled },
                set: { model.setSegmentMusicEnabled($0) }
            ))

            HStack {
                Button("Connect Apple Music") {
                    model.connectAppleMusic()
                }
                .buttonStyle(VeloGlassSecondaryButtonStyle())
            }

            Text(model.musicStatus)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(2)

            Text("Playback control only — MusicKit queues playlists by interval energy.")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
    }

    private var routeSection: some View {
        VeloGlassSection("Route (M3)") {
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
    }

    private var bikeSection: some View {
        VeloGlassSection("Bike (M3c)") {
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
    }

    private var workoutSection: some View {
        VeloGlassSection("Workout (M5)") {
            HStack {
                Text("FTP")
                Slider(value: Binding(
                    get: { model.ftp },
                    set: { model.applyFtp($0) }
                ), in: 100...400, step: 5)
                Text("\(Int(model.ftp)) W")
                    .monospacedDigit()
                    .frame(width: 56, alignment: .trailing)
            }

            WorkoutBuilderView(model: model)
        }
    }

    private var rideModeSection: some View {
        VeloGlassSection("Ride mode") {
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
    }

    private var rideHistorySection: some View {
        VeloGlassSection("Ride history") {
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
                                        Text(RideSummaryFormatting.formatRideDate(ride.startedAtUnix))
                                            .font(.caption.bold())
                                        Text("\(RideSummaryFormatting.formatDistance(ride.distanceM)) · \(RideSummaryFormatting.formatElapsed(ride.elapsedS))")
                                            .font(.caption2)
                                        if let avg = ride.avgPowerW {
                                            Text("Avg \(RideSummaryFormatting.formatPower(avg))")
                                                .font(.caption2)
                                                .foregroundStyle(.secondary)
                                        }
                                    }
                                    Spacer()
                                    VeloPublishBadge(status: ride.publishStatus)
                                }
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
                .frame(maxHeight: 140)
            }
        }
    }

    private var rideStateSection: some View {
        VeloGlassSection("RideState") {
            VStack(alignment: .leading, spacing: 4) {
                stat("Power", value: model.rideState.powerW, suffix: "W")
                stat("Cadence", value: model.rideState.cadenceRpm, suffix: "rpm")
                stat("HR", value: model.rideState.heartRateBpm, suffix: "bpm")
                Text(String(format: "Speed: %.1f km/h", model.rideState.speedMps * 3.6))
                Text(String(format: "Distance: %.0f m", model.rideState.distanceM))
            }
        }
    }

    private var rustLogSection: some View {
        VeloGlassSection("Rust log") {
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

struct SetupChromeView_Previews: PreviewProvider {
    static var previews: some View {
        SetupChromeView(model: VeloSimModel())
            .frame(width: 360, height: 900)
    }
}
