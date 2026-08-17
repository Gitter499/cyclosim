import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct ActivitiesCatalogView: View {
    @ObservedObject var model: VeloSimModel
    @State private var selectedRouteId: String?

    var body: some View {
        NavigationSplitView {
            catalogSidebar
                .navigationSplitViewColumnWidth(min: 260, ideal: 300, max: 360)
        } detail: {
            detailColumn
        }
        .navigationTitle("Activities")
        .onAppear { syncSelection() }
        .onChange(of: model.activeRouteId) { _, newValue in
            selectedRouteId = newValue
        }
    }

    private var catalogSidebar: some View {
        VStack(spacing: 0) {
            Picker("Section", selection: Binding(
                get: { model.activitiesTab },
                set: { model.activitiesTab = $0 }
            )) {
                ForEach(ActivitiesTab.allCases) { tab in
                    Text(tab.title).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .padding()

            List(selection: $selectedRouteId) {
                switch model.activitiesTab {
                case .routes:
                    routesList
                case .workouts:
                    workoutsList
                }
            }
            .listStyle(.sidebar)
        }
    }

    @ViewBuilder
    private var routesList: some View {
        if model.availableRoutes.isEmpty {
            ContentUnavailableView(
                "No routes",
                systemImage: "map",
                description: Text("Import a GPX file to get started.")
            )
        } else {
            ForEach(model.availableRoutes, id: \.routeId) { route in
                RouteCatalogRow(
                    route: route,
                    isSelected: model.activeRouteId == route.routeId,
                    samples: model.routeProfiles[route.routeId]
                )
                .tag(route.routeId)
                .onAppear { model.loadRouteProfile(route.routeId) }
                    .onTapGesture {
                        selectedRouteId = route.routeId
                        model.selectRoute(route.routeId)
                    }
            }
        }
    }

    @ViewBuilder
    private var workoutsList: some View {
        Section("FTP Tests") {
            // Row metadata computed from the real workout definition (#49).
            let workout = model.sampleWorkout
            let totalS = workout.intervals.reduce(0) { $0 + $1.durationS }
            WorkoutCatalogRow(
                name: workout.name,
                duration: "\(Int((totalS / 60).rounded())) min",
                tss: String(format: "%.0f", model.estimatedTss(for: workout)),
                blocks: workout.intervals.map { interval in
                    switch interval.target {
                    case let .ergWatts(watts): return model.ftp > 0 ? watts / model.ftp : 0.6
                    case let .ftpPercent(percent): return percent / 100.0
                    case .freeRide: return 0.6
                    }
                }
            ) {
                model.startSampleWorkout()
            }
        }

        Section("Custom") {
            Text("Use the builder in the detail pane.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    @ViewBuilder
    private var detailColumn: some View {
        ScrollView {
            // Route/workout content left, ride setup in its own right rail —
            // the single stacked column read as one long unrelated form
            // (caught by the CI screenshot review).
            HStack(alignment: .top, spacing: Tok.s4) {
                VStack(alignment: .leading, spacing: Tok.s4) {
                    switch model.activitiesTab {
                    case .routes:
                        routeDetail
                    case .workouts:
                        workoutDetail
                    }
                }
                .frame(maxWidth: .infinity, alignment: .topLeading)

                PreRidePanel(model: model)
                    .frame(width: 360, alignment: .topLeading)
            }
            .padding(20)
        }
        .background(Color(nsColor: .windowBackgroundColor))
    }

    @ViewBuilder
    private var routeDetail: some View {
        if let route = selectedRoute {
            VStack(alignment: .leading, spacing: Tok.s3) {
                Text(route.name)
                    .font(.title2.bold())

                HStack(spacing: Tok.s4) {
                    RouteElevationSparkline(samples: model.routeProfiles[route.routeId])
                        .frame(width: 120, height: 40)
                        .id(route.routeId)
                        .onAppear { model.loadRouteProfile(route.routeId) }
                    Label("\(Int(route.totalDistanceM / 1000)) km", systemImage: "ruler")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                }

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

                GroupBox("3D Tiles") {
                    Toggle("Photorealistic tiles (online)", isOn: Binding(
                        get: { model.tiles3dEnabled },
                        set: { model.setTiles3d($0) }
                    ))
                    Text(model.tilesProviderStatus)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    if let err = model.tilesLastError, model.tiles3dEnabled {
                        Text(err)
                            .font(.caption2)
                            .foregroundStyle(.orange)
                            .lineLimit(3)
                    }
                }

                GroupBox("Bike") {
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
                                if id.isEmpty { model.clearBike() }
                                else { model.selectBike(id) }
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
        } else {
            ContentUnavailableView(
                "Select a route",
                systemImage: "map",
                description: Text("Choose a route from the list or import GPX.")
            )
        }
    }

    private var workoutDetail: some View {
        VStack(alignment: .leading, spacing: Tok.s3) {
            Text("Workouts")
                .font(.title2.bold())

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

            WorkoutLibraryView(model: model)
        }
    }

    private var selectedRoute: RouteInfoDto? {
        guard let id = selectedRouteId ?? model.activeRouteId else { return nil }
        return model.availableRoutes.first { $0.routeId == id }
    }

    private func syncSelection() {
        selectedRouteId = model.activeRouteId ?? model.availableRoutes.first?.routeId
    }
}

private struct RouteCatalogRow: View {
    let route: RouteInfoDto
    let isSelected: Bool
    var samples: [Double]?

    var body: some View {
        HStack(spacing: Tok.s3) {
            RouteElevationSparkline(samples: samples)
                .frame(width: 64, height: 28)

            VStack(alignment: .leading, spacing: 2) {
                Text(route.name)
                    .font(.subheadline.weight(.semibold))
                Text("\(Int(route.totalDistanceM / 1000)) km")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }

            Spacer()

            if isSelected {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
                    .accessibilityLabel("Selected")
            }
        }
        .padding(.vertical, 4)
        .accessibilityElement(children: .combine)
    }
}

private struct WorkoutCatalogRow: View {
    let name: String
    let duration: String
    let tss: String
    let blocks: [Double]
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: Tok.s3) {
                IntervalGraphPreview(blocks: blocks)
                    .frame(width: 72, height: 28)
                VStack(alignment: .leading, spacing: 2) {
                    Text(name)
                        .font(.subheadline.weight(.semibold))
                    Text("\(duration) · TSS \(tss)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(name), \(duration), TSS \(tss). Starts the workout.")
    }
}

struct ActivitiesCatalogView_Previews: PreviewProvider {
    static var previews: some View {
        ActivitiesCatalogView(model: VeloSimModel())
            .frame(width: 900, height: 720)
    }
}
