import SwiftUI
import VeloSimSupport

@MainActor
struct RideModeView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        ZStack(alignment: .top) {
            MetalRideView(model: model)
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            RideHUDOverlay(model: model)

            VStack {
                rideControlBar
                    .padding(.horizontal, 16)
                    .padding(.top, 12)
                Spacer()
            }
        }
    }

    private var rideControlBar: some View {
        HStack(spacing: 12) {
            Label("Recording", systemImage: "record.circle.fill")
                .foregroundStyle(.red)
                .font(.caption.weight(.semibold))

            Spacer()

            if !model.rideFlowStatus.isEmpty, model.rideFlowStatus != "idle", model.rideFlowStatus != "recording" {
                Text(model.rideFlowStatus)
                    .font(.caption)
                    .foregroundStyle(.white.opacity(0.85))
                    .lineLimit(1)
            }

            Button("Stop & publish") {
                model.stopRideAndPublish()
            }
            .disabled(model.isFinishingRide)
            .buttonStyle(VeloGlassPrimaryButtonStyle())
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(.black.opacity(0.35), in: RoundedRectangle(cornerRadius: 12))
    }
}

struct RideModeView_Previews: PreviewProvider {
    static var previews: some View {
        RideModeView(model: VeloSimModel())
            .frame(width: 960, height: 640)
    }
}
