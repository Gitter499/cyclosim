import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct RideHistoryView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        NavigationStack {
            Group {
                if model.rideHistory.isEmpty {
                    VStack(spacing: 14) {
                        Image(systemName: "figure.outdoor.cycle")
                            .font(.system(size: 40, weight: .semibold))
                            .foregroundStyle(.white)
                            .frame(width: 88, height: 88)
                            .background(Color.blue.gradient, in: Circle())
                            .accessibilityHidden(true)
                        Text("No rides yet")
                            .font(.title3.bold())
                        Text("Finish your first ride and it shows up here with distance, power, and publish status.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .multilineTextAlignment(.center)
                            .frame(maxWidth: 300)
                        Button("Go to Activities") {
                            model.shellDestination = .activities
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    ScrollView {
                        VStack(alignment: .leading, spacing: 16) {
                            totalsStrip
                            VStack(spacing: 8) {
                                ForEach(model.rideHistory, id: \.id) { ride in
                                    rideRow(ride)
                                }
                            }
                        }
                        .frame(maxWidth: 760)
                        .padding(20)
                        .frame(maxWidth: .infinity)
                    }
                }
            }
            .navigationTitle("History")
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    // MARK: - Totals

    private var totalsStrip: some View {
        let rides = model.rideHistory
        let distanceM = rides.reduce(0) { $0 + $1.distanceM }
        let elapsedS = rides.reduce(0) { $0 + $1.elapsedS }
        let powers = rides.compactMap(\.avgPowerW)
        return HStack(spacing: 8) {
            totalTile(
                label: "Rides", value: "\(rides.count)",
                systemImage: "figure.outdoor.cycle", tint: .blue
            )
            totalTile(
                label: "Distance", value: RideSummaryFormatting.formatDistance(distanceM),
                systemImage: "point.topleft.down.curvedto.point.bottomright.up", tint: .teal
            )
            totalTile(
                label: "Time", value: RideSummaryFormatting.formatElapsed(elapsedS),
                systemImage: "clock", tint: .indigo
            )
            if !powers.isEmpty {
                totalTile(
                    label: "Avg power",
                    value: RideSummaryFormatting.formatPower(
                        powers.reduce(0, +) / Double(powers.count)),
                    systemImage: "bolt.fill", tint: .orange
                )
            }
        }
    }

    private func totalTile(label: String, value: String, systemImage: String, tint: Color)
        -> some View
    {
        VStack(alignment: .leading, spacing: 4) {
            Label(label, systemImage: systemImage)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .minimumScaleFactor(0.7)
            Text(value)
                .font(.title3.weight(.semibold))
                .monospacedDigit()
                .lineLimit(1)
                .minimumScaleFactor(0.7)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(tint.opacity(0.14), in: RoundedRectangle(cornerRadius: 10))
    }

    // MARK: - Rows

    private func rideRow(_ ride: RideRecordDto) -> some View {
        let zoneTint: Color = {
            guard let avg = ride.avgPowerW else { return .secondary.opacity(0.6) }
            return PowerZone.of(watts: Int(avg), ftp: Int(model.ftp)).color
        }()
        return Button {
            model.openRide(ride)
        } label: {
            HStack(spacing: 12) {
                Image(systemName: "figure.outdoor.cycle")
                    .font(.system(size: 17, weight: .semibold))
                    .foregroundStyle(zoneTint)
                    .frame(width: 40, height: 40)
                    .background(zoneTint.opacity(0.16), in: Circle())
                VStack(alignment: .leading, spacing: 3) {
                    Text(routeName(ride.routeId))
                        .font(.subheadline.weight(.semibold))
                    Text(RideSummaryFormatting.formatRideDate(ride.startedAtUnix))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    HStack(spacing: 4) {
                        Text(RideSummaryFormatting.formatDistance(ride.distanceM))
                        Text("·").foregroundStyle(.tertiary)
                        Text(RideSummaryFormatting.formatElapsed(ride.elapsedS))
                        if let avg = ride.avgPowerW {
                            Text("·").foregroundStyle(.tertiary)
                            Text("\(RideSummaryFormatting.formatPower(avg)) avg")
                        }
                    }
                    .font(.caption)
                    .monospacedDigit()
                    .foregroundStyle(.secondary)
                }
                Spacer()
                VeloPublishBadge(status: ride.publishStatus)
                Image(systemName: "chevron.right")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 12))
            .contentShape(RoundedRectangle(cornerRadius: 12))
        }
        .buttonStyle(.plain)
    }

    private func routeName(_ routeId: String?) -> String {
        guard let routeId else { return "Free ride" }
        return model.availableRoutes.first { $0.routeId == routeId }?.name ?? routeId
    }
}

struct RideHistoryView_Previews: PreviewProvider {
    static var previews: some View {
        RideHistoryView(model: VeloSimModel())
            .frame(width: 640, height: 560)
    }
}
