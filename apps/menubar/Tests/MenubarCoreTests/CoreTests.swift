import Darwin
import Foundation
import Testing
@testable import MenubarCore

private let emptyReport = #"{"v":1,"now":"2026-11-10T22:20:00.000Z","waiting":0,"running":false,"repos":[]}"#

@Test func fixtureContractAndByteFraming() throws {
    let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        .appendingPathComponent("../../../../fixtures/expected/menubar").standardized
    let files = try FileManager.default.contentsOfDirectory(at: root, includingPropertiesForKeys: nil)
    for file in files where file.pathExtension == "json" && file.lastPathComponent != "watch-script.json" {
        guard case .report(let report) = try WireMessage.decode(Data(contentsOf: file)) else {
            Issue.record("Expected report: \(file.lastPathComponent)"); continue
        }
        #expect(report.v == 1)
        #expect(!report.repos.isEmpty)
    }
    let stream = try Data(contentsOf: root.appendingPathComponent("watch-codex-done.jsonl"))
    var reader = LineReader()
    var messages: [WireMessage] = []
    for byte in stream { messages += reader.append(Data([byte])) }
    #expect(messages.count == 9)
    guard case .completion(let event) = messages[1] else { Issue.record("Missing event"); return }
    #expect(event.run == "toolu_watch_a")
    #expect(event.step == "2.4")
    #expect(messages[2] == .tick("2026-11-10T22:20:03.000Z"))
    #expect(messages[4] == .error("synthetic scan failure"))
    var unicode = LineReader()
    let bytes = Data("{\"v\":1,\"error\":\"表示幅🙂\"}\n".utf8)
    var decoded: [WireMessage] = []
    for byte in bytes { decoded += unicode.append(Data([byte])) }
    #expect(decoded == [.error("表示幅🙂")])
    #expect(unicode.append(Data("{\"v\":9}\n{bad}\n{\"v\":1,\"tick\":\"now\"}\n".utf8)) == [
        .tooOld(9), .malformed, .tick("now"),
    ])
    #expect(unicode.append(Data(emptyReport.utf8)).isEmpty)
    #expect(unicode.append(Data("\n".utf8)).count == 1)
}

@Test func lookupOrderAndNotFound() throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let lookup = BinaryLookup(home: dir.path, systemDirectories: [dir.appendingPathComponent("brew").path, dir.appendingPathComponent("local").path])
    let saved = dir.appendingPathComponent("saved path/multi-link").path
    let paths = lookup.candidates(savedPath: saved)
    #expect(paths == [saved, dir.path + "/.cargo/bin/multi-link", dir.path + "/brew/multi-link", dir.path + "/local/multi-link"])
    #expect(lookup.resolve(savedPath: saved) == .notFound(paths))
    for path in paths.reversed() {
        try script(at: URL(fileURLWithPath: path), body: "exit 0")
        #expect(lookup.resolve(savedPath: saved) == .found(path))
    }
}

@Test func instanceLockIsExclusiveAndCloseOnExec() throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let path = dir.appendingPathComponent("state/menubar.lock").path
    let first = InstanceLock(path: path)
    let second = InstanceLock(path: path)
    #expect(first.acquire())
    #expect(first.closeOnExec)
    #expect(!second.acquire())
    first.release()
    #expect(second.acquire())
}

actor Permission: NotificationPermission {
    var allowed = false
    func canDeliver() async -> Bool { allowed }
    func set(_ value: Bool) { allowed = value }
}
actor Updates {
    var values: [WatcherUpdate] = []
    func add(_ update: WatcherUpdate) { values.append(update) }
    func statuses() -> [WatcherStatus] { values.compactMap { if case .status(let s) = $0 { s } else { nil } } }
    func completions() -> [Completion] { values.compactMap { if case .line(.completion(let event)) = $0 { event } else { nil } } }
    func deliveries() -> [Bool] { values.compactMap { if case .delivery(let b) = $0 { b } else { nil } } }
}

@Test func watcherDrainsStderrRestartsWithBackoffAndQuits() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("fake binary")
    try script(at: binary, body: """
    [ "$1 $2 $3" = 'menubar --json --watch' ] || exit 20
    dd if=/dev/zero bs=65536 count=4 >&2 2>/dev/null
    echo '\(emptyReport)'
    exit 7
    """)
    let updates = Updates()
    let watcher = Watcher(binary: binary.path, permission: Permission(),
                          timing: .init(initialBackoff: 0.02, maximumBackoff: 0.08, maintenance: 0.01, permissionPoll: 60)) {
        await updates.add($0)
    }
    await watcher.start()
    try await eventually { await updates.statuses().filter { if case .stopped = $0 { true } else { false } }.count >= 4 }
    await watcher.quit()
    let before = await updates.statuses()
    let delays = before.compactMap { if case .stopped(_, let delay) = $0 { delay } else { nil } }
    #expect(Array(delays.prefix(4)) == [0.02, 0.04, 0.08, 0.08])
    try await Task.sleep(for: .milliseconds(150))
    #expect(await updates.statuses() == before)
    await watcher.start()
    #expect(await updates.statuses() == before)
}

