import AppKit
import Foundation
import UserNotifications

/// System effects live here; views only render the model and forward user actions.
@MainActor
public final class AppController {
    public let model = MenubarModel()
    // UNUserNotificationCenter keeps a weak delegate: retain it for the app's lifetime.
    public let notifications = CompletionNotifications()
    private var watcher: Watcher?
    private var binary: String?
    private var started = false
    private var quitting = false

    private let resolveBinary: @MainActor () -> BinaryResolution
    private let permission: any NotificationPermission
    private let instanceLock: InstanceLock
    public private(set) var alreadyRunning = false
    private let requestAuthorization: @MainActor (CompletionNotifications) async -> Void

    public init(
        resolveBinary: @escaping @MainActor () -> BinaryResolution = {
            BinaryLookup().resolve(savedPath: UserDefaults.standard.string(forKey: "multiLinkBinary"))
        },
        permission: any NotificationPermission = SystemNotificationPermission(),
        lockPath: String = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".multi-link/menubar.lock").path,
        requestAuthorization: @escaping @MainActor (CompletionNotifications) async -> Void = { delegate in
            let center = UNUserNotificationCenter.current()
            center.delegate = delegate
            _ = try? await center.requestAuthorization(options: [.alert, .sound])
        }
    ) {
        self.resolveBinary = resolveBinary
        self.permission = permission
        self.instanceLock = InstanceLock(path: lockPath)
        self.requestAuthorization = requestAuthorization
    }

    public func start() async {
        guard !started, !quitting else { return }
        started = true
        guard instanceLock.acquire() else {
            alreadyRunning = true
            model.actionNotice = "multi-link is already running"
            return
        }
        model.onCompletion = { [weak self] event in
            guard let self else { return }
            Task {
                do { try await self.notifications.deliver(event) }
                catch { self.model.actionNotice = "notification failed: \(error.localizedDescription)" }
            }
        }
        let permission = ReportingPermission(model: model, source: permission)
        switch resolveBinary() {
        case .notFound(let paths):
            model.receive(.status(.notFound(paths)))
            _ = await permission.canDeliver()
        case .found(let path):
            binary = path
            let watcher = Watcher(binary: path, permission: permission) { [weak model] update in
                await model?.receive(update)
            }
            self.watcher = watcher
            await watcher.start()
        }
        // The watcher runs while the owner answers the prompt. The callback's result
        // never drives delivery: re-read settings when it returns, even after an error.
        guard !quitting else { return }
        await requestAuthorization(notifications)
        guard !quitting else { return }
        if let watcher {
            await watcher.refreshPermission()
        } else {
            _ = await permission.canDeliver()
        }
    }

    public func popoverOpened() async { await watcher?.refreshPermission() }

    public func activate(_ row: RowPresentation) async {
        switch row.action {
        case .copy:
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(row.id, forType: .string)
            model.actionNotice = "copied id"
        case .open:
            guard let binary else { return }
            do {
                let result = try await MenubarActions.run(binary: binary, arguments: MenubarActions.openArguments(id: row.id))
                model.actionNotice = MenubarActions.result(status: result.status, stdout: result.stdout,
                                                          stderr: result.stderr, opens: row.opens)
            } catch { model.actionNotice = error.localizedDescription }
        }
    }

    public func openTUI() async {
        guard let binary else { model.actionNotice = "multi-link not found"; return }
        do {
            let result = try await MenubarActions.run(binary: "/usr/bin/osascript", arguments: MenubarActions.tuiArguments(binary: binary))
            model.actionNotice = MenubarActions.tuiResult(result)
        } catch { model.actionNotice = "could not open TUI: \(error.localizedDescription)" }
    }

    public func openNotificationSettings() {
        NSWorkspace.shared.open(URL(string: MenubarActions.notificationSettings)!)
        model.dismissNotificationsOff()
    }

    public func quit() async {
        quitting = true
        await watcher?.quit()
        instanceLock.release()
    }
}

private struct ReportingPermission: NotificationPermission {
    let model: MenubarModel
    let source: any NotificationPermission
    func canDeliver() async -> Bool {
        let allowed = await source.canDeliver()
        await model.updatePermission(allowed)
        return allowed
    }
}
