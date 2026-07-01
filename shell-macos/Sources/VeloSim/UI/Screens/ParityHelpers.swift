import SwiftUI
import VeloFFI
import VeloSimSupport

// MARK: - Ride controls (§7.5)

@MainActor
struct RideControlCluster: View {
    @ObservedObject var model: VeloSimModel
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency

    var body: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            VStack(spacing: Tok.s2) {
                controlButton("Pause", systemImage: "pause.fill") { model.pauseRide() }
                controlButton(
                    model.chaseCameraWide ? "Narrow" : "Wide",
                    systemImage: "camera.aperture"
                ) { model.toggleChaseCamera() }
                controlButton("Shot", systemImage: "camera") { model.captureRideScreenshot() }
                controlButton("U-turn", systemImage: "arrow.uturn.backward") { model.requestUTurn() }
            }
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Ride controls")
    }

    private func controlButton(_ title: String, systemImage: String, action: @escaping () -> Void) -> some View {
        Button(title, systemImage: systemImage, action: action)
            .font(.caption.weight(.semibold))
            .labelStyle(.iconOnly)
            .buttonBorderShape(.circle)
            .controlSize(.large)
            .hudSurface(Circle(), reduceTransparency: reduceTransparency)
            .accessibilityLabel(title)
    }
}

// MARK: - Workout bar (§6.2 / §7.5)

@MainActor
struct WorkoutBarView: View {
    let workout: WorkoutHUD
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency

    var body: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            HStack(spacing: Tok.s4) {
                VStack(alignment: .leading, spacing: Tok.s1) {
                    Text(workout.blockName)
                        .font(Typo.label())
                        .foregroundStyle(.secondary)
                    Text("\(workout.actualWatts) / \(workout.targetWatts) W")
                        .font(Typo.metric())
                        .monospacedDigit()
                        .contentTransition(.numericText())
                        .foregroundStyle(
                            abs(workout.actualWatts - workout.targetWatts) <= 10 ? Color.green : Color.primary
                        )
                }
                Spacer()
                Text(HUDDurationFormat.mmss(seconds: workout.intervalRemainingS))
                    .font(Typo.metric())
                    .monospacedDigit()
                    .contentTransition(.numericText())
            }
            .padding(Tok.s4)
            .hudSurface(RoundedRectangle(cornerRadius: Tok.rCard), reduceTransparency: reduceTransparency)
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(
            "Workout \(workout.blockName), \(workout.actualWatts) of \(workout.targetWatts) watts"
        )
    }
}

// MARK: - Pause menu (§7.6)

@MainActor
struct PauseMenuOverlay: View {
    @ObservedObject var model: VeloSimModel
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency

    var body: some View {
        ZStack {
            Color.black.opacity(0.45)
                .ignoresSafeArea()

            VStack(spacing: Tok.s4) {
                Text("Paused")
                    .font(.title2.bold())

                Button("Resume") { model.resumeRide() }
                    .buttonStyle(VeloGlassPrimaryButtonStyle())

                Button("End ride") { model.stopRideAndPublish() }
                    .buttonStyle(VeloGlassSecondaryButtonStyle())
                    .disabled(model.isFinishingRide)

                Button("Discard", role: .destructive) { model.discardRide() }
                    .buttonStyle(.plain)
            }
            .padding(Tok.s6)
            .hudSurface(RoundedRectangle(cornerRadius: Tok.rCard), reduceTransparency: reduceTransparency)
        }
    }
}

// MARK: - Home quick start (§7.1)

@MainActor
struct QuickStartRow: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VeloGlassContainer(spacing: Tok.s3) {
            HStack(spacing: Tok.s3) {
                veloGlassProminentButton("Just Ride", systemImage: "bicycle") {
                    model.beginJustRide()
                }
                veloGlassButton("Workout", systemImage: "list.bullet.rectangle") {
                    model.shellDestination = .activities
                    model.activitiesTab = .workouts
                }
                veloGlassButton("FTP Test", systemImage: "gauge.high") {
                    model.showFTPTestPicker = true
                }
                veloGlassButton("Route", systemImage: "map") {
                    model.shellDestination = .activities
                    model.activitiesTab = .routes
                }
            }
        }
    }
}

// MARK: - Pairing (§7.2)

@MainActor
struct PairingView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        NavigationStack {
            List {
                pairRow(role: "Power / Trainer", connected: model.sensorMode == .bluetooth && model.bleState.contains("connected"))
                pairRow(role: "Cadence", connected: false)
                pairRow(role: "Heart Rate", connected: false)
            }
            .navigationTitle("Pair devices")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") { model.showPairingSheet = false }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Ride") {
                        model.showPairingSheet = false
                        model.beginJustRide()
                    }
                    .disabled(model.sensorMode != .bluetooth)
                }
            }
        }
        .frame(minWidth: 420, minHeight: 360)
    }

    private func pairRow(role: String, connected: Bool) -> some View {
        HStack {
            Label(role, systemImage: connected ? "checkmark.circle.fill" : "dot.radiowaves.left.and.right")
                .foregroundStyle(connected ? .green : .secondary)
            Spacer()
            Text(connected ? "Connected" : "Searching…")
                .foregroundStyle(.secondary)
                .monospacedDigit()
            Button(connected ? "Change" : "Connect") {
                model.setSensorMode(.bluetooth)
            }
            .buttonStyle(VeloGlassSecondaryButtonStyle())
            .controlSize(.small)
        }
        .padding(.vertical, Tok.s1)
    }
}

