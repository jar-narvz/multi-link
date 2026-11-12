import Foundation

public enum BoardTint: Sendable { case foreground, dim, claude, codex, ok, warn, stale }

public struct BoardPalette: Sendable {
    public let foreground: UInt32
    public let background: UInt32
    public let line: UInt32
    public let dim: UInt32
    public let selection: UInt32
    public let claude: UInt32
    public let codex: UInt32
    public let ok: UInt32
    public let warn: UInt32
    public let stale: UInt32

    public init(dark: Bool) {
        foreground = dark ? 0xece8e1 : 0x1d1c1a
        background = dark ? 0x201f1d : 0xfbfaf8
        line = dark ? 0x34312d : 0xe2ded8
        dim = dark ? 0x958f85 : 0x7a756d
        selection = dark ? 0x2c2a27 : 0xece8e2
        claude = dark ? 0xdf9b79 : 0xc2714a
        codex = dark ? 0x8cb9f1 : 0x3f7fd0
        ok = dark ? 0x7fbf8e : 0x3f8f55
        warn = dark ? 0xd6a243 : 0xb07a12
        stale = dark ? 0xe08a7c : 0xa3463a
    }

    public func color(_ tint: BoardTint) -> UInt32 {
        switch tint {
        case .foreground: foreground
        case .dim: dim
        case .claude: claude
        case .codex: codex
        case .ok: ok
        case .warn: warn
        case .stale: stale
        }
    }
}

public enum RowAction: Equatable, Sendable {
    case open(String), copy
    public var title: String {
        switch self { case .open(let title): title; case .copy: "Copy id" }
    }
}

public struct RowPresentation: Identifiable, Sendable {
    public let id: String
    public let label: String
    public let phrase: String
    public let symbol: String
    public let tint: BoardTint
    public let isChild: Bool
    public let action: RowAction
    public let opens: String?

    public init(id: String, label: String, phrase: String, state: String, harness: String,
                opens: String?, isChild: Bool = false) {
        self.id = id
        self.label = label
        self.phrase = phrase
        self.symbol = Self.symbol(state: state)
        self.tint = Self.tint(state: state, harness: harness)
        self.isChild = isChild
        self.action = Self.action(opens: opens)
        self.opens = opens
    }

    public static func symbol(state: String) -> String {
        switch state {
        case "busy": "●"
        case "waiting", "stale": "◆"
        case "idle": "○"
        case "finished": "✓"
        default: "·"
        }
    }

    public static func tint(state: String, harness: String) -> BoardTint {
        switch state {
        case "busy": harness == "claude" ? .claude : .codex
        case "waiting": .warn
        case "stale": .stale
        case "finished": .ok
        default: .dim
        }
    }

    public static func action(opens: String?) -> RowAction {
        switch opens {
        case "app": .open("Open in app")
        case "terminal": .open("Jump to terminal")
        default: .copy
        }
    }

    public static func childLabel(harness: String, match: String) -> String {
        "└ \(harness) · \(match == "heuristic" ? "likely" : match)"
    }
}

public struct RepoBlock: Identifiable, Sendable {
    public let id: String
    public let header: String
    public let rows: [RowPresentation]

    public init(_ repo: Repository) {
        id = repo.repo
        header = repo.name + (repo.step.map {
            " · step \($0.id)" + ($0.title.map { " " + $0 } ?? "") + ($0.more > 0 ? " +\($0.more)" : "")
        } ?? "")
        rows = repo.rows.flatMap { row in
            [RowPresentation(id: row.id, label: row.harness + " · " + row.label, phrase: row.phrase, state: row.state,
                             harness: row.harness, opens: row.opens)] + row.children.map { child in
                RowPresentation(id: child.id, label: RowPresentation.childLabel(harness: child.harness, match: child.match),
                                phrase: child.phrase, state: child.state, harness: child.harness,
                                opens: child.opens, isChild: true)
            }
        }
    }
}

extension MenubarModel {
    public var iconText: String {
        if let report, report.waiting > 0 { return "ml \(report.waiting)" }
        return report?.running == true ? "ml" : "ml ○"
    }

    // The CLI is the ordering authority (tier, activity, path); preserve its array verbatim.
    public var blocks: [RepoBlock] { report?.repos.map(RepoBlock.init) ?? [] }

    public func footer(now: Date) -> String {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        let date = checkedAt.flatMap { formatter.date(from: $0) }
        let checked = date.map { "checked \(max(0, Int(now.timeIntervalSince($0))))s ago" } ?? "not checked yet"
        let message: String?
        switch status {
        case .idle: message = "Starting watcher…"
        case .watching, .quit: message = nil
        case .tooOld: message = "multi-link too old · rebuild the CLI"
        case .notFound(let paths): message = "multi-link not found · tried " + paths.joined(separator: ", ")
        case .failed(let error): message = error
        case .stopped(let error, let delay): message = "Watcher stopped: \(error) · retrying in \(Int(delay))s"
        }
        return [checked, message, actionNotice].compactMap { $0 }.joined(separator: " · ")
    }
}
