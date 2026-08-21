import AppKit
import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct RideSummarySheet: View {
    @ObservedObject var model: VeloSimModel
    let summary: RideSummaryDto
    let publishResult: PublishResultDto?
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var sealShown = false

    var body: some View {
        VStack(spacing: 0) {
            headerChrome

            statsBody
                .padding(16)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(.quaternary)

            actionBar
                .padding(16)
        }
        .frame(minWidth: 520, minHeight: 560)
    }

    private var headerChrome: some View {
        VStack(spacing: 6) {
            // Celebration seal: springs in once; static under Reduce Motion.
            Image(systemName: "checkmark.seal.fill")
                .font(.system(size: 34))
                .foregroundStyle(.green)
                .scaleEffect(sealShown || reduceMotion ? 1.0 : 0.4)
                .opacity(sealShown || reduceMotion ? 1.0 : 0.0)
                .onAppear {
                    guard !reduceMotion else { return }
                    withAnimation(.spring(response: 0.45, dampingFraction: 0.6)) {
                        sealShown = true
                    }
                }
                .accessibilityHidden(true)

            Text("Ride complete")
                .font(.title2.bold())
            Text(RideSummaryFormatting.formatRideDate(summary.startedAtUnix))
                .font(.subheadline)
                .foregroundStyle(.secondary)
            if let publishResult {
                HStack(spacing: 8) {
                    VeloPublishBadge(status: publishBadgeStatus(for: publishResult))
                    Text(RideSummaryFormatting.publishStatusLabel(for: publishResult))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(.vertical, 16)
        .padding(.horizontal, 20)
        .frame(maxWidth: .infinity)
        .veloGlassRoundedRect(cornerRadius: 0)
    }

    private var statsBody: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Headline stats as tinted tiles — Home's tile language, colored.
            LazyVGrid(
                columns: [GridItem(.flexible()), GridItem(.flexible())],
                spacing: Tok.s3
            ) {
                heroTile(
                    "Distance", RideSummaryFormatting.formatDistance(summary.distanceM),
                    systemImage: "point.topleft.down.to.point.bottomright.curvepath",
                    tint: .blue
                )
                heroTile(
                    "Time", RideSummaryFormatting.formatElapsed(summary.elapsedS),
                    systemImage: "clock.fill",
                    tint: .teal
                )
                heroTile(
                    "Avg power", RideSummaryFormatting.formatPower(summary.avgPowerW),
                    systemImage: "bolt.fill",
                    tint: avgPowerTint
                )
                heroTile(
                    "Max power", RideSummaryFormatting.formatPower(summary.maxPowerW),
                    systemImage: "flame.fill",
                    tint: .orange
                )
            }

            if let metrics = model.lastRideMetrics {
                Divider()
                statRow("Normalized power", RideSummaryFormatting.formatPower(metrics.normalizedPowerW))
                if let intensity = metrics.intensityFactor {
                    statRow("Intensity factor", String(format: "%.2f", intensity))
                }
                if let tss = metrics.tss {
                    statRow("Training stress", String(format: "%.0f TSS", tss))
                }
                statRow("Elevation gain", String(format: "%.0f m", metrics.elevationGainM))
            }

            if let publishResult {
                Divider()
                Text(RideSummaryFormatting.activityLinkLabel(for: publishResult))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(3)
                    .textSelection(.enabled)
            }

            if let clipPath = publishResult?.highlightClipPath, !clipPath.isEmpty {
                Divider()
                Text("Highlight clip saved")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text(URL(fileURLWithPath: clipPath).lastPathComponent)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }

    private var highlightActions: some View {
        Group {
            if let clipPath = publishResult?.highlightClipPath, !clipPath.isEmpty {
                Button("Play highlight") {
                    model.openHighlightClip()
                }
                .buttonStyle(VeloGlassSecondaryButtonStyle())
                Button("Reveal clip") {
                    model.revealHighlightClipInFinder()
                }
                .buttonStyle(VeloGlassSecondaryButtonStyle())
            }
        }
    }

    private var actionBar: some View {
        VeloGlassContainer(spacing: 12) {
            VStack(spacing: 12) {
                highlightActions
                HStack(spacing: 12) {
                    if publishResult != nil {
                        Button("Open activity") {
                            model.openLastRideActivity()
                        }
                        .buttonStyle(VeloGlassPrimaryButtonStyle())
                    }
                    Button("Done") {
                        model.dismissRideSummary()
                    }
                    .buttonStyle(VeloGlassSecondaryButtonStyle())
                }
            }
        }
    }

    /// Avg power tile tints by zone vs FTP — power is the only zone-coded color.
    private var avgPowerTint: Color {
        guard let avg = summary.avgPowerW else { return .gray }
        return PowerZone.of(watts: Int(avg.rounded()), ftp: max(1, Int(model.ftp.rounded()))).color
    }

    private func heroTile(
        _ label: String, _ value: String, systemImage: String, tint: Color
    ) -> some View {
        VStack(alignment: .leading, spacing: Tok.s1) {
            HStack(spacing: Tok.s1) {
                Image(systemName: systemImage)
                    .font(.caption)
                    .foregroundStyle(tint)
                Text(label)
                    .font(Typo.label())
                    .foregroundStyle(.secondary)
            }
            Text(value)
                .font(.title3.weight(.bold))
                .monospacedDigit()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(Tok.s3)
        .background(tint.opacity(0.14), in: RoundedRectangle(cornerRadius: Tok.rTile))
        .accessibilityElement(children: .combine)
    }

    private func statRow(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .font(.body.weight(.semibold))
                .monospacedDigit()
        }
    }

    private func publishBadgeStatus(for result: PublishResultDto) -> PublishStatus {
        if result.activityUrl.hasPrefix("error:") {
            return .failed
        }
        return result.savedLocally ? .local : .strava
    }
}

struct RideSummarySheet_Previews: PreviewProvider {
    static var previews: some View {
        RideSummarySheet(
            model: VeloSimModel(),
            summary: RideSummaryDto(
                elapsedS: 3720,
                distanceM: 42195,
                sampleCount: 100,
                avgPowerW: 210,
                maxPowerW: 380,
                startedAtUnix: 1_700_000_000,
                highlightClips: []
            ),
            publishResult: PublishResultDto(
                activityUrl: "/Users/me/Documents/VeloSim/rides/abc123",
                savedLocally: true,
                rideId: "abc123",
                highlightClipPath: "/Users/me/Documents/VeloSim/rides/abc123/highlight.mp4"
            )
        )
    }
}
