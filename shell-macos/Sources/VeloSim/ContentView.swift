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
        .sheet(isPresented: $model.showPairingSheet) {
            PairingView(model: model)
        }
        .onAppear {
            model.refreshRideHistory()
            if model.tiles3dEnabled { model.refreshServiceStatus() }
        }
        .onDisappear { model.stopSimLoop() }
    }
}
