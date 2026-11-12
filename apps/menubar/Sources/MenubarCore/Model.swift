import Foundation
import Observation

public struct Report: Decodable, Equatable, Sendable {
    public let v: Int
    public let now: String
    public let waiting: Int
    public let running: Bool
    public let repos: [Repository]
}

public struct Repository: Decodable, Equatable, Sendable {
    public let repo: String
    public let name: String
    public let tier: String
    public let last_activity: String
    public let step: RepoStep?
    public let rows: [Row]
}

public struct RepoStep: Decodable, Equatable, Sendable {
    public let id: String
    public let title: String?
    public let more: Int
}

public struct Row: Decodable, Equatable, Sendable {
    public let section: String
    public let id: String
    public let harness: String
    public let label: String
    public let step: String?
    public let state: String
    public let phrase: String
    public let asked: String?
    public let said: String?
    public let openable: Bool
    public let opens: String?
    public let children: [Child]
    public let handoffs: [Handoff]
}

public struct Child: Decodable, Equatable, Sendable {
    public let id: String
    public let harness: String
    public let state: String
    public let phrase: String
    public let asked: String?
    public let said: String?
    public let openable: Bool
    public let opens: String?
    public let match: String
}

public struct Handoff: Decodable, Equatable, Sendable {
    public let step: String?
    public let outcome: String
    public let candidates: [String]
}

public struct Completion: Decodable, Equatable, Sendable {
    public let kind: String
    public let run: String
    public let id: String
    public let step: String?
    public let said: String?
}

public enum WireMessage: Equatable, Sendable {
    case report(Report)
    case tick(String)
    case error(String)
    case completion(Completion)
    case tooOld(Int)
    case malformed

    public static func decode(_ data: Data) throws -> Self {
        struct Envelope: Decodable {
            let v: Int
            let tick: String?
            let error: String?
            let event: Completion?
        }
        let decoder = JSONDecoder()
        // Check version before decoding any version-specific payload.
        struct Version: Decodable { let v: Int }
        let version = try decoder.decode(Version.self, from: data).v
        guard version == 1 else { return .tooOld(version) }
        let envelope = try decoder.decode(Envelope.self, from: data)
        if let error = envelope.error { return .error(error) }
        if let tick = envelope.tick { return .tick(tick) }
        if let event = envelope.event {
            return event.kind == "codex_done" ? .completion(event) : .malformed
        }
        return .report(try decoder.decode(Report.self, from: data))
    }
}

/// Buffer bytes, not strings: a pipe read can end inside a UTF-8 scalar.
public struct LineReader: Sendable {
    private var pending = Data()
    public init() {}

    public mutating func append(_ data: Data) -> [WireMessage] {
        pending.append(data)
        var messages: [WireMessage] = []
        while let end = pending.firstIndex(of: 10) {
            let line = Data(pending[..<end])
            pending.removeSubrange(...end)
            messages.append((try? WireMessage.decode(line)) ?? .malformed)
        }
        return messages
    }
}

public enum WatcherStatus: Equatable, Sendable {
    case idle, watching, tooOld, quit
    case notFound([String])
    case failed(String)
    case stopped(String, retryIn: Double)
}

public enum WatcherUpdate: Sendable {
    case line(WireMessage)
    case status(WatcherStatus)
    case delivery(Bool)
}

@MainActor @Observable
public final class MenubarModel {
    public private(set) var report: Report?
    public private(set) var checkedAt: String?
    public private(set) var status: WatcherStatus = .idle
    public private(set) var ownsDelivery = false
    public var actionNotice: String?
    public private(set) var showsNotificationsOff = false
    private var notificationsOffDismissed = false

    public func updatePermission(_ allowed: Bool) {
        showsNotificationsOff = !allowed && !notificationsOffDismissed
    }

    public func dismissNotificationsOff() {
        notificationsOffDismissed = true
        showsNotificationsOff = false
    }

    public var onCompletion: ((Completion) -> Void)?

    public init() {}

    public func receive(_ update: WatcherUpdate) {
        switch update {
        case .status(let value): status = value
        case .delivery(let value): ownsDelivery = value
        case .line(let line):
            switch line {
            case .report(let value):
                report = value
                checkedAt = value.now
                status = .watching
            case .tick(let value): checkedAt = value
            case .error(let error): status = .failed(error)
            case .tooOld: status = .tooOld
            case .malformed: status = .failed("Invalid watcher output")
            case .completion(let event): onCompletion?(event)
            }
        }
    }
}
