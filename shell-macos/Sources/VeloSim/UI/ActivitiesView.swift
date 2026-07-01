import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct ActivitiesView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            headerChrome

            Picker("Section", selection: Binding(
                get: { model.activitiesTab },
                set: { model.activitiesTab = $0 }
            )) {
                ForEach(ActivitiesTab.allCases) { tab in
                    Text(tab.title).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, 4)

            ScrollView {
                switch model.activitiesTab {
                case .routes:
                    routesTab
                case .workouts:
                    workoutsTab
                case .history:
                    historyTab
                }
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private var headerChrome: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Activities")
                .font(.title2.bold())
            Text("Routes, workouts, and ride history.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloGlassRoundedRect(cornerRadius: 14)
    }

    private var routesTab: some View {
        VStack(alignment: .leading, spacing: 12) {
            VeloGlassSection("Routes") {
                HStack {
                    Button("Import GPX…") { model.importGpxFile() }
                    if model.activeRouteId != nil {
                        Button("Clear route") { model.clearRoute() }
                    }
                    Button("Go to ride") { model.selectedTab = .ride }
                        .buttonStyle(VeloGlassPrimaryButtonStyle())
                }

                Text(model.routeImportStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)

                if !model.availableRoutes.isEmpty {
                    Picker("Active route", selection: Binding(
                        get: { model.activeRouteId ?? "" },
                        set: { id in
                            if id.isEmpty {
                                model.clearRoute()
                            } else {
                                model.selectRoute(id)
                            }
                        }
                    )) {
                        Text("None (flat)").tag("")
                        ForEach(model.availableRoutes, id: \.routeId) { route in
                            Text("\(route.name) (\(Int(route.totalDistanceM)) m)")
                                .tag(route.routeId)
                        }
                    }
                }

                if model.activeRouteId != nil {
                    Toggle("3D Tiles (online)", isOn: Binding(
                        get: { model.tiles3dEnabled },
                        set: { model.setTiles3d($0) }
                    ))

                    Text(model.tilesProviderStatus)
                        .font(.caption2)
                        .foregroundStyle(.secondary)

                    if let err = model.tilesLastError, model.tiles3dEnabled {
                        Text("Tiles: \(err)")
                            .font(.caption2)
                            .foregroundStyle(.orange)
                            .lineLimit(3)
                    }
                }
            }

            VeloGlassSection("Bike") {
                HStack {
                    Button("Import photos…") { model.importBikePhotos() }
                    if model.activeBikeId != nil {
                        Button("Clear bike") { model.clearBike() }
                    }
                }

                Text(model.bikeImportStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if !model.availableBikes.isEmpty {
                    Picker("Active bike", selection: Binding(
                        get: { model.activeBikeId ?? "" },
                        set: { id in
                            if id.isEmpty {
                                model.clearBike()
                            } else {
                                model.selectBike(id)
                            }
                        }
                    )) {
                        Text("None (default)").tag("")
                        ForEach(model.availableBikes, id: \.bikeId) { bike in
                            Text(bike.name).tag(bike.bikeId)
                        }
                    }
                }
            }
        }
    }

    private var workoutsTab: some View {
        VStack(alignment: .leading, spacing: 12) {
            VeloGlassSection("FTP & workouts") {
                HStack {
                    Text("FTP")
                    Slider(value: Binding(
                        get: { model.ftp },
                        set: { model.applyFtp($0) }
                    ), in: 100...400, step: 5)
                    Text("\(Int(model.ftp)) W")
                        .monospacedDigit()
                        .frame(width: 56, alignment: .trailing)
                }

                WorkoutBuilderView(model: model)

                Button("Start ride with workout") {
                    model.selectedTab = .ride
                }
                .buttonStyle(VeloGlassPrimaryButtonStyle())
                .disabled(!model.workoutLive.active)
            }
        }
    }

    private var historyTab: some View {
        VeloGlassSection("Ride history") {
            if model.rideHistory.isEmpty {
                Text("No rides yet")
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

struct ActivitiesView_Previews: PreviewProvider {
    static var previews: some View {
        ActivitiesView(model: VeloSimModel())
            .frame(width: 640, height: 720)
    }
}
