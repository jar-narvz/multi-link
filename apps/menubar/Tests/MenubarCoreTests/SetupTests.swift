import Foundation
import Testing
import UserNotifications
@testable import MenubarCore

@MainActor private final class SetupFakes: SetupRunning, SetupNotificationSource, LoginItemService, SetupDefaults {
    var calls: [String] = []
    var hooks = "fixable"
    var badJSON = false
    var commandError = false
    var notification = SetupNotificationStatus(authorization: .notDetermined, alerts: .disabled)
    var status: LoginItemStatus = .notRegistered
    var loginError = false
    var shown = false
    func run(_ arguments: [String]) async throws -> CommandOutput {
        calls.append(arguments.joined(separator: " "))
        if arguments == ["setup", "--json"] {
            return CommandOutput(status: 1, stdout: badJSON ? "bad" : """
            {"v":1,"items":[{"id":"hooks","state":"\(hooks)","file":"/fake/settings.json","exists":true,"events":[{"event":"PreToolUse","status":"customized","details":[]}]}]}
            """, stderr: "")
        }
        return CommandOutput(status: commandError ? 1 : 0, stdout: "", stderr: commandError ? "apply refused" : "")
    }
    func settings() async -> SetupNotificationStatus { calls.append("settings"); return notification }
    func request() async throws { calls.append("request") }
    func register() throws { calls.append("register"); if loginError { throw FakeError.refused } }
    func unregister() throws { calls.append("unregister"); if loginError { throw FakeError.refused } }
    func openSettings() { calls.append("login settings") }
    func hasShownSetup() -> Bool { shown }
    func markSetupShown() { shown = true }
    func model(path: String = "/Applications/multi-link.app") -> SetupModel {
        SetupModel(runner: self, notifications: self, login: self, defaults: self, bundlePath: path)
    }
}
private enum FakeError: LocalizedError { case refused; var errorDescription: String? { "Operation not permitted" } }

@Test @MainActor func hooksStatesAndInvalidJSON() async {
    let fake = SetupFakes(); let model = fake.model()
    for state in ["ok", "attention", "fixable", "unreadable"] {
        fake.hooks = state; await model.refresh()
        #expect(model.hooks?.state.rawValue == state)
        #expect(model.hooksGreen == ["ok", "attention"].contains(state))
        #expect(model.remaining == (model.hooksGreen ? 1 : 2))
    }
    fake.hooks = "attention"; await model.refresh()
    #expect(model.hooksMessage == "On, except PreToolUse: you changed that one, so it's left as is.")
    fake.badJSON = true; await model.refresh()
    #expect(!model.hooksGreen); #expect(model.readError != nil)
}

@Test @MainActor func notificationOnlyAuthorizedWithAlertsIsGreen() async {
    let fake = SetupFakes(); let model = fake.model()
    for status: UNAuthorizationStatus in [.notDetermined, .denied, .authorized, .provisional, UNAuthorizationStatus(rawValue: 4)!] {
        for alerts: UNNotificationSetting in [.disabled, .enabled] {
            fake.notification = SetupNotificationStatus(authorization: status, alerts: alerts)
            await model.refresh()
            #expect(model.notificationsGreen == (status == .authorized && alerts == .enabled))
        }
    }
    await model.turnOnNotifications()
    #expect(!model.notificationsGreen) // request result cannot grant delivery
}

@Test @MainActor func everythingOrderAndReread() async {
    let fake = SetupFakes(); let model = fake.model(); await model.refresh(); fake.calls = []
    await model.turnOnEverything()
    #expect(fake.calls == ["hook apply", "request", "register", "setup --json", "settings"])
    #expect(model.hooksChanged)
    for state in ["ok", "attention", "unreadable"] {
        fake.hooks = state; await model.refresh(); fake.calls = []; await model.turnOnEverything()
        #expect(fake.calls == ["request", "register", "setup --json", "settings"])
    }
    fake.commandError = true; await model.removeHooks()
    #expect(model.actionError == "apply refused")
    #expect(fake.calls.suffix(3) == ["hook remove", "setup --json", "settings"])
}

