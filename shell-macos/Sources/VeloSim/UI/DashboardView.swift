import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct DashboardView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                welcomeHeader
                quickStartSection
                recentRidesSection
            }
            .padding(20)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private var welcomeHeader: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Welcome back")
                .font(.largeTitle.bold())
            Text("Pick a quick start or browse activities.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            Text(model.tilesProviderStatus)
                .font(.caption)
                .foregroundStyle(.tertiary)
                .lineLimit(2)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloGlassRoundedRect(cornerRadius: 14)
    }

    private var quickStartSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Quick start")
                .font(.headline)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                quickStartCard(
                    title: "Free ride",
                    subtitle: "Flat plane · ERG optional",
                    systemImage: "figure.outdoor.cycle",
                    tint: .blue
                ) {
                    model.beginFreeRide()
                }

                quickStartCard(
                    title: "Route ride",
                    subtitle: model.activeRouteId.map { "Active: \($0)" } ?? "Pick a GPX route",
                    systemImage: "map",
                    tint: .green
                ) {
                    model.selectedTab = .activities
                    model.activitiesTab = .routes
                }

                quickStartCard(
                    title: "Workout",
                    subtitle: model.workoutLive.active ? model.workoutLive.workoutName : "Structured intervals",
                    systemImage: "bolt.fill",
                    tint: .orange
                ) {
                    model.selectedTab = .activities
                    model.activitiesTab = .workouts
                }
            }
        }
    }

    private func quickStartCard(
        title: String,
        subtitle: String,
        systemImage: String,
        tint: Color,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 8) {
                Image(systemName: systemImage)
                    .font(.title2)
                    .foregroundStyle(tint)
                Text(title)
                    .font(.headline)
                    .foregroundStyle(.primary)
                Text(subtitle)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .multilineTextAlignment(.leading)
            }
            .padding(14)
            .frame(maxWidth: .infinity, minHeight: 120, alignment: .leading)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: 12))
        }
        .buttonStyle(.plain)
    }

    private var recentRidesSection: some View {
        VeloGlassSection("Recent rides") {
            if model.rideHistory.isEmpty {
                Text("No rides yet — start from Quick start or the Ride tab.")
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
                                    Text("\(RideSummaryFormatting.formatDistance(ride.distanceM)) · \(RideSummaryFormatting.formatElapsed(ride.elapsedS))")
                                        .font(.caption2)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                VeloPublishBadge(status: ride.publishStatus)
                            }
                        }
                        .buttonStyle(.plain)
                    }

                    Button("View all history") {
                        model.selectedTab = .activities
                        model.activitiesTab = .history
                    }
                    .font(.caption)
                }
            }
        }
    }
}

struct DashboardView_Previews: PreviewProvider {
    static var previews: some View {
        DashboardView(model: VeloSimModel())
            .frame(width: 720, height: 560)
    }
}
