//! Controls for the worktree completion records of hook changes and selected corrections.

use super::*;
use crate::direct_fix_install::InstallChange;
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;

/// A context that runs the host's Git with no global options.
fn context() -> GitContext {
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: vec![
            (OsString::from("GIT_AUTHOR_NAME"), OsString::from("Fixture")),
            (OsString::from("GIT_AUTHOR_EMAIL"), OsString::from("fixture@example.invalid")),
        ],
        global_prefix: Vec::new(),
        // The context names no repository, so input files are made in the scratch directory beside the fixtures.
        common_dir: Some(crate::test_support::fixture_root()),
    };
}

/// A corrected file whose worktree copy still holds the original bytes is recorded with both blob names;
///  a copy that changed since,
/// a missing copy and a symbolic link are not.
#[test]
fn selected_corrections_are_recorded_only_while_the_worktree_holds_the_original() {
    let root: PathBuf = fixture("worktree-records-selected");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"one").expect("a");
    std::fs::write(repo.join("b.txt"), b"two").expect("b");
    std::fs::write(repo.join("c.txt"), b"three").expect("c");
    git(repo.as_path(), &["add", "--", "a.txt", "b.txt", "c.txt"]);
    let index: PathBuf = repo.join(".git/index");
    // The private index holds the corrected blobs.
    let corrected = |name: &str, bytes: &[u8]| {
        let blob = {
            let mut child = std::process::Command::new(REAL_GIT)
                .current_dir(&repo)
                .args(["hash-object", "-w", "--stdin"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("hash-object");
            std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), bytes).expect("write");
            String::from_utf8_lossy(child.wait_with_output().expect("output").stdout.as_slice()).trim().to_string()
        };
        git(repo.as_path(), &["update-index", "--cacheinfo", "100644", blob.as_str(), name]);
    };
    corrected("a.txt", b"one\n");
    corrected("b.txt", b"two\n");
    corrected("c.txt", b"three\n");
    std::fs::write(repo.join("b.txt"), b"two edited").expect("edited since");
    std::fs::remove_file(repo.join("c.txt")).expect("removed since");
    let change = |name: &str, original: &[u8], replacement: &[u8]| {
        return InstallChange {
            path: name.as_bytes().to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(original),
            replacement: Rc::from(replacement),
        };
    };
    let records: Vec<AddedPath> = selected_worktree_records(
        &context(),
        repo.as_path(),
        repo.as_path(),
        index.as_path(),
        &[
            change("a.txt", b"one", b"one\n"),
            change("b.txt", b"two", b"two\n"),
            change("c.txt", b"three", b"three\n"),
        ],
    )
    .expect("records");
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0].path, b"a.txt");
    assert_eq!(records[0].git_mode, "100644");
    assert_ne!(records[0].original_oid, records[0].intended_oid);
    remove(root.as_path());
}

/// The entry is asked about by its repository-relative name,
/// and `git ls-files` answers relative to the working directory unless it is told otherwise:
/// a selected correction below the top is recorded only while the listing names the path it was asked about.
#[test]
fn a_correction_below_the_top_is_recorded_by_its_repository_relative_name() {
    let root: PathBuf = fixture("worktree-records-nested-cwd");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(repo.join("sub")).expect("sub directory");
    std::fs::write(repo.join("sub/a.txt"), b"old").expect("a");
    git(repo.as_path(), &["add", "--", "sub/a.txt"]);
    let index: PathBuf = repo.join(".git/index");
    // The private index holds the corrected bytes, and the worktree copy still holds the original.
    let corrected: String = {
        let mut child = std::process::Command::new(REAL_GIT)
            .current_dir(&repo)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("hash-object");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), b"old\n").expect("write");
        String::from_utf8_lossy(child.wait_with_output().expect("output").stdout.as_slice()).trim().to_string()
    };
    git(repo.as_path(), &["update-index", "--cacheinfo", "100644", corrected.as_str(), "sub/a.txt"]);
    let records: Vec<AddedPath> = selected_worktree_records(
        &context(),
        repo.join("sub").as_path(),
        repo.as_path(),
        index.as_path(),
        &[InstallChange {
            path: b"sub/a.txt".to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(b"old".as_slice()),
            replacement: Rc::from(b"old\n".as_slice()),
        }],
    )
    .expect("records");
    assert_eq!(records.len(), 1, "the selected correction is recorded: {records:?}");
    assert_eq!(records[0].path, b"sub/a.txt", "the record names the path it was asked about");
    remove(root.as_path());
}

/// A caller whose two glob modes disagree makes Git refuse every pathspec it is given,
/// and the completion-record step still reads the corrected entry:
/// its pathspec is the wrapper's own, so it runs without the caller's modes.
#[test]
fn a_completion_record_reads_its_entry_under_the_callers_conflicting_glob_modes() {
    let root: PathBuf = fixture("worktree-records-glob-modes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"old").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    let index: PathBuf = repo.join(".git/index");
    let corrected: String = {
        let mut child = std::process::Command::new(REAL_GIT)
            .current_dir(&repo)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("hash-object");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), b"old\n").expect("write");
        String::from_utf8_lossy(child.wait_with_output().expect("output").stdout.as_slice()).trim().to_string()
    };
    git(repo.as_path(), &["update-index", "--cacheinfo", "100644", corrected.as_str(), "a.txt"]);
    let mut inherited: GitContext = context();
    inherited.overlay.push((OsString::from("GIT_GLOB_PATHSPECS"), OsString::from("1")));
    inherited.overlay.push((OsString::from("GIT_NOGLOB_PATHSPECS"), OsString::from("1")));
    let records: Vec<AddedPath> = selected_worktree_records(
        &inherited,
        repo.as_path(),
        repo.as_path(),
        index.as_path(),
        &[InstallChange {
            path: b"a.txt".to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(b"old".as_slice()),
            replacement: Rc::from(b"old\n".as_slice()),
        }],
    )
    .expect("the corrected entry is read despite the caller's modes");
    assert_eq!(records.len(), 1, "the correction is recorded: {records:?}");
    assert_eq!(records[0].path, b"a.txt");
    remove(root.as_path());
}

