import SwiftUI

@main
struct VeloSimApp: App {
    @StateObject private var model = VeloSimModel()

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
