# Phase 0 — dependency questions (c) and (d)

Step 0.3 of `PLAN.md`. Merged into `docs/phase0.md` in Step 0.6.

**Method and its limit.** The owner's safety policy for this step: no cloning, downloading to disk,
building, installing or running third-party code. Both projects were read **only through the GitHub
API with `gh`**, streamed to the terminal and never saved. So every answer below comes from reading
source at one commit, not from running anything. PLAN.md Step 0.3 also asked to run agtrace on the
0.2a sandbox sessions; that was not done (see (d), "What running it would add").

Shared helper used for every file read below (`$REPO` and `$SHA` per section):

```
gh api "repos/$REPO/contents/<path>?ref=$SHA" --jq .content | base64 -d   # piped to grep / sed -n
```

---

## (c) Is zoetrope's core usable as a library crate?

- Repo: https://github.com/furkankly/zoetrope
- Commit read: `b1f31dd26bd4e9e513885e39edb78d0850a5d1fe` (HEAD of `main`, committed 2026-09-11)
- License: MIT (`gh api repos/furkankly/zoetrope --jq .license.spdx_id`, plus `license = "MIT"` in `Cargo.toml`)

Commands used:

```
gh api repos/furkankly/zoetrope/commits/HEAD --jq '.sha+" "+.commit.committer.date'
gh api repos/furkankly/zoetrope --jq '.license.spdx_id, .default_branch'
gh api "repos/furkankly/zoetrope/git/trees/$SHA?recursive=1" --jq '.tree[] | select(.type=="blob") | .path'
# then the contents helper on each file below, filtered with grep -n '^\s*pub ...' or sed -n ranges
```

Files relied on: `Cargo.toml`, `src/lib.rs`, `src/provider/mod.rs`, `src/fact.rs`,
`src/state/mod.rs`, `src/state/graph.rs`, `src/state/session.rs`, `src/tailer/mod.rs`,
`src/provider/summary.rs`, `src/provider/codex/mod.rs`, `src/provider/codex/discovery.rs`,
`src/provider/claude/discovery.rs`, `docs/DISCOVERY.md`, `README.md`.

### Findings

**Layout.** One package, `zoetrope` 0.2.0 (edition 2024, MSRV 1.88), with a lib target
(`src/lib.rs`) and a bin target `zoe` (`src/main.rs`, `required-features = ["native"]`). The
`[workspace]` only exists to exclude `web/wasm` (the browser frontend, its own crate
`zoetrope-web`). Published on crates.io as `zoetrope` (README badge and `cargo install zoetrope`).

**Is there a library?** Yes. `src/lib.rs` documents a "portable core" usable with
`default-features = false`, and that is exactly how `zoetrope-web` uses it. Public modules:
`fact`, `provider`, `state`, `tailer`, `ui`; `autopilot`, `handler`, `tui` are native-only and
explicitly have "no stability promise". `lib.rs` says the parts meant to be depended on are the
domain (`state`) and the vocabulary (`fact`). But the crate docs also say "this is an application",
and it is at 0.x, so there is no API stability guarantee.

**Public parsing API** (`src/provider/mod.rs`):
- `enum Provider { Claude, Codex }` with `name`, `parse`, `all_paths`, `session_file`,
  `session_file_from`, `related_paths`, `project_key`, `stream_for`, `sidecar`.
- `enum Stream` with `push(&mut self, line) -> Option<Statement>` — the per-file parser.
- `fn provider_of(head)` — sniffs the format from the first line's content.
- `struct SessionFile { provider, path, session, role, read, project_key, modified }`,
  `enum FileRole { Root, Agent { parent }, Sidecar }`, `enum ReadMode`, `struct Session`
  (`every_file`, `rescan`), `struct Scope`, `enum Target`, `enum OpenError`.
- Free functions `assemble`, `sweep(scope, only)`, `open(target, only)`.
- The format-specific parsers themselves (`provider::claude`, `provider::codex`, their `wire.rs`
  serde models, and `summary`) are **`pub(crate)`**. A dependent only ever sees `Fact`s.

**Public model** (`src/fact.rs`, `src/state/`):
- `fact::Fact { agent, ts, kind }`, `fact::FactKind` (`Agent { kind, parent, agent_type,
  description, spawned_by, interactive }`, `Activity`, `Label`, `Model`, `Tokens`, `Prompt`,
  `Reasoning`, `ToolStart { call, name, summary }`, `ToolEnd { call, outcome }`, `Spawn`, `Ended`,
  `Session { label, value }`, `Tally`, `Title`), `fact::Statement { at, facts }`,
  `fact::Outcome { Ok, Err }`, `AgentKind`, `AgentStatus`.
- `state::session::SessionModel` (the fold of facts), `AgentInfo`, `ToolCallInfo`, `SpawnContext`.
- `state::AgentFlow` = `rataflow::Flow<AgentNode, AgentEdge>`; `state::graph` (`new_flow`, `sync`,
  `relayout`) is a **UI layout projection** for the TUI, not a general graph model.

