import SwiftUI
import VeloSimSupport

struct ContentView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        TabView(selection: $model.selectedTab) {
            DashboardView(model: model)
                .tabItem { Label(AppTab.home.title, systemImage: AppTab.home.systemImage) }
                .tag(AppTab.home)

            ActivitiesView(model: model)
                .tabItem { Label(AppTab.activities.title, systemImage: AppTab.activities.systemImage) }
                .tag(AppTab.activities)

            RideView(model: model)
                .tabItem { Label(AppTab.ride.title, systemImage: AppTab.ride.systemImage) }
                .tag(AppTab.ride)

            SettingsView(model: model, embeddedInTab: true)
                .tabItem { Label(AppTab.settings.title, systemImage: AppTab.settings.systemImage) }
                .tag(AppTab.settings)
        }
        .sheet(isPresented: $model.showRideSummarySheet) {
            if let summary = model.lastRideSummary {
                RideSummarySheet(
                    model: model,
                    summary: summary,
                    publishResult: model.lastPublishResult
                )
            }
        }
        .onAppear {
            model.startSimLoop()
            model.refreshRideHistory()
        }
        .onDisappear { model.stopSimLoop() }
    }
}
