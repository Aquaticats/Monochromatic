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

/// One raw `diff-tree -z` record.
fn record(from_mode: &str, to_mode: &str, from: &str, to: &str, path: &str) -> String {
    return format!(":{from_mode} {to_mode} {from} {to} M\0{path}\0");
}

/// Modified ordinary files get a completion; mode changes, symbolic links and malformed records do not.
#[test]
fn hook_changes_list_every_path_and_complete_only_ordinary_modifications() {
    let text: String = [
        record("100644", "100644", "aaaa", "bbbb", "a.txt"),
        record("100755", "100755", "cccc", "dddd", "run.sh"),
        record("100644", "100755", "eeee", "ffff", "mode.txt"),
        record("120000", "120000", "1111", "2222", "link"),
    ]
    .concat();
    let changes: HookChanges = parse_hook_changes(text.as_bytes()).expect("parsed");
    assert_eq!(changes.paths, vec![b"a.txt".to_vec(), b"run.sh".to_vec(), b"mode.txt".to_vec(), b"link".to_vec()]);
    assert_eq!(
        changes.worktree_records,
        vec![
            AddedPath {
                path: b"a.txt".to_vec(),
                git_mode: String::from("100644"),
                original_oid: String::from("aaaa"),
                intended_oid: String::from("bbbb"),
            },
            AddedPath {
                path: b"run.sh".to_vec(),
                git_mode: String::from("100755"),
                original_oid: String::from("cccc"),
                intended_oid: String::from("dddd"),
            },
        ]
    );
    assert!(parse_hook_changes(b":100644 100644 a b\0x\0").is_err());
    assert!(parse_hook_changes(b":100644 100644 a b M").is_err());
    assert_eq!(parse_hook_changes(b"").expect("empty"), HookChanges::default());
}

/// A path is bytes: names that are not UTF-8 are kept exactly in the path list and in their completion records,
///  and two names that read alike stay apart.
#[test]
fn hook_changes_keep_the_exact_bytes_of_a_name_that_is_not_utf8() {
    let changes: HookChanges =
        parse_hook_changes(b":100644 100644 aaaa bbbb M\0caf\xe9.txt\0:100644 100644 cccc dddd M\0caf\xe8.txt\0")
            .expect("parsed");
    assert_eq!(changes.paths, vec![b"caf\xe9.txt".to_vec(), b"caf\xe8.txt".to_vec()]);
    assert_eq!(changes.worktree_records[0].path, b"caf\xe9.txt");
    assert_eq!(changes.worktree_records[1].path, b"caf\xe8.txt");
}

/// Combining keeps each path once in first-seen order and every completion in order.
#[test]
fn combined_hook_changes_keep_paths_once_and_every_completion() {
    let earlier: HookChanges = parse_hook_changes(record("100644", "100644", "a", "b", "x").as_bytes()).expect("earlier");
    let later: HookChanges = parse_hook_changes(
        [record("100644", "100644", "c", "d", "x"), record("100644", "100644", "e", "f", "y")]
            .concat()
            .as_bytes(),
    )
    .expect("later");
    let combined: HookChanges = HookChanges::combine(&earlier, &later);
    assert_eq!(combined.paths, vec![b"x".to_vec(), b"y".to_vec()]);
    assert_eq!(combined.worktree_records.len(), 3);
    assert_eq!(combined.worktree_records[0].intended_oid, "b");
    assert_eq!(combined.worktree_records[2].path, b"y");
}

/// Equal trees describe no change and ask Git nothing.
#[test]
fn equal_trees_have_no_hook_changes() {
    let changes: HookChanges = describe_hook_changes(
        &context(),
        Path::new("/nonexistent"),
        Path::new("/nonexistent/objects"),
        "same",
        "same",
    )
    .expect("no Git needed");
    assert_eq!(changes, HookChanges::default());
}

/// A tree difference read from a real repository lists the changed paths and the ordinary completion.
#[test]
fn hook_changes_are_read_from_two_real_trees() {
    let root: PathBuf = fixture("worktree-records-trees");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    let before: String = String::from_utf8_lossy(git(repo.as_path(), &["write-tree"]).stdout.as_slice()).trim().to_string();
    std::fs::write(repo.join("a.txt"), b"two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    let after: String = String::from_utf8_lossy(git(repo.as_path(), &["write-tree"]).stdout.as_slice()).trim().to_string();
    let changes: HookChanges =
        describe_hook_changes(&context(), repo.as_path(), repo.join(".git/objects").as_path(), before.as_str(), after.as_str())
            .expect("changes");
    assert_eq!(changes.paths, vec![b"a.txt".to_vec()]);
    assert_eq!(changes.worktree_records.len(), 1);
    assert_eq!(changes.worktree_records[0].git_mode, "100644");
    remove(root.as_path());
}

/// A corrected file whose worktree copy still holds the original bytes is recorded with both blob names;
///  a copy that changed since, a missing copy and a symbolic link are not.
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

/// A symbolic link or submodule correction is never recorded.
#[test]
fn non_ordinary_modes_get_no_completion() {
    assert_eq!(ordinary_git_mode(CandidateMode::Regular), Some("100644"));
    assert_eq!(ordinary_git_mode(CandidateMode::Executable), Some("100755"));
    assert_eq!(ordinary_git_mode(CandidateMode::Symlink), None);
    assert_eq!(ordinary_git_mode(CandidateMode::Gitlink), None);
}

/// A correction whose index entry changed mode, or whose bytes equal the original, is not recorded.
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

/// A path whose index entries are all unmerged has no settled entry, and a stage-zero entry reports its mode and object.
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