/// A symbolic link or submodule correction is never recorded.
#[test]
fn non_ordinary_modes_get_no_completion() {
    assert_eq!(ordinary_git_mode(CandidateMode::Regular), Some("100644"));
    assert_eq!(ordinary_git_mode(CandidateMode::Executable), Some("100755"));
    assert_eq!(ordinary_git_mode(CandidateMode::Symlink), None);
    assert_eq!(ordinary_git_mode(CandidateMode::Gitlink), None);
}

/// A correction whose index entry changed mode,
/// or whose bytes equal the original,
/// is not recorded.
#[test]
fn a_changed_mode_or_an_unchanged_blob_gets_no_completion() {
    let root: PathBuf = fixture("worktree-records-skips");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("same.txt"), b"same").expect("same");
    std::fs::write(repo.join("moded.txt"), b"moded").expect("moded");
    git(repo.as_path(), &["add", "--", "same.txt", "moded.txt"]);
    // `same.txt` keeps its bytes in the index; `moded.txt` gets other bytes and an executable mode.
    let blob = |bytes: &[u8]| -> String {
        let mut child = std::process::Command::new(REAL_GIT)
            .current_dir(&repo)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("hash-object");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), bytes).expect("write");
        return String::from_utf8_lossy(child.wait_with_output().expect("output").stdout.as_slice()).trim().to_string();
    };
    git(repo.as_path(), &["update-index", "--cacheinfo", "100755", blob(b"moded\n").as_str(), "moded.txt"]);
    let change = |name: &str, original: &[u8], replacement: &[u8]| {
        return InstallChange {
            path: name.as_bytes().to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(original),
            replacement: Rc::from(replacement),
        };
    };
    let records: Vec<AddedPath> = selected_worktree_records(
        &context(),
        repo.as_path(),
        repo.as_path(),
        repo.join(".git/index").as_path(),
        &[change("same.txt", b"same", b"same\n"), change("moded.txt", b"moded", b"moded\n")],
    )
    .expect("records");
    assert_eq!(records, Vec::<AddedPath>::new());
    remove(root.as_path());
}

/// Another file whose name differs only in case is never taken for the corrected path:
/// `GIT_ICASE_PATHSPECS=1` makes Git list both entries,
/// and the first one it lists is not the one asked about.
#[test]
fn a_case_insensitive_lookup_records_the_path_it_asked_about() {
    let root: PathBuf = fixture("worktree-records-icase");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("A.txt"), b"upper\n").expect("A");
    std::fs::write(repo.join("a.txt"), b"old").expect("a");
    git(repo.as_path(), &["add", "--", "A.txt", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=both"]);
    let upper: String = String::from_utf8_lossy(git(repo.as_path(), &["rev-parse", ":A.txt"]).stdout.as_slice()).trim().to_string();
    let corrected: String = {
        let mut child = std::process::Command::new(REAL_GIT)
            .current_dir(&repo)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("hash-object");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), b"old\n").expect("write");
        String::from_utf8_lossy(child.wait_with_output().expect("output").stdout.as_slice()).trim().to_string()
    };
    // The private index holds the corrected bytes of `a.txt`, and the worktree copy still holds the original.
    git(repo.as_path(), &["update-index", "--cacheinfo", "100644", corrected.as_str(), "a.txt"]);
    let mut icase: GitContext = context();
    icase.overlay.push((OsString::from("GIT_ICASE_PATHSPECS"), OsString::from("1")));
    let records: Vec<AddedPath> = selected_worktree_records(
        &icase,
        repo.as_path(),
        repo.as_path(),
        repo.join(".git/index").as_path(),
        &[InstallChange {
            path: b"a.txt".to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(b"old".as_slice()),
            replacement: Rc::from(b"old\n".as_slice()),
        }],
    )
    .expect("records");
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0].intended_oid, corrected, "the record names the blob committed for a.txt");
    assert_ne!(records[0].intended_oid, upper, "the record must not name A.txt's blob");
    remove(root.as_path());
}

/// A path whose index entries are all unmerged has no settled entry,
/// and a stage-zero entry reports its mode and object.
#[test]
fn only_a_stage_zero_entry_is_settled() {
    let root: PathBuf = fixture("worktree-records-settled");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"settled").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    let blob: String = String::from_utf8_lossy(git(repo.as_path(), &["rev-parse", ":a.txt"]).stdout.as_slice())
        .trim()
        .to_string();
    let entries: String = format!("100644 {blob} 1\tu.txt\n100644 {blob} 2\tu.txt\n100644 {blob} 3\tu.txt\n");
    let mut child = std::process::Command::new(REAL_GIT)
        .current_dir(&repo)
        .args(["update-index", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("update-index");
    std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), entries.as_bytes()).expect("write");
    assert!(child.wait().expect("wait").success());
    let index: PathBuf = repo.join(".git/index");
    let settled = settled_entry(&context(), repo.as_path(), index.as_path(), b"a.txt").expect("settled");
    assert_eq!(settled, Some((String::from("100644"), blob.clone())));
    let unmerged = settled_entry(&context(), repo.as_path(), index.as_path(), b"u.txt").expect("unmerged");
    assert_eq!(unmerged, None);
    remove(root.as_path());
}
