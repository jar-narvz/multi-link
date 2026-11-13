import Darwin
import Foundation

public struct WatcherTiming: Sendable {
    public var initialBackoff: Double
    public var maximumBackoff: Double
    public var maintenance: Double
    public var permissionPoll: Double

    public init(initialBackoff: Double = 1, maximumBackoff: Double = 30,
                maintenance: Double = 1, permissionPoll: Double = 60) {
        precondition(initialBackoff > 0 && maximumBackoff >= initialBackoff && maintenance > 0 && permissionPoll > 0)
        self.initialBackoff = initialBackoff
        self.maximumBackoff = maximumBackoff
        self.maintenance = maintenance
        self.permissionPoll = permissionPoll
    }
}

/// Owns child-process work off the main actor; the consumer marshals updates to its UI actor.
public actor Watcher {
    private let binary: String
    private let permission: any NotificationPermission
    private let timing: WatcherTiming
    private let publish: @Sendable (WatcherUpdate) async -> Void
    private var runner: Task<Void, Never>?
    private var maintenanceTask: Task<Void, Never>?
    private var process: Process?
    private var quitting = false
    private var incompatible = false
    private var allowed = false
    private var lastPermissionCheck = Date.distantPast
    private var reader = LineReader()
    private var stderrTail = Data()
    private var oldCommand = false
    private var notifyMode = false
    private var rotating = false
    private var delivery = false

    public init(binary: String, permission: any NotificationPermission,
                timing: WatcherTiming = .init(),
                onUpdate: @escaping @Sendable (WatcherUpdate) async -> Void) {
        self.binary = binary
        self.permission = permission
        self.timing = timing
        self.publish = onUpdate
    }

    public func start() async {
        guard runner == nil, !quitting else { return }
        // Install tasks before any await, so a concurrent start cannot create a second child.
        runner = Task { await self.supervise() }
        maintenanceTask = Task { await self.maintain() }
    }

    /// Call at launch (also done by start), when the popover opens, and after requesting permission.
    /// The requestAuthorization callback deliberately never enters this state machine.
    public func refreshPermission() async {
        let value = await permission.canDeliver()
        guard !quitting else { return }
        allowed = value
        lastPermissionCheck = Date()
        await reconcileDelivery()
    }

    public func quit() async {
        guard !quitting else { return }
        quitting = true
        runner?.cancel()
        maintenanceTask?.cancel()
        stopChild()
        await reconcileDelivery()
        await runner?.value
        await publish(.status(.quit))
    }

    private func maintain() async {
        while !Task.isCancelled && !quitting {
            if Date().timeIntervalSince(lastPermissionCheck) >= timing.permissionPoll {
                await refreshPermission()
            }
            await reconcileDelivery()
            do { try await Task.sleep(for: .seconds(timing.maintenance)) }
            catch { return }
        }
    }

    private func reconcileDelivery() async {
        let held = allowed && !quitting && !incompatible
        if held != delivery {
            delivery = held
            await publish(.delivery(held))
        }
        // A non-delivering watcher must not claim permanent notified/<run> markers.
        // Rotate once when permission changes.
        if let process, process.isRunning, notifyMode != held, !rotating {
            rotating = true
            stopChild()
        }
    }

    private func supervise() async {
        await refreshPermission()
        var delay = timing.initialBackoff
        while !quitting && !Task.isCancelled && !incompatible {
            reader = LineReader()
            stderrTail = Data()
            oldCommand = false
            rotating = false
            notifyMode = delivery
            let child = Process()
            let stdout = Pipe()
            let stderr = Pipe()
            child.executableURL = URL(fileURLWithPath: binary)
            child.arguments = ["menubar", "--json", "--watch"] + (notifyMode ? ["--notify"] : [])
            child.standardOutput = stdout
            child.standardError = stderr
            child.standardInput = FileHandle.nullDevice
            var failure: String
            do {
                let outputTask = Task.detached { [self] in
                    let handle = stdout.fileHandleForReading
                    defer { try? handle.close() }
                    while let data = Self.readChunk(handle) {
                        await self.readOutput(data)
                    }
                }
                let errorTask = Task.detached { [self] in
                    let handle = stderr.fileHandleForReading
                    defer { try? handle.close() }
                    while let data = Self.readChunk(handle) {
                        await self.readError(data)
                    }
                }
                // Install the handler before launch, including for children that exit immediately.
                // Suspension leaves every executor (and the main run loop) free to make progress.
                let launchError: Error? = await withCheckedContinuation { continuation in
                    child.terminationHandler = { _ in continuation.resume(returning: nil) }
                    do {
                        try child.run()
                        process = child
                    } catch {
                        child.terminationHandler = nil
                        continuation.resume(returning: error)
                    }
                    // Parent write ends must close even when launch fails, so readers reach EOF.
                    try? stdout.fileHandleForWriting.close()
                    try? stderr.fileHandleForWriting.close()
                }
                await outputTask.value
                await errorTask.value
                if let launchError { throw launchError }
                failure = String(decoding: stderrTail, as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)
                if failure.isEmpty { failure = "Watcher exited (\(child.terminationStatus))" }
            } catch {
                try? stdout.fileHandleForWriting.close()
                try? stderr.fileHandleForWriting.close()
                try? stdout.fileHandleForReading.close()
                try? stderr.fileHandleForReading.close()
                failure = error.localizedDescription
            }
            process = nil
            guard !quitting && !Task.isCancelled else { return }
            if oldCommand || incompatible {
                incompatible = true
                await reconcileDelivery()
                await publish(.status(.tooOld))
                return
            }
            if rotating {
                // Intentional mode changes do not introduce restart backoff.
                continue
            }
            await reconcileDelivery()
            await publish(.status(.stopped(failure, retryIn: delay)))
            do { try await Task.sleep(for: .seconds(delay)) }
            catch { return }
            delay = min(delay * 2, timing.maximumBackoff)
        }
    }

    private func readOutput(_ data: Data) async {
        guard !quitting, !incompatible else { return }
        for message in reader.append(data) {
            if case .tooOld = message {
                incompatible = true
                stopChild()
            }
            await reconcileDelivery()
            if case .completion = message {
                if delivery && notifyMode { await publish(.line(message)) }
            } else {
                await publish(.line(message))
                if case .report = message { await publish(.status(.watching)) }
            }
            if incompatible { return }
        }
    }

    private func readError(_ data: Data) {
        guard !quitting else { return }
        // Bounded retention, continuous draining. Keep enough overlap to recognize split diagnostics.
        stderrTail.append(data)
        let diagnostic = String(decoding: stderrTail, as: UTF8.self).lowercased()
        if diagnostic.contains("menubar") &&
            (diagnostic.contains("unrecognized subcommand") || diagnostic.contains("unknown subcommand")) {
            oldCommand = true
        }
        if stderrTail.count > 16384 { stderrTail = Data(stderrTail.suffix(16384)) }
    }

    /// Foundation's read(upToCount:) may wait to fill the requested size on pipes.
    /// One POSIX read returns the bytes available now, keeping heartbeats live.
    private nonisolated static func readChunk(_ handle: FileHandle) -> Data? {
        var bytes = [UInt8](repeating: 0, count: 65536)
        while true {
            let count = Darwin.read(handle.fileDescriptor, &bytes, bytes.count)
            if count > 0 { return Data(bytes.prefix(count)) }
            if count < 0 && errno == EINTR { continue }
            return nil
        }
    }

    private func stopChild() {
        guard let child = process, child.isRunning else { return }
        child.terminate()
        // A stuck child must not prevent Quit or a mode change forever.
        Task.detached {
            try? await Task.sleep(for: .seconds(2))
            if child.isRunning { Darwin.kill(child.processIdentifier, SIGKILL) }
        }
    }
}
