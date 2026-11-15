import Darwin
import Foundation

public enum TerminalLinkState: Equatable {
    case linked
    case other(detail: String)
    case broken(target: String)
    case absent
}

public struct TerminalLink: Equatable {
    public let path: String
    public let state: TerminalLinkState
    public init(path: String, state: TerminalLinkState) { self.path = path; self.state = state }
    public var message: String {
        switch state {
        case .linked: "multi-link is linked at \(path)."
        case .other(let detail): "\(path) is something else (\(detail)), so it's left as is."
        case .broken(let target): "\(path) is a broken link to \(target), so it's left as is."
        case .absent: ""
        }
    }
}

@MainActor public protocol TerminalLinkProbe {
    func state(at path: String, bundledCLI: String) -> TerminalLinkState
}

/// Inspection only: lstat preserves dangling links; realpath follows the entire
/// chain and fails on missing targets and loops. Never launches a process.
public struct FileSystemTerminalLinkProbe: TerminalLinkProbe {
    public init() {}
    public func state(at path: String, bundledCLI: String) -> TerminalLinkState {
        var info = stat()
        guard lstat(path, &info) == 0 else {
            let failure = errno
            if failure == ENOENT || failure == ENOTDIR { return .absent }
            if failure == ELOOP { return .broken(target: path) }
            return .other(detail: "could not inspect: \(String(cString: strerror(failure)))")
        }
        guard info.st_mode & S_IFMT == S_IFLNK else {
            return .other(detail: info.st_mode & S_IFMT == S_IFDIR ? "directory" :
                            info.st_mode & S_IFMT == S_IFREG ? "regular file" : "occupied path")
        }
        let target = (try? FileManager.default.destinationOfSymbolicLink(atPath: path)) ?? path
        guard let resolved = realpath(path, nil) else {
            let failure = errno
            if failure == ENOENT || failure == ENOTDIR || failure == ELOOP { return .broken(target: target) }
            return .other(detail: "\(target) (could not resolve: \(String(cString: strerror(failure))))")
        }
        defer { free(resolved) }
        let resolvedPath = String(cString: resolved)
        guard let cli = realpath(bundledCLI, nil) else { return .other(detail: resolvedPath) }
        defer { free(cli) }
        return resolvedPath == String(cString: cli) ? .linked : .other(detail: resolvedPath)
    }
}
