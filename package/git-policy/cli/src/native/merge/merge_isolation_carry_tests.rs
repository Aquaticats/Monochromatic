//! Controls for reading the person's own work and keeping it outside the checkout:
//! both layers of a path,
//! the objects that hold them,
//! the ref that keeps those from `git gc`,
//! and the shapes a capture refuses.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A repository with `a.txt` committed,
/// then edited in the worktree and staged for `b.txt`.
fn edited(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"a base\n").expect("a");
    std::fs::write(repo.join("b.txt"), b"b base\n").expect("b");
    git(repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"a base\nthe person's line\n").expect("the person's edit");
    std::fs::write(repo.join("b.txt"), b"b base\nstaged\n").expect("the staged edit");
    git(repo.as_path(), &["add", "--", "b.txt"]);
    return (root, repo);
}

/// What:
///  The worktree's bytes and the index's entry of one path are two layers,
/// and each is kept with its own object.
/// Why:
///   A staged edit is what a commit would record and the worktree may hold more on top of it,
/// so one object for both
///  would lose one of them.
#[test]
fn both_layers_of_one_path_are_captured_separately() {
    let (root, repo) = edited("carry-both-layers");
    let context = git_context();
    let paths: Vec<Vec<u8>> = vec![b"a.txt".to_vec(), b"b.txt".to_vec()];
    let capture: Capture = capture_work(&context, repo.as_path(), paths.as_slice()).expect("the capture");
    assert_eq!(capture.paths.len(), 2);
    let a: &CapturedPath = &capture.paths[0];
    assert_eq!(a.path, b"a.txt");
    assert_eq!(a.worktree.as_ref().map(|file| { return file.mode.as_str(); }), Some("100644"));
    assert_ne!(a.worktree.as_ref().map(|file| { return file.raw.clone(); }), None);
    // The entry is captured as the index holds it, which for a path nobody staged is the branch's own.
    assert_eq!(
        a.index.as_ref().map(|staged| { return staged.oid.clone(); }),
        Some(git_text(repo.as_path(), &["rev-parse", "HEAD:a.txt"]).trim().to_owned()),
        "the index layer of a path with only a worktree edit is the branch's entry"
    );
    let b: &CapturedPath = &capture.paths[1];
    assert!(b.index.is_some(), "the staged edit is a layer of its own");
    assert_eq!(b.index.as_ref().map(|staged| { return staged.oid.clone(); }), Some(git_text(repo.as_path(), &["rev-parse", ":b.txt"]).trim().to_owned()));
    // The raw bytes read back exactly, which is what a restore writes.
    let raw: String = a.worktree.as_ref().expect("the worktree layer").raw.clone();
    let bytes: String = git_text(repo.as_path(), &["cat-file", "blob", raw.as_str()]);
    assert_eq!(bytes, "a base\nthe person's line");
    remove(root.as_path());
}

/// What:
///  The capture is kept by one commit on a ref of its own,
/// and the ref goes when the merge is done with it.
/// Why:
///   A blob nothing points at is collected,
/// and a ref left behind keeps the person's bytes for ever.
#[test]
fn the_capture_is_kept_on_a_ref_and_the_ref_goes() {
    let (root, repo) = edited("carry-kept-on-a-ref");
    let context = git_context();
    let paths: Vec<Vec<u8>> = vec![b"a.txt".to_vec()];
    let capture: Capture = keep_capture(&context, repo.as_path(), &capture_work(&context, repo.as_path(), paths.as_slice()).expect("the capture")).expect("kept");
    let reference: String = capture.reference.clone().expect("a ref");
    assert!(reference.starts_with(CARRIED_REF_PREFIX), "{reference}");
    assert_eq!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", CARRIED_REF_PREFIX]).trim(), reference);
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", capture.commit.as_str()]).trim(), "commit");
    drop_capture_ref(&context, repo.join(".git").as_path(), repo.as_path(), &capture);
    assert_eq!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", CARRIED_REF_PREFIX]).trim(), "");
    remove(root.as_path());
}

/// What:
///  A path that stands as a directory is not one edit,
/// so the capture refuses it and the command falls back to native Git's own answer.
/// Why:
///   Guessing which files in it are the person's would lose the rest.
#[test]
fn a_directory_at_a_paths_place_is_refused() {
    let root: PathBuf = fixture("carry-directory");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("d"), b"was a file\n").expect("d");
    git(repo.as_path(), &["add", "--", "d"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::remove_file(repo.join("d")).expect("remove d");
    std::fs::create_dir(repo.join("d")).expect("a directory");
    std::fs::write(repo.join("d/x"), b"inside\n").expect("inside");
    let context = git_context();
    let refused: Result<Capture, String> = capture_work(&context, repo.as_path(), &[b"d".to_vec()]);
    assert!(refused.is_err(), "a directory is refused");
    remove(root.as_path());
}
