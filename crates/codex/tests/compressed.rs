use std::{fs, path::PathBuf};

use multi_link_codex::{discover, parse_file, read_details};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase9")
}

fn compressed() -> PathBuf {
    fs::read_dir(fixtures().join(".codex/sessions/2026/11/09"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            let name = path.file_name().unwrap().to_str().unwrap();
            name.contains("91c00001") && name.ends_with(".jsonl.zst")
        })
        .unwrap()
}

#[test]
fn compressed_discovery_prefers_plain_and_counts_corruption() {
    let actual = discover(&fixtures()).unwrap();
    let rollouts: Vec<_> = actual
        .rollouts
        .iter()
        .filter(|r| r.id.starts_with("91"))
        .collect();
    assert_eq!(rollouts.len(), 2);
    assert!(rollouts[0].id.starts_with("91c00001"));
    assert!(rollouts[0].transcript.ends_with(".jsonl.zst"));
    assert!(rollouts[1].id.starts_with("91c00002"));
    assert!(rollouts[1].transcript.ends_with(".jsonl"));
    assert_eq!(actual.unreadable.len(), 1);
    assert!(actual.unreadable[0].file.contains("91c00003"));
}

#[test]
fn compressed_parser_reads_calls_and_turns() {
    let path = compressed();
    let parsed = parse_file(&fixtures(), &path, &mut Vec::new())
        .unwrap()
        .unwrap();
    assert!(parsed.id.starts_with("91c00001"));
    assert_eq!(parsed.turns.len(), 1);
    assert_eq!(parsed.last_turn_complete(), Some(true));
    let (calls, turns) = read_details(&path).unwrap();
    assert_eq!(calls.calls.len(), 3);
    assert_eq!(
        calls
            .calls
            .iter()
            .filter(|c| c.label.as_deref() == Some("$ cargo test") && c.ok == Some(true))
            .count(),
        1
    );
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].last_agent_message.as_deref(), Some("<text>"));
}

// Minimal standard zstd frames use RLE blocks, avoiding an encoder or
// external program in tests. The streaming header omits the content size.
fn rle_frame(size: usize) -> Vec<u8> {
    let mut frame = vec![0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x50];
    let mut remaining = size;
    while remaining > 0 {
        let block = remaining.min(128 * 1024);
        remaining -= block;
        let header = ((block as u32) << 3) | 2 | u32::from(remaining == 0);
        frame.extend_from_slice(&header.to_le_bytes()[..3]);
        frame.push(b'x');
    }
    frame
}

#[test]
fn compressed_limit_is_inclusive_and_plain_reads_are_uncapped() {
    use std::io::{self, Read};
    const LIMIT: usize = 64 * 1024 * 1024;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("rollout-limit.jsonl.zst");
    fs::write(&path, rle_frame(LIMIT)).unwrap();
    let mut reader = multi_link_codex::open_rollout(&path).unwrap();
    assert_eq!(
        io::copy(&mut reader, &mut io::sink()).unwrap(),
        LIMIT as u64
    );
    drop(reader);
    fs::write(&path, rle_frame(LIMIT + 1)).unwrap();
    assert!(multi_link_codex::open_rollout(&path).is_err());
    assert!(parse_file(root.path(), &path, &mut Vec::new()).is_err());
    assert!(read_details(&path).is_err());
    let plain = path.with_extension("");
    io::copy(
        &mut io::repeat(b'x').take((LIMIT + 1) as u64),
        &mut fs::File::create(&plain).unwrap(),
    )
    .unwrap();
    let mut reader = multi_link_codex::open_rollout(&plain).unwrap();
    assert_eq!(
        io::copy(&mut reader, &mut io::sink()).unwrap(),
        (LIMIT + 1) as u64
    );
}

#[test]
fn empty_truncated_checksum_and_trailing_corruption_are_read_failures() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("rollout-bad.jsonl.zst");
    let mut truncated = rle_frame(5);
    truncated.pop();
    let mut checksum = vec![0x28, 0xb5, 0x2f, 0xfd, 0x24, 1, 9, 0, 0, b'x'];
    checksum.extend_from_slice(&[0; 4]);
    let mut trailing = rle_frame(1);
    trailing.extend_from_slice(b"bad");
    for data in [Vec::new(), truncated, checksum, trailing] {
        fs::write(&path, data).unwrap();
        assert!(parse_file(root.path(), &path, &mut Vec::new()).is_err());
        assert!(read_details(&path).is_err());
    }
}

#[test]
fn concatenated_frames_share_the_decompressed_limit() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("rollout-joined.jsonl.zst");
    let mut frames = rle_frame(32 * 1024 * 1024);
    frames.extend(rle_frame(32 * 1024 * 1024 + 1));
    fs::write(&path, frames).unwrap();
    assert!(multi_link_codex::open_rollout(&path).is_err());
}

#[test]
fn metadata_id_prefers_plain_even_with_different_filenames() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".codex/sessions");
    fs::create_dir_all(&dir).unwrap();
    fs::copy(compressed(), dir.join("rollout-archived.jsonl.zst")).unwrap();
    let plain = dir.join("rollout-live.jsonl");
    fs::write(&plain, "{\"type\":\"session_meta\",\"timestamp\":\"2026-11-09T09:00:45Z\",\"payload\":{\"id\":\"91c00001-0000-4000-8000-000091c00001\",\"cwd\":\"/plain\"}}\n").unwrap();
    let actual = discover(root.path()).unwrap();
    assert!(actual.unreadable.is_empty());
    assert_eq!(actual.rollouts.len(), 1);
    assert_eq!(actual.rollouts[0].cwd, "/plain");
}
