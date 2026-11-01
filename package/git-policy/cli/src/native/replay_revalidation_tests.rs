//! Controls for the revalidation steps that need no policy: the replayed index, the shadow `HEAD` move,
//!  and the `pre-commit` decision.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// The re-run decision: hooks not bypassed and a changed tree.
#[test]
fn pre_commit_reruns_only_for_a_changed_tree_without_no_verify() {
    assert!(pre_commit_rerun_required(false, "aaa", "bbb"));
    assert!(!pre_commit_rerun_required(false, "aaa", "aaa"));
    assert!(!pre_commit_rerun_required(true, "aaa", "bbb"));
    assert!(!pre_commit_rerun_required(true, "aaa", "aaa"));
}

/// The replayed index lives under its replay's directory.
#[test]
fn the_replayed_index_is_named_by_replay() {
    assert_eq!(
        replayed_index_path(Path::new("/tx"), 3),
        PathBuf::from("/tx/replay-3/commit.index")
    );
}

/// The merged tree reads into the private index and not into the real one.
#[test]
fn the_merged_tree_reads_into_the_private_index() {
    let root: PathBuf = fixture("replay-revalidation-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("f.txt"), "one\n").expect("file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tree"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]).trim().to_string();
    let real_index: PathBuf = repo.join(".git").join("index");
    let real_before: Vec<u8> = std::fs::read(&real_index).expect("real index");
    let index: PathBuf = replayed_index_path(root.as_path(), 1);
    read_replayed_index(&git_context(), repo.as_path(), repo.join(".git").join("objects").as_path(), index.as_path(), &tree)
        .expect("read-tree");
    let mut request: GitRequest = GitRequest::new(repo.as_path(), &["ls-files"]);
    request.index = Some(index.clone());
    let listed: String = String::from_utf8_lossy(&run_git_checked(&git_context(), &request).expect("ls-files").stdout).into_owned();
    assert_eq!(listed, "f.txt\n");
    assert_eq!(std::fs::read(&real_index).expect("real index after"), real_before);
    remove(root.as_path());
}

/// A symbolic shadow `HEAD` moves its branch, and a detached one moves itself.
#[test]
fn the_shadow_head_moves_its_branch_or_detached_head() {
    let root: PathBuf = fixture("replay-revalidation-head");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]);
    let second: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let shadow: PathBuf = repo.join(".git");
    move_shadow_head(&git_context(), shadow.as_path(), repo.as_path(), "refs/heads/main", first.as_str(), false)
        .expect("branch move");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "refs/heads/main"]).trim(), first);
    git(repo.as_path(), &["checkout", "--quiet", "--detach", second.as_str()]);
    move_shadow_head(&git_context(), shadow.as_path(), repo.as_path(), "HEAD", first.as_str(), true)
        .expect("detached move");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim(), first);
    remove(root.as_path());
}
