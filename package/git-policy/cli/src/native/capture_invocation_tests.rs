//! Controls for the invocation capture:
//!  the layout, the symbolic `HEAD`, the conclusion, the ref backend and the base.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, repository, remove};
#[cfg(unix)]
use crate::recording_git::{recorded_starts, recording_context};

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

/// The empty tree of a SHA-1 or SHA-256 repository is known without hashing:
///  no `hash-object` start happens and the recorded name is what Git hashes.
#[test]
#[cfg(unix)]
fn the_empty_tree_of_a_known_format_starts_no_hash_object() {
    for format in ["sha1", "sha256"] {
        let root: PathBuf = fixture(format!("capture-invocation-no-hash-{format}").as_str());
        let repo: PathBuf = root.join("repo");
        std::fs::create_dir(&repo).expect("fixture");
        git(repo.as_path(), &["init", "--quiet", format!("--object-format={format}").as_str()]);
        let (context, log): (GitContext, PathBuf) = recording_context(root.as_path());
        let layout: InvocationLayout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
        let starts: Vec<String> = recorded_starts(log.as_path());
        assert!(!starts.is_empty(), "the recorder must see the layout's other starts");
        for start in &starts {
            assert!(!start.starts_with("hash-object"), "unexpected start: {start}");
        }
        let oracle: String = git_text(repo.as_path(), &["hash-object", "-t", "tree", "--stdin"]);
        assert_eq!(layout.empty_tree_oid, oracle);
        remove(root.as_path());
    }
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