@MainActor @Test func exitingChildPublishesStoppedAndRestartsWithMainActorFree() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("exits")
    try script(at: binary, body: """
    cd "$(dirname "$0")"
    echo start >> starts
    echo '\(emptyReport)'
    exit 7
    """)
    let model = MenubarModel()
    let updates = Updates()
    let watcher = Watcher(binary: binary.path, permission: Permission(),
                          timing: .init(initialBackoff: 0.02, maximumBackoff: 0.02)) {
        await updates.add($0)
        await model.receive($0)
    }
    await watcher.start()
    var heartbeats = 0
    for _ in 0..<200 {
        heartbeats += 1 // This loop and model updates must keep running on the main actor.
        if await updates.statuses().filter({ if case .stopped = $0 { true } else { false } }).count >= 3 { break }
        try await Task.sleep(for: .milliseconds(10))
    }
    await watcher.quit()
    let statuses = await updates.statuses()
    #expect(statuses.contains(.stopped("Watcher exited (7)", retryIn: 0.02)))
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8).split(separator: "\n").count >= 3)
    #expect(heartbeats > 1)
    #expect(model.status == .quit)
}

@Test func watcherRecognizesOldBinaryAndUnknownVersion() async throws {
    for body in ["echo \"error: unrecognized subcommand 'menubar'\" >&2; exit 2", "echo '{\"v\":99}'; sleep 1"] {
        let dir = try temporaryDirectory()
        defer { try? FileManager.default.removeItem(at: dir) }
        let binary = dir.appendingPathComponent("old")
        try script(at: binary, body: body)
        let updates = Updates()
        let watcher = Watcher(binary: binary.path, permission: Permission()) { await updates.add($0) }
        await watcher.start()
        try await eventually { await updates.statuses().contains(.tooOld) }
        await watcher.quit()
        #expect(!FileManager.default.fileExists(atPath: dir.appendingPathComponent("lock").path))
    }
}

@Test func permissionRotatesOnceWithoutClaimingMarkersWhilePendingOrDenied() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("fake")
    try script(at: binary, body: """
    cd "$(dirname "$0")"
    echo "$*" >> starts
    if [ "$4" = '--notify' ]; then
        touch marker
        echo '{"v":1,"event":{"kind":"codex_done","run":"one","id":"session","step":null,"said":"done"}}'
    fi
    echo '\(emptyReport)'
    exec sleep 30
    """)
    let permission = Permission()
    let updates = Updates()
    let watcher = Watcher(binary: binary.path, permission: permission,
                          timing: .init(maintenance: 0.01, permissionPoll: 0.03)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.statuses().contains(.watching) }
    #expect(!FileManager.default.fileExists(atPath: dir.appendingPathComponent("marker").path))
    await watcher.refreshPermission() // Pending -> denied: still cannot deliver.
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8) == "menubar --json --watch\n")
    #expect(!FileManager.default.fileExists(atPath: dir.appendingPathComponent("marker").path))
    await permission.set(true)
    await watcher.refreshPermission()
    try await eventually { await updates.completions().count == 1 }
    await watcher.refreshPermission()
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8) == "menubar --json --watch\nmenubar --json --watch --notify\n")
    #expect(FileManager.default.fileExists(atPath: dir.appendingPathComponent("marker").path))
    await permission.set(false) // Polling also revokes notify without opening the popover.
    try await eventually {
        (try? String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8).split(separator: "\n").count) == 3
    }
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8) == "menubar --json --watch\nmenubar --json --watch --notify\nmenubar --json --watch\n")
    await watcher.quit()
}

@Test func quietWatcherKeepsNotifyPastFormerHealthDeadline() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("quiet")
    try script(at: binary, body: """
    cd "$(dirname "$0")"
    echo "$*" >> starts
    echo '\(emptyReport)'
    exec sleep 30
    """)
    let permission = Permission()
    await permission.set(true)
    let updates = Updates()
    let watcher = Watcher(binary: binary.path, permission: permission,
                          timing: .init(maintenance: 0.01)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.statuses().contains(.watching) }
    try await Task.sleep(for: .milliseconds(10200))
    #expect(await updates.deliveries() == [true])
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8) == "menubar --json --watch --notify\n")
    await watcher.quit()
}

@Test func watcherLaunchFailurePublishesStoppedAndCanQuit() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let updates = Updates()
    let watcher = Watcher(binary: dir.appendingPathComponent("missing").path, permission: Permission()) {
        await updates.add($0)
    }
    await watcher.start()
    try await eventually { await updates.statuses().contains { if case .stopped = $0 { true } else { false } } }
    await watcher.quit()
    #expect(await updates.statuses().last == .quit)
}

