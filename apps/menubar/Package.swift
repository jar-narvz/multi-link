// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "multi-link-menubar",
    platforms: [.macOS(.v15)],
    products: [.executable(name: "multi-link-menubar", targets: ["MenubarApp"])],
    targets: [
        .target(name: "MenubarCore"),
        .executableTarget(name: "MenubarApp", dependencies: ["MenubarCore"]),
        .testTarget(name: "MenubarCoreTests", dependencies: ["MenubarCore"]),
    ]
)
