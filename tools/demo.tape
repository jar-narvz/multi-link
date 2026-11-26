# Records the README demo GIF from the synthetic fixture (no real prompts or paths).
# Run from the repo root:
#   cargo build --release -p multi-link
#   vhs tools/demo.tape
# Screens: home -> repo timeline -> step graph -> live grid.

Output "docs/img/demo-phase14-fixture.gif"

Set Shell bash
Set Width 1400
Set Height 800
Set FontSize 14
Set Padding 20
Set Theme "Builtin Dark"

Hide
Type "clear; TZ=America/Los_Angeles ./target/release/multi-link tui --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --since all"
Enter
Sleep 3s
Show

# Home: every repo, who needs you, what is working.
Sleep 3s

# Repo timeline: Claude and Codex lanes, with the confirmed link.
Enter
Sleep 3s

# Step graph: plan -> Codex implements -> verify + commit.
Type "s"
Sleep 3.5s

# Live grid: what every agent is doing now.
Escape
Sleep 500ms
Type "l"
Sleep 4s

Hide
Type "q"
Sleep 300ms
Type "q"