**Dependency weight (core, `default-features = false`).** `ratatui` 0.30 (no default features),
`rataflow` 0.1 (with `sugiyama` layout), `serde`, `serde_json`, `chrono`, `web-time`,
`unicode-width`, `imbl` 7. The `native` default adds `tokio` (multi-thread), `crossterm`,
`futures`, `anyhow`. So even the "core" pulls in a terminal UI framework and a graph-layout crate
that multi-link (a CLI that prints JSON) does not need.

**Fit against what multi-link needs** (field names here are assumptions until Phase 0 confirms
them on real data):
1. **Lossy by design.** Tool-call input reaches a dependent only as `ToolStart.summary`, a one-line
   rendering capped at 200 chars (`SUMMARY_MAX` in `src/provider/summary.rs`). Tool output is
   reduced to `ToolEnd.outcome` = `Ok`/`Err`; the text is dropped. multi-link's `exact` link needs
   the child Codex session id, which (per the design) we expect to see in the `codex exec` tool
   result text or in the full command. Output text never survives the `Fact` boundary, and full
   commands are not reliably preserved (short Bash commands ≤ 200 chars survive whole, so a
   `resume <id>` argument can survive; longer ones are cut).
2. **No cross-harness linking.** zoetrope's parent/child is within one harness: Claude subagents
   and workflows (`<uuid>/subagents/agent-*.jsonl`) and Codex child threads
   (`session_meta.source.subagent.thread_spawn.parent_thread_id`). `assemble` groups by
   `(provider, session)`; it never relates a Claude session to a Codex one.
3. **Raw records are private.** We could not reach Codex `session_meta` fields like `originator`
   except via `FactKind::Session { label: "app", value }`, a display string.

### Decision

No-go on depending on it as a crate: the public surface is a lossy display vocabulary (truncated
summaries, outputs reduced to Ok/Err), the raw parsers are `pub(crate)`, the core drags in
`ratatui` + `rataflow` + `imbl`, and the API has no stability promise. We write our own parsers.

Worth learning from (credit zoetrope, MIT, in our README / source comments if we adapt anything):
- The provider split: `wire.rs` (defensive serde: unknown types, missing fields and bad lines parse
  to something skippable, never a panic), `discovery.rs` (pure path logic, at most a head read),
  `mod.rs` (per-file stream). Same shape suits our parsers.
- "A provider states what a file is; the core decides what a session is" (`docs/DISCOVERY.md`).
- Content-based format sniffing (`provider_of`: Codex lines carry `payload` / `session_meta`).
- Layout facts to check in Phase 0 (assumptions, from their docs): Claude
  `~/.claude/projects/<sanitized-cwd>/<uuid>.jsonl` plus `<uuid>/subagents/`; Codex
  `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`, with `thread_source` and
  `source.subagent.thread_spawn.parent_thread_id` in `session_meta`, and a child rollout replaying
  its parent's `session_meta` further down (only the first one is the file's own).
- Their golden-file conformance tests (`assets/<name>/<fixture>.model.txt` / `.timeline.txt`).

Decision: no-go

---

## (d) Does agtrace already link a Claude Code session to the Codex session it launched?

Definition for this section:
- **Decision: go** = agtrace does NOT already link a Claude session to the Codex session it
  launched (Claude → Codex hand-off), so multi-link's differentiator holds.
- **Decision: no-go** = it does.

- Repo: https://github.com/lanegrid/agtrace
- Commit read: `7649b7ec861d3d199985bbc5fb6d25c2009e0048` (HEAD of `main`, committed 2026-10-29;
  workspace version 0.8.0)
- License: `gh api` reports Apache-2.0; the repo ships both `LICENSE-APACHE` and `LICENSE-MIT`.

Commands used:

```
gh api repos/lanegrid/agtrace/commits/HEAD --jq '.sha+" "+.commit.committer.date'
gh api repos/lanegrid/agtrace --jq '.license.spdx_id, .default_branch'
gh api "repos/lanegrid/agtrace/git/trees/$SHA?recursive=1" --jq '.tree[] | select(.type=="blob") | .path'
# keyword sweep over every .rs file under crates/agtrace-{providers,engine,index,runtime,core}/src:
#   contents helper | grep -n -i -E 'codex exec|"codex"|originator|cross.?provider|cross.?harness|
#                                   launched|external.?session|linked_session|child_session|parent_session'
# then targeted reads (grep / sed -n) of the files below; and `grep -n -i codex` over the Claude
# provider files listed below.
```

Files relied on:
- Where transcripts are read: `crates/agtrace-providers/src/claude/` (`decoder.rs`, `header.rs`,
  `discovery.rs`, `tools.rs`, `tool_mapping.rs`, `mapper.rs`) and
  `crates/agtrace-providers/src/codex/` (`header.rs`, `records.rs`, `collab.rs`, `exec.rs`,
  `discovery.rs`); roots in `crates/agtrace-core/src/roots.rs`.
