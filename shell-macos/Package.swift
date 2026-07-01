// swift-tools-version: 5.9
import Foundation
import PackageDescription

let rustLibPath = "../target/release"

let supportExclude = [
    "VeloSimApp.swift",
    "ContentView.swift",
    "VeloSimModel.swift",
    "WorkoutBuilderView.swift",
    "UI/AppNavigation.swift",
    "UI/DashboardView.swift",
    "UI/ActivitiesView.swift",
    "UI/RideView.swift",
    "UI/RideHUDOverlay.swift",
    "UI/MetalRideView.swift",
    "UI/PreRideSetupSheet.swift",
    "UI/SetupChromeView.swift",
    "UI/SettingsView.swift",
    "UI/RideSummarySheet.swift",
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
                "Strava/StravaConfig.swift",
                "Strava/StravaOAuth.swift",
                "Strava/StravaTokenStore.swift",
                "Strava/StravaUploader.swift",
                "Ride",
                "Input",
                "Music",
                "Workout",
                "PlatformCallbacks.swift",
                "UI/VeloGlass.swift",
                "UI/RideSummaryFormatting.swift",
                "UI/RideHUDFormatting.swift",
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
