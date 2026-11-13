import Darwin
import Foundation
import UserNotifications

public protocol NotificationPermission: Sendable {
    func canDeliver() async -> Bool
}

public struct SystemNotificationPermission: NotificationPermission {
    public init() {}
    public func canDeliver() async -> Bool {
        let settings = await UNUserNotificationCenter.current().notificationSettings()
        return Self.canDeliver(authorizationStatus: settings.authorizationStatus, alertSetting: settings.alertSetting)
    }

    static func canDeliver(authorizationStatus: UNAuthorizationStatus, alertSetting: UNNotificationSetting) -> Bool {
        authorizationStatus == .authorized && alertSetting == .enabled
    }
}

/// Held by AppController for the app lifetime, independent of permission or watcher health.
public final class InstanceLock {
    private let path: String
    private var descriptor: Int32 = -1
    public init(path: String) { self.path = path }
    deinit { release() }

    public func acquire() -> Bool {
        if descriptor >= 0 { return true }
        do {
            try FileManager.default.createDirectory(
                at: URL(fileURLWithPath: path).deletingLastPathComponent(), withIntermediateDirectories: true)
        } catch { return false }
        let fd = Darwin.open(path, O_CREAT | O_RDWR | O_CLOEXEC, mode_t(0o600))
        guard fd >= 0 else { return false }
        guard flock(fd, LOCK_EX | LOCK_NB) == 0 else {
            Darwin.close(fd)
            return false
        }
        descriptor = fd
        return true
    }

    public var closeOnExec: Bool {
        descriptor >= 0 && fcntl(descriptor, F_GETFD) & FD_CLOEXEC != 0
    }

    public func release() {
        guard descriptor >= 0 else { return }
        flock(descriptor, LOCK_UN)
        Darwin.close(descriptor)
        descriptor = -1
    }
}
