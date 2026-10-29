# AGENTS.md — rules for Codex (implementer)

multi-link is a terminal dashboard for every Claude Code and Codex session on this machine.
It links each Codex session to the Claude session that handed it work, and groups sessions
by PLAN.md step. Design: `docs/design.md`.

## Roles
- **Claude** plans, reviews, and decides when a step is done. Only Claude edits `PLAN.md`.
- **Codex** (you) implements one PLAN.md step at a time and writes its tests.
- Whoever didn't write the code reviews it.

## How to work a step
1. Read the step in `PLAN.md` and the "Current state" in `PROGRESS.md`.
2. Change only the files the step names. If another file must change, stop and say why.
3. Run the step's **Verify** command. The step is done only when it produces the expected result.
4. Add a `PROGRESS.md` log entry (newest first) and update "Current state".
5. Do not commit or push. Stage nothing. Claude commits after review.

## Environment
- Rust: `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` before `cargo`.
- You run with network on only so cargo can fetch crates. Project code makes no network calls.
- If something blocks you (missing tool, unclear step), stop and say exactly what; don't work around
  the step's file list.

## Hard rules
- **Read-only access to agent data.** Never write to `~/.claude/` or `~/.codex/`.
- **No real transcripts in git.** Real session files go in `fixtures/raw/` (git-ignored).
  Only scrubbed copies go in `fixtures/`. See `fixtures/README.md`.
- **No network calls** in project code.
- **Fail soft on unknown records.** Skip and count them; never crash on a new transcript format.
- **No invented numbers.** Any metric in README or docs must come from a real output file.
- No secrets, tokens or personal paths in committed files.

## PROGRESS.md entry format
```
### YYYY-MM-DD — Step <id>: <title> — <agent>
- Did:
- Changed: <files>
- Verify: <command> → <actual output, pasted>
- Review:
- Gotchas:
```
