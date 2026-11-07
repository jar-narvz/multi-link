use multi_link_model::{CallKind, classify_bash, edit_label, quoted_string};

#[test]
fn bash_prefixes_and_fixed_labels() {
    for prefix in [
        "",
        "export PATH=/synthetic; ",
        "export PATH=/synthetic\n",
        "cd /synthetic && ",
        "VAR=x ",
        "export A=b; cd /synthetic && VAR=x ",
    ] {
        assert_eq!(
            classify_bash(&format!("{prefix}git diff -- private")),
            (CallKind::Read, Some("$ git diff".into()))
        );
    }
    for command in [
        "cat",
        "sed",
        "grep",
        "rg",
        "head",
        "tail",
        "ls",
        "awk",
        "wc",
        "find",
        "git status",
        "git log",
        "git show",
    ] {
        assert_eq!(
            classify_bash(&format!("{command} 'private; text'")),
            (CallKind::Read, None)
        );
    }
    for (command, label) in [
        ("git commit -m private", "$ git commit"),
        ("cargo test --workspace", "$ cargo test"),
        ("cargo clippy", "$ cargo clippy"),
        ("cargo fmt", "$ cargo fmt"),
        ("cargo build", "$ cargo build"),
        ("pytest -q", "$ pytest"),
        ("npm test -- private", "$ npm test"),
    ] {
        assert_eq!(classify_bash(command), (CallKind::Run, Some(label.into())));
    }
    assert_eq!(classify_bash("python3 write.py"), (CallKind::Run, None));
    assert_eq!(classify_bash("export A=b"), (CallKind::Run, None));
}

#[test]
fn quoted_strings_and_basename_only() {
    assert_eq!(quoted_string(r#""a\nb\"c" rest"#), Some("a\nb\"c".into()));
    assert_eq!(quoted_string("'a\\nb'"), Some("a\nb".into()));
    assert_eq!(quoted_string("variable"), None);
    assert_eq!(
        edit_label(" /synthetic/private/parser.rs "),
        Some("edit parser.rs".into())
    );
}

#[test]
fn compound_labels_prefer_commit_and_skip_heredoc_bodies() {
    for (command, kind, label) in [
        (
            "git add a && git commit -m x",
            CallKind::Run,
            Some("$ git commit"),
        ),
        (
            "cargo test && git commit -m x",
            CallKind::Run,
            Some("$ git commit"),
        ),
        (
            "git diff; git commit -m x",
            CallKind::Read,
            Some("$ git commit"),
        ),
        ("cat > f <<'E'\ngit commit -m x\nE", CallKind::Read, None),
        (
            "cat > f <<'E'\ngit commit -m x\nE\ngit commit -m y",
            CallKind::Read,
            Some("$ git commit"),
        ),
        (
            "git commit -m \"$(cat <<'EOF'\nsynthetic message\nEOF\n)\"",
            CallKind::Run,
            Some("$ git commit"),
        ),
        ("echo \"git commit\"", CallKind::Run, None),
        ("echo x | VAR=x git diff", CallKind::Run, Some("$ git diff")),
        (
            "echo x & cd /synthetic && export A=b; VAR=x cargo test",
            CallKind::Run,
            Some("$ cargo test"),
        ),
        (
            "echo x\ncargo fmt; cargo test",
            CallKind::Run,
            Some("$ cargo fmt"),
        ),
        (
            "cat <<- E\ngit commit -m x\n  E  \ncargo test",
            CallKind::Read,
            Some("$ cargo test"),
        ),
        ("cat <<\"E\"\ngit commit -m x", CallKind::Read, None),
        (
            "cat <<< text\ngit commit -m x",
            CallKind::Read,
            Some("$ git commit"),
        ),
        ("echo 'x; git commit'", CallKind::Run, None),
    ] {
        assert_eq!(
            classify_bash(command),
            (kind, label.map(str::to_owned)),
            "{command}"
        );
    }
}
