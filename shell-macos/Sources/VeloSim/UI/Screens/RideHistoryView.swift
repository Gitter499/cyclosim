import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct RideHistoryView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            headerChrome

            ScrollView {
                VeloBrowseSection("Ride history") {
                    if model.rideHistory.isEmpty {
                        Text("No rides yet — start from Activities.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    } else {
                        LazyVStack(alignment: .leading, spacing: 8) {
                            ForEach(model.rideHistory, id: \.id) { ride in
                                Button {
                                    model.openRide(ride)
                                } label: {
                                    HStack {
                                        VStack(alignment: .leading, spacing: 2) {
                                            Text(RideSummaryFormatting.formatRideDate(ride.startedAtUnix))
                                                .font(.caption.bold())
                                            Text("\(RideSummaryFormatting.formatDistance(ride.distanceM)) · \(RideSummaryFormatting.formatElapsed(ride.elapsedS))")
                                                .font(.caption2)
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
                        }
                    }
                }
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var headerChrome: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("History")
                .font(.title2.bold())
            Text("Past rides saved locally or published to Strava.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloBrowseHeader(cornerRadius: 14)
    }
}

struct RideHistoryView_Previews: PreviewProvider {
    static var previews: some View {
        RideHistoryView(model: VeloSimModel())
            .frame(width: 640, height: 560)
    }
}
