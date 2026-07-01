import SwiftUI
import VeloSimSupport

struct ContentView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        Group {
            if model.shellPhase == .riding {
                RideModeView(model: model)
            } else {
                AppShellView(model: model)
            }
        }
        .sheet(isPresented: $model.showRideSummarySheet) {
            if let summary = model.lastRideSummary {
                RideSummarySheet(
                    model: model,
                    summary: summary,
                    publishResult: model.lastPublishResult
                )
                .frame(minWidth: 520, minHeight: 560)
            }
        }
        .onAppear {
            model.startSimLoop()
            model.refreshRideHistory()
        }
        .onDisappear { model.stopSimLoop() }
    }
}
