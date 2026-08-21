import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct PreRidePanel: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            inputSection
            steeringSection
            musicSection
            rideModeSection
            if let workout = model.armedWorkout, model.workoutLive.active {
                workoutSection(workout)
            }
            if model.activeRouteId != nil {
                tilesSection
            }
            startSection
        }
    }

    /// Colored duration-weighted interval preview of the armed workout, so the
    /// rider sees what they're about to start right next to the Start button.
    private func workoutSection(_ workout: WorkoutDto) -> some View {
        let totalS = workout.intervals.reduce(0) { $0 + $1.durationS }
        let blocks = workout.intervals.map { interval -> Double in
            switch interval.target {
            case let .ergWatts(watts): return model.ftp > 0 ? watts / model.ftp : 0.6
            case let .ftpPercent(percent): return percent / 100.0
            case .freeRide: return 0.6
            }
        }
        return VeloGlassSection("Workout") {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline) {
                    Text(workout.name)
                        .font(.subheadline.weight(.semibold))
                    Spacer()
                    Text("\(Int((totalS / 60).rounded())) min · TSS \(String(format: "%.0f", model.estimatedTss(for: workout)))")
                        .font(.caption)
                        .monospacedDigit()
                        .foregroundStyle(.secondary)
                }
                IntervalGraphPreview(
                    blocks: blocks,
                    weights: workout.intervals.map(\.durationS)
                )
                .frame(height: 44)
                Button("Remove workout") { model.clearWorkout() }
                    .buttonStyle(VeloGlassSecondaryButtonStyle())
            }
        }
    }

    private var inputSection: some View {
        VeloGlassSection("Trainer & sensors") {
            Picker("Input", selection: Binding(
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
        VeloGlassSection("Steering") {
            Picker("Steering", selection: Binding(
                get: { model.steeringMode },
                set: { model.setSteeringMode($0) }
            )) {
                ForEach(SteeringInputMode.allCases) { mode in
                    Text(mode.label).tag(mode)
                }
            }
            .pickerStyle(.segmented)

            if model.steeringMode == .airpods {
                Button("Recenter heading") { model.recenterSteering() }
                    .buttonStyle(VeloGlassSecondaryButtonStyle())
            }
        }
    }

    private var musicSection: some View {
        VeloGlassSection("Music") {
            Toggle("Segment music at intervals", isOn: Binding(
                get: { model.segmentMusicEnabled },
                set: { model.setSegmentMusicEnabled($0) }
            ))

            Button("Connect Apple Music") { model.connectAppleMusic() }
                .buttonStyle(VeloGlassSecondaryButtonStyle())

            Text(model.musicStatus)
                .font(.caption)
                .foregroundStyle(.secondary)
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

            if model.rideMode == .sim, model.activeRouteId == nil {
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
    }

    private var tilesSection: some View {
        VeloGlassSection("3D Tiles") {
            Toggle("Photorealistic tiles (online)", isOn: Binding(
                get: { model.tiles3dEnabled },
                set: { model.setTiles3d($0) }
            ))

            Text(model.tilesProviderStatus)
                .font(.caption2)
                .foregroundStyle(.secondary)

            if let err = model.tilesLastError, model.tiles3dEnabled {
                Text(err)
                    .font(.caption2)
                    .foregroundStyle(.orange)
                    .lineLimit(3)
            }
        }
    }

    private var startSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            readinessBanner

            Button("Start ride") {
                model.startRideFromActivities()
            }
            .buttonStyle(VeloGlassPrimaryButtonStyle())
            .disabled(model.isFinishingRide || model.preRideBlockReason != nil)
        }
    }

    @ViewBuilder
    private var readinessBanner: some View {
        let checks = model.preRideChecks
        if !checks.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                ForEach(checks) { check in
                    HStack(alignment: .firstTextBaseline, spacing: 6) {
                        Image(systemName: icon(for: check.severity))
                            .foregroundStyle(color(for: check.severity))
                            .font(.caption)
                        Text("\(check.label): \(check.detail)")
                            .font(.caption)
                            .foregroundStyle(check.severity == .ready ? .secondary : .primary)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
            }
        }
    }

    private func icon(for severity: PreRideValidation.Severity) -> String {
        switch severity {
        case .ready: return "checkmark.circle.fill"
        case .warning: return "exclamationmark.triangle.fill"
        case .blocked: return "xmark.octagon.fill"
        }
    }

    private func color(for severity: PreRideValidation.Severity) -> Color {
        switch severity {
        case .ready: return .green
        case .warning: return .orange
        case .blocked: return .red
        }
    }
}
