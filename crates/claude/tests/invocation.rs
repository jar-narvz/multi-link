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
