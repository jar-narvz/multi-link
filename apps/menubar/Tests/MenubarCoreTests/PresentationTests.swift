import Foundation
import Testing
import UserNotifications
@testable import MenubarCore

@Test @MainActor func presentationContract() throws {
    let model = MenubarModel()
    #expect(model.iconText == "ml ○")
    let report = Report(v: 1, now: "2026-11-10T22:20:00.000Z", waiting: 2, running: true, repos: [
        Repository(repo: "/b", name: "b", tier: "busy", last_activity: "z", step: nil, rows: []),
        Repository(repo: "/a", name: "a", tier: "waiting", last_activity: "a",
                   step: RepoStep(id: "19.3", title: "Popover", more: 2), rows: []),
    ])
    model.receive(.line(.report(report)))
    #expect(model.iconText == "ml 2")
    #expect(model.blocks.map(\.header) == ["b", "a · step 19.3 Popover +2"])
    #expect(model.blocks.map(\.id) == ["/b", "/a"])
    model.receive(.line(.report(Report(v: 1, now: report.now, waiting: 0, running: true, repos: []))))
    #expect(model.iconText == "ml")
    let now = ISO8601DateFormatter().date(from: "2026-11-10T22:20:03Z")!
    #expect(model.footer(now: now) == "checked 3s ago")
    model.receive(.status(.stopped("gone", retryIn: 2)))
    #expect(model.footer(now: now) == "checked 3s ago · Watcher stopped: gone · retrying in 2s")
    model.receive(.status(.tooOld))
    #expect(model.footer(now: now).contains("multi-link too old"))
    model.receive(.status(.notFound(["/one", "/two"])))
    #expect(model.footer(now: now).contains("multi-link not found · tried /one, /two"))
    model.actionNotice = MenubarActions.result(status: 1, stdout: "", stderr: "osascript timed out after 5 s\n", opens: "terminal")
    #expect(model.actionNotice == "osascript timed out after 5 s · if macOS asked for permission, try again")
    #expect(model.footer(now: now).hasSuffix(model.actionNotice!))
    model.updatePermission(false)
    #expect(model.showsNotificationsOff)
    model.dismissNotificationsOff()
    model.updatePermission(true)
    model.updatePermission(false)
    #expect(!model.showsNotificationsOff)
}

@Test func symbolsActionsAndColors() {
    #expect(["busy", "waiting", "stale", "idle", "finished", "unknown"].map { RowPresentation.symbol(state: $0) } == ["●", "◆", "◆", "○", "✓", "·"])
    #expect(RowPresentation.action(opens: "app") == .open("Open in app"))
    #expect(RowPresentation.action(opens: "terminal") == .open("Jump to terminal"))
    #expect(RowPresentation.action(opens: nil) == .copy)
    #expect(RowPresentation.action(opens: "future") == .copy)
    #expect(RowPresentation.childLabel(harness: "codex", match: "heuristic") == "└ codex · likely")
    #expect(RowPresentation.childLabel(harness: "codex", match: "exact") == "└ codex · exact")
    #expect(BoardPalette(dark: false).claude == 0xc2714a)
    #expect(BoardPalette(dark: true).codex == 0x8cb9f1)
    #expect(RowPresentation.tint(state: "stale", harness: "claude") == .stale)
    #expect(RowPresentation.tint(state: "finished", harness: "codex") == .ok)
}

@Test func commandsAndResults() {
    #expect(MenubarActions.openArguments(id: "abc") == ["open", "--no-resume", "abc"])
    #expect(MenubarActions.tuiCommand(binary: "/a path/it's multi-link") == "'/a path/it'\\''s multi-link' tui")
    #expect(MenubarActions.tuiArguments(binary: "/a path/ml") == [
        "-e", "on run argv", "-e", "tell application \"Terminal\" to do script (item 1 of argv)",
        "-e", "tell application \"Terminal\" to activate", "-e", "end run", "'/a path/ml' tui",
    ])
    #expect(MenubarActions.result(status: 0, stdout: "jumped\n", stderr: "", opens: "terminal") == "jumped")
    #expect(MenubarActions.result(status: 0, stdout: "", stderr: "", opens: "app") == "opened in app")
    #expect(MenubarActions.result(status: 0, stdout: "", stderr: "", opens: "terminal") == "jumped to terminal")
    #expect(MenubarActions.result(status: 1, stdout: "", stderr: "", opens: nil) == "could not open session")
    #expect(MenubarActions.tuiResult(CommandOutput(status: 0, stdout: "tab 1", stderr: "")) == "opened TUI")
    #expect(MenubarActions.tuiResult(CommandOutput(status: 1, stdout: "", stderr: "denied\n")) == "could not open TUI: denied")
    #expect(MenubarActions.notificationSettings == "x-apple.systempreferences:com.apple.Notifications-Settings.extension")
}

