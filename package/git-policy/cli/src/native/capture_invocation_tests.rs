//! Controls for the invocation capture:
//!  the layout, the symbolic `HEAD`, the conclusion, the ref backend and the base.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, repository, remove};

/// Git's empty tree under SHA-1, the object format of every fixture.
const SHA1_EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// A branch repository reports its branch as the target, a conclusion of none, and its layout.
#[test]
fn branch_layout_names_the_branch_and_the_store() {
    let root: PathBuf = fixture("capture-invocation-branch");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let layout: InvocationLayout =
        capture_invocation_layout(&git_context(), repo.as_path(), false).expect("layout");
    assert_eq!(
        layout.symbolic_head,
        SymbolicHead::Branch(String::from("refs/heads/main"))
    );
    assert_eq!(layout.target_ref, "refs/heads/main");
    assert_eq!(layout.conclusion, Conclusion::None);
    assert_eq!(layout.ref_format, RefFormat::Files);
    assert_eq!(layout.object_format, "sha1");
    assert_eq!(layout.empty_tree_oid, SHA1_EMPTY_TREE);
    assert_eq!(layout.repository_root, std::fs::canonicalize(&repo).expect("fixture"));
    assert_eq!(layout.git_dir, repo.join(".git"));
    assert_eq!(layout.common_dir, repo.join(".git"));
    assert_eq!(layout.real_index_path, repo.join(".git").join("index"));
    assert_eq!(layout.object_directory, repo.join(".git").join("objects"));
    assert_eq!(
        layout.registry_root,
        repo.join(".git").join("cli-git-transactions")
    );
    assert!(!layout.invoked_at.is_empty());
    remove(root.as_path());
}

/// A detached `HEAD` makes `HEAD` itself the ref the landing advances.
#[test]
fn detached_head_targets_head_itself() {
    let root: PathBuf = fixture("capture-invocation-detached");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let layout: InvocationLayout =
        capture_invocation_layout(&git_context(), repo.as_path(), false).expect("layout");
    assert_eq!(layout.symbolic_head, SymbolicHead::Detached);
    assert_eq!(layout.target_ref, "HEAD");
    remove(root.as_path());
}

/// Amend wins over any conclusion marker, and each marker decides its own kind.
#[test]
fn conclusion_markers_decide_the_kind() {
    let root: PathBuf = fixture("capture-invocation-conclusion");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let context = git_context();
    assert_eq!(
        resolve_conclusion_kind(&context, repo.as_path(), true),
        Ok(Conclusion::Amend)
    );
    std::fs::write(repo.join(".git").join("MERGE_HEAD"), format!("{head}\n")).expect("fixture");
    assert_eq!(
        resolve_conclusion_kind(&context, repo.as_path(), false),
        Ok(Conclusion::Merge)
    );
    std::fs::remove_file(repo.join(".git").join("MERGE_HEAD")).expect("fixture");
    std::fs::write(repo.join(".git").join("CHERRY_PICK_HEAD"), format!("{head}\n")).expect("fixture");
    assert_eq!(
        resolve_conclusion_kind(&context, repo.as_path(), false),
        Ok(Conclusion::CherryPick)
    );
    std::fs::remove_file(repo.join(".git").join("CHERRY_PICK_HEAD")).expect("fixture");
    std::fs::write(repo.join(".git").join("REVERT_HEAD"), format!("{head}\n")).expect("fixture");
    assert_eq!(
        resolve_conclusion_kind(&context, repo.as_path(), false),
        Ok(Conclusion::Revert)
    );
    remove(root.as_path());
}

/// An unresolved marker file is not a conclusion: only a resolving pseudoref counts.
#[test]
fn garbage_marker_is_not_a_conclusion() {
    let root: PathBuf = fixture("capture-invocation-garbage-marker");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".git").join("MERGE_HEAD"), "not-an-object\n").expect("fixture");
    assert_eq!(
        resolve_conclusion_kind(&git_context(), repo.as_path(), false),
        Ok(Conclusion::None)
    );
    remove(root.as_path());
}

/// A marker that names an object the repository lacks is still a conclusion in progress.
///  `rev-parse --verify` accepts it and the transaction then fails on the missing object;
///  a batched `cat-file --batch-check` read reports such a marker as missing,
///  which would silently turn a merge into a plain commit that drops the merge parent.
///  This pins why the three markers are probed with `rev-parse --verify` and not read in one call.
#[test]
fn a_marker_naming_a_missing_object_is_still_a_conclusion() {
    let root: PathBuf = fixture("capture-invocation-dangling-marker");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".git").join("MERGE_HEAD"), format!("{}\n", "a".repeat(40))).expect("fixture");
    assert_eq!(
        resolve_conclusion_kind(&git_context(), repo.as_path(), false),
        Ok(Conclusion::Merge)
    );
    remove(root.as_path());
}

