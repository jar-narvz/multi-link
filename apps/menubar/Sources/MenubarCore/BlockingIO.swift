import Foundation

/// Blocking calls (pipe reads, waiting for a child) run on their own threads, never on Swift's cooperative pool.
/// That pool has one thread per CPU core; a blocking read parked there holds a thread until the child writes or
/// exits. A few watchers and actions in parallel (2 readers per watcher) then use every thread of a 3-core CI
/// runner, and the tasks that would quit those children can never run: the 2026-11-14 CI hang (Step 22.5).
enum BlockingIO {
    /// Runs `work` on a new thread and suspends (without holding a pool thread) until it returns.
    static func run<T: Sendable>(_ work: @escaping @Sendable () -> T) async -> T {
        await withCheckedContinuation { continuation in
            Thread.detachNewThread { continuation.resume(returning: work()) }
        }
    }

    /// Reads `handle` to EOF on its own thread and calls `body` with each chunk, in order, as soon as it arrives;
    /// closes the handle at the end. The thread reads the next chunk only after `body` returns, so a chatty child
    /// is held back by the full pipe (as with a plain read loop) instead of filling memory. Cancelling the caller
    /// cannot interrupt a pending read: the reader ends at EOF, which callers get by ending the child (`stopChild`).
    static func forEachChunk(of handle: FileHandle, _ body: (Data) async -> Void) async {
        let consumed = DispatchSemaphore(value: 0)
        let chunks = AsyncStream<Data> { continuation in
            // Wakes a waiting reader if the stream ends early (the consuming task was cancelled).
            continuation.onTermination = { _ in consumed.signal() }
            Thread.detachNewThread {
                defer {
                    try? handle.close()
                    continuation.finish()
                }
                while let data = readChunk(handle) {
                    if case .terminated = continuation.yield(data) { return }
                    consumed.wait()
                }
            }
        }
        for await data in chunks {
            await body(data)
            consumed.signal()
        }
    }

    /// Foundation's read(upToCount:) may wait to fill the requested size on pipes.
    /// One POSIX read returns the bytes available now, keeping heartbeats live.
    static func readChunk(_ handle: FileHandle) -> Data? {
        var bytes = [UInt8](repeating: 0, count: 65536)
        while true {
            let count = Darwin.read(handle.fileDescriptor, &bytes, bytes.count)
            if count > 0 { return Data(bytes.prefix(count)) }
            if count < 0 && errno == EINTR { continue }
            return nil
        }
    }
}
