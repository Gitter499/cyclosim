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
                nextRideHero
                QuickStartRow(model: model)

                // Two columns at desktop width: rides left, glanceables right.
                HStack(alignment: .top, spacing: Tok.s4) {
                    recentRidesSection
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                    VStack(alignment: .leading, spacing: Tok.s4) {
                        pinnedListSection
                        lifetimeStatsSection
                    }
                    .frame(width: 320, alignment: .topLeading)
                }
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

    // MARK: - Next ride hero

    /// Featured card: the pinned route if set, else the first installed
    /// route, else a free-ride prompt — Home always leads with a ride to take.
    private var nextRideHero: some View {
        let route = model.pinnedRouteId
            .flatMap { id in model.availableRoutes.first(where: { $0.routeId == id }) }
            ?? model.availableRoutes.first

        return Button {
            if let route {
                model.selectRoute(route.routeId)
                model.startRideFromActivities()
            } else {
                model.beginJustRide()
            }
        } label: {
            HStack(spacing: Tok.s4) {
                VStack(alignment: .leading, spacing: Tok.s1) {
                    Text(route == nil ? "JUST RIDE" : "NEXT RIDE")
                        .font(Typo.label())
                        .foregroundStyle(.secondary)
                    Text(route?.name ?? "Free ride on open terrain")
                        .font(.title2.bold())
                    if let route {
                        Text("\(Int(route.totalDistanceM / 1000)) km")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .monospacedDigit()
                    } else {
                        Text("No route loaded — ERG, SIM, and free mode all work here.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                }
                Spacer()
                if let route {
                    RouteElevationSparkline(samples: model.routeProfiles[route.routeId])
                        .frame(width: 180, height: 48)
                        .id(route.routeId)
                        .onAppear { model.loadRouteProfile(route.routeId) }
                }
                Image(systemName: "play.circle.fill")
                    .font(.system(size: 36))
                    .foregroundStyle(Color.accentColor)
            }
            .padding(Tok.s4)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: Tok.rCard))
        }
        .buttonStyle(.plain)
        .accessibilityLabel(
            route.map { "Next ride: \($0.name), \(Int($0.totalDistanceM / 1000)) kilometers. Starts the ride." }
                ?? "Just ride: free ride on open terrain. Starts the ride."
        )
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
                VeloGlassSection("My list") {
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
        VeloGlassSection("Recent rides") {
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
                                    Text(recentRideDetail(ride))
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

    private func recentRideDetail(_ ride: RideRecordDto) -> String {
        var parts = [
            RideSummaryFormatting.formatDistance(ride.distanceM),
            RideSummaryFormatting.formatElapsed(ride.elapsedS),
        ]
        if let avg = ride.avgPowerW {
            parts.append("\(Int(avg.rounded())) W avg")
        }
        return parts.joined(separator: " · ")
    }

    private var lifetimeStatsSection: some View {
        VeloGlassSection("Lifetime") {
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
