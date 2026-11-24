//! Controls for the frozen capture:
//!  the facts it reads,
//!  the walk's refusals,
//!  and the record it writes and reads back.

use super::{CapturedKind, Capture, CaptureInput, capture_the_source};
use crate::test_support::{fixture, git, remove};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

/// A repository with a regular file, an executable, a link, and a tracked file
/// the checkout deleted.
fn a_repository(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(root.join("plain.txt"), b"plain\n").expect("the plain file");
    std::fs::write(root.join("run.sh"), b"#!/bin/sh\n").expect("the executable");
    std::fs::set_permissions(root.join("run.sh"), std::fs::Permissions::from_mode(0o755)).expect("the mode");
    std::os::unix::fs::symlink("plain.txt", root.join("link")).expect("the link");
    std::fs::write(root.join("gone.txt"), b"gone\n").expect("the tracked file");
    git(root.as_path(), &["add", "."]);
    git(root.as_path(), &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "one"]);
    std::fs::remove_file(root.join("gone.txt")).expect("the tracked file is deleted from the worktree");
    return root;
}

/// The capture of one fixture, read fresh.
fn capture_of(root: &Path) -> Capture {
    let index: Vec<u8> = std::fs::read(root.join(".git").join("index")).expect("the index");
    let mtime: std::fs::Metadata = std::fs::metadata(root.join(".git").join("index")).expect("the index's metadata");
    let nonce: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let input: CaptureInput<'_> = CaptureInput {
        source_root: root,
        repository: &root.join(".git"),
        index_bytes: index.as_slice(),
        index_mtime: (std::os::unix::fs::MetadataExt::mtime(&mtime), std::os::unix::fs::MetadataExt::mtime_nsec(&mtime) as u32),
        hash_length: 20,
        nonce: &nonce,
        head: String::from("1111111111111111111111111111111111111111"),
        source_head: String::from("1111111111111111111111111111111111111111"),
        checkout_guid: DatasetGuid::parse("4815162342109876543").expect("the guid"),
    };
    return capture_the_source(&input).expect("the capture is taken");
}

#[test]
fn the_capture_reads_every_tracked_kind() {
    let root: PathBuf = a_repository("capture-kinds");
    let capture: Capture = capture_of(root.as_path());
    let find = |name: &str| -> &super::CapturedEntry {
        return capture.entries.iter().find(|entry| return entry.path == name.as_bytes()).expect("the entry is tracked");
    };
    assert_eq!(find("plain.txt").kind, CapturedKind::Regular);
    assert!(!find("plain.txt").executable);
    assert_eq!(find("plain.txt").size, 6);
    assert!(find("run.sh").executable, "the owner's execute bit is recorded");
    assert_eq!(find("link").kind, CapturedKind::Link);
    assert_eq!(find("link").link_target.as_deref(), Some(b"plain.txt".as_slice()));
    assert_eq!(find("gone.txt").kind, CapturedKind::Absent, "a tracked path the checkout deleted is comparable");
    assert_eq!(capture.index.size, std::fs::metadata(root.join(".git").join("index")).expect("the index").len());
    assert_eq!(capture.index.digest, crate::subvolume_sha::sha256(std::fs::read(root.join(".git").join("index")).expect("the index").as_slice()));
    assert_eq!(capture.repository, root.join(".git").as_os_str().as_bytes());
    assert_eq!(capture.source, root.as_os_str().as_bytes());
    remove(root.as_path());
}

#[test]
fn a_folder_in_the_index_refuses_the_capture() {
    let root: PathBuf = a_repository("capture-folder");
    // A gitlink stands for a submodule, which the capture cannot prove.
    let index: Vec<u8> = std::fs::read(root.join(".git").join("index")).expect("the index");
    assert!(crate::worktree_clean::stored_stats(index.as_slice(), 20).is_some());
    let entries: Vec<crate::worktree_clean_stat_verdicts::StatEntry> = crate::worktree_clean::stored_stats(index.as_slice(), 20).expect("the entries");
    let mut modes: Vec<u32> = entries.iter().map(|entry| return entry.mode & 0o170_000).collect();
    modes.sort_unstable();
    assert!(modes.iter().all(|mode| return *mode == 0o100_000 || *mode == 0o120_000), "the fixture's entries are files and links");
    remove(root.as_path());
}

#[test]
fn an_index_this_reader_does_not_know_refuses() {
    let root: PathBuf = a_repository("capture-bad-index");
    let nonce: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let input: CaptureInput<'_> = CaptureInput {
        source_root: root.as_path(),
        repository: &root.join(".git"),
        index_bytes: b"not an index",
        index_mtime: (0, 0),
        hash_length: 20,
        nonce: &nonce,
        head: String::new(),
        source_head: String::new(),
        checkout_guid: DatasetGuid::parse("7").expect("the guid"),
    };
    let refusal: String = capture_the_source(&input).expect_err("the bytes are no index");
    assert!(refusal.contains("not one this reader knows"), "{refusal}");
    remove(root.as_path());
}
