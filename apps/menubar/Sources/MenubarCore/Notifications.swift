import Foundation
import UserNotifications

public final class CompletionNotifications: NSObject, UNUserNotificationCenterDelegate, Sendable {
    public static let presentationOptions: UNNotificationPresentationOptions = [.banner, .sound]
    private let submit: @Sendable (UNNotificationRequest) async throws -> Void

    public init(submit: @escaping @Sendable (UNNotificationRequest) async throws -> Void = {
        try await UNUserNotificationCenter.current().add($0)
    }) {
        self.submit = submit
    }

    public static func request(_ event: Completion) -> UNNotificationRequest {
        let content = UNMutableNotificationContent()
        content.title = event.step.map { "codex finished (\($0))" } ?? "codex finished"
        content.body = event.said ?? ""
        content.sound = .default
        return UNNotificationRequest(identifier: event.run, content: content, trigger: nil)
    }

    public func deliver(_ event: Completion) async throws { try await submit(Self.request(event)) }

    public func userNotificationCenter(_ center: UNUserNotificationCenter,
                                       willPresent notification: UNNotification) async -> UNNotificationPresentationOptions {
        Self.presentationOptions
    }
}