- Where sessions are related: `crates/agtrace-runtime/src/ops/index.rs` (fills
  `parent_session_id` / `root_session_id`), `crates/agtrace-index/src/schema.rs`,
  `crates/agtrace-index/src/queries/session.rs` (`get_children`),
  `crates/agtrace-index/src/db.rs` (`get_child_sessions`),
  `crates/agtrace-runtime/src/client/sessions.rs`, `crates/agtrace-engine/src/workspace/parent.rs`.
- `README.md` (feature claims).

### Findings

**What agtrace links (all within one harness):**
1. **Codex → Codex child threads.** `codex/header.rs` reads `session_meta.parent_thread_id` or
   `source.subagent.thread_spawn.parent_thread_id` (plus `agent_path`, `agent_nickname`) into
   `agent.parent` via `AgentId::codex_thread`. `codex/collab.rs` handles Codex multi_agent v2
   `spawn_agent` / `send_message` addressing (`/root/judge`).
2. **Claude → Claude subagents, forks and teammates.** `engine/workspace/parent.rs`
   (`teammate_parent`, `team_lead_agent`, `RuntimeAliases`, `teammate_spawns`) puts a teammate under
   the agent whose log holds its `AgentSpawn`, else under the team lead (`leadSessionId`).
3. **The index.** In `runtime/src/ops/index.rs`, `parent_session_id` is set by provider:
   `ProviderId::Codex` → the Codex parent thread's session id; `ProviderId::ClaudeCode` → the team
   lead. There is no third branch and no case that crosses providers.
4. **Project grouping, not session linking.** `index/src/schema.rs` notes a repository hash that
   "allows cross-provider session grouping": sessions of both tools in the same repo are listed
   together. That says "same project", not "this Claude session launched that Codex run".

**What it does not link:** a Claude Code session to the Codex session it launched (e.g. a Bash
tool call running `codex exec`, or a Codex MCP call). Evidence:
- The keyword sweep found no `codex exec`, `originator`, or cross-provider/session-link code in
  any of the five library crates. The only `codex` hits are provider registration
  (`registry.rs`, `traits.rs`), log roots, and tests.
- `grep -i codex` over the Claude provider files (`decoder.rs`, `tools.rs`, `tool_mapping.rs`,
  `mapper.rs`, `discovery.rs`, `header.rs`) and `tool_analyzer.rs` returned nothing: the Claude
  side never looks for Codex in a command or tool result.
- `codex/exec.rs` is about Codex's *own* `exec` tool (a JS program inside a Codex session calling
  `tools.exec_command`), not the `codex exec` CLI launched from Claude.
- The README describes the tree as "the lead, its teammates, background subagents, forks, and
  Codex child threads"; it claims no Claude→Codex linking.

**Confidence:** high for this commit. The stored relation is one column (`parent_session_id`)
whose only writer branches on provider as described. The live tree also assigns parents in memory
(`workspace/view.rs::link`, found in Codex review), but it is fed only by each provider's own spawn
evidence (Claude teammates/subagents, Codex `thread_spawn`); neither provider emits a cross-provider
spawn target, so no Claude→Codex edge can form there either. Limits: this is from
reading source, not running it, and grep can miss a link built under a name we did not search for
(the parent derivation code was read directly to reduce that risk). agtrace moves fast (HEAD was
committed today), so re-check before publishing a comparison.

**Run on the 0.2a sandbox sessions** (owner approved, 2026-10-29, after the source read): built
`agtrace` at 7649b7e outside the repo; ran with `HOME` and `AGTRACE_PATH` set to a throwaway folder
holding copies of only the sandbox transcripts (`agtrace init --all-projects`, then
`agtrace session list --all-projects --format json`). Its index held 9 `claude_code` and 10 `codex`
sessions; `select count(*) from sessions where parent_session_id is not null` → 0. It grouped
sessions by checkout (`project_root`) but linked no Claude session to its Codex child, matching the
source reading. The original list of what a run would add, kept for reference:
- Confirm end to end that `agtrace session show` / the agent tree / MCP `get_agent_tree` show the
  Claude session and the Codex run it launched as separate, unlinked roots (expected from source).
- Confirm it parses the owner's real Claude Code and Codex versions at all (README: Claude Code
  ≥ 2.1.24x, Codex ≥ 0.153), checked against `fixtures/raw/ground-truth.md`.
- Show whether the repository-hash grouping puts both sessions under the same project, which is the
  closest thing it has to a link and the baseline multi-link is compared against.
- If ever run: point it at a throwaway tree with `AGTRACE_CLAUDE_HOME` / `AGTRACE_CODEX_HOME`
  (`agtrace-core/src/roots.rs`), and check where its SQLite index is written and whether `init`
  touches `~/.claude` / `~/.codex` before doing so.

Decision: go
