//! Controls for the frozen proof:
//!  the unchanged tree passes,
//!  a tracked write,
//!  an index write,
//!  and a path that came back
//!  are each refused with the path and the field named.

use super::prove_the_clone;
use crate::test_support::{fixture, git, remove};
use crate::worktree_clone_add_route::add_capture::{Capture, CaptureInput, capture_the_source};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::PathBuf;

/// A committed repository with one plain file and one tracked-then-deleted file.
fn a_repository(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(root.join("plain.txt"), b"plain\n").expect("the file");
    std::fs::write(root.join("gone.txt"), b"gone\n").expect("the tracked file");
    git(root.as_path(), &["add", "."]);
    git(root.as_path(), &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "one"]);
    std::fs::remove_file(root.join("gone.txt")).expect("the tracked file is deleted");
    return root;
}

/// The capture of one fixture.
fn capture_of(root: &std::path::Path) -> Capture {
    let index: Vec<u8> = std::fs::read(root.join(".git").join("index")).expect("the index");
    let metadata: std::fs::Metadata = std::fs::metadata(root.join(".git").join("index")).expect("the metadata");
    let nonce: RequestNonce = RequestNonce::fresh().expect("a nonce");
    let input: CaptureInput<'_> = CaptureInput {
        source_root: root,
        repository: &root.join(".git"),
        index_bytes: index.as_slice(),
        index_mtime: (std::os::unix::fs::MetadataExt::mtime(&metadata), std::os::unix::fs::MetadataExt::mtime_nsec(&metadata) as u32),
        hash_length: 20,
        nonce: &nonce,
        head: String::from("1111111111111111111111111111111111111111"),
        source_head: String::from("1111111111111111111111111111111111111111"),
        checkout_guid: DatasetGuid::parse("7").expect("the guid"),
    };
    return capture_the_source(&input).expect("the capture is taken");
}

#[test]
fn the_unchanged_tree_passes() {
    let root: PathBuf = a_repository("proof-unchanged");
    let capture: Capture = capture_of(root.as_path());
    prove_the_clone(&capture, root.as_path()).expect("the fixture is the stand-in the capture read");
    remove(root.as_path());
}

#[test]
fn a_tracked_write_between_the_capture_and_the_clone_is_refused() {
    let root: PathBuf = a_repository("proof-written");
    let capture: Capture = capture_of(root.as_path());
    std::fs::write(root.join("plain.txt"), b"written later\n").expect("the control's write");
    let refusal: String = prove_the_clone(&capture, root.as_path()).expect_err("a write is refused");
    assert!(refusal.contains("plain.txt differs in its size"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
    remove(root.as_path());
}

#[test]
fn an_index_written_between_the_capture_and_the_clone_is_refused() {
    let root: PathBuf = a_repository("proof-indexed");
    let capture: Capture = capture_of(root.as_path());
    git(root.as_path(), &["add", "-A"]);
    let refusal: String = prove_the_clone(&capture, root.as_path()).expect_err("an index write is refused first");
    assert!(refusal.contains("copied index differs in its length and SHA-256"), "{refusal}");
    remove(root.as_path());
}

#[test]
fn a_digested_path_whose_bytes_differ_is_refused_and_the_unchanged_one_passes() {
    let root: PathBuf = a_repository("proof-digested");
    let mut capture: Capture = capture_of(root.as_path());
    // The control gives the capture a digest its file cannot have, so the guard
    // is what the assertion meets rather than a write that D2 would catch first.
    capture.digests = vec![crate::worktree_clone_add_route::add_capture::CapturedDigest { path: b"plain.txt".to_vec(), digest: [0; 32] }];
    let refusal: String = prove_the_clone(&capture, root.as_path()).expect_err("the bytes are not the capture's");
    assert!(refusal.contains("plain.txt differs in its content"), "{refusal}");
    assert!(refusal.contains("SHA-256"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
    let mut unchanged: Capture = capture_of(root.as_path());
    let bytes: Vec<u8> = std::fs::read(root.join("plain.txt")).expect("the file");
    unchanged.digests = vec![crate::worktree_clone_add_route::add_capture::CapturedDigest { path: b"plain.txt".to_vec(), digest: crate::subvolume_sha::sha256(bytes.as_slice()) }];
    prove_the_clone(&unchanged, root.as_path()).expect("a digest the file's own bytes answer passes");
    remove(root.as_path());
}

#[test]
fn a_digested_link_is_compared_through_its_target_bytes() {
    let root: PathBuf = fixture("proof-digested-link");
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::os::unix::fs::symlink("plain.txt", root.join("link")).expect("the link");
    std::fs::write(root.join("plain.txt"), b"plain\n").expect("the file");
    git(root.as_path(), &["add", "."]);
    git(root.as_path(), &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "one"]);
    let mut capture: Capture = capture_of(root.as_path());
    let real: [u8; 32] = crate::subvolume_sha::sha256(b"plain.txt");
    capture.digests = vec![crate::worktree_clone_add_route::add_capture::CapturedDigest { path: b"link".to_vec(), digest: real }];
    prove_the_clone(&capture, root.as_path()).expect("a link's digest is its target's bytes");
    capture.digests = vec![crate::worktree_clone_add_route::add_capture::CapturedDigest { path: b"link".to_vec(), digest: crate::subvolume_sha::sha256(b"other.txt") }];
    let refusal: String = prove_the_clone(&capture, root.as_path()).expect_err("another target is refused");
    assert!(refusal.contains("link differs in its content"), "{refusal}");
    remove(root.as_path());
}

#[test]
fn a_path_the_capture_recorded_absent_and_the_clone_holds_is_refused() {
    let root: PathBuf = a_repository("proof-came-back");
    let capture: Capture = capture_of(root.as_path());
    std::fs::write(root.join("gone.txt"), b"back\n").expect("the control brings the path back");
    let refusal: String = prove_the_clone(&capture, root.as_path()).expect_err("a returned path is refused");
    assert!(refusal.contains("gone.txt differs in its kind"), "{refusal}");
    remove(root.as_path());
}
