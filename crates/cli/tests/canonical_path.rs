use serde_json::Value;
use std::{
    fs,
    os::unix::fs::symlink,
    path::Path,
    process::{Command, Output},
};

const BIN: &str = env!("CARGO_BIN_EXE_multi-link");

fn run(binary: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("CLAUDE_CONFIG_DIR", home.join("config"))
        .env("MULTI_LINK_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn assert_commands(value: &Value, target: &Path) {
    let expected = format!("'{}' hook record", target.display());
    let hooks = value["hooks"].as_object().unwrap();
    assert_eq!(hooks.len(), 5);
    for entries in hooks.values() {
        let entries = entries.as_array().unwrap();
        assert_eq!(entries.len(), 1);
        let commands = entries[0]["hooks"].as_array().unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0]["command"], expected);
    }
}

#[test]
fn symlink_names_share_canonical_hooks_and_setup_path() {
    let target = fs::canonicalize(BIN).unwrap();
    // Start with each name so an other-named first apply cannot escape recognition.
    for names in [
        ["multi-link", "multi-link-other"],
        ["multi-link-other", "multi-link"],
    ] {
        let home = tempfile::tempdir().unwrap();
        fs::create_dir(home.path().join("config")).unwrap();
        let file = home.path().join("config/settings.json");
        let links = names.map(|name| {
            let link = home.path().join(name);
            symlink(&target, &link).unwrap();
            link
        });
        for link in &links {
            assert!(fs::symlink_metadata(link).unwrap().is_symlink());
        }
        let apply_args = ["hook", "apply", "--settings", file.to_str().unwrap()];
        let first = run(&links[0], home.path(), &apply_args);
        assert!(first.status.success(), "{first:?}");
        let bytes = fs::read(&file).unwrap();
        assert_commands(&serde_json::from_slice(&bytes).unwrap(), &target);
        for link in &links {
            let check = run(
                link,
                home.path(),
                &["hook", "check", "--settings", file.to_str().unwrap()],
            );
            assert!(check.status.success(), "{check:?}");
            assert!(
                String::from_utf8(check.stdout)
                    .unwrap()
                    .contains("are current")
            );
            let setup = run(link, home.path(), &["setup", "--json"]);
            assert!(setup.status.success(), "{setup:?}");
            let report: Value = serde_json::from_slice(&setup.stdout).unwrap();
            assert_eq!(report["binary"], target.to_str().unwrap());
            assert_eq!(report["items"][1]["state"], "ok");
            assert_eq!(
                report["items"][1]["file"],
                fs::canonicalize(&file).unwrap().to_str().unwrap()
            );
            for action in ["install", "uninstall"] {
                let snippet = run(link, home.path(), &["hook", action]);
                assert!(snippet.status.success(), "{snippet:?}");
                assert_commands(&serde_json::from_slice(&snippet.stdout).unwrap(), &target);
            }
        }
        let second = run(&links[1], home.path(), &apply_args);
        assert!(second.status.success(), "{second:?}");
        assert_eq!(
            String::from_utf8(second.stdout).unwrap(),
            format!(
                "no change: {}\n",
                fs::canonicalize(&file).unwrap().display()
            )
        );
        assert_eq!(fs::read(&file).unwrap(), bytes);
        assert_eq!(fs::read_dir(file.parent().unwrap()).unwrap().count(), 1);
    }
}
