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
