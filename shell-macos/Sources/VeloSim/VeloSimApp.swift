import SwiftUI

@main
struct VeloSimApp: App {
    @StateObject private var model = VeloSimModel()

    init() {
        // CI eval loop: render key screens to PNGs and exit (no window).
        MainActor.assumeIsolated {
            if ScreenshotMode.runIfRequested() {
                exit(0)
            }
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView(model: model)
                .frame(minWidth: 960, minHeight: 640)
                .onOpenURL { url in
                    model.handleOAuthCallback(url: url)
                }
        }
    }
}
