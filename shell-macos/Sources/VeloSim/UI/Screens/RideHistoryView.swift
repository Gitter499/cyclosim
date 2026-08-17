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
                    ContentUnavailableView(
                        "No rides yet",
                        systemImage: "clock",
                        description: Text("Start from Activities.")
                    )
                } else {
                    List(model.rideHistory, id: \.id) { ride in
                        Button {
                            model.openRide(ride)
                        } label: {
                            HStack {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(RideSummaryFormatting.formatRideDate(ride.startedAtUnix))
                                        .font(.subheadline.weight(.semibold))
                                    Text("\(RideSummaryFormatting.formatDistance(ride.distanceM)) · \(RideSummaryFormatting.formatElapsed(ride.elapsedS))")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                    if let avg = ride.avgPowerW {
                                        Text("Avg \(RideSummaryFormatting.formatPower(avg))")
                                            .font(.caption2)
                                            .foregroundStyle(.secondary)
                                    }
                                }
                                Spacer()
                                VeloPublishBadge(status: ride.publishStatus)
                            }
                        }
                        .buttonStyle(.plain)
                    }
                    .listStyle(.inset)
                }
            }
            .navigationTitle("History")
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
    }
}

struct RideHistoryView_Previews: PreviewProvider {
    static var previews: some View {
        RideHistoryView(model: VeloSimModel())
            .frame(width: 640, height: 560)
    }
}
