import SwiftUI
import VeloFFI
import VeloSimSupport

@MainActor
struct RideView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        ZStack(alignment: .bottom) {
            MetalRideView(model: model)
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            RideHUDOverlay(model: model)

            VStack(spacing: 0) {
                rideTopChrome
                Spacer()
                tilesStatusBar
            }
        }
        .sheet(isPresented: $model.showPreRideSetup) {
            PreRideSetupSheet(model: model)
        }
    }

    private var rideTopChrome: some View {
        HStack(spacing: 12) {
            if model.isRideRecording {
                Label("Recording", systemImage: "record.circle.fill")
                    .foregroundStyle(.red)
                    .font(.caption.weight(.semibold))
            }

            Spacer()

            Button {
                model.showPreRideSetup = true
            } label: {
                Label("Setup", systemImage: "slider.horizontal.3")
            }
            .buttonStyle(VeloGlassSecondaryButtonStyle())

            if model.isRideRecording {
                Button("Stop & publish") {
                    model.stopRideAndPublish()
                }
                .disabled(model.isFinishingRide)
                .buttonStyle(VeloGlassPrimaryButtonStyle())
            } else {
                Button("Start ride") {
                    model.startRide()
                }
                .disabled(model.isFinishingRide)
                .buttonStyle(VeloGlassPrimaryButtonStyle())
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.black.opacity(0.35))
    }

    @ViewBuilder
    private var tilesStatusBar: some View {
        VStack(spacing: 4) {
            if model.tiles3dEnabled {
                HStack {
                    Text(model.tilesProviderStatus)
                        .lineLimit(1)
                    Spacer()
                    if !model.tilesAttribution.isEmpty {
                        Text(model.tilesAttribution)
                            .lineLimit(1)
                    }
                }
                .font(.caption2)
                .foregroundStyle(.white.opacity(0.85))

                if let err = model.tilesLastError {
                    Text(err)
                        .font(.caption2)
                        .foregroundStyle(.orange)
                        .lineLimit(2)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            } else if model.activeRouteId != nil {
                Text("3D Tiles off — enable in Activities → Routes")
                    .font(.caption2)
                    .foregroundStyle(.white.opacity(0.7))
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            if !model.rideFlowStatus.isEmpty, model.rideFlowStatus != "idle" {
                Text(model.rideFlowStatus)
                    .font(.caption2)
                    .foregroundStyle(.white.opacity(0.75))
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.black.opacity(0.45))
    }
}

struct RideView_Previews: PreviewProvider {
    static var previews: some View {
        RideView(model: VeloSimModel())
            .frame(width: 960, height: 640)
    }
}
