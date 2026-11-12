import Foundation

public enum BinaryResolution: Equatable, Sendable {
    case found(String)
    case notFound([String])
}

public struct BinaryLookup: Sendable {
    public let home: String
    public let systemDirectories: [String]

    public init(home: String = FileManager.default.homeDirectoryForCurrentUser.path,
                systemDirectories: [String] = ["/opt/homebrew/bin", "/usr/local/bin"]) {
        self.home = home
        self.systemDirectories = systemDirectories
    }

    public func candidates(savedPath: String?) -> [String] {
        var paths: [String] = []
        if let savedPath, !savedPath.isEmpty { paths.append(savedPath) }
        paths.append(home + "/.cargo/bin/multi-link")
        paths += systemDirectories.map { $0 + "/multi-link" }
        return paths
    }

    public func resolve(savedPath: String?) -> BinaryResolution {
        let paths = candidates(savedPath: savedPath)
        if let path = paths.first(where: {
            var directory: ObjCBool = false
            return FileManager.default.fileExists(atPath: $0, isDirectory: &directory)
                && !directory.boolValue && FileManager.default.isExecutableFile(atPath: $0)
        }) { return .found(path) }
        return .notFound(paths)
    }
}
