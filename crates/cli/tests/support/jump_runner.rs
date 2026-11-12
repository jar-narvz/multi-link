// Included as binary unit tests so the executable path and deadline can be injected.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{Duration, Instant},
};

pub(crate) fn fake_program(body: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("osascript");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    (dir, path)
}

#[test]
fn hung_child_is_killed_and_reaped_at_injected_deadline() {
    let (_dir, program) = fake_program("printf '%s' \"$$\" > \"$0.pid\"\nexec /bin/sleep 60");
    assert_eq!(super::JUMP_DEADLINE, Duration::from_secs(5));
    let started = Instant::now();
    let result = super::run_jump(
        multi_link_core::open::TerminalApp::Terminal,
        "/dev/ttys004",
        program.as_os_str(),
        Duration::from_millis(500),
    );
    assert_eq!(
        result.unwrap_err(),
        "cannot jump: osascript timed out after 5 s"
    );
    assert!(started.elapsed() >= Duration::from_millis(500));
    assert!(started.elapsed() < Duration::from_secs(2));
    let pid = fs::read_to_string(program.with_extension("pid")).unwrap();
    // waitpid returns ECHILD only after our runner has reaped this child.
    // Unlike ps, this checks our own child table and works inside the sandbox.
    unsafe extern "C" {
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }
    let mut status = 0;
    let result = unsafe { waitpid(pid.parse().unwrap(), &mut status, 1) };
    assert_eq!(result, -1);
    assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(10));
}