/// The base is the target's commit now, and an unborn target starts from the empty tree.
#[test]
fn base_follows_the_target_and_unborn_uses_the_empty_tree() {
    let root: PathBuf = fixture("capture-invocation-base");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context = git_context();
    let layout: InvocationLayout =
        capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture =
        capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(capture.base, Base::Commit(head.clone()));
    assert_eq!(base_revision(&capture), head);

    let unborn_root: PathBuf = fixture("capture-invocation-unborn");
    git(unborn_root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let unborn_layout: InvocationLayout =
        capture_invocation_layout(&context, unborn_root.as_path(), false).expect("layout");
    let unborn: InvocationCapture =
        capture_invocation_base(&context, unborn_root.as_path(), unborn_layout).expect("capture");
    assert_eq!(unborn.base, Base::Unborn);
    assert_eq!(base_revision(&unborn), SHA1_EMPTY_TREE);
    remove(unborn_root.as_path());
    remove(root.as_path());
}

/// A linked worktree reports its own Git directory apart from the common directory.
#[test]
fn linked_worktree_separates_git_dir_from_common_dir() {
    let root: PathBuf = fixture("capture-invocation-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let linked: PathBuf = root.join("linked");
    git(
        repo.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature",
            linked.to_str().expect("fixture"),
        ],
    );
    let layout: InvocationLayout =
        capture_invocation_layout(&git_context(), linked.as_path(), false).expect("layout");
    assert_eq!(layout.common_dir, repo.join(".git"));
    assert_eq!(layout.git_dir, repo.join(".git").join("worktrees").join("linked"));
    assert_eq!(layout.repository_root, std::fs::canonicalize(&linked).expect("fixture"));
    assert_ne!(layout.git_dir, layout.common_dir);
    remove(root.as_path());
}

/// A directory outside any repository stops the transaction before any mutation.
#[test]
fn outside_a_repository_the_layout_fails() {
    let root: PathBuf = fixture("capture-invocation-outside");
    let error: GitFailure =
        capture_invocation_layout(&git_context(), root.as_path(), false).expect_err("no repo");
    assert!(error.0.contains("rev-parse"), "{}", error.0);
    remove(root.as_path());
}

/// The incomplete-layout guard rejects an empty report line.
#[test]
fn an_empty_layout_line_is_incomplete() {
    let lines: Vec<&str> = vec!["/repo", "", "/repo/.git"];
    assert_eq!(
        required_line(&lines, 1),
        Err(GitFailure(String::from("Git returned incomplete repository layout.")))
    );
    assert_eq!(
        required_line(&lines, 9),
        Err(GitFailure(String::from("Git returned incomplete repository layout.")))
    );
    assert_eq!(required_line(&lines, 0), Ok(String::from("/repo")));
}

/// A stand-in Git that logs each start and fails, so a read that starts Git shows in the log.
fn logging_stand_in(root: &Path) -> (GitContext, PathBuf) {
    let log: PathBuf = root.join("starts.log");
    let path: PathBuf = root.join("stand-in");
    let content: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexit 99\n", log.display());
    crate::test_support::executable(path.as_path(), content.as_bytes());
    return (
        GitContext { real_git: path, overlay: Vec::new(), global_prefix: Vec::new() },
        log,
    );
}

/// A snapshot parsed from `(scope, record)` pairs, read with or without global options.
fn listing_snapshot(records: &[(&str, &str)], without_global_options: bool) -> ConfigSnapshot {
    let mut bytes: Vec<u8> = Vec::new();
    for (scope, record) in records {
        bytes.extend_from_slice(scope.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(record.as_bytes());
        bytes.push(0);
    }
    return ConfigSnapshot::parse(bytes.as_slice(), without_global_options).expect("a well-formed listing");
}

/// A snapshot without the ref storage key proves `files` with no Git start; every other snapshot asks Git.
#[test]
fn the_ref_format_comes_from_a_snapshot_that_proves_files_and_from_git_otherwise() {
    let root: PathBuf = fixture("capture-invocation-ref-format-snapshot");
    let (context, log) = logging_stand_in(root.as_path());
    let proving: ConfigSnapshot = listing_snapshot(&[("local", "core.bare\nfalse")], true);
    assert_eq!(resolve_ref_format_with(&context, root.as_path(), Some(&proving)), Ok(RefFormat::Files));
    assert!(!log.exists(), "a proving snapshot started Git");
    let naming: ConfigSnapshot = listing_snapshot(&[("local", "extensions.refstorage\nreftable")], true);
    let optioned: ConfigSnapshot = listing_snapshot(&[("local", "core.bare\nfalse")], false);
    for snapshot in [Some(&naming), Some(&optioned), None] {
        let _ = resolve_ref_format_with(&context, root.as_path(), snapshot);
    }
    assert_eq!(
        std::fs::read_to_string(log.as_path()).expect("log"),
        "rev-parse --show-ref-format\n".repeat(3)
    );
    remove(root.as_path());
}

/// Real Git: the layout's ref format equals `rev-parse --show-ref-format` for files and reftable repositories.
#[test]
fn the_layout_ref_format_matches_rev_parse_for_files_and_reftable() {
    let root: PathBuf = fixture("capture-invocation-ref-format-real");
    let files: PathBuf = repository(root.as_path(), "files");
    let reftable: PathBuf = root.join("reftable");
    std::fs::create_dir(&reftable).expect("directory");
    git(reftable.as_path(), &["init", "--quiet", "--ref-format=reftable"]);
    git(reftable.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let context: GitContext = git_context();
    let for_files: InvocationLayout = capture_invocation_layout(&context, files.as_path(), false).expect("layout");
    assert_eq!(for_files.ref_format, RefFormat::Files);
    assert!(for_files.config_snapshot.is_some());
    let for_reftable: InvocationLayout =
        capture_invocation_layout(&context, reftable.as_path(), false).expect("layout");
    assert_eq!(for_reftable.ref_format, RefFormat::Reftable);
    remove(root.as_path());
}
