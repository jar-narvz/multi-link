import Foundation
import Testing
import UserNotifications
@testable import MenubarCore

@MainActor private final class SetupFakes: SetupRunning, SetupNotificationSource, LoginItemService, SetupDefaults, TerminalLinkProbe {
    var calls: [String] = []
    var terminalStates: [String: TerminalLinkState] = [:]
    var terminalReads: [String] = []
    var terminalCLIs: [String] = []
    func state(at path: String, bundledCLI: String) -> TerminalLinkState {
        terminalReads.append(path)
        terminalCLIs.append(bundledCLI)
        return terminalStates[path] ?? .absent
    }
    var hooks = "fixable"
    var owners: [String]? = nil
    var takeOverClearsOwners = false
    var arguments: [[String]] = []
    var badJSON = false
    var commandError = false
    var notificationError = false
    var notification = SetupNotificationStatus(authorization: .notDetermined, alerts: .disabled)
    var status: LoginItemStatus = .notRegistered
    var loginError = false
    var shown = false
    func run(_ arguments: [String]) async throws -> CommandOutput {
        self.arguments.append(arguments)
        calls.append(arguments.joined(separator: " "))
        if arguments == ["setup", "--json"] {
            let ownersField = try owners.map { ",\"owners\":" + String(decoding: try JSONEncoder().encode($0), as: UTF8.self) } ?? ""
            return CommandOutput(status: 1, stdout: badJSON ? "bad" : """
            {"v":1,"items":[{"id":"hooks","state":"\(hooks)","file":"/fake/settings.json","exists":true\(ownersField),"events":[{"event":"PreToolUse","status":"customized","details":[]}]}]}
            """, stderr: "")
        }
        if arguments == ["hook", "apply", "--take-over"], takeOverClearsOwners, !commandError {
            owners = []; hooks = "ok"
        }
        return CommandOutput(status: commandError ? 1 : 0, stdout: "", stderr: commandError ? "apply refused" : "")
    }
    func settings() async -> SetupNotificationStatus { calls.append("settings"); return notification }
    func request() async throws {
        calls.append("request")
        if notificationError { throw FakeError.refused }
    }
    func register() throws { calls.append("register"); if loginError { throw FakeError.refused } }
    func unregister() throws { calls.append("unregister"); if loginError { throw FakeError.refused } }
    func openSettings() { calls.append("login settings") }
    func hasShownSetup() -> Bool { shown }
    func markSetupShown() { shown = true }
    func model(path: String = "/Applications/multi-link.app") -> SetupModel {
        SetupModel(runner: self, notifications: self, login: self, defaults: self, bundlePath: path, terminalProbe: self)
    }
}
private enum FakeError: LocalizedError { case refused; var errorDescription: String? { "Operation not permitted" } }

