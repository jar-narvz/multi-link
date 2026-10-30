# PLAN.md — repo-l

Source of truth for what gets built, in order.

## Goal
Show linked sessions on one home screen.

## Steps

### Step 4.1 — Linker: current-directory flags
- Files: `src/invocation.rs`
- **Verify:** `cargo test invocation`

### Step 4.3 — Core: merge quiet rows
- Files: `src/overview.rs`
- **Verify:** `cargo test overview`

### Step 4.4 — TUI: header summary
- Files: `src/home.rs`
- **Verify:** `cargo test home`

### Step 4.5 — TUI: mini time bars
- Files: `src/home.rs`
- **Verify:** `cargo test home`

### Step 4.6 — Real check and README
- Files: `README.md`
- **Verify:** `cargo test --workspace`
