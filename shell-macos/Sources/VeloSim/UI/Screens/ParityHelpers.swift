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
                controlButton("Lap", systemImage: "flag.fill") { model.markLap() }
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
    var ergBiasPct: Double = 100.0
    var ftp: Int = 0
    var onBiasDown: (() -> Void)?
    var onBiasUp: (() -> Void)?
    var onSkip: (() -> Void)?
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            VStack(spacing: Tok.s2) {
                HStack(spacing: Tok.s4) {
                    VStack(alignment: .leading, spacing: Tok.s1) {
                        Text(workout.blockName)
                            .font(Typo.label())
                            .foregroundStyle(.secondary)
                        // Zone color codes power elsewhere on the HUD, so
                        // on-target state stays neutral (hud-design skill §4).
                        Text("\(workout.actualWatts) / \(workout.targetWatts) W")
                            .font(Typo.metric())
                            .monospacedDigit()
                            .contentTransition(reduceMotion ? .identity : .numericText())
                            .foregroundStyle(.white)
                        if let next = workout.nextBlockName {
                            Text("Next · \(next)")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                    Spacer()
                    if onBiasDown != nil || onBiasUp != nil || onSkip != nil {
                        HStack(spacing: Tok.s2) {
                            biasButton("minus", label: "Lower target") { onBiasDown?() }
                            Text(String(format: "%.0f%%", ergBiasPct))
                                .font(Typo.label())
                                .monospacedDigit()
                                .foregroundStyle(ergBiasPct == 100 ? Color.secondary : Color.primary)
                                .frame(minWidth: 40)
                                .accessibilityLabel("ERG bias \(Int(ergBiasPct)) percent")
                            biasButton("plus", label: "Raise target") { onBiasUp?() }
                            biasButton("forward.end.fill", label: "Skip interval") { onSkip?() }
                        }
                    }
                    Text(HUDDurationFormat.mmss(seconds: workout.intervalRemainingS))
                        .font(Typo.metric())
                        .monospacedDigit()
                        .contentTransition(reduceMotion ? .identity : .numericText())
                }

                intervalProgressBar
            }
            .padding(Tok.s4)
            .hudSurface(RoundedRectangle(cornerRadius: Tok.rCard), reduceTransparency: reduceTransparency)
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(
            "Workout \(workout.blockName), \(workout.actualWatts) of \(workout.targetWatts) watts"
        )
    }

    /// Track + fill per hud-design skill: white 12% track, fill in the zone
    /// color of the *target* watts. Gauges may animate at frame rate.
    private var intervalProgressBar: some View {
        GeometryReader { geo in
            let zone = PowerZone.of(watts: workout.targetWatts, ftp: ftp)
            ZStack(alignment: .leading) {
                Capsule()
                    .fill(.white.opacity(0.12))
                Capsule()
                    .fill(zone.color)
                    .frame(width: max(0, geo.size.width * workout.intervalProgress))
            }
        }
        .frame(height: 5)
        .allowsHitTesting(false)
    }
    private func biasButton(_ systemImage: String, label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: systemImage)
                .font(.caption.weight(.semibold))
        }
        .buttonBorderShape(.circle)
        .controlSize(.small)
        .accessibilityLabel(label)
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
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        NavigationStack {
            List {
                pairRow(
                    role: "Power / Trainer",
                    connected: model.bleDevices.trainerConnected,
                    detail: model.bleDevices.trainerName
                )
                pairRow(
                    role: "Cadence",
                    connected: model.bleDevices.trainerConnected,
                    detail: model.bleDevices.trainerConnected ? "via trainer" : nil
                )
                pairRow(
                    role: "Heart Rate",
                    connected: model.bleDevices.hrConnected,
                    detail: hrDetail
                )
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

    private var hrDetail: String? {
        guard model.bleDevices.hrConnected else { return nil }
        let name = model.bleDevices.hrName ?? "HR strap"
        if let bpm = model.bleDevices.latestHeartRateBpm {
            return "\(name) · \(bpm) bpm"
        }
        return name
    }

    private func pairRow(role: String, connected: Bool, detail: String? = nil) -> some View {
        HStack {
            Label(role, systemImage: connected ? "checkmark.circle.fill" : "dot.radiowaves.left.and.right")
                .foregroundStyle(connected ? .green : .secondary)
            Spacer()
            Text(detail ?? (connected ? "Connected" : "Searching…"))
                .foregroundStyle(.secondary)
                .monospacedDigit()
                .contentTransition(reduceMotion ? .identity : .numericText())
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
                            RouteElevationSparkline(samples: model.routeProfiles[route.routeId])
                                .frame(width: 72, height: 28)
                                .onAppear { model.loadRouteProfile(route.routeId) }

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
    /// Real elevation samples (m); nil renders a flat placeholder while loading.
    private let samples: [CGFloat]

    init(samples: [Double]?) {
        guard let samples, samples.count > 1,
              let minE = samples.min(), let maxE = samples.max(), maxE > minE
        else {
            self.samples = Array(repeating: 0.4, count: 24)
            return
        }
        let range = maxE - minE
        self.samples = samples.map { CGFloat(0.1 + 0.8 * (($0 - minE) / range)) }
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
        // Decorative: route rows carry name/distance as text.
        .accessibilityHidden(true)
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

            workoutRow(for: model.sampleWorkout) {
                model.startSampleWorkout()
            }

            Text("Custom workouts")
                .font(.headline)
                .padding(.top, Tok.s2)

            WorkoutBuilderView(model: model)
        }
    }

    /// Row metadata computed from the real workout definition (#49).
    private func workoutRow(for workout: WorkoutDto, action: @escaping () -> Void) -> some View {
        let totalS = workout.intervals.reduce(0) { $0 + $1.durationS }
        let blocks = workout.intervals.map { interval -> Double in
            switch interval.target {
            case let .ergWatts(watts): return model.ftp > 0 ? watts / model.ftp : 0.6
            case let .ftpPercent(percent): return percent / 100.0
            case .freeRide: return 0.6
            }
        }
        return workoutRow(
            name: workout.name,
            duration: "\(Int((totalS / 60).rounded())) min",
            tss: String(format: "%.0f", model.estimatedTss(for: workout)),
            blocks: blocks,
            weights: workout.intervals.map(\.durationS),
            action: action
        )
    }

    private func workoutRow(
        name: String,
        duration: String,
        tss: String,
        blocks: [Double],
        weights: [Double]? = nil,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: Tok.s3) {
                IntervalGraphPreview(blocks: blocks, weights: weights)
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
                    .foregroundStyle(Color.accentColor)
            }
            .padding(Tok.s3)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: Tok.rTile))
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(name), \(duration), TSS \(tss). Starts the workout.")
    }
}

struct IntervalGraphPreview: View {
    let blocks: [Double]
    /// Relative bar widths (interval durations). nil → equal widths.
    var weights: [Double]?

    var body: some View {
        GeometryReader { geo in
            let n = blocks.count
            let total = weights.map { $0.reduce(0, +) } ?? Double(n)
            let available = geo.size.width - CGFloat(max(0, n - 1))
            HStack(spacing: 1) {
                ForEach(Array(blocks.enumerated()), id: \.offset) { i, pct in
                    let weight = weights.flatMap { $0.indices.contains(i) ? $0[i] : nil } ?? 1
                    let width = total > 0 ? available * CGFloat(weight / total) : 0
                    RoundedRectangle(cornerRadius: 2)
                        .fill(PowerZone.of(watts: Int(pct * 250), ftp: 250).color.opacity(0.85))
                        .frame(width: max(2, width),
                               height: geo.size.height * CGFloat(min(1, pct)))
                        .frame(maxHeight: .infinity, alignment: .bottom)
                }
            }
        }
        // Decorative: the row's name/duration/TSS text carries the info.
        .accessibilityHidden(true)
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
