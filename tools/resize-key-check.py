"""Resize + key check for the TUI (Phase 13, item 3). Synthetic fixture only; reads no real data.

Usage, from the repo root: python3 -I tools/resize-key-check.py BIN [ROUNDS]
Runs `BIN tui` on fixtures/sessions-phase6 in a pseudo-terminal, stops reading its output so the TUI blocks
while writing, resizes the terminal several times, sends `q` together with one more resize, then starts
reading again. Pass: the TUI exits cleanly (status 0) on that single `q` within 4 s. Fail: it is still running (the key was
delayed); a second `q` is then sent so the run ends. Prints one line per round and exits 1 if any failed.
"""
import fcntl, os, pty, select, signal, struct, sys, termios, time

BIN = sys.argv[1]
ROUNDS = int(sys.argv[2]) if len(sys.argv) > 2 else 5
ARGS = [BIN, "tui", "--root", "fixtures/sessions-phase6", "--now", "2026-11-03T06:25:00Z", "--since", "2h"]
SIZES = [(40, 160), (32, 120)]
FINAL = (36, 140)  # differs from both, so the last resize is a real SIGWINCH


def size(fd, rows, cols):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def drain(fd, seconds, pid):
    """Read output for up to `seconds`; return None while running, else the exit status (0 = clean exit)."""
    end = time.time() + seconds
    while time.time() < end:
        done, status = os.waitpid(pid, os.WNOHANG)
        if done:
            return os.waitstatus_to_exitcode(status)
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try:
                os.read(fd, 65536)
            except OSError:
                pass
    done, status = os.waitpid(pid, os.WNOHANG)
    return os.waitstatus_to_exitcode(status) if done else None


def round_once():
    pid, fd = pty.fork()
    if pid == 0:
        os.environ["COLORTERM"] = "truecolor"
        os.execv(BIN, ARGS)
    size(fd, *SIZES[0])
    if drain(fd, 3.0, pid) is not None:  # first frame; the TUI must still be running
        os.close(fd)
        return "FAIL (exited before the check)"
    for i in range(8):  # stalled reader: redraws fill the pty buffer and block the TUI in a write
        size(fd, *SIZES[(i + 1) % 2])
        time.sleep(0.15)
    os.write(fd, b"q")
    size(fd, *FINAL)  # SIGWINCH and the key become ready together
    time.sleep(2.0)
    code = drain(fd, 4.0, pid)
    if code == 0:
        result = "pass"
    elif code is not None:
        result = f"FAIL (exit status {code})"
    else:
        os.write(fd, b"q")
        if drain(fd, 3.0, pid) is not None:
            result = "FAIL (key delayed until a second key)"
        else:
            result = "FAIL (no exit)"
            os.kill(pid, signal.SIGKILL)
            os.waitpid(pid, 0)
    os.close(fd)
    return result


results = [round_once() for _ in range(ROUNDS)]
for n, r in enumerate(results, 1):
    print(f"round {n}: {r}")
sys.exit(0 if all(r == "pass" for r in results) else 1)
