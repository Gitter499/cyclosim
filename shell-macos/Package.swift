// swift-tools-version: 5.9
import Foundation
import PackageDescription

let rustLibPath = "../target/release"

let supportExclude = [
    "VeloSimApp.swift",
    "ContentView.swift",
    "VeloSimModel.swift",
    "WorkoutBuilderView.swift",
    "UI/Screens/DashboardView.swift",
    "UI/HUD/RideHUDOverlay.swift",
    "UI/Screens/MetalRideView.swift",
    "UI/Screens/SettingsView.swift",
    "UI/Screens/RideSummarySheet.swift",
    "UI/Screens/AppShellView.swift",
    "UI/Screens/HomeDashboardView.swift",
    "UI/Screens/ActivitiesCatalogView.swift",
    "UI/Screens/RideHistoryView.swift",
    "UI/Screens/RideModeView.swift",
    "UI/Screens/PreRidePanel.swift",
    "UI/Screens/ParityHelpers.swift",
    "UI/Settings/ConnectionWizardChrome.swift",
    "UI/Settings/StravaConnectionWizard.swift",
    "UI/Settings/AppleMusicConnectionWizard.swift",
    "UI/Settings/IntegrationsKeysWizard.swift",
    "UI/HUD/FTPTestEngine.swift",
    "BLE",
    "Strava/StravaAuthCoordinator.swift",
]

/// Liquid Glass SwiftUI APIs require the macOS 26 SDK (Xcode 26+). CI uses macOS 14 runners.
private func macOSSDKSupportsLiquidGlass() -> Bool {
    guard let major = macOSSDKMajorVersion() else { return false }
    return major >= 26
}

private func macOSSDKMajorVersion() -> Double? {
    let sdkRoot = ProcessInfo.processInfo.environment["SDKROOT"] ?? ""
    let pipe = Pipe()
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: "/usr/bin/xcrun")
    if sdkRoot.isEmpty {
        proc.arguments = ["--show-sdk-version", "--sdk", "macosx"]
    } else {
        proc.arguments = ["--show-sdk-version", "--sdk", sdkRoot]
    }
    proc.standardOutput = pipe
    proc.standardError = FileHandle.nullDevice
    do {
        try proc.run()
    } catch {
        return nil
    }
    proc.waitUntilExit()
    guard proc.terminationStatus == 0 else { return nil }
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    guard let version = String(data: data, encoding: .utf8)?
        .trimmingCharacters(in: .whitespacesAndNewlines),
        let major = Double(version.split(separator: ".").first ?? "")
    else {
        return nil
    }
    return major
}

private let liquidGlassSwiftSettings: [SwiftSetting] =
    macOSSDKSupportsLiquidGlass() ? [.define("VELO_LIQUID_GLASS")] : []

let package = Package(
    name: "VeloSim",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "VeloSim", targets: ["VeloSim"]),
        .library(name: "VeloSimSupport", targets: ["VeloSimSupport"]),
    ],
    targets: [
        .target(
            name: "VeloFFIBridge",
            path: "Bridge",
            publicHeadersPath: "include",
            cSettings: [
                .headerSearchPath("include"),
            ],
            linkerSettings: [
                .linkedLibrary("velo_ffi"),
                .linkedFramework("Metal"),
                .linkedFramework("QuartzCore"),
                .unsafeFlags([
                    "-L\(rustLibPath)",
                    "-Xlinker", "-rpath", "-Xlinker", "\(rustLibPath)",
                ], .when(platforms: [.macOS])),
            ]
        ),
        .target(
            name: "VeloFFI",
            dependencies: ["VeloFFIBridge"],
            path: "Generated",
            sources: ["velo_ffi.swift"]
        ),
        .target(
            name: "VeloSimBLE",
            path: "Sources/VeloSimBLE"
        ),
        .target(
            name: "VeloSimSupport",
            dependencies: ["VeloFFI"],
            path: "Sources/VeloSim",
            exclude: supportExclude,
            swiftSettings: liquidGlassSwiftSettings,
            linkerSettings: [
                .linkedFramework("Security"),
                .linkedFramework("AVFoundation"),
                .linkedFramework("CoreMotion"),
                .linkedFramework("MusicKit"),
            ]
        ),
        .executableTarget(
            name: "VeloSim",
            dependencies: ["VeloFFI", "VeloSimBLE", "VeloSimSupport"],
            path: "Sources/VeloSim",
            exclude: [
                "AppSecretsStore.swift",
                "AppSettingsStore.swift",
                "SettingsApplyLogic.swift",
                "ConnectionWizardStep.swift",
                "Strava/StravaConfig.swift",
                "Strava/StravaOAuth.swift",
                "Strava/StravaTokenStore.swift",
                "Strava/StravaUploader.swift",
                "Ride",
                "Input",
                "Music",
                "Workout",
                "PlatformCallbacks.swift",
                "TelemetrySamplePoll.swift",
                "UI/Components/VeloGlass.swift",
                "UI/Components/HUDSurface.swift",
                "UI/RideSummaryFormatting.swift",
                "UI/HUD/RideHUDFormatting.swift",
                "UI/HUD/HUDModel.swift",
                "UI/HUD/HUDCoordinator.swift",
                "UI/Design/Tok.swift",
                "UI/Design/Typo.swift",
                "UI/Design/PowerZone.swift",
                "UI/Screens/ShellDestination.swift",
            ],
            swiftSettings: liquidGlassSwiftSettings,
            linkerSettings: [
                .linkedFramework("CoreBluetooth"),
            ]
        ),
        .testTarget(
            name: "VeloSimTests",
            dependencies: ["VeloSimBLE", "VeloSimSupport", "VeloFFI"],
            path: "Tests/VeloSimTests"
        ),
    ]
)
