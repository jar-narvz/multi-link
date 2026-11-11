use std::{fs, path::Path};

use multi_link_claude::{invocations, recognise, scan};
use multi_link_model::InvocationForm::{Exec, Plugin, Resume};
use serde_json::{Value, json};

const UUID: &str = "01a189e2-d57f-7a91-b74c-ae85abd1a6a1";

#[test]
fn recognition_table() {
    for (command, form, resume_id, moved) in [
        ("codex exec \"x\"", Some(Exec), None, false),
        ("codex exec", Some(Exec), None, false),
        (
            "codex exec resume 01a189e2-d57f-7a91-b74c-ae85abd1a6a1",
            Some(Resume),
            Some(UUID),
            false,
        ),
        (
            "codex resume '01a189e2-d57f-7a91-b74c-ae85abd1a6a1'",
            Some(Resume),
            Some(UUID),
            false,
        ),
        ("codex exec resume non-uuid", Some(Exec), None, false),
        ("codex exec resume", Some(Exec), None, false),
        ("codex resume", None, None, false),
        ("codex resume non-uuid", None, None, false),
        (
            "codex resume 01a189e2-d57f-7a91-b74c-ae85abd1a6a1x",
            None,
            None,
            false,
        ),
        (
            "codex resume 01a189e2_d57f-7a91-b74c-ae85abd1a6a1",
            None,
            None,
            false,
        ),
        (
            "codex resume g1a189e2-d57f-7a91-b74c-ae85abd1a6a1",
            None,
            None,
            false,
        ),
        ("cd d && codex exec \"a; b\"", Some(Exec), None, true),
        ("cd d || codex exec x", Some(Exec), None, true),
        ("echo x; codex exec 'a; b'", Some(Exec), None, false),
        ("echo x | codex exec x", Some(Exec), None, false),
        ("echo x\ncodex exec x", Some(Exec), None, false),
        ("echo 'a; codex exec x'", None, None, false),
        (
            "echo \"a && codex exec x || codex exec y | codex exec z\"",
            None,
            None,
            false,
        ),
        ("echo 'a\ncodex exec x'", None, None, false),
        ("echo a\\;codex exec x", None, None, false),
        ("echo \"a\\\"; codex exec x\"", None, None, false),
        ("codex exec 'unterminated", None, None, false),
        (
            "codex exec x; cd d; codex resume 01a189e2-d57f-7a91-b74c-ae85abd1a6a1",
            Some(Exec),
            None,
            false,
        ),
        ("codex exec -C d x", Some(Exec), None, true),
        ("codex exec -Cd x", Some(Exec), None, true),
        ("codex exec --cd d x", Some(Exec), None, true),
        ("codex exec --cd=d x", Some(Exec), None, true),
        ("codex exec -- '-C'", Some(Exec), None, false),
        ("codex exec 'please cd d'", Some(Exec), None, false),
        (
            "node /p/codex-companion.mjs task x",
            Some(Plugin),
            None,
            false,
        ),
        (
            "node \"/p with spaces/codex-companion.mjs\" task x",
            Some(Plugin),
            None,
            false,
        ),
        (
            "cd d; node /p/codex-companion.mjs task x",
            Some(Plugin),
            None,
            true,
        ),
        (
            "node /p/codex-companion.mjs task-resume-candidate",
            None,
            None,
            false,
        ),
        ("node /p/codex-companion.mjs.bak task x", None, None, false),
        (
            "node --flag /p/codex-companion.mjs task x",
            None,
            None,
            false,
        ),
        ("codex --version", None, None, false),
        ("ls ~/.codex/sessions", None, None, false),
        ("FOO=1 codex exec", None, None, false),
        ("echo codex exec x", None, None, false),
    ] {
        let actual = recognise(command);
        assert_eq!(actual.as_ref().map(|launch| launch.form), form, "{command}");
        if let Some(actual) = actual {
            assert_eq!(actual.resume_id.as_deref(), resume_id, "{command}");
            assert_eq!(actual.dir_moved, moved, "{command}");
        }
    }
}