@MainActor @Test func secondAppInstanceReportsAlreadyRunningWithoutStartingWatcher() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let path = dir.appendingPathComponent("lock").path
    var resolutions = 0
    var prompts = 0
    func controller() -> AppController {
        AppController(resolveBinary: { resolutions += 1; return .notFound([]) },
                      permission: Permission(), lockPath: path,
                      requestAuthorization: { _ in prompts += 1 })
    }
    let first = controller()
    await first.start()
    let probe = InstanceLock(path: path)
    #expect(!probe.acquire()) // Held even with permission denied and no binary.
    probe.release()
    let second = controller()
    await second.start()
    #expect(second.model.actionNotice == "multi-link is already running")
    #expect(resolutions == 1)
    #expect(prompts == 1)
    await second.quit()
    #expect(!probe.acquire())
    await first.quit()
    #expect(probe.acquire())
    probe.release()
    let third = controller()
    await third.start()
    #expect(resolutions == 2)
    await third.quit()
}

@Test(arguments: [#"{"v":1,"error":"transient scan failure"}"#, "{malformed}"])
func transientScanOutputKeepsDeliveryAndChild(_ line: String) async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("fake")
    try script(at: binary, body: """
    cd "$(dirname "$0")"
    echo "$$ $4" >> starts
    echo '\(emptyReport)'
    if [ "$4" = '--notify' ]; then
        while [ ! -f emit ]; do echo '{"v":1,"tick":"ready"}'; sleep 0.05; done
        echo '\(line)'
    fi
    while :; do echo '{"v":1,"tick":"after scan"}'; sleep 0.05; done
    """)
    let permission = Permission()
    await permission.set(true)
    let updates = Updates()
    let path = dir.appendingPathComponent("lock").path
    let watcher = Watcher(binary: binary.path, permission: permission,
                          timing: .init(maintenance: 0.01)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.values.contains { if case .line(.tick("ready")) = $0 { true } else { false } } }
    let starts = try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8)
    #expect(starts.split(separator: "\n").count == 1) // Permission allows notify on the first child.
    try Data().write(to: dir.appendingPathComponent("emit"))
    let expected: WireMessage = line == "{malformed}" ? .malformed : .error("transient scan failure")
    try await eventually { await updates.values.contains { if case .line(let message) = $0 { message == expected } else { false } } }
    try await Task.sleep(for: .milliseconds(200))
    #expect(await updates.deliveries() == [true])
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8) == starts)
    let probe = InstanceLock(path: path)
    #expect(probe.acquire()) // Watchers no longer own the instance lock.
    probe.release()
    await watcher.quit()
}

@Test func unexpectedExitPreservesPermissionDuringBackoff() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("fake")
    // Use an argv-independent relative sentinel in the executable's own directory.
    try script(at: binary, body: """
    cd "$(dirname "$0")"
    while [ ! -f stop ]; do echo '\(emptyReport)'; sleep 0.05; done
    exit 7
    """)
    let permission = Permission()
    await permission.set(true)
    let updates = Updates()
    let path = dir.appendingPathComponent("lock").path
    let watcher = Watcher(binary: binary.path, permission: permission,
                          timing: .init(maintenance: 0.01)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.deliveries().last == true }
    try Data().write(to: dir.appendingPathComponent("stop"))
    try await eventually { await updates.statuses().contains { if case .stopped = $0 { true } else { false } } }
    #expect(await updates.deliveries().last == true)
    let probe = InstanceLock(path: path)
    #expect(probe.acquire())
    probe.release()
    await watcher.quit()
}

@MainActor @Test func modelKeepsLastReportOnErrors() throws {
    let model = MenubarModel()
    model.receive(.line(try WireMessage.decode(Data(emptyReport.utf8))))
    model.receive(.line(.error("scan failed")))
    #expect(model.report?.waiting == 0)
    #expect(model.status == .failed("scan failed"))
    model.receive(.line(.tick("checked")))
    #expect(model.checkedAt == "checked")
    model.receive(.line(.tooOld(2)))
    #expect(model.status == .tooOld)
}

private func temporaryDirectory() throws -> URL {
    let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    return url
}
private func script(at url: URL, body: String) throws {
    try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    try Data(("#!/bin/sh\n" + body + "\n").utf8).write(to: url)
    try FileManager.default.setAttributes([.posixPermissions: 0o700], ofItemAtPath: url.path)
}
private func eventually(_ condition: () async -> Bool) async throws {
    for _ in 0..<300 {
        if await condition() { return }
        try await Task.sleep(for: .milliseconds(10))
    }
    Issue.record("Timed out waiting for watcher")
}
