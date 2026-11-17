import Foundation
import Observation
import ServiceManagement
import UserNotifications

@MainActor public protocol SetupRunning { func run(_ arguments: [String]) async throws -> CommandOutput }
@MainActor public protocol SetupNotificationSource {
    func settings() async -> SetupNotificationStatus
    func request() async throws
}
public struct SetupNotificationStatus {
    public let authorization: UNAuthorizationStatus
    public let alerts: UNNotificationSetting
    public init(authorization: UNAuthorizationStatus, alerts: UNNotificationSetting) {
        self.authorization = authorization; self.alerts = alerts
    }
    public var green: Bool { authorization == .authorized && alerts == .enabled }
}
public enum SetupLaunchAction: Equatable { case gate, show, none }
public enum LoginItemStatus: Equatable { case enabled, notRegistered, notFound, requiresApproval, unknown }
@MainActor public protocol LoginItemService {
    var status: LoginItemStatus { get }
    func register() throws
    func unregister() throws
    func openSettings()
}
@MainActor public protocol SetupDefaults {
    func hasShownSetup() -> Bool
    func markSetupShown()
}
public struct HooksSetup: Decodable {
    public enum State: String, Decodable { case ok, attention, fixable, unreadable }
    public struct Event: Decodable { public let event: String; public let status: String }
    public let state: State
    public let file: String
    public let events: [Event]
}
private struct SetupReport: Decodable {
    let v: Int
    let items: [Item]
    struct Item: Decodable {
        let id: String
        let state: String
        let file: String?
        let events: [HooksSetup.Event]?
    }
}

/// Reads and explicit actions are separate so opening/polling can never request permissions.
@MainActor @Observable public final class SetupModel {
    public private(set) var hooks: HooksSetup?
    public private(set) var readError: String?
    public private(set) var actionError: String?
    public private(set) var notificationError: String?
    public private(set) var loginError: String?
    public private(set) var notification = SetupNotificationStatus(authorization: .notDetermined, alerts: .disabled)
    public private(set) var loginStatus: LoginItemStatus = .notRegistered
    public private(set) var hooksChanged = false
    public private(set) var hooksFixableAtLaunch = false
    public private(set) var busy = false
    public private(set) var terminalLinks: [TerminalLink] = []
    public let isTemporary: Bool
    public let bundlePath: String
    private let terminalProbe: any TerminalLinkProbe
    private let terminalPlaces: [String]
    private let runner: any SetupRunning
    private let notifications: any SetupNotificationSource
    private let login: any LoginItemService
    private let defaults: any SetupDefaults
    private var readAtLaunch = false
    public var onPermissionChange: (@MainActor () async -> Void)?

