import SwiftUI
import VeloSimSupport

@MainActor
struct HomeDashboardView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        DashboardView(model: model)
    }
}