@Test @MainActor func hooksStatesAndInvalidJSON() async {
    let fake = SetupFakes(); let model = fake.model()
    for state in ["ok", "attention", "fixable", "unreadable"] {
        fake.hooks = state; await model.refresh()
        #expect(model.hooks?.state.rawValue == state)
        #expect(model.hooks?.owners == [])
        #expect(model.canTurnOnHooks == (state == "fixable"))
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
    #expect(fake.calls == ["hook apply --take-over", "request", "register", "setup --json", "settings"])
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
        #expect(model.terminalLinks.isEmpty)
        #expect(model.terminalCommand == nil)
        await model.refresh(); await model.turnOnEverything(); await model.turnOnHooks()
        await model.turnOnNotifications(); await model.removeHooks(); await model.setLoginEnabled(true)
        #expect(fake.calls.isEmpty)
        #expect(fake.terminalReads.isEmpty)
        #expect(model.terminalLinks.isEmpty)
        #expect(model.terminalCommand == nil)
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
    #expect(fake.calls == ["hook apply --take-over", "setup --json", "settings"])
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

@Test @MainActor func notificationActionsPreserveHookErrors() async {
    let fake = SetupFakes(); let model = fake.model()
    await model.refresh()
    fake.commandError = true
    await model.turnOnHooks()
    #expect(model.actionError == "apply refused")

    fake.notificationError = true
    fake.notification = SetupNotificationStatus(authorization: .denied, alerts: .disabled)
    await model.turnOnNotifications()
    #expect(model.actionError == "apply refused")
    fake.notification = SetupNotificationStatus(authorization: .authorized, alerts: .enabled)
    await model.pollTick()
    #expect(model.actionError == "apply refused")
    fake.notificationError = false
    await model.turnOnNotifications()
    #expect(model.actionError == "apply refused")

    for state in ["ok", "attention", "unreadable"] {
        fake.hooks = state
        await model.refresh()
        await model.turnOnEverything()
        #expect(model.actionError == "apply refused")
    }
    fake.hooks = "fixable"
    fake.commandError = false
    fake.notificationError = false
    await model.refresh()
    await model.turnOnEverything()
    #expect(model.actionError == nil) // A new hook apply clears its previous error.
}

@Test @MainActor func notificationErrorClearsOnlyWhenPollingReadsGreen() async {
    let fake = SetupFakes(); let model = fake.model()
    fake.notificationError = true
    fake.notification = SetupNotificationStatus(authorization: .denied, alerts: .disabled)
    await model.turnOnNotifications()
    #expect(model.notificationError == "Operation not permitted")
    #expect(model.actionError == nil)

    await model.pollTick()
    #expect(model.notificationError == "Operation not permitted")
    #expect(!model.notificationsGreen)
    for status in [
        SetupNotificationStatus(authorization: .authorized, alerts: .disabled),
        SetupNotificationStatus(authorization: .denied, alerts: .enabled),
        SetupNotificationStatus(authorization: .provisional, alerts: .enabled)
    ] {
        fake.notification = status
        await model.pollTick()
        #expect(model.notificationError == "Operation not permitted")
    }
    fake.notification = SetupNotificationStatus(authorization: .authorized, alerts: .enabled)
    await model.pollTick()
    #expect(model.notificationError == nil)
    #expect(model.notificationsGreen)
}

@Test @MainActor func notificationRetryClearsPreviousErrorEvenWhenStillDenied() async {
    for everything in [false, true] {
        let fake = SetupFakes(); let model = fake.model()
        fake.hooks = "ok"
        fake.notification = SetupNotificationStatus(authorization: .denied, alerts: .disabled)
        fake.notificationError = true
        if everything { await model.turnOnEverything() } else { await model.turnOnNotifications() }
        #expect(model.notificationError == "Operation not permitted")
        #expect(model.actionError == nil)
        fake.notificationError = false
        if everything { await model.turnOnEverything() } else { await model.turnOnNotifications() }
        #expect(model.notificationError == nil)
        #expect(!model.notificationsGreen)
    }
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

@Test @MainActor func terminalCommandQuotesSpaceAndSingleQuote() async {
    for (path, expected) in [
        ("/Applications/My App.app", "sudo mkdir -p /usr/local/bin && sudo ln -s '/Applications/My App.app/Contents/MacOS/multi-link' /usr/local/bin/multi-link"),
        ("/Applications/Jared's App.app", "sudo mkdir -p /usr/local/bin && sudo ln -s '/Applications/Jared'\\''s App.app/Contents/MacOS/multi-link' /usr/local/bin/multi-link")
    ] {
        let fake = SetupFakes(); let model = fake.model(path: path)
        #expect(model.terminalCommand == nil) // No offer until both paths have been checked.
        await model.refresh()
        #expect(model.terminalCommand == expected)
    }
}

@Test @MainActor func terminalPlacesRefreshIndependentlyAndNeverCountAsSetup() async {
    let fake = SetupFakes()
    let model = SetupModel(runner: fake, notifications: fake, login: fake, defaults: fake,
                           bundlePath: "/Applications/multi-link.app", terminalProbe: fake,
                           terminalPlaces: ["/test/local", "/test/brew"])
    for (state, message) in [
        (TerminalLinkState.linked, "multi-link is linked at /test/local."),
        (.other(detail: "/another/cli"), "/test/local is something else (/another/cli), so it's left as is."),
        (.broken(target: "missing"), "/test/local is a broken link to missing, so it's left as is."),
        (.absent, "")
    ] {
        fake.terminalStates = ["/test/local": state]
        await model.refresh()
        #expect(model.terminalLinks == [TerminalLink(path: "/test/local", state: state), TerminalLink(path: "/test/brew", state: .absent)])
        #expect(model.terminalLinks.map(\.message) == [message, ""])
        #expect((model.terminalCommand != nil) == (state == .absent))
        #expect(model.remaining == 2)
    }
    fake.terminalStates = ["/test/local": .broken(target: "missing"), "/test/brew": .other(detail: "/other")]
    await model.refresh()
    #expect(model.terminalCommand == nil)
    #expect(model.terminalLinks.map(\.message) == ["/test/local is a broken link to missing, so it's left as is.", "/test/brew is something else (/other), so it's left as is."])
    fake.calls = []
    await model.turnOnEverything()
    #expect(fake.calls == ["hook apply --take-over", "request", "register", "setup --json", "settings"])
    #expect(model.remaining == 2)
    #expect(model.terminalLinks.map(\.state) == [.broken(target: "missing"), .other(detail: "/other")])
    fake.hooks = "ok"
    fake.notification = SetupNotificationStatus(authorization: .authorized, alerts: .enabled)
    await model.refresh()
    #expect(model.remaining == 0)
    #expect(model.terminalCommand == nil)
    #expect(fake.terminalReads == Array(repeating: ["/test/local", "/test/brew"], count: 7).flatMap { $0 })
}

@Test @MainActor func terminalOfferRequiresBothPlacesAbsent() async {
    let states: [TerminalLinkState] = [.absent, .linked, .other(detail: "/other/cli"), .broken(target: "missing")]
    let fake = SetupFakes(); let model = fake.model()
    for local in states {
        for brew in states {
            fake.terminalStates = ["/usr/local/bin/multi-link": local, "/opt/homebrew/bin/multi-link": brew]
            await model.refresh()
            #expect(model.terminalLinks.map(\.state) == [local, brew])
            #expect((model.terminalCommand != nil) == (local == .absent && brew == .absent))
            #expect(model.remaining == 2)
        }
    }
    #expect(fake.terminalCLIs.count == 32)
    #expect(fake.terminalCLIs.allSatisfy { $0 == "/Applications/multi-link.app/Contents/MacOS/multi-link" })
    fake.terminalStates = ["/usr/local/bin/multi-link": .linked]
    await model.refresh()
    #expect(model.terminalLinks.first?.message == "multi-link is linked at /usr/local/bin/multi-link.")
}

@Test @MainActor func otherCopyHooksNeedAttentionAndExplicitTakeover() async {
    for state in ["ok", "fixable", "attention"] {
        for everything in [false, true] {
            let fake = SetupFakes(); let model = fake.model()
            fake.hooks = state
            fake.owners = ["/other/bin/multi-link"]
            fake.notification = SetupNotificationStatus(authorization: .authorized, alerts: .enabled)
            await model.refresh()
            #expect(!model.hooksGreen)
            #expect(model.hooksHaveOtherOwners)
            #expect(model.canTurnOnHooks) // SetupView uses this for Turn on visibility.
            #expect(model.hooks?.owners == ["/other/bin/multi-link"])
            #expect(model.remaining == 1)
            #expect(model.hooksFixableAtLaunch)
            #expect(model.hooksMessage == "Hooks use another copy: /other/bin/multi-link")
            await model.pollTick()
            #expect(model.hooksFixableAtLaunch)
            fake.calls = []; fake.arguments = []
            fake.takeOverClearsOwners = true
            if everything { await model.turnOnEverything() } else { await model.turnOnHooks() }
            #expect(fake.arguments.first == ["hook", "apply", "--take-over"])
            #expect(fake.calls == (everything
                ? ["hook apply --take-over", "request", "register", "setup --json", "settings"]
                : ["hook apply --take-over", "setup --json", "settings"]))
            #expect(model.hooksGreen)
            #expect(model.hooks?.owners == [])
            #expect(!model.hooksHaveOtherOwners)
            #expect(!model.canTurnOnHooks)
            #expect(model.remaining == 0)
            #expect(!model.hooksFixableAtLaunch)
            #expect(model.hooksChanged)
            fake.arguments = []
            await model.removeHooks()
            #expect(fake.arguments.first == ["hook", "remove"])
        }
    }
}

@Test @MainActor func multipleHookOwnersShowFirstAndCount() async {
    let fake = SetupFakes(); let model = fake.model()
    fake.hooks = "ok"
    fake.owners = ["/a/multi-link", "/b/multi-link", "/c/multi-link"]
    await model.refresh()
    #expect(model.hooksMessage == "Hooks use another copy: /a/multi-link +2 more")
}

@Test func hooksSetupDecodesOwnersAndOlderCLI() throws {
    for (field, expected) in [
        ("", [String]()),
        (",\"owners\":[]", []),
        (",\"owners\":[\"/other/bin/multi-link\"]", ["/other/bin/multi-link"])
    ] {
        let json = """
        {"state":"ok","file":"/fake/settings.json","events":[]\(field)}
        """
        let hooks = try JSONDecoder().decode(HooksSetup.self, from: Data(json.utf8))
        #expect(hooks.owners == expected)
        #expect(hooks.state == .ok)
        #expect(hooks.file == "/fake/settings.json")
        #expect(hooks.events.isEmpty)
    }
}

@Test @MainActor func ownerFreeHooksOnlyApplyWhenFixable() async {
    for owners: [String]? in [nil, []] {
        for state in ["ok", "attention", "fixable", "unreadable"] {
            let fake = SetupFakes(); let model = fake.model()
            fake.hooks = state; fake.owners = owners
            await model.refresh()
            #expect(model.hooks?.owners == [])
            #expect(!model.hooksHaveOtherOwners)
            #expect(model.canTurnOnHooks == (state == "fixable"))
            fake.arguments = []
            await model.turnOnHooks()
            #expect(fake.arguments == (state == "fixable"
                ? [["hook", "apply", "--take-over"], ["setup", "--json"]] : []))
        }
    }
}