@Test @MainActor func loginStatusesAndErrors() async {
    let fake = SetupFakes(); let model = fake.model()
    for status in [LoginItemStatus.enabled, .notRegistered, .notFound, .requiresApproval, .unknown] {
        fake.status = status; await model.refresh(); #expect(model.loginStatus == status)
        #expect(model.loginEnabled == (status == .enabled))
    }
    fake.loginError = true
    await model.setLoginEnabled(true); #expect(model.loginError == "Operation not permitted")
    await model.setLoginEnabled(false); #expect(model.loginError == "Operation not permitted")
    #expect(fake.calls.contains("unregister"))
}

@Test @MainActor func gateRunsNothingAndFirstLaunchIsOnce() async {
    for path in ["/Volumes/test/multi-link.app", "/private/test/AppTranslocation/random/multi-link.app"] {
        let fake = SetupFakes(); let model = fake.model(path: path)
        #expect(model.isTemporary)
        await model.refresh(); await model.turnOnEverything(); await model.turnOnHooks()
        await model.turnOnNotifications(); await model.removeHooks(); await model.setLoginEnabled(true)
        #expect(fake.calls.isEmpty)
        #expect(model.launchAction() == .gate)
        #expect(!fake.shown)
        #expect(fake.model(path: path).launchAction() == .gate)
        let installed = fake.model()
        #expect(installed.launchAction() == .show)
        #expect(fake.shown)
        #expect(fake.model().launchAction() == .none)
        #expect(model.launchAction() == .gate) // Gate remains visible even after real setup was shown.
        #expect(fake.calls.isEmpty)
    }
}

@Test @MainActor func pollingTicksAreReadOnly() async {
    let fake = SetupFakes(); let model = fake.model()
    for _ in 0..<4 { await model.pollTick() }
    #expect(fake.calls == Array(repeating: ["setup --json", "settings"], count: 4).flatMap { $0 })
    #expect(model.hooksFixableAtLaunch)
}

@Test @MainActor func firstLaunchUsesInjectedStoreAcrossModels() async {
    let fake = SetupFakes(); let first = fake.model()
    await first.refresh()
    #expect(!fake.shown) // reads never mark the window as shown
    #expect(first.launchAction() == .show)
    #expect(fake.model().launchAction() == .none)
}

@Test @MainActor func hookActionsSurfaceStderrAndRefresh() async {
    let fake = SetupFakes(); let model = fake.model()
    await model.refresh(); fake.commandError = true; fake.calls = []
    await model.turnOnHooks()
    #expect(model.actionError == "apply refused")
    #expect(!model.hooksChanged)
    #expect(fake.calls == ["hook apply", "setup --json", "settings"])
    fake.commandError = false; fake.hooks = "ok"
    await model.removeHooks()
    #expect(model.actionError == nil)
    #expect(model.hooksChanged)
    #expect(model.hooks?.state == .ok)
}

@Test @MainActor func permissionChangesNotifyWatcherAndRequestRereadsSettings() async {
    let fake = SetupFakes(); let model = fake.model(); var refreshes = 0
    model.onPermissionChange = { refreshes += 1 }
    await model.refresh()
    #expect(refreshes == 0)
    fake.notification = SetupNotificationStatus(authorization: .authorized, alerts: .enabled)
    await model.pollTick()
    #expect(refreshes == 1)
    fake.notification = SetupNotificationStatus(authorization: .denied, alerts: .disabled)
    await model.turnOnNotifications()
    #expect(!model.notificationsGreen)
    #expect(refreshes == 3) // permission transition plus explicit post-request refresh
    #expect(fake.calls.suffix(3) == ["request", "setup --json", "settings"])
}

@Test @MainActor func hooksAttentionDotClearsWhenRefreshReadsGreen() async {
    for state in ["ok", "attention"] {
        let fake = SetupFakes(); let model = fake.model()
        await model.refresh()
        #expect(model.hooksFixableAtLaunch)
        fake.hooks = state
        await model.pollTick()
        #expect(model.hooksGreen)
        #expect(!model.hooksFixableAtLaunch)
        fake.hooks = "fixable"
        await model.refresh()
        #expect(!model.hooksFixableAtLaunch) // Launch attention does not reappear later.
    }
}
