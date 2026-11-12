import Foundation

public enum MenubarActions {
    public static let notificationSettings = "x-apple.systempreferences:com.apple.Notifications-Settings.extension"
    public static func openArguments(id: String) -> [String] { ["open", "--no-resume", id] }
    public static func tuiCommand(binary: String) -> String {
        "'" + binary.replacingOccurrences(of: "'", with: "'\\''") + "' tui"
    }
    public static func tuiArguments(binary: String) -> [String] {
        ["-e", "on run argv", "-e", "tell application \"Terminal\" to do script (item 1 of argv)",
         "-e", "tell application \"Terminal\" to activate", "-e", "end run", tuiCommand(binary: binary)]
    }
    public static func result(status: Int32, stdout: String, stderr: String, opens: String?) -> String {
        let output = (status == 0 ? stdout : stderr).trimmingCharacters(in: .whitespacesAndNewlines)
        if status == 0 { return output.isEmpty ? (opens == "terminal" ? "jumped to terminal" : "opened in app") : output }
        let message = output.isEmpty ? "could not open session" : output
        return message + (message.contains("osascript timed out after 5 s")
                          ? " · if macOS asked for permission, try again" : "")
    }

    public static func tuiResult(_ result: CommandOutput) -> String {
        if result.status == 0 { return "opened TUI" }
        let error = result.stderr.trimmingCharacters(in: .whitespacesAndNewlines)
        return "could not open TUI" + (error.isEmpty ? "" : ": " + error)
    }

    /// Drain both pipes concurrently, away from the UI actor.
    public static func run(binary: String, arguments: [String]) async throws -> CommandOutput {
        try await Task.detached {
            let process = Process()
            let output = Pipe()
            let error = Pipe()
            process.executableURL = URL(fileURLWithPath: binary)
            process.arguments = arguments
            process.standardInput = FileHandle.nullDevice
            process.standardOutput = output
            process.standardError = error
            try process.run()
            try? output.fileHandleForWriting.close()
            try? error.fileHandleForWriting.close()
            async let stdout = drain(output.fileHandleForReading)
            async let stderr = drain(error.fileHandleForReading)
            process.waitUntilExit()
            return await CommandOutput(status: process.terminationStatus, stdout: stdout, stderr: stderr)
        }.value
    }

    private static func drain(_ handle: FileHandle) async -> String {
        await Task.detached {
            defer { try? handle.close() }
            return String(decoding: handle.readDataToEndOfFile(), as: UTF8.self)
        }.value
    }
}

public struct CommandOutput: Sendable {
    public let status: Int32
    public let stdout: String
    public let stderr: String
}
