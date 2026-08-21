import AppKit
import SwiftUI
import VeloFFI
import VeloSimSupport

/// Headless screenshot mode for the agent eval loop: `VeloSim --screenshots
/// <dir>` renders key screens off-screen via `ImageRenderer`, writes PNGs, and
/// exits before any window appears. CI base64-dumps the files into the job log
/// so the real SwiftUI shell can be reviewed without a Mac at hand.
@MainActor
enum ScreenshotMode {
    /// Returns true when screenshot mode ran (caller should exit).
    static func runIfRequested() -> Bool {
        let args = CommandLine.arguments
        guard let flag = args.firstIndex(of: "--screenshots"), flag + 1 < args.count else {
            return false
        }
        // Windows from a CLI process need the shared app set up as an
        // accessory (no Dock icon, no activation needed).
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        app.finishLaunching()

        let dir = URL(fileURLWithPath: args[flag + 1], isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)

        let model = VeloSimModel()
        seedDemoLibrary(model)
        seedDemoRide(model)
        // Arm the sample workout through the real engine so the pre-ride
        // rail's interval preview shows in the activities capture.
        model.startSampleWorkout()

        capture(HomeDashboardView(model: model), size: CGSize(width: 1100, height: 720),
                to: dir, name: "home")
        capture(ActivitiesCatalogView(model: model), size: CGSize(width: 1100, height: 720),
                to: dir, name: "activities")
        capture(SettingsView(model: model), size: CGSize(width: 1100, height: 720),
                to: dir, name: "settings")
        capture(RideHistoryView(model: model), size: CGSize(width: 1100, height: 720),
                to: dir, name: "history")
        let summary = RideSummaryDto(
            elapsedS: 3_612,
            distanceM: 31_240,
            sampleCount: 3_612,
            avgPowerW: 212,
            maxPowerW: 640,
            startedAtUnix: 1_754_913_600,
            highlightClips: [
                HighlightClipRequestDto(startElapsedS: 1_204, durationS: 20, label: "Power surge"),
                HighlightClipRequestDto(startElapsedS: 3_312, durationS: 20, label: "Finish"),
            ]
        )
        model.lastRideMetrics = RideMetricsDto(
            normalizedPowerW: 228,
            intensityFactor: 0.91,
            tss: 92,
            elevationGainM: 412
        )
        capture(
            RideSummarySheet(model: model, summary: summary, publishResult: nil)
                // In the app this sits over the window; give the capture the
                // same backing or dark-scheme text lands on default white.
                .background(Color(nsColor: .windowBackgroundColor)),
            size: CGSize(width: 640, height: 760), to: dir, name: "ride-summary"
        )

        capture(
            RideHUDOverlay(model: model)
                .background(
                    LinearGradient(
                        colors: [Color(red: 0.33, green: 0.55, blue: 0.83),
                                 Color(red: 0.30, green: 0.42, blue: 0.28)],
                        startPoint: .top, endPoint: .bottom
                    )
                ),
            size: CGSize(width: 1280, height: 720), to: dir, name: "ride-hud"
        )

        // Minimal mode hides the secondary chrome — capture it too so the
        // pared-down layout stays reviewed. The overlay reads the shell-level
        // flag, not HUDModel.minimalMode (r16: both captures came out
        // byte-identical with the wrong flag set).
        model.hudMinimalMode = true
        capture(
            RideHUDOverlay(model: model)
                .background(
                    LinearGradient(
                        colors: [Color(red: 0.33, green: 0.55, blue: 0.83),
                                 Color(red: 0.30, green: 0.42, blue: 0.28)],
                        startPoint: .top, endPoint: .bottom
                    )
                ),
            size: CGSize(width: 1280, height: 720), to: dir, name: "ride-hud-minimal"
        )
        model.hudMinimalMode = false
        return true
    }

