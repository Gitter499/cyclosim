import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct ActivitiesCatalogView: View {
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
                    routesSection
                case .workouts:
                    workoutsSection
                }

                PreRidePanel(model: model)
                    .padding(.top, 8)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var headerChrome: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Activities")
                .font(.title2.bold())
            Text("Routes, workouts, and pre-ride setup.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloGlassRoundedRect(cornerRadius: 14)
    }

    private var routesSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            VeloGlassSection("Routes") {
                HStack {
                    Button("Import GPX…") { model.importGpxFile() }
                    if model.activeRouteId != nil {
                        Button("Clear route") { model.clearRoute() }
                    }
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

    private var workoutsSection: some View {
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
        }
    }
}

struct ActivitiesCatalogView_Previews: PreviewProvider {
    static var previews: some View {
        ActivitiesCatalogView(model: VeloSimModel())
            .frame(width: 640, height: 720)
    }
}