#[test]
fn help_options_are_not_launches() {
    for command in [
        "codex exec --help",
        "codex exec -h",
        "codex exec resume --help",
        "codex exec resume -h",
        "codex resume --help",
        "codex exec -C d --help",
        "codex exec -c v -o p -C d -s m --add-dir d --help",
        "env TASK=x timeout 10 codex exec --help",
    ] {
        assert_eq!(recognise(command), None, "{command}");
    }
    for command in [
        r#"codex exec "use --help""#,
        r#"codex exec -o --help.txt "x""#,
        "codex exec -- --help",
        "codex exec prompt --help",
        r#"codex exec "--help""#,
        "codex exec '--help'",
        r"codex exec \--help",
        "codex exec --he'lp'",
        "codex exec -c --help x",
        "codex exec -o --help x",
        "codex exec -C --help x",
        "codex exec -s --help x",
        "codex exec --add-dir --help x",
        "codex exec --config=--help x",
        "codex exec resume -- --help",
    ] {
        assert_eq!(recognise(command).unwrap().form, Exec, "{command}");
    }
    let launch = recognise(r#"codex exec --help; codex exec "Step 1.2 implement""#).unwrap();
    assert_eq!(launch.form, Exec);
    assert_eq!(launch.step.as_deref(), Some("1.2"));
}

#[test]
fn directory_options_distinguish_current_directory_from_moves() {
    for (options, moved) in [
        (r#"-C "$PWD""#, false),
        ("-C $PWD", false),
        ("-C .", false),
        ("-C${PWD}", false),
        ("--cd=$PWD", false),
        ("--cd .", false),
        (r#"-C "${PWD}""#, false),
        ("-C$PWD", false),
        ("-C.", false),
        ("--cd=${PWD}", false),
        (r#"--cd="$PWD""#, false),
        ("--cd $PWD", false),
        ("-C '.'", false),
        ("-C '$PWD'", true),
        (r"-C \$PWD", true),
        (r#"-C "\$PWD""#, true),
        ("-C'${PWD}'", true),
        (r"--cd=\${PWD}", true),
        ("-C /tmp/x", true),
        (r#"-C "$HOME/x""#, true),
        (r#"-C "$PWD/x""#, true),
        ("-C ${PWD}/x", true),
        ("--cd=", true),
        ("-C", true),
        ("--cd", true),
        ("-C . --cd /tmp/x", true),
        ("-C /tmp/x --cd .", true),
        ("-C . --cd=$PWD", false),
        ("-- -C /tmp/x", false),
        ("-- --cd=/tmp/x", false),
        ("-C . -- -C /tmp/x", false),
        // The next word is consumed as the option's value, even if option-shaped.
        ("--cd -C .", true),
    ] {
        let command = format!("codex exec {options}");
        let launch = recognise(&command).unwrap();
        assert_eq!(launch.form, Exec, "{command}");
        assert_eq!(launch.dir_moved, moved, "{command}");
    }
    for command in [
        "cd x && codex exec -C .",
        "cd x; codex exec --cd=$PWD",
        r#"cd x && codex exec -C "$PWD""#,
    ] {
        assert!(recognise(command).unwrap().dir_moved, "{command}");
    }
}

#[test]
fn fixture_invocations_match_all_answer_key_tuples() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let expected: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/links.json")).unwrap()).unwrap();
    let root = fixtures.join("sessions");
    let calls = invocations(&root, &scan(&root).unwrap()).unwrap();
    assert_eq!(calls.len(), 13);
    let actual: Vec<_> = calls
        .iter()
        .map(|call| {
            json!({
                "parent": call.parent, "invocation": call.invocation, "form": call.form,
                "background": call.background, "at": call.at, "checkout": call.checkout,
                "subagent": call.subagent,
            })
        })
        .collect();
    let expected: Vec<_> = expected["links"].as_array().unwrap().iter().map(|link| {
        let call = calls.iter().find(|call| call.invocation == link["invocation"].as_str().unwrap()).unwrap();
        if call.form == Resume {
            assert_eq!(call.resume_id.as_deref(), link["child"].as_str());
        }
        if call.form == Plugin {
            assert!(call.result_text.as_ref().unwrap().contains("Thread ready"));
        }
        json!({"parent": link["parent"], "invocation": link["invocation"], "form": link["form"],
            "background": link["background"], "at": link["at"], "checkout": link["checkout"], "subagent": link["subagent"]})
    }).collect();
    assert_eq!(actual, expected);
    assert!(
        !calls
            .iter()
            .any(|call| call.parent == "df403bfa-454f-4993-b67c-e44c6e7a9f7e")
    );
}

fn write_records(root: &Path, file: &str, records: &[Value]) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        records
            .iter()
            .map(|value| format!("{value}\n"))
            .collect::<String>(),
    )
    .unwrap();
}

fn call(id: &str, command: &str) -> Value {
    json!({"type":"assistant", "sessionId":"record-parent", "agentId":"record-agent",
        "timestamp":"2026-10-29T00:00:00Z", "cwd":"/checkout",
        "message":{"content":[{"type":"tool_use", "name":"Bash", "id":id,
            "input":{"command":command, "run_in_background":true}}]}})
}

#[test]
fn extracts_only_assistant_bash_and_pairs_results_by_id_within_each_file() {
    let root = tempfile::tempdir().unwrap();
    let launch = call("launch", "cd x && codex exec 'a; b'; codex exec second");
    let mut blob = launch.clone();
    blob["type"] = json!("api-request-blob");
    let mut user = launch.clone();
    user["type"] = json!("user");
    let mut other_tool = launch.clone();
    other_tool["message"]["content"][0]["id"] = json!("other-tool");
    other_tool["message"]["content"][0]["name"] = json!("Other");
    let mut missing_time = call("missing-time", "codex exec x");
    missing_time["timestamp"] = json!("invalid");
    write_records(
        root.path(),
        ".claude/projects/p/main.jsonl",
        &[
            blob,
            user,
            other_tool,
            missing_time,
            launch,
            call("no-result", "codex exec x"),
            call("unrelated", "codex exec x"),
            call("not-launch", "echo codex exec x"),
            json!({"type":"user", "message":{"content":[
            {"type":"tool_result", "tool_use_id":"unrelated", "content":"wrong"},
            {"type":"tool_result", "tool_use_id":"launch", "content":[
                {"type":"text", "text":"first"}, {"type":"image", "text":"ignored"},
                {"type":"text", "text":"second"}]}]}, "toolUseResult":{"stdout":"stdout", "stderr":"ignored"}}),
            json!({"type":"user", "message":{"content":[
            {"type":"tool_result", "tool_use_id":"launch", "content":"later"}]},
                "toolUseResult":{"stdout":"single stdout"}}),
        ],
    );
    write_records(
        root.path(),
        ".claude/projects/p/main/subagents/agent-file-agent.jsonl",
        &[
            call("launch", "node /p/codex-companion.mjs task x"),
            json!({"type":"user", "message":{"content":[
            {"type":"tool_result", "tool_use_id":"launch", "content":"subagent output"}]}}),
        ],
    );
    let calls = invocations(root.path(), &scan(root.path()).unwrap()).unwrap();
    assert_eq!(calls.len(), 4);
    let main = calls
        .iter()
        .find(|call| call.parent == "main" && call.invocation == "launch")
        .unwrap();
    assert_eq!(main.subagent, None);
    assert_eq!(main.form, Exec);
    assert!(main.background);
    assert!(main.dir_moved);
    assert_eq!(main.checkout, "/checkout");
    assert_eq!(main.command, "cd x && codex exec 'a; b'; codex exec second");
    assert_eq!(
        main.result_text.as_deref(),
        Some("first\nsecond\nlater\nsingle stdout")
    );
    assert_eq!(
        calls
            .iter()
            .find(|call| call.invocation == "unrelated")
            .unwrap()
            .result_text
            .as_deref(),
        Some("wrong")
    );
    assert_eq!(
        calls
            .iter()
            .find(|call| call.invocation == "no-result")
            .unwrap()
            .result_text,
        None
    );
    let sub = calls
        .iter()
        .find(|call| call.parent == "record-parent")
        .unwrap();
    assert_eq!(sub.subagent.as_deref(), Some("record-agent"));
    assert_eq!(sub.form, Plugin);
    assert_eq!(sub.result_text.as_deref(), Some("subagent output"));
}

#[test]
fn repeated_tool_use_ids_keep_the_first_occurrence_in_each_file() {
    let root = tempfile::tempdir().unwrap();
    let first = call("repeated", "codex exec first");
    let mut rewritten = call("repeated", "codex exec second");
    rewritten["timestamp"] = json!("2026-10-29T00:00:01Z");
    rewritten["cwd"] = json!("/rewritten");
    for file in [
        ".claude/projects/p/first.jsonl",
        ".claude/projects/p/second.jsonl",
    ] {
        write_records(root.path(), file, &[first.clone(), rewritten.clone()]);
    }
    let calls = invocations(root.path(), &scan(root.path()).unwrap()).unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls
            .iter()
            .map(|call| call.parent.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    for call in calls {
        assert_eq!(call.invocation, "repeated");
        assert_eq!(call.command, "codex exec first");
        assert_eq!(call.checkout, "/checkout");
        assert_eq!(call.at.to_string(), "2026-10-29T00:00:00.000Z");
    }
}

#[test]
fn step_comes_from_raw_text_of_the_first_launch_segment_only() {
    for (command, step) in [
        ("echo 'Step 9'; codex exec x | echo 'Step 8'", None),
        ("codex exec x; codex exec 'Step 2'", None),
        ("codex exec 'Step 1; Step 2'", Some("1")),
        (
            "echo 'Step 9' && codex exec 'Step 0.2a' || echo 'Step 8'",
            Some("0.2a"),
        ),
        ("echo x\ncodex exec 'Step 4'\necho 'Step 9'", Some("4")),
        ("codex exec 'Step '1.2", None),
        ("codex exec 'Step 1'.2", Some("1")),
        ("codex exec 'St'ep' 1.2'", None),
        (r"codex exec Step\ 1.2", None),
        (r"codex exec St\ep 1.2", None),
        (r"codex exec 'Step 1.2ab; Step 3'", Some("3")),
        ("node /p/codex-companion.mjs task 'Step 5'", Some("5")),
        (
            "codex resume 01a189e2-d57f-7a91-b74c-ae85abd1a6a1 'Step 6'",
            Some("6"),
        ),
        (
            "echo é || codex exec \"Step 7 | quoted\" | tail -1",
            Some("7"),
        ),
        ("codex exec \"$(cat prompt.txt)\"", None),
    ] {
        let root = tempfile::tempdir().unwrap();
        write_records(
            root.path(),
            ".claude/projects/p/main.jsonl",
            &[call("launch", command)],
        );
        let calls = invocations(root.path(), &scan(root.path()).unwrap()).unwrap();
        assert_eq!(calls.len(), 1, "{command}");
        assert_eq!(calls[0].step.as_deref(), step, "{command}");
        assert_eq!(
            recognise(command).unwrap().step.as_deref(),
            step,
            "{command}"
        );
    }
}

#[test]
fn shell_groups_heredocs_and_wrappers() {
    for (command, form, moved) in [
        ("(codex exec 'Synthetic task' &)", Some(Exec), false),
        ("( cd synthetic && codex exec x )", Some(Exec), true),
        ("{ codex exec x; }", Some(Exec), false),
        ("nohup codex exec x", Some(Exec), false),
        (
            "env TASK=synthetic OTHER='two words' codex exec x",
            Some(Exec),
            false,
        ),
        ("timeout 30s codex exec x", Some(Exec), false),
        (
            "gtimeout 1.5m node /p/codex-companion.mjs task x",
            Some(Plugin),
            false,
        ),
        (
            "nohup env TASK=x timeout 1h codex exec -C other x",
            Some(Exec),
            true,
        ),
        (
            "env TASK=x codex resume 01a189e2-d57f-7a91-b74c-ae85abd1a6a1",
            Some(Resume),
            false,
        ),
        ("env -i codex exec x", None, false),
        ("env 1INVALID=x codex exec x", None, false),
        ("timeout --signal=KILL 2s codex exec x", None, false),
        ("timeout nonsense codex exec x", None, false),
        ("sudo codex exec x", None, false),
        ("'{' codex exec x", None, false),
        (r"\{ codex exec x", None, false),
        ("echo '(codex exec fake)'", None, false),
        ("echo $(codex exec x)", None, false),
        ("echo \"$(echo \"x\"; codex exec x)\"", None, false),
        ("echo $(echo $(codex exec x))", None, false),
        ("codex exec $(cat synthetic.txt)", Some(Exec), false),
        (
            "cat <<END\nDon't parse \"this\n(codex exec fake)\nEND\ncodex exec x",
            Some(Exec),
            false,
        ),
        (
            "cat <<'END'\nSynthetic ' and \" quotes\nEND\n(codex exec x)",
            Some(Exec),
            false,
        ),
        (
            "cat <<\"END\"\nSynthetic ' and \" quotes\nEND\n{ codex exec x; }",
            Some(Exec),
            false,
        ),
        (
            "cat <<-END\n\tSynthetic ' and \"\n\tEND\ncodex exec x",
            Some(Exec),
            false,
        ),
        ("cat <<END\ncodex exec fake", None, false),
        ("cat <<END\n END\ncodex exec fake", None, false),
        (
            "cat <<A <<'B'\n'\nA\n\"\nB\ncodex exec x",
            Some(Exec),
            false,
        ),
        ("echo '<<END'; codex exec x", Some(Exec), false),
        (
            "cat <<'END' > brief.txt\nSynthetic brief: don't forget the \"sample\".\nEND\n(codex exec --sandbox workspace-write \"$(cat brief.txt)\" < /dev/null > /dev/null 2>&1 &)",
            Some(Exec),
            false,
        ),
    ] {
        let launch = recognise(command);
        assert_eq!(launch.as_ref().map(|launch| launch.form), form, "{command}");
        if let Some(launch) = launch {
            assert_eq!(launch.dir_moved, moved, "{command}");
            assert_eq!(launch.step, None, "{command}");
        }
    }
    assert_eq!(
        recognise("cat <<END\nStep 9\nEND\n(codex exec 'Step 6.6')")
            .unwrap()
            .step
            .as_deref(),
        Some("6.6")
    );
}

#[test]
fn redirect_targets_are_literal_and_on_the_launch_segment() {
    for (command, expected) in [
        (
            "echo hi > /tmp/other; codex exec x > logs/run",
            Some("logs/run"),
        ),
        ("codex exec x > 'logs/a b' 2>&1", Some("logs/a b")),
        ("cd sub && codex exec x > logs/run", None),
        ("pushd sub; codex exec x > logs/run", None),
        ("(codex exec x > logs/run)", None),
        ("cd sub; codex exec x > /tmp/run", Some("/tmp/run")),
        ("codex exec x > /tmp/a > /tmp/b", None),
        ("codex exec x > /tmp/a 1>&2", None),
        ("codex exec x 2> /tmp/error", None),
        ("codex exec x > \"$S/run\"", None),
        ("codex exec x > '$S/run'", None),
        ("codex exec x > `pwd`/run", None),
        ("codex exec x > /tmp/*.log", None),
        ("codex exec x > \"$(pwd)/run\"", None),
        ("codex exec x 'prompt > /tmp/fake'", None),
        ("codex exec x; echo hi > /tmp/other", None),
    ] {
        assert_eq!(
            multi_link_claude::recognise(command)
                .unwrap()
                .stdout_redirect
                .as_deref(),
            expected,
            "{command}"
        );
    }
}

#[test]
fn earlier_unquoted_directory_words_refuse_relative_redirects() {
    for (prefix, suffix) in [
        ("{ cd sub &&", "; }"),
        ("if cd sub; then", "; fi"),
        ("! cd sub;", ""),
        ("time cd sub;", ""),
        ("'cd' sub;", ""),
        ("echo cd;", ""),
        ("echo pushd;", ""),
        ("codex exec --help cd;", ""),
    ] {
        let relative = format!("{prefix} codex exec x > out.log{suffix}");
        assert_eq!(
            recognise(&relative).unwrap().stdout_redirect,
            None,
            "{relative}"
        );
        let absolute = format!("{prefix} codex exec x > /tmp/out.log{suffix}");
        assert_eq!(
            recognise(&absolute).unwrap().stdout_redirect.as_deref(),
            Some("/tmp/out.log"),
            "{absolute}"
        );
    }
    for prefix in ["echo 'cd';", "echo \"pushd\";", r"echo c\d;"] {
        let command = format!("{prefix} codex exec x > out.log");
        assert_eq!(
            recognise(&command).unwrap().stdout_redirect.as_deref(),
            Some("out.log"),
            "{command}"
        );
    }
}

#[test]
fn redirect_outside_roots_is_refused_before_open_and_missing_is_retriable() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("home/repo")).unwrap();
    std::fs::create_dir_all(root.path().join("tmp")).unwrap();
    let fifo = root.path().join("home/private.fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let launch = recognise("codex exec x > /home/private.fifo").unwrap();
    // This specific containment error must precede open/fstat's non-regular error.
    assert_eq!(
        launch
            .redirect_child(std::path::Path::new("/home/repo"), Some(root.path()))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    std::os::unix::fs::symlink("/dev/null", root.path().join("tmp/device")).unwrap();
    let launch = recognise("codex exec x > /tmp/device").unwrap();
    assert_eq!(
        launch
            .redirect_child(std::path::Path::new("/home/repo"), Some(root.path()))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let launch = recognise("codex exec x > /tmp/missing").unwrap();
    assert_eq!(
        launch
            .redirect_child(std::path::Path::new("/home/repo"), Some(root.path()))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound
    );
}

#[test]
fn redirect_hard_links_are_refused_until_only_one_link_remains() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("tmp")).unwrap();
    std::fs::create_dir_all(root.path().join("home/repo")).unwrap();
    let private = root.path().join("home/private.log");
    let id = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    std::fs::write(&private, format!("session id: {id}\n")).unwrap();
    std::fs::hard_link(&private, root.path().join("tmp/run.log")).unwrap();
    let launch = recognise("codex exec x > /tmp/run.log").unwrap();
    let read = || launch.redirect_child(std::path::Path::new("/home/repo"), Some(root.path()));
    assert!(read().is_err(), "multiple links must never supply a child");
    std::fs::remove_file(private).unwrap();
    assert_eq!(read().unwrap().as_deref(), Some(id));
}

#[test]
fn resume_last_recognition_obeys_option_and_segment_boundaries() {
    for command in [
        "codex exec resume --last 'Step 2.1'",
        "nohup env X=y timeout 30 codex exec resume --last --json 'Step 2.1'",
        "codex exec resume -c model=x --last 'Step 2.1'",
        "codex exec resume --last -C . 'Step 2.1'",
    ] {
        let launch = recognise(command).unwrap();
        assert_eq!(launch.form, Resume, "{command}");
        assert_eq!(launch.resume_id, None, "{command}");
        assert!(!launch.dir_moved, "{command}");
    }
    for command in [
        "codex exec resume 'prompt containing --last'",
        "codex exec resume prompt --last",
        "codex exec resume -- --last",
        "codex exec resume -c --last prompt",
        "codex exec '--last'",
    ] {
        assert_eq!(recognise(command).unwrap().form, Exec, "{command}");
    }
    assert!(recognise("codex exec resume --last --help").is_none());
    assert!(recognise("codex exec resume --last -h").is_none());
    let launch = recognise("cd /elsewhere; codex exec resume --last x").unwrap();
    assert_eq!(launch.form, Resume);
    assert!(!launch.dir_moved);
    assert_eq!(launch.launch_dir.as_deref(), Some("/elsewhere"));
}

#[test]
fn resume_last_after_exec_options_preserves_directory_and_prompt_boundaries() {
    for (command, moved) in [
        (
            r#"codex exec -s workspace-write -c k=v --add-dir D -C "$PWD" -o F resume --last "prompt""#,
            false,
        ),
        ("codex exec --json -C /elsewhere resume --last prompt", true),
        (
            "nohup env X=y timeout 30 codex exec --json -c a=b resume --last prompt",
            false,
        ),
    ] {
        let launch = recognise(command).unwrap();
        assert_eq!(launch.form, Resume, "{command}");
        assert_eq!(launch.resume_id, None, "{command}");
        assert_eq!(launch.dir_moved, moved, "{command}");
    }
    for command in [
        "codex exec --json 'resume' --last",
        "codex exec --json prompt resume --last",
        "codex exec --json -- resume --last",
        "codex exec --json > resume --last",
        "codex exec -c resume --last prompt",
    ] {
        let launch = recognise(command).unwrap();
        assert_eq!(launch.form, Exec, "{command}");
        assert_eq!(launch.resume_id, None, "{command}");
    }
    for command in [
        "codex exec -s read-only resume --help",
        "codex exec --json -c a=b resume --last --help",
        "codex exec --help resume --last prompt",
    ] {
        assert!(recognise(command).is_none(), "{command}");
    }
    let id = "01a189e2-d57f-7a91-b74c-ae85abd1a6a1";
    for command in [
        format!("codex exec -s read-only resume {id}"),
        format!("codex exec resume {id}"),
        format!("codex resume {id}"),
    ] {
        let launch = recognise(&command).unwrap();
        assert_eq!(launch.form, Resume);
        assert_eq!(launch.resume_id.as_deref(), Some(id));
    }
}

#[test]
fn resume_last_skips_all_supported_option_values_without_consuming_equals_forms() {
    for option in [
        "-m",
        "--model",
        "-i",
        "--image",
        "-p",
        "--profile",
        "--enable",
        "--disable",
        "--output-schema",
        "--thread-source",
        "--cyber-access-program",
        "--local-provider",
        "--color",
    ] {
        for command in [
            format!("codex exec resume {option} gpt-x --last prompt"),
            format!("codex exec {option} gpt-x resume --last prompt"),
            format!("codex exec {option}=gpt-x resume --last prompt"),
            format!("codex exec resume {option}=gpt-x --last prompt"),
        ] {
            let launch = recognise(&command).unwrap();
            assert_eq!(launch.form, Resume, "{command}");
            assert_eq!(launch.resume_id, None, "{command}");
        }
        for command in [
            format!("codex exec resume {option} --last prompt"),
            format!("codex exec {option} resume --last prompt"),
        ] {
            assert_eq!(recognise(&command).unwrap().form, Exec, "{command}");
        }
        for command in [
            format!("codex exec resume {option} --help --last prompt"),
            format!("codex exec {option} --help resume --last prompt"),
        ] {
            assert_eq!(recognise(&command).unwrap().form, Resume, "{command}");
        }
    }
}

#[test]
fn literal_cd_tracks_only_groups_open_at_the_move() {
    for command in [
        "cd /target && codex exec x",
        "(cd /target && codex exec x)",
        "cd /target && (true) && codex exec x",
        "cd /target && S=$(date +%s) && codex exec x",
        "(cd /target && (true) && codex exec x)",
    ] {
        assert!(!recognise(command).unwrap().dir_moved, "{command}");
    }
    for command in [
        "(cd /target); codex exec x",
        "((cd /target) && codex exec x)",
        "(cd /target); (codex exec x)",
    ] {
        assert!(recognise(command).unwrap().dir_moved, "{command}");
    }
}

#[test]
fn option_first_resume_id_obeys_known_option_boundaries() {
    for prefix in [
        "codex exec resume -c model=x",
        "codex exec -s read-only resume --json",
        "codex resume --oss",
        "codex exec resume --config=k=v --ephemeral --all",
    ] {
        let command = format!("{prefix} {UUID} prompt");
        let launch = recognise(&command).unwrap();
        assert_eq!(launch.form, Resume, "{command}");
        assert_eq!(launch.resume_id.as_deref(), Some(UUID), "{command}");
    }
    for options in ["--unknown", "--", "-", "> out", "-c", "--json non-uuid"] {
        let command = format!("codex exec resume {options} {UUID}");
        let launch = recognise(&command).unwrap();
        assert_eq!(launch.form, Exec, "{command}");
        assert_eq!(launch.resume_id, None, "{command}");
    }
    let launch = recognise(&format!("codex exec resume -c k=v --last {UUID}")).unwrap();
    assert_eq!(launch.form, Resume);
    assert_eq!(launch.resume_id, None);
}

#[test]
fn literal_cd_resolves_against_record_cwd_without_filesystem_access() {
    for (prefix, expected) in [
        ("cd /target &&", "/target"),
        ("cd ./sub &&", "/checkout/sub"),
        ("cd ../x/./y/ &&", "/x/y"),
        ("cd /target && cd ./inner &&", "/target/inner"),
        ("cd /target; export A=1;", "/target"),
        ("cd /target\n", "/target"),
        (
            "cd '/target with spaces/+@%:,=-_0' &&",
            "/target with spaces/+@%:,=-_0",
        ),
        ("cd /../../target//./ &&", "/target"),
        ("cd .. &&", "/"),
        ("cd /target && (true) &&", "/target"),
        ("cd /target && S=$(date +%s) &&", "/target"),
    ] {
        let root = tempfile::tempdir().unwrap();
        write_records(
            root.path(),
            ".claude/projects/p/main.jsonl",
            &[call(
                "launch",
                &format!("{prefix} codex exec -C \"$PWD\" x"),
            )],
        );
        let calls = invocations(root.path(), &scan(root.path()).unwrap()).unwrap();
        assert_eq!(calls.len(), 1, "{prefix}");
        assert_eq!(calls[0].checkout, "/checkout");
        assert_eq!(calls[0].launch_dir.as_deref(), Some(expected), "{prefix}");
        assert!(!calls[0].dir_moved, "{prefix}");
    }
    for prefix in ["cd . &&", "cd /checkout/sub/.. &&"] {
        let root = tempfile::tempdir().unwrap();
        write_records(
            root.path(),
            ".claude/projects/p/main.jsonl",
            &[call("launch", &format!("{prefix} codex exec x"))],
        );
        let calls = invocations(root.path(), &scan(root.path()).unwrap()).unwrap();
        assert_eq!(calls[0].launch_dir, None, "{prefix}");
        assert!(!calls[0].dir_moved, "{prefix}");
    }
}

#[test]
fn unknown_cd_forms_and_folder_builtins_disable_heuristics() {
    for prefix in [
        "cd $HOME/x &&",
        "cd ~/x &&",
        "cd - &&",
        "cd sub &&",
        "cd &&",
        "cd /x /y &&",
        "cd /x/{a,b} &&",
        "cd /x/* &&",
        "cd /x? &&",
        "cd /x ||",
        "cd /x |",
        "false || cd /x &&",
        "false | cd /x &&",
        "env cd /x &&",
        "nohup cd /x &&",
        "timeout 3 cd /x &&",
        "pushd /x &&",
        "popd;",
        "cd /x && eval true &&",
        "cd /x && source f;",
        "cd /x && . f;",
        "cd /x && exec true;",
        "cd /x && builtin true;",
        "cd /x && command true;",
        "cd /x && cd sub &&",
        "(cd /x);",
    ] {
        let command = format!("{prefix} codex exec x");
        let launch = recognise(&command).unwrap();
        assert!(launch.dir_moved, "{command}");
        assert_eq!(launch.launch_dir, None, "{command}");
    }
    assert!(recognise("cd /x & codex exec x").is_none());
}

#[test]
fn single_ampersand_is_not_a_literal_cd_boundary() {
    for command in [
        "true & cd /target && codex exec x",
        "cd /target & (true); codex exec x",
        "cd /target &\ncodex exec x",
    ] {
        let launch = recognise(command).unwrap();
        assert!(launch.dir_moved, "{command}");
        assert_eq!(launch.launch_dir, None, "{command}");
    }
    assert!(recognise("cd /target & codex exec x").is_none());
}

#[test]
fn folder_builtin_deny_list_is_scoped_to_a_prior_literal_cd() {
    for builtin in [
        "command -v codex",
        "source env.sh",
        ". env.sh",
        "eval true",
        "exec true",
        "builtin true",
    ] {
        let command = format!("{builtin} && codex exec x");
        let launch = recognise(&command).unwrap();
        assert!(!launch.dir_moved, "{command}");
        assert_eq!(launch.launch_dir, None, "{command}");

        let command = format!("cd /target && {builtin} && codex exec x");
        let launch = recognise(&command).unwrap();
        assert!(launch.dir_moved, "{command}");
        assert_eq!(launch.launch_dir, None, "{command}");
    }
    for builtin in ["cd sub", "env cd sub", "pushd /target", "popd"] {
        let command = format!("{builtin} && codex exec x");
        let launch = recognise(&command).unwrap();
        assert!(launch.dir_moved, "{command}");
        assert_eq!(launch.launch_dir, None, "{command}");
    }
}

#[test]
fn interactive_resume_requires_an_explicit_uuid_without_last() {
    for command in [
        "codex resume --last".to_owned(),
        "codex resume --last prompt".to_owned(),
        format!("codex resume --last {UUID}"),
        format!("codex resume -c model=x --last {UUID}"),
    ] {
        assert!(recognise(&command).is_none(), "{command}");
    }
    let launch = recognise(&format!("codex resume --oss {UUID}")).unwrap();
    assert_eq!(launch.form, Resume);
    assert_eq!(launch.resume_id.as_deref(), Some(UUID));
    let launch = recognise("codex exec resume --last").unwrap();
    assert_eq!(launch.form, Resume);
    assert_eq!(launch.resume_id, None);
}
