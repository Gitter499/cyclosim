import SwiftUI
import VeloSimSupport

// MARK: - Home quick start (§7.1)

@MainActor
struct QuickStartRow: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VeloGlassContainer(spacing: Tok.s3) {
            HStack(spacing: Tok.s3) {
                veloGlassProminentButton("Just Ride", systemImage: "bicycle") {
                    model.beginJustRide()
                }
                veloGlassButton("Workout", systemImage: "list.bullet.rectangle") {
                    model.shellDestination = .activities
                    model.activitiesTab = .workouts
                }
                veloGlassButton("FTP Test", systemImage: "gauge.high") {
                    model.showFTPTestPicker = true
                }
                veloGlassButton("Route", systemImage: "map") {
                    model.shellDestination = .activities
                    model.activitiesTab = .routes
                }
            }
        }
    }
}
