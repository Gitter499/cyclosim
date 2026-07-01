import SwiftUI
import VeloFFI
import VeloSimSupport

/// Zwift-style metrics overlay on the Metal viewport (not sidebar chrome).
@MainActor
struct RideHUDOverlay: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        ZStack {
            VStack {
                topRow
                Spacer()
                bottomRow
            }
            .padding(20)

            if let banner = RideHUDFormatting.workoutBanner(live: model.workoutLive) {
                VStack {
                    Text(banner)
                        .font(.headline.weight(.semibold))
                        .monospacedDigit()
                        .padding(.horizontal, 14)
                        .padding(.vertical, 8)
                        .background(.black.opacity(0.45), in: Capsule())
                    Spacer()
                }
                .padding(.top, 16)
            }
        }
        .allowsHitTesting(false)
    }

    private var topRow: some View {
        HStack(alignment: .top) {
            hudMetric(title: "POWER", value: RideHUDFormatting.formatPower(model.rideState.powerW))
                .frame(maxWidth: .infinity, alignment: .leading)
            hudMetric(title: "HR", value: RideHUDFormatting.formatHeartRate(model.rideState.heartRateBpm))
                .frame(maxWidth: .infinity, alignment: .trailing)
        }
    }

    private var bottomRow: some View {
        VStack(spacing: 8) {
            if let hint = RideHUDFormatting.steeringHint(
                mode: model.steeringMode,
                routeLoaded: model.activeRouteId != nil
            ) {
                Text(hint)
                    .font(.caption2)
                    .foregroundStyle(.white.opacity(0.75))
            }

            HStack(alignment: .bottom) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(RideHUDFormatting.formatSpeedKmh(model.rideState.speedMps))
                        .font(.title3.weight(.semibold))
                        .monospacedDigit()
                    Text(RideHUDFormatting.formatDistance(model.rideState.distanceM))
                        .font(.caption)
                        .monospacedDigit()
                        .foregroundStyle(.white.opacity(0.85))
                }
                Spacer()
                VStack(alignment: .trailing, spacing: 4) {
                    Text(RideHUDFormatting.formatCadence(model.rideState.cadenceRpm))
                        .font(.title3.weight(.semibold))
                        .monospacedDigit()
                    Text(RideHUDFormatting.formatElapsed(model.rideState.elapsedS))
                        .font(.caption)
                        .monospacedDigit()
                        .foregroundStyle(.white.opacity(0.85))
                }
            }
        }
    }

    private func hudMetric(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.caption2.weight(.bold))
                .foregroundStyle(.white.opacity(0.7))
            Text(value)
                .font(.system(size: 36, weight: .bold, design: .rounded))
                .monospacedDigit()
                .foregroundStyle(.white)
                .shadow(color: .black.opacity(0.5), radius: 2, x: 0, y: 1)
        }
    }
}
