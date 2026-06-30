import AppKit
import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct RideSummarySheet: View {
    @ObservedObject var model: VeloSimModel
    let summary: RideSummaryDto
    let publishResult: PublishResultDto?

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
        .frame(width: 380)
    }

    private var headerChrome: some View {
        VStack(spacing: 6) {
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
            statRow("Distance", RideSummaryFormatting.formatDistance(summary.distanceM))
            statRow("Elapsed", RideSummaryFormatting.formatElapsed(summary.elapsedS))
            statRow("Avg power", RideSummaryFormatting.formatPower(summary.avgPowerW))
            statRow("Max power", RideSummaryFormatting.formatPower(summary.maxPowerW))

            if let publishResult {
                Divider()
                Text(RideSummaryFormatting.activityLinkLabel(for: publishResult))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(3)
                    .textSelection(.enabled)
            }
        }
    }

    private var actionBar: some View {
        VeloGlassContainer(spacing: 12) {
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
                startedAtUnix: 1_700_000_000
            ),
            publishResult: PublishResultDto(
                activityUrl: "/Users/me/Documents/VeloSim/rides/abc123",
                savedLocally: true,
                rideId: "abc123"
            )
        )
    }
}