// MARK: - Route select with sparkline stub (§7.3)

@MainActor
struct RouteSelectView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: Tok.s3) {
            if model.availableRoutes.isEmpty {
                Text("Import a GPX route to ride with elevation.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(model.availableRoutes, id: \.routeId) { route in
                    Button {
                        model.selectRoute(route.routeId)
                    } label: {
                        HStack(spacing: Tok.s3) {
                            RouteElevationSparkline(routeId: route.routeId)
                                .frame(width: 72, height: 28)

                            VStack(alignment: .leading, spacing: 2) {
                                Text(route.name)
                                    .font(.subheadline.weight(.semibold))
                                    .foregroundStyle(.primary)
                                Text("\(Int(route.totalDistanceM / 1000)) km")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()
                            }

                            Spacer()

                            if model.activeRouteId == route.routeId {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundStyle(.green)
                            }
                        }
                        .padding(.vertical, Tok.s2)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }
}

struct RouteElevationSparkline: View {
    let routeId: String

    private var samples: [CGFloat] {
        var hash: UInt64 = 14695981039346656037
        for byte in routeId.utf8 {
            hash = (hash ^ UInt64(byte)) &* 1099511628211
        }
        return (0 ..< 24).map { i in
            let seed = Double((hash &+ UInt64(i * 17)) % 1000) / 1000.0
            return CGFloat(0.2 + seed * 0.6)
        }
    }

    var body: some View {
        GeometryReader { geo in
            Path { path in
                let w = geo.size.width
                let h = geo.size.height
                let step = w / CGFloat(max(samples.count - 1, 1))
                for (i, y) in samples.enumerated() {
                    let x = CGFloat(i) * step
                    let py = h * (1 - y)
                    if i == 0 { path.move(to: CGPoint(x: x, y: py)) }
                    else { path.addLine(to: CGPoint(x: x, y: py)) }
                }
            }
            .stroke(Color.accentColor, lineWidth: 1.5)
        }
    }
}

// MARK: - Workout library stub (§7.4)

@MainActor
struct WorkoutLibraryView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: Tok.s4) {
            Text("FTP Tests")
                .font(.headline)

            workoutRow(
                name: "2x20 Threshold",
                duration: "60 min",
                tss: "~65",
                blocks: [0.55, 0.75, 1.0, 0.55, 1.0, 0.55]
            ) {
                model.startSampleWorkout()
            }

            Text("Custom workouts")
                .font(.headline)
                .padding(.top, Tok.s2)

            WorkoutBuilderView(model: model)
        }
    }

    private func workoutRow(
        name: String,
        duration: String,
        tss: String,
        blocks: [Double],
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: Tok.s3) {
                IntervalGraphPreview(blocks: blocks)
                    .frame(width: 96, height: 32)

                VStack(alignment: .leading, spacing: 2) {
                    Text(name)
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(.primary)
                    Text("\(duration) · TSS \(tss)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Image(systemName: "play.circle.fill")
                    .foregroundStyle(.orange)
            }
            .padding(Tok.s3)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: Tok.rTile))
        }
        .buttonStyle(.plain)
    }
}

struct IntervalGraphPreview: View {
    let blocks: [Double]

    var body: some View {
        GeometryReader { geo in
            HStack(spacing: 1) {
                ForEach(Array(blocks.enumerated()), id: \.offset) { _, pct in
                    RoundedRectangle(cornerRadius: 2)
                        .fill(PowerZone.of(watts: Int(pct * 250), ftp: 250).color.opacity(0.85))
                        .frame(height: geo.size.height * CGFloat(min(1, pct)))
                        .frame(maxHeight: .infinity, alignment: .bottom)
                }
            }
        }
    }
}

// MARK: - FTP test picker (§6.3)

@MainActor
struct FTPTestPickerView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        NavigationStack {
            List(RampTestEngine.ProtocolKind.allCases) { kind in
                Button {
                    model.startFTPTest(kind)
                    model.showFTPTestPicker = false
                } label: {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(kind.rawValue)
                            .font(.headline)
                        Text(kind.subtitle)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("FTP Test")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { model.showFTPTestPicker = false }
                }
            }
        }
        .frame(minWidth: 440, minHeight: 320)
    }
}

@MainActor
struct FTPAnnouncementSheet: View {
    @ObservedObject var model: VeloSimModel
    let oldFTP: Int
    let newFTP: Int

    var body: some View {
        VStack(spacing: Tok.s4) {
            Text("New FTP set!")
                .font(.title.bold())
            Text("\(oldFTP) → \(newFTP) W")
                .font(Typo.metric())
                .monospacedDigit()
            Button("Done") { model.pendingFTPAnnouncement = nil }
                .buttonStyle(VeloGlassPrimaryButtonStyle())
        }
        .padding(Tok.s6)
        .frame(minWidth: 320)
    }
}