    /// Demo library content so Home/Activities screenshots show populated
    /// states instead of first-launch emptiness.
    private static func seedDemoLibrary(_ model: VeloSimModel) {
        model.availableRoutes = [
            RouteInfoDto(routeId: "col-du-demo", name: "Col du Demo", totalDistanceM: 18_400),
            RouteInfoDto(routeId: "rolling-hills", name: "Rolling Hills Loop", totalDistanceM: 24_800),
        ]
        model.pinnedRouteId = "col-du-demo"
        model.routeProfiles["col-du-demo"] = (0..<48).map { (i: Int) -> Double in
            let x = Double(i)
            return 300.0 + x * 18.0 + 40.0 * sin(x / 5.0)
        }

        let now = UInt64(1_755_000_000)
        let day: UInt64 = 86_400
        model.rideHistory = [
            RideRecordDto(
                id: "demo-1", startedAtUnix: now - day, elapsedS: 3_612, distanceM: 31_240,
                avgPowerW: 212, maxPowerW: 640, fitPath: "", screenshotPath: nil,
                highlightClipPath: nil, stravaActivityId: nil, publishStatus: .strava,
                routeId: "col-du-demo"
            ),
            RideRecordDto(
                id: "demo-2", startedAtUnix: now - 3 * day, elapsedS: 2_710, distanceM: 21_050,
                avgPowerW: 235, maxPowerW: 588, fitPath: "", screenshotPath: nil,
                highlightClipPath: nil, stravaActivityId: nil, publishStatus: .local,
                routeId: "rolling-hills"
            ),
            RideRecordDto(
                id: "demo-3", startedAtUnix: now - 6 * day, elapsedS: 4_505, distanceM: 38_900,
                avgPowerW: 198, maxPowerW: 512, fitPath: "", screenshotPath: nil,
                highlightClipPath: nil, stravaActivityId: nil, publishStatus: .strava,
                routeId: nil
            ),
        ]
    }

    /// Plausible mid-ride values so the HUD screenshot shows every element.
    private static func seedDemoRide(_ model: VeloSimModel) {
        let hud = model.hudModel
        hud.ftp = 250
        hud.power = 264
        hud.cadence = 92
        hud.heartRate = 163
        hud.speedMps = 10.4
        hud.distanceM = 24_350
        hud.gradient = 0.042
        hud.elapsedS = 3_725
        hud.wattsPerKg = 3.5
        hud.lapCount = 2
        hud.currentLapElapsedS = 412
        hud.rollingPower = (0..<48).map { (i: Int) -> Double in
            let x = Double(i)
            let wave: Double = 40.0 * sin(x / 6.0)
            let jitter: Double = Double(i % 5) * 3.0
            return 240.0 + wave + jitter
        }
        hud.elevationProfile = (0..<48).map { (i: Int) -> Double in
            let x = Double(i)
            let hill: Double = 60.0 * sin(x / 8.0)
            return 120.0 + hill + x * 1.5
        }
        hud.routeTotalM = 42_000
        hud.transientEvent = TransientHUDEvent(
            title: "Threshold 1",
            detail: "250 W",
            raisedAt: CFAbsoluteTimeGetCurrent()
        )
        hud.workout = WorkoutHUD(
            targetWatts: 250,
            actualWatts: 264,
            intervalRemainingS: 512,
            intervalProgress: 0.57,
            blockName: "Threshold 1",
            nextBlockName: "Recover"
        )
    }

    /// Renders in an offscreen NSWindow instead of ImageRenderer: navigation
    /// containers (SplitView/Stack) and SF Symbols only resolve with a real
    /// window backing (caught by the first CI screenshot round, where nav
    /// screens came back blank and every symbol was a missing-glyph box).
    private static func capture<V: View>(_ view: V, size: CGSize, to dir: URL, name: String) {
        let hosting = NSHostingView(
            rootView: view
                .frame(width: size.width, height: size.height)
                .preferredColorScheme(.dark)
        )
        hosting.frame = CGRect(origin: .zero, size: size)

        let window = NSWindow(
            contentRect: CGRect(origin: .zero, size: size),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        // AppKit-backed materials read the window appearance, not just the
        // SwiftUI color scheme.
        window.appearance = NSAppearance(named: .darkAqua)
        window.contentView = hosting
        window.orderFrontRegardless()

        // Let async layout (navigation columns, lists) settle.
        RunLoop.main.run(until: Date().addingTimeInterval(0.8))
        hosting.layoutSubtreeIfNeeded()

        guard let rep = hosting.bitmapImageRepForCachingDisplay(in: hosting.bounds) else {
            FileHandle.standardError.write(Data("screenshot \(name): no bitmap rep\n".utf8))
            window.orderOut(nil)
            return
        }
        hosting.cacheDisplay(in: hosting.bounds, to: rep)
        window.orderOut(nil)

        guard let png = rep.representation(using: .png, properties: [:]) else {
            FileHandle.standardError.write(Data("screenshot \(name): png encode failed\n".utf8))
            return
        }
        do {
            try png.write(to: dir.appendingPathComponent("\(name).png"))
        } catch {
            FileHandle.standardError.write(Data("screenshot \(name): \(error)\n".utf8))
        }
    }
}
