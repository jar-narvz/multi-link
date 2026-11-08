#!/usr/bin/env python3
"""Print the owner's Codex usage and say whether to use Codex or Claude-only mode.

Reads the newest `rate_limits` object Codex wrote into its rollouts under
~/.codex/sessions (undocumented field, seen in codex-cli 0.164.0 on 2026-11-07).
Exit 0 = Codex OK, 1 = Codex LOW (a live window at or above the threshold),
2 = unknown (no rate_limits found). Read-only; prints no prompt text.
"""
import glob
import json
import os
import sys
import time

THRESHOLD = 90.0  # percent of a live window


def latest_rate_limits():
    pattern = os.path.expanduser("~/.codex/sessions/*/*/*/*.jsonl")
    files = sorted(glob.glob(pattern), key=os.path.getmtime, reverse=True)[:20]
    for path in files:
        found = None
        with open(path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                if '"rate_limits"' not in line:
                    continue
                try:
                    payload = json.loads(line).get("payload", {})
                except ValueError:
                    continue
                limits = payload.get("rate_limits") or payload.get("info", {}).get("rate_limits")
                if isinstance(limits, dict):
                    found = limits
        if found:
            return found
    return None


def main():
    limits = latest_rate_limits()
    if not limits:
        print("CODEX: UNKNOWN (no rate_limits in recent rollouts)")
        return 2
    now = time.time()
    low = False
    for name in ("primary", "secondary"):
        window = limits.get(name) or {}
        resets = window.get("resets_at") or 0
        used = float(window.get("used_percent") or 0) if resets > now else 0.0
        when = time.strftime("%a %H:%M", time.localtime(resets)) if resets else "?"
        print(f"{name}: {used:.0f}% of {window.get('window_minutes')} min window, resets {when}")
        low = low or used >= THRESHOLD
    print("CODEX: LOW -> Claude-only mode until the window resets" if low else "CODEX: OK")
    return 1 if low else 0


if __name__ == "__main__":
    sys.exit(main())
