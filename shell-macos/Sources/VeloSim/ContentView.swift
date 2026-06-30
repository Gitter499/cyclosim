import AppKit
import SwiftUI
import QuartzCore
import Metal
import VeloFFI
import VeloSimSupport

struct ContentView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        HSplitView {
            MetalRideView(model: model)
                .frame(minWidth: 640)

            SetupChromeView(model: model)
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

struct MetalRideView: NSViewRepresentable {
    @ObservedObject var model: VeloSimModel

    func makeNSView(context: Context) -> MetalHostView {
        let view = MetalHostView()
        view.onLayerReady = { layer, size in
            model.initRenderer(layer: layer, size: size)
        }
        view.onResize = { size in
            model.resizeRenderer(size: size)
        }
        return view
    }

    func updateNSView(_ nsView: MetalHostView, context: Context) {}
}

final class MetalHostView: NSView {
    var onLayerReady: ((CAMetalLayer, CGSize) -> Void)?
    var onResize: ((CGSize) -> Void)?

    override func makeBackingLayer() -> CALayer {
        let layer = CAMetalLayer()
        layer.device = MTLCreateSystemDefaultDevice()
        layer.pixelFormat = .bgra8Unorm
        layer.framebufferOnly = false
        self.layer = layer
        self.wantsLayer = true
        return layer
    }

    override func layout() {
        super.layout()
        guard let metalLayer = layer as? CAMetalLayer else { return }
        let scale = window?.backingScaleFactor ?? 2.0
        let size = bounds.size
        metalLayer.drawableSize = CGSize(width: size.width * scale, height: size.height * scale)
        onResize?(metalLayer.drawableSize)
        onLayerReady?(metalLayer, metalLayer.drawableSize)
    }
}
