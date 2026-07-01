import SwiftUI
import VeloSimSupport

/// Ride screen: Metal world + SwiftUI HUD + pause overlay (guide §2).
@MainActor
struct RideModeView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        ZStack {
            MetalRideView(model: model)
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            RideHUDOverlay(model: model)

            if model.ridePaused {
                PauseMenuOverlay(model: model)
            }

            if !model.hudMinimalMode {
                VStack {
                    Spacer()
                    tilesStatusBar
                }
            }
        }
    }

    @ViewBuilder
    private var tilesStatusBar: some View {
        if model.tiles3dEnabled, let err = model.tilesLastError {
            Text(err)
                .font(.caption2)
                .foregroundStyle(.orange)
                .lineLimit(2)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, Tok.s4)
                .padding(.vertical, Tok.s2)
                .background(Color.black.opacity(0.45))
        }
    }
}

struct RideModeView_Previews: PreviewProvider {
    static var previews: some View {
        RideModeView(model: VeloSimModel())
            .frame(width: 960, height: 640)
    }
}