    public init(runner: any SetupRunning, notifications: any SetupNotificationSource,
                login: any LoginItemService, defaults: any SetupDefaults, bundlePath: String,
                terminalProbe: any TerminalLinkProbe = FileSystemTerminalLinkProbe(),
                terminalPlaces: [String] = ["/usr/local/bin/multi-link", "/opt/homebrew/bin/multi-link"]) {
        self.runner = runner; self.notifications = notifications; self.login = login; self.defaults = defaults
        self.terminalProbe = terminalProbe; self.terminalPlaces = terminalPlaces
        self.bundlePath = bundlePath
        isTemporary = bundlePath.hasPrefix("/Volumes/") || bundlePath.contains("/AppTranslocation/")
    }
    public var terminalCommand: String? {
        guard !isTemporary, terminalLinks.count == 2,
              terminalLinks.allSatisfy({ $0.state == .absent }) else { return nil }
        let cli = bundlePath + "/Contents/MacOS/multi-link"
        let quoted = "'" + cli.replacingOccurrences(of: "'", with: "'\\''") + "'"
        return "sudo mkdir -p /usr/local/bin && sudo ln -s \(quoted) /usr/local/bin/multi-link"
    }
    public var hooksGreen: Bool { hooks?.state == .ok || hooks?.state == .attention }
    public var notificationsGreen: Bool { notification.green }
    public var loginEnabled: Bool { loginStatus == .enabled }
    public var remaining: Int { (hooksGreen ? 0 : 1) + (notificationsGreen ? 0 : 1) }
    public var hooksMessage: String {
        switch hooks?.state {
        case .ok: "On. Exact hand-off links and \"waiting on you\"."
        case .attention:
            "On, except \(hooks!.events.filter { $0.status == "customized" || $0.status == "unknown event" || $0.status == "unknown_event" }.map(\.event).joined(separator: ", ")): you changed that one, so it's left as is."
        case .fixable: "Exact hand-off links and \"waiting on you\"."
        case .unreadable: "Couldn't read Claude settings. Check the file, then try again."
        case nil: readError ?? "Checking Claude hooks…"
        }
    }
    public func launchAction() -> SetupLaunchAction {
        guard !isTemporary else { return .gate }
        guard !defaults.hasShownSetup() else { return .none }
        defaults.markSetupShown()
        return .show
    }
    public func refresh() async {
        guard !isTemporary else { return }
        terminalLinks = terminalPlaces.map {
            TerminalLink(path: $0, state: terminalProbe.state(at: $0, bundledCLI: bundlePath + "/Contents/MacOS/multi-link"))
        }
        do {
            let output = try await runner.run(["setup", "--json"])
            guard output.status == 0 || output.status == 1 else {
                throw SetupFailure.message(output.stderr.isEmpty ? "Could not read setup." : output.stderr)
            }
            let report = try JSONDecoder().decode(SetupReport.self, from: Data(output.stdout.utf8))
            guard report.v == 1, let item = report.items.first(where: { $0.id == "hooks" }),
                  let state = HooksSetup.State(rawValue: item.state), let file = item.file, let events = item.events else {
                throw SetupFailure.message("Unsupported setup response.")
            }
            hooks = HooksSetup(state: state, file: file, events: events); readError = nil
        } catch { hooks = nil; readError = error.localizedDescription }
        if !readAtLaunch { hooksFixableAtLaunch = hooks?.state == .fixable; readAtLaunch = true }
        if hooksGreen { hooksFixableAtLaunch = false }
        let previous = notification.green
        notification = await notifications.settings()
        if notification.green { notificationError = nil }
        loginStatus = login.status
        if previous != notification.green { await onPermissionChange?() }
    }
    public func pollTick() async { guard !busy else { return }; await refresh() }
    private func changeHooks(_ verb: String) async {
        do {
            let output = try await runner.run(["hook", verb])
            if output.status != 0 { throw SetupFailure.message(output.stderr.isEmpty ? "Hook \(verb) failed." : output.stderr.trimmingCharacters(in: .whitespacesAndNewlines)) }
            hooksChanged = true
        } catch { actionError = error.localizedDescription }
    }
    public func turnOnHooks() async {
        guard !isTemporary, !busy, hooks?.state == .fixable else { return }
        busy = true; actionError = nil; await changeHooks("apply"); await refresh(); busy = false
    }
    public func removeHooks() async {
        guard !isTemporary, !busy else { return }
        busy = true; actionError = nil; await changeHooks("remove"); await refresh(); busy = false
    }
    private func requestNotifications() async {
        notificationError = nil
        do { try await notifications.request() } catch { notificationError = error.localizedDescription }
    }
    public func turnOnNotifications() async {
        guard !isTemporary, !busy else { return }
        busy = true; await requestNotifications(); await refresh()
        await onPermissionChange?(); busy = false
    }
    private func changeLogin(_ enabled: Bool) {
        loginError = nil
        do { if enabled { try login.register() } else { try login.unregister() } }
        catch { loginError = error.localizedDescription }
        loginStatus = login.status
    }
    public func setLoginEnabled(_ enabled: Bool) async {
        guard !isTemporary, !busy else { return }
        changeLogin(enabled)
    }
    public func openLoginSettings() { guard !isTemporary else { return }; login.openSettings() }
    public func turnOnEverything() async {
        guard !isTemporary, !busy else { return }
        busy = true
        if hooks?.state == .fixable { actionError = nil; await changeHooks("apply") }
        await requestNotifications()
        changeLogin(true)
        await refresh(); await onPermissionChange?(); busy = false
    }
}
private enum SetupFailure: LocalizedError {
    case message(String)
    var errorDescription: String? { switch self { case .message(let text): text } }
}

@MainActor public struct BundledSetupRunner: SetupRunning {
    public init() {}
    public func run(_ arguments: [String]) async throws -> CommandOutput {
        guard let binary = Bundle.main.executableURL?.deletingLastPathComponent()
            .appendingPathComponent("multi-link").path,
              FileManager.default.isExecutableFile(atPath: binary) else {
            throw SetupFailure.message("Bundled multi-link CLI not found.")
        }
        return try await MenubarActions.run(binary: binary, arguments: arguments)
    }
}
@MainActor public struct SystemSetupNotifications: SetupNotificationSource {
    public init() {}
    public func settings() async -> SetupNotificationStatus {
        let value = await UNUserNotificationCenter.current().notificationSettings()
        return SetupNotificationStatus(authorization: value.authorizationStatus, alerts: value.alertSetting)
    }
    public func request() async throws {
        _ = try await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])
    }
}
@MainActor public struct SystemLoginItem: LoginItemService {
    public init() {}
    public var status: LoginItemStatus {
        switch SMAppService.mainApp.status {
        case .enabled: .enabled
        case .notRegistered: .notRegistered
        case .notFound: .notFound
        case .requiresApproval: .requiresApproval
        @unknown default: .unknown
        }
    }
    public func register() throws { try SMAppService.mainApp.register() }
    public func unregister() throws { try SMAppService.mainApp.unregister() }
    public func openSettings() { SMAppService.openSystemSettingsLoginItems() }
}
@MainActor public struct SystemSetupDefaults: SetupDefaults {
    private let store: UserDefaults
    public init(_ store: UserDefaults = .standard) { self.store = store }
    public func hasShownSetup() -> Bool { store.bool(forKey: "multiLinkSetupShown") }
    public func markSetupShown() { store.set(true, forKey: "multiLinkSetupShown") }
}
