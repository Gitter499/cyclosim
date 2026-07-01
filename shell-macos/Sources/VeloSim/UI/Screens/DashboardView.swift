import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct DashboardView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Tok.s4) {
                profileHeader
                QuickStartRow(model: model)
                pinnedListSection
                recentRidesSection
                lifetimeStatsSection
            }
            .padding(Tok.s4)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
        .sheet(isPresented: $model.showFTPTestPicker) {
            FTPTestPickerView(model: model)
        }
        .sheet(item: $model.pendingFTPAnnouncement) { announcement in
            FTPAnnouncementSheet(model: model, oldFTP: announcement.oldFTP, newFTP: announcement.newFTP)
        }
    }

    private var profileHeader: some View {
        VStack(alignment: .leading, spacing: Tok.s2) {
            Text(model.riderName)
                .font(.largeTitle.bold())
            HStack(spacing: Tok.s4) {
                profileChip("FTP", "\(Int(model.ftp)) W") {
                    model.showFTPTestPicker = true
                }
                profileChip("Weight", String(format: "%.0f kg", model.riderWeightKg), action: nil)
                profileChip(
                    "W/kg",
                    String(format: "%.1f", model.ftp / max(model.riderWeightKg, 1)),
                    action: nil
                )
            }
            Button("Pair devices…") { model.showPairingSheet = true }
                .font(.caption)
        }
        .padding(Tok.s3)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.quaternary, in: RoundedRectangle(cornerRadius: Tok.rTile))
    }

    private func profileChip(_ label: String, _ value: String, action: (() -> Void)?) -> some View {
        Group {
            if let action {
                Button(action: action) {
                    chipContent(label: label, value: value)
                }
                .buttonStyle(.plain)
            } else {
                chipContent(label: label, value: value)
            }
        }
    }

    private func chipContent(label: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(Typo.label())
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline.weight(.semibold))
                .monospacedDigit()
        }
    }

    private var pinnedListSection: some View {
        Group {
            if model.pinnedRouteId != nil || model.pinnedWorkoutName != nil {
                VeloBrowseSection("My list") {
                    VStack(alignment: .leading, spacing: 10) {
                        if let routeId = model.pinnedRouteId {
                            pinnedRow(
                                title: pinnedRouteTitle(routeId),
                                subtitle: "Pinned route",
                                systemImage: "map.fill",
                                tint: .green
                            ) {
                                model.beginPinnedRouteRide()
                            }
                        }

                        if let workout = model.pinnedWorkoutName {
                            pinnedRow(
                                title: workout,
                                subtitle: "Pinned workout",
                                systemImage: "bolt.fill",
                                tint: .orange
                            ) {
                                model.beginPinnedWorkout()
                            }
                        }
                    }
                }
            }
        }
    }

    private func pinnedRouteTitle(_ routeId: String) -> String {
        model.availableRoutes.first(where: { $0.routeId == routeId })?.name ?? routeId
    }

    private func pinnedRow(
        title: String,
        subtitle: String,
        systemImage: String,
        tint: Color,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: systemImage)
                    .font(.title3)
                    .foregroundStyle(tint)
                    .frame(width: 28)
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(.primary)
                    Text(subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Image(systemName: "play.circle.fill")
                    .foregroundStyle(tint)
            }
            .padding(.vertical, 4)
        }
        .buttonStyle(.plain)
    }

    private var recentRidesSection: some View {
        VeloBrowseSection("Recent rides") {
            if model.rideHistory.isEmpty {
                Text("No rides yet — start from Quick start or Activities.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(model.rideHistory.prefix(5), id: \.id) { ride in
                        Button {
                            model.openRide(ride)
                        } label: {
                            HStack {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(RideSummaryFormatting.formatRideDate(ride.startedAtUnix))
                                        .font(.caption.bold())
                                    Text(
                                        "\(RideSummaryFormatting.formatDistance(ride.distanceM)) · \(RideSummaryFormatting.formatElapsed(ride.elapsedS))"
                                    )
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()
                                }
                                Spacer()
                                VeloPublishBadge(status: ride.publishStatus)
                            }
                            .padding(8)
                            .background(
                                model.highlightedRideId == ride.id
                                    ? Color.accentColor.opacity(0.15)
                                    : Color.clear,
                                in: RoundedRectangle(cornerRadius: 8)
                            )
                        }
                        .buttonStyle(.plain)
                    }

                    Button("View all history") {
                        model.shellDestination = .history
                    }
                    .font(.caption)
                }
            }
        }
    }

    private var lifetimeStatsSection: some View {
        VeloBrowseSection("Lifetime") {
            let totalDist = model.rideHistory.reduce(0.0) { $0 + $1.distanceM }
            let totalTime = model.rideHistory.reduce(0.0) { $0 + $1.elapsedS }
            HStack(spacing: Tok.s4) {
                lifetimeTile("Distance", RideSummaryFormatting.formatDistance(totalDist))
                lifetimeTile("Time", RideSummaryFormatting.formatElapsed(totalTime))
                lifetimeTile("Rides", "\(model.rideHistory.count)")
            }
        }
    }

    private func lifetimeTile(_ label: String, _ value: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(Typo.label())
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline.weight(.semibold))
                .monospacedDigit()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(Tok.s3)
        .background(.quaternary, in: RoundedRectangle(cornerRadius: Tok.rTile))
    }
}

struct DashboardView_Previews: PreviewProvider {
    static var previews: some View {
        DashboardView(model: VeloSimModel())
            .frame(width: 720, height: 560)
    }
}
