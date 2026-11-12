import Darwin
import Foundation
import Testing
@testable import MenubarCore

private let emptyReport = #"{"v":1,"now":"2026-11-10T22:20:00.000Z","waiting":0,"running":false,"repos":[]}"#

@Test func fixtureContractAndByteFraming() throws {
    let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        .appendingPathComponent("../../../../fixtures/expected/menubar").standardized
    let files = try FileManager.default.contentsOfDirectory(at: root, includingPropertiesForKeys: nil)
    for file in files where file.pathExtension == "json" {
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

@Test func deliveryLockRequiresPermissionAndHealthAndRetries() throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let path = dir.appendingPathComponent("state/menubar.lock").path
    let lock = DeliveryLock(path: path)
    var gate = DeliveryGate(lock: lock)
    let now = Date(timeIntervalSince1970: 100)
    let denied = !gate.update(canDeliver: false, lastLine: now, now: now)
    #expect(denied)
    let unhealthy = !gate.update(canDeliver: true, lastLine: nil, now: now)
    #expect(unhealthy)
    let probe = DeliveryLock(path: path)
    #expect(probe.acquire())
    let contended = !gate.update(canDeliver: true, lastLine: now, now: now)
    #expect(contended)
    probe.release()
    let waitingForRetry = !gate.update(canDeliver: true, lastLine: now, now: now.addingTimeInterval(1))
    #expect(waitingForRetry)
    let acquiredOnRetry = gate.update(canDeliver: true, lastLine: now, now: now.addingTimeInterval(2))
    #expect(acquiredOnRetry)
    #expect(!probe.acquire())
    #expect(lock.closeOnExec)
    let permissionLost = !gate.update(canDeliver: false, lastLine: now, now: now.addingTimeInterval(3))
    #expect(permissionLost)
    #expect(probe.acquire())
    probe.release()
    let reacquired = gate.update(canDeliver: true, lastLine: now, now: now.addingTimeInterval(4))
    #expect(reacquired)
    let healthExpired = !gate.update(canDeliver: true, lastLine: now, now: now.addingTimeInterval(10.01))
    #expect(healthExpired)
    #expect(probe.acquire())
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
    let watcher = Watcher(binary: binary.path, permission: Permission(), lockPath: dir.appendingPathComponent("lock").path,
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

@Test func watcherRecognizesOldBinaryAndUnknownVersion() async throws {
    for body in ["echo \"error: unrecognized subcommand 'menubar'\" >&2; exit 2", "echo '{\"v\":99}'; sleep 1"] {
        let dir = try temporaryDirectory()
        defer { try? FileManager.default.removeItem(at: dir) }
        let binary = dir.appendingPathComponent("old")
        try script(at: binary, body: body)
        let updates = Updates()
        let watcher = Watcher(binary: binary.path, permission: Permission(), lockPath: dir.appendingPathComponent("lock").path) { await updates.add($0) }
        await watcher.start()
        try await eventually { await updates.statuses().contains(.tooOld) }
        await watcher.quit()
        #expect(!FileManager.default.fileExists(atPath: dir.appendingPathComponent("lock").path))
    }
}

@Test func injectedPermissionControlsDeliveryAndQuitReleasesLock() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("fake")
    try script(at: binary, body: "while :; do echo '\(emptyReport)'; sleep 0.05; done")
    let permission = Permission()
    let updates = Updates()
    let path = dir.appendingPathComponent("lock").path
    let watcher = Watcher(binary: binary.path, permission: permission, lockPath: path,
                          timing: .init(maintenance: 0.01, permissionPoll: 60)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.statuses().contains(.watching) }
    #expect(await updates.deliveries().last != true)
    await permission.set(true)
    await watcher.refreshPermission()
    try await eventually { await updates.deliveries().last == true }
    let probe = DeliveryLock(path: path)
    #expect(!probe.acquire())
    await permission.set(false)
    await watcher.refreshPermission()
    #expect(probe.acquire())
    probe.release()
    await watcher.quit()
    #expect(probe.acquire())
}

@Test func secondInstanceCannotClaimAndTakesOverAfterQuit() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("delivery fake")
    try script(at: binary, body: """
    if [ "$4" = '--notify' ]; then
        echo '{"v":1,"event":{"kind":"codex_done","run":"invocation","id":"session","step":null,"said":"done"}}'
    fi
    while :; do echo '\(emptyReport)'; sleep 0.05; done
    """)
    let permission = Permission()
    await permission.set(true)
    let first = Updates()
    let second = Updates()
    let path = dir.appendingPathComponent("lock").path
    let timing = WatcherTiming(maintenance: 0.01, permissionPoll: 0.03)
    let a = Watcher(binary: binary.path, permission: permission, lockPath: path, timing: timing) { await first.add($0) }
    let b = Watcher(binary: binary.path, permission: permission, lockPath: path, timing: timing) { await second.add($0) }
    await a.start()
    try await eventually { await first.completions().count == 1 }
    await b.start()
    try await eventually { await second.statuses().contains(.watching) }
    #expect(await second.completions().isEmpty)
    #expect(await second.deliveries().last != true)
    await a.quit()
    try await eventually { await second.completions().count == 1 }
    #expect(await second.completions().first?.run == "invocation")
    await permission.set(false) // Automatic polling, without a popover refresh.
    try await eventually { await second.deliveries().last == false }
    let probe = DeliveryLock(path: path)
    #expect(probe.acquire())
    probe.release()
    await b.quit()
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
    let watcher = Watcher(binary: binary.path, permission: permission, lockPath: path,
                          timing: .init(maintenance: 0.01)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.values.contains { if case .line(.tick("ready")) = $0 { true } else { false } } }
    let starts = try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8)
    #expect(starts.split(separator: "\n").count == 2) // Initial child, then delivery-enabled child.
    try Data().write(to: dir.appendingPathComponent("emit"))
    let expected: WireMessage = line == "{malformed}" ? .malformed : .error("transient scan failure")
    try await eventually { await updates.values.contains { if case .line(let message) = $0 { message == expected } else { false } } }
    try await Task.sleep(for: .milliseconds(200))
    #expect(await updates.deliveries() == [true])
    #expect(try String(contentsOf: dir.appendingPathComponent("starts"), encoding: .utf8) == starts)
    let probe = DeliveryLock(path: path)
    #expect(!probe.acquire())
    probe.release()
    await watcher.quit()
}

@Test func unexpectedExitReleasesDeliveryDuringBackoff() async throws {
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
    let watcher = Watcher(binary: binary.path, permission: permission, lockPath: path,
                          timing: .init(maintenance: 0.01)) { await updates.add($0) }
    await watcher.start()
    try await eventually { await updates.deliveries().last == true }
    try Data().write(to: dir.appendingPathComponent("stop"))
    try await eventually { await updates.statuses().contains { if case .stopped = $0 { true } else { false } } }
    #expect(await updates.deliveries().last == false)
    let probe = DeliveryLock(path: path)
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
