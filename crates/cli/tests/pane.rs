use std::{path::Path, process::Command};

#[test]
fn pane_matches_all_ten_pretty_json_keys_byte_for_byte() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = [
        (
            "busy",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-p --session 14150000-0000-4000-8000-000000001415",
        ),
        (
            "waiting-stale",
            "--root fixtures/sessions-phase9 --now 2026-11-09T07:30:00Z --cwd /home/dev/repo-p9c --session 92000012-0000-4000-8000-00009200000c",
        ),
        (
            "done-2h",
            "--root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z --cwd /home/dev/repo-p9l --session 95000004-0000-4000-8000-000095000004",
        ),
        (
            "children-exact-likely",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session 14120000-0000-4000-8000-000000001412",
        ),
        (
            "unseen",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session aaaaaaaa-0000-4000-8000-000000000000",
        ),
        (
            "ambiguous",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-r --session 141a0000-0000-4000-8000-00000000141a",
        ),
        (
            "child-other-repo",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-q --session 14150000-0000-4000-8000-000000001415",
        ),
        (
            "idle-recent",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session 14180000-0000-4000-8000-000000001418",
        ),
        (
            "empty",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-none --session aaaaaaaa-0000-4000-8000-000000000000",
        ),
        (
            "waiting-quiet",
            "--root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z --cwd /home/dev/repo-p9c --session 92000012-0000-4000-8000-00009200000c",
        ),
    ];
    assert_eq!(cases.len(), 10);
    for (name, args) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .current_dir(&root)
            .env("TZ", "UTC")
            .args(["pane", "--json"])
            .args(args.split_whitespace())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let expected =
            std::fs::read(root.join(format!("fixtures/expected/pane/{name}.json"))).unwrap();
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(expected).unwrap(),
            "{name}"
        );
    }
}
