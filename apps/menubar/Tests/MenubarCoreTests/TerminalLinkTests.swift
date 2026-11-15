import Foundation
import Testing
@testable import MenubarCore

// These cases catch following only one symlink, trusting existence alone, and
// treating occupied paths as safe destinations for an install command.
@Test @MainActor func terminalProbeResolvesRealLinksWithoutChangingThem() throws {
    let fm = FileManager.default
    let root = URL(fileURLWithPath: "/private/tmp").appendingPathComponent("multi-link-terminal-" + UUID().uuidString)
    try fm.createDirectory(at: root, withIntermediateDirectories: true)
    defer { try? fm.removeItem(at: root) }
    let cli = root.appendingPathComponent("App With Space.app/Contents/MacOS/multi-link")
    try fm.createDirectory(at: cli.deletingLastPathComponent(), withIntermediateDirectories: true)
    try Data("cli".utf8).write(to: cli)
    let probe = FileSystemTerminalLinkProbe()
    let place = root.appendingPathComponent("multi-link")
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .absent)
    try fm.createSymbolicLink(atPath: place.path, withDestinationPath: cli.path)
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .linked)
    try fm.removeItem(at: place)
    let chain = root.appendingPathComponent("chain")
    try fm.createSymbolicLink(atPath: chain.path, withDestinationPath: "App With Space.app/Contents/MacOS/multi-link")
    try fm.createSymbolicLink(atPath: place.path, withDestinationPath: "chain")
    let bundleAlias = root.appendingPathComponent("alias")
    try fm.createSymbolicLink(atPath: bundleAlias.path, withDestinationPath: cli.path)
    #expect(probe.state(at: place.path, bundledCLI: bundleAlias.path) == .linked)
    #expect(try fm.destinationOfSymbolicLink(atPath: place.path) == "chain")
    try fm.removeItem(at: place)
    try fm.createSymbolicLink(atPath: place.path, withDestinationPath: "missing")
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .broken(target: "missing"))
    try fm.removeItem(at: place)
    try fm.createSymbolicLink(atPath: place.path, withDestinationPath: "loop")
    try fm.createSymbolicLink(atPath: root.appendingPathComponent("loop").path, withDestinationPath: "multi-link")
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .broken(target: "loop"))
    try fm.removeItem(at: place)
    let other = root.appendingPathComponent("other")
    try Data("other".utf8).write(to: other)
    try fm.createSymbolicLink(atPath: place.path, withDestinationPath: "other")
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .other(detail: other.path))
    try fm.removeItem(at: place)
    try Data("keep".utf8).write(to: place)
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .other(detail: "regular file"))
    #expect(try String(contentsOf: place, encoding: .utf8) == "keep")
    try fm.removeItem(at: place)
    try fm.createDirectory(at: place, withIntermediateDirectories: false)
    #expect(probe.state(at: place.path, bundledCLI: cli.path) == .other(detail: "directory"))
}
