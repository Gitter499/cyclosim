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
            if model.activeRouteId != nil {
                tilesSection
            }
            startSection
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
            if let reason = model.preRideBlockReason {
                Text(reason)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Button("Start ride") {
                model.startRideFromActivities()
            }
            .buttonStyle(VeloGlassPrimaryButtonStyle())
            .disabled(model.isFinishingRide || model.preRideBlockReason != nil)
        }
    }
}
