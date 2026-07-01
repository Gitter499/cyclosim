import SwiftUI
import VeloSimSupport

@MainActor
struct AppShellView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        Group {
            #if VELO_LIQUID_GLASS
            if #available(macOS 26, *) {
                modernTabShell
            } else {
                legacyTabShell
            }
            #else
            legacyTabShell
            #endif
        }
        .background(Color(nsColor: .windowBackgroundColor))
    }

    #if VELO_LIQUID_GLASS
    @available(macOS 26, *)
    private var modernTabShell: some View {
        TabView(selection: $model.shellDestination) {
            Tab(ShellDestination.home.title, systemImage: ShellDestination.home.systemImage, value: ShellDestination.home) {
                destinationView(.home)
            }
            Tab(ShellDestination.activities.title, systemImage: ShellDestination.activities.systemImage, value: ShellDestination.activities) {
                destinationView(.activities)
            }
            Tab(ShellDestination.history.title, systemImage: ShellDestination.history.systemImage, value: ShellDestination.history) {
                destinationView(.history)
            }
            Tab(ShellDestination.settings.title, systemImage: ShellDestination.settings.systemImage, value: ShellDestination.settings) {
                destinationView(.settings)
            }
        }
        .tabViewStyle(.sidebarAdaptable)
    }
    #endif

    private var legacyTabShell: some View {
        TabView(selection: $model.shellDestination) {
            destinationView(.home)
                .tabItem { Label(ShellDestination.home.title, systemImage: ShellDestination.home.systemImage) }
                .tag(ShellDestination.home)
            destinationView(.activities)
                .tabItem { Label(ShellDestination.activities.title, systemImage: ShellDestination.activities.systemImage) }
                .tag(ShellDestination.activities)
            destinationView(.history)
                .tabItem { Label(ShellDestination.history.title, systemImage: ShellDestination.history.systemImage) }
                .tag(ShellDestination.history)
            destinationView(.settings)
                .tabItem { Label(ShellDestination.settings.title, systemImage: ShellDestination.settings.systemImage) }
                .tag(ShellDestination.settings)
        }
    }

    @ViewBuilder
    private func destinationView(_ destination: ShellDestination) -> some View {
        Group {
            switch destination {
            case .home:
                HomeDashboardView(model: model)
            case .activities:
                ActivitiesCatalogView(model: model)
            case .history:
                RideHistoryView(model: model)
            case .settings:
                SettingsBlockedView(isBlocked: model.shellPhase == .riding) {
                    SettingsView(model: model, embeddedInTab: true)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .transition(.opacity)
        .id(destination)
    }
}

struct AppShellView_Previews: PreviewProvider {
    static var previews: some View {
        AppShellView(model: VeloSimModel())
            .frame(width: 900, height: 640)
    }
}
