// swift-tools-version: 6.2

import PackageDescription

let package = Package(
    name: "HexSight",
    platforms: [
        .macOS(.v26)
    ],
    targets: [
        .executableTarget(
            name: "HexSight",
            path: "Sources/HexSight",
            linkerSettings: [
                .linkedLibrary("hexsight_ffi"),
                .unsafeFlags(["-L../target/release"])
            ]
        ),
        .testTarget(
            name: "HexSightTests",
            dependencies: ["HexSight"],
            path: "Tests/HexSightTests"
        )
    ]
)
