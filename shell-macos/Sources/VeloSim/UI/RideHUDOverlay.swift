import SwiftUI
import VeloFFI
import VeloSimSupport

/// MyWhoosh-inspired metrics overlay on the Metal viewport.
@MainActor
struct RideHUDOverlay: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        if model.hudMinimalMode {
            EmptyView()
        } else {
            hudContent
        }
    }

    private var hudContent: some View {
        ZStack {
            VStack(spacing: 0) {
                intervalSection
                    .padding(.top, 16)

                Spacer(minLength: 0)

                powerCluster
                    .padding(.bottom, 8)

                gradeElevationStrip
                    .padding(.bottom, 6)

                bottomMetricsRow
                    .padding(.bottom, 8)

                if !model.tilesAttribution.isEmpty {
                    tilesAttributionRow
                        .padding(.bottom, 10)
                }
            }
            .padding(.horizontal, 20)
        }
        .allowsHitTesting(false)
    }

    @ViewBuilder
    private var intervalSection: some View {
        if let bar = RideHUDFormatting.intervalBar(live: model.workoutLive) {
            VStack(spacing: 6) {
                HStack {
                    Text(bar.intervalName)
                        .font(.subheadline.weight(.semibold))
                    Spacer()
                    Text(bar.targetLabel)
                        .font(.subheadline.weight(.medium))
                        .foregroundStyle(.white.opacity(0.9))
                    Text(RideHUDFormatting.formatElapsed(bar.remainingS))
                        .font(.subheadline.weight(.semibold))
                        .monospacedDigit()
                }

                GeometryReader { geo in
                    ZStack(alignment: .leading) {
                        Capsule()
                            .fill(.white.opacity(0.2))
                        Capsule()
                            .fill(.orange.gradient)
                            .frame(width: max(4, geo.size.width * bar.fraction))
                    }
                }
                .frame(height: 8)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(.black.opacity(0.45), in: RoundedRectangle(cornerRadius: 12))
        } else if let banner = RideHUDFormatting.workoutBanner(live: model.workoutLive) {
            Text(banner)
                .font(.headline.weight(.semibold))
                .monospacedDigit()
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
                .background(.black.opacity(0.45), in: Capsule())
        }
    }

    private var powerCluster: some View {
        HStack(alignment: .lastTextBaseline, spacing: 28) {
            VStack(alignment: .leading, spacing: 2) {
                Text("POWER")
                    .font(.caption2.weight(.bold))
                    .foregroundStyle(.white.opacity(0.7))
                Text(RideHUDFormatting.formatPower(model.rideState.powerW))
                    .font(.system(size: 52, weight: .bold, design: .rounded))
                    .monospacedDigit()
                    .foregroundStyle(.white)
                    .shadow(color: .black.opacity(0.5), radius: 3, x: 0, y: 1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            VStack(alignment: .trailing, spacing: 10) {
                secondaryMetric(title: "HR", value: RideHUDFormatting.formatHeartRate(model.rideState.heartRateBpm))
                secondaryMetric(title: "CAD", value: RideHUDFormatting.formatCadence(model.rideState.cadenceRpm))
            }
        }
    }

    private var gradeElevationStrip: some View {
        HStack(spacing: 16) {
            Label {
                Text(RideHUDFormatting.formatGradePercent(model.rideState.grade))
                    .monospacedDigit()
            } icon: {
                Image(systemName: "arrow.up.right")
            }

            Label {
                Text(RideHUDFormatting.formatElevation(model.rideState.elevationM))
                    .monospacedDigit()
            } icon: {
                Image(systemName: "mountain.2")
            }

            Spacer()

            if let hint = RideHUDFormatting.steeringHint(
                mode: model.steeringMode,
                routeLoaded: model.activeRouteId != nil
            ) {
                Text(hint)
                    .font(.caption2)
                    .foregroundStyle(.white.opacity(0.75))
            }
        }
        .font(.caption.weight(.semibold))
        .foregroundStyle(.white.opacity(0.9))
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.black.opacity(0.35), in: Capsule())
    }

    private var bottomMetricsRow: some View {
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
                Text(RideHUDFormatting.formatElapsed(model.rideState.elapsedS))
                    .font(.title3.weight(.semibold))
                    .monospacedDigit()
                Text(model.rideMode == .erg ? "ERG" : model.rideMode == .sim ? "SIM" : "FREE")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.white.opacity(0.85))
            }
        }
        .foregroundStyle(.white)
    }

    private var tilesAttributionRow: some View {
        Text(model.tilesAttribution)
            .font(.caption2)
            .foregroundStyle(.white.opacity(0.7))
            .lineLimit(2)
            .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func secondaryMetric(title: String, value: String) -> some View {
        VStack(alignment: .trailing, spacing: 2) {
            Text(title)
                .font(.caption2.weight(.bold))
                .foregroundStyle(.white.opacity(0.7))
            Text(value)
                .font(.title2.weight(.bold))
                .monospacedDigit()
                .foregroundStyle(.white)
        }
    }
}