@Test func completionRequestAndForegroundDelegate() async throws {
    let line = #"{"v":1,"event":{"kind":"codex_done","run":"run-a","id":"session","step":"19.3","said":"Finished it"}}"#
    guard case .completion(let event) = try WireMessage.decode(Data(line.utf8)) else {
        Issue.record("Expected completion event"); return
    }
    let request = CompletionNotifications.request(event)
    #expect(request.identifier == "run-a")
    #expect(request.content.title == "codex finished (19.3)")
    #expect(request.content.body == "Finished it")
    #expect(request.content.sound != nil)
    #expect(request.trigger == nil)
    #expect(CompletionNotifications.request(Completion(kind: "codex_done", run: "b", id: "session", step: nil, said: nil)).content.title == "codex finished")
    #expect(CompletionNotifications.presentationOptions == [.banner, .sound])
    let sink = NotificationSink()
    let notifications = CompletionNotifications { request in await sink.add(request.identifier) }
    try await notifications.deliver(event)
    #expect(await sink.ids == ["run-a"])
}

private actor NotificationSink {
    var ids: [String] = []
    func add(_ id: String) { ids.append(id) }
}

@Test @MainActor func fixtureBlocksAndChildLines() throws {
    let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        .appendingPathComponent("../../../../fixtures/expected/menubar")
    let model = MenubarModel()
    model.receive(.line(try WireMessage.decode(Data(contentsOf: root.appendingPathComponent("multi-repo.json")))))
    #expect(model.blocks.map(\.header) == ["repo-n · step 4.1", "repo-m · step 2.4 +2", "repo-q · step 3.2", "repo-p · step 3.2", "repo-r", "repo-s"])
    #expect(model.blocks[0].rows[0].label == "claude · step 4.1")
    #expect(model.blocks[0].rows[0].symbol == "◆")
    let children = model.blocks.flatMap(\.rows).filter(\.isChild)
    #expect(!children.isEmpty)
    #expect(children.allSatisfy { $0.label.hasPrefix("└ codex · ") })
    #expect(model.blocks[1].rows[0].symbol == "●")
    model.receive(.line(try WireMessage.decode(Data(contentsOf: root.appendingPathComponent("open-actions.json")))))
    #expect(model.blocks[0].rows[0].action == .open("Jump to terminal"))
}

@Test func actionProcessUsesExactArgvAndDrainsBothPipes() async throws {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("binary with spaces")
    try Data("""
    #!/bin/sh
    [ "$#" = 3 ] && [ "$1" = open ] && [ "$2" = --no-resume ] && [ "$3" = 'id with spaces' ] || exit 42
    dd if=/dev/zero bs=65536 count=4 >&2 2>/dev/null
    printf 'jumped to Terminal\\n'
    exit 0
    """.utf8).write(to: binary)
    try FileManager.default.setAttributes([.posixPermissions: 0o700], ofItemAtPath: binary.path)
    let result = try await MenubarActions.run(binary: binary.path, arguments: MenubarActions.openArguments(id: "id with spaces"))
    #expect(result.status == 0)
    #expect(result.stdout == "jumped to Terminal\n")
    #expect(result.stderr.utf8.count == 262144)
}

@Test func headersWithAndWithoutStepTitle() {
    func header(title: String?, more: Int) -> String {
        RepoBlock(Repository(repo: "/repo", name: "repo", tier: "busy", last_activity: "now",
                             step: RepoStep(id: "19.3", title: title, more: more), rows: [])).header
    }
    #expect(header(title: "Popover", more: 0) == "repo · step 19.3 Popover")
    #expect(header(title: "Popover", more: 2) == "repo · step 19.3 Popover +2")
    #expect(header(title: nil, more: 0) == "repo · step 19.3")
    #expect(header(title: nil, more: 2) == "repo · step 19.3 +2")
}

@Test @MainActor func startupWatchesWhileAuthorizationPendingAndRefreshesAfterward() async throws {
    let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(at: dir) }
    let binary = dir.appendingPathComponent("fake watcher")
    try Data("""
    #!/bin/sh
    echo '{"v":1,"now":"2026-11-10T22:20:00.000Z","waiting":0,"running":false,"repos":[]}'
    exec sleep 30
    """.utf8).write(to: binary)
    try FileManager.default.setAttributes([.posixPermissions: 0o700], ofItemAtPath: binary.path)
    let permission = Permission()
    let prompt = PendingAuthorization()
    let controller = AppController(
        resolveBinary: { .found(binary.path) }, permission: permission,
        lockPath: dir.appendingPathComponent("menubar.lock").path,
        requestAuthorization: { _ in await prompt.request() })
    let startup = Task { await controller.start() }
    for _ in 0..<200 {
        let requests = await prompt.requests
        if controller.model.report != nil && requests == 1 { break }
        try await Task.sleep(for: .milliseconds(10))
    }
    let reportBeforeAnswer = controller.model.report
    let ownsBeforeAnswer = controller.model.ownsDelivery
    await controller.start() // A second start must not request authorization again.
    await permission.set(true)
    await prompt.answer()
    await startup.value
    #expect(reportBeforeAnswer?.repos.isEmpty == true)
    #expect(!ownsBeforeAnswer)
    #expect(await prompt.requests == 1)
    #expect(!controller.model.showsNotificationsOff)
    await controller.quit()
}

private actor PendingAuthorization {
    var requests = 0
    private var continuation: CheckedContinuation<Void, Never>?
    func request() async {
        requests += 1
        await withCheckedContinuation { continuation = $0 }
    }
    func answer() {
        continuation?.resume()
        continuation = nil
    }
}
