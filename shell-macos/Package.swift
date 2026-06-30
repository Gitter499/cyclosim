// swift-tools-version: 5.9
import PackageDescription

let rustLibPath = "../target/release"

let supportExclude = [
    "VeloSimApp.swift",
    "ContentView.swift",
    "VeloSimModel.swift",
    "WorkoutBuilderView.swift",
    "BLE",
    "Strava/StravaAuthCoordinator.swift",
]

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
            linkerSettings: [
                .linkedFramework("Security"),
            ]
        ),
        .executableTarget(
            name: "VeloSim",
            dependencies: ["VeloFFI", "VeloSimBLE", "VeloSimSupport"],
            path: "Sources/VeloSim",
            exclude: [
                "Strava/StravaConfig.swift",
                "Strava/StravaOAuth.swift",
                "Strava/StravaTokenStore.swift",
                "Strava/StravaUploader.swift",
                "Ride",
                "PlatformCallbacks.swift",
            ],
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
