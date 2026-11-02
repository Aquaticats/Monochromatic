//! Controls for the native check that decides whether a command left the branch ahead of its upstream: the walk over the
//! commit parents against real Git, and the tip decision on disposable repositories.

use super::*;
use crate::test_support::{fixture, git, remove};

/// A repository with a bare remote, `main` pushed and tracking `origin/main`.
fn repository(name: &str) -> (PathBuf, PathBuf) {
    let root = fixture(name);
    let remote = root.join("remote.git");
    let repo = root.join("repo");
    git(&root, &["init", "--bare", "--quiet", "--initial-branch=main", "remote.git"]);
    git(&root, &["init", "--quiet", "--initial-branch=main", "repo"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=base"]);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    return (root, repo);
}

/// The commit a revision names.
fn oid(repo: &Path, revision: &str) -> String {
    return String::from_utf8_lossy(&git(repo, &["rev-parse", revision]).stdout).trim().to_owned();
}

/// Whether real Git says `ancestor` is an ancestor of `descendant`.
fn git_says_ancestor(repo: &Path, ancestor: &str, descendant: &str) -> bool {
    return crate::test_support::git_output(repo, &["merge-base", "--is-ancestor", ancestor, descendant]).status.success();
}

/// A watch of the repository as the wrapper would start it, with the fixture's environment.
fn watch(repo: &Path) -> TipWatch {
    let overlay: Vec<(OsString, OsString)> = crate::test_support::isolated_overlay();
    return watch_tip(repo, Path::new("/usr/bin/git"), overlay.as_slice(), &[]).expect("a branch with a tip");
}

/// The walk answers like `git merge-base --is-ancestor` for ancestors, non-ancestors and the commit itself.
#[test]
fn the_walk_agrees_with_git_about_ancestry() {
    let (root, repo) = repository("atr-walk");
    let base = oid(&repo, "HEAD");
    git(&repo, &["switch", "--quiet", "--create", "side"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=side"]);
    let side = oid(&repo, "HEAD");
    git(&repo, &["switch", "--quiet", "main"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=main"]);
    git(&repo, &["merge", "--quiet", "--no-ff", "--no-edit", "side"]);
    let merged = oid(&repo, "HEAD");
    let layout = read_layout_in(&repo, crate::test_support::isolated_overlay().as_slice()).expect("layout");
    let database = ObjectDatabase::open(layout.common_dir.as_path(), None, layout.object_format).expect("database");
    for (from, target) in [(&merged, &base), (&merged, &side), (&side, &base), (&base, &side), (&side, &merged), (&base, &base)] {
        assert_eq!(
            reaches(&database, &layout, from, target, WALK_BUDGET),
            Some(git_says_ancestor(&repo, target, from)),
            "{target} from {from}"
        );
    }
    assert_eq!(reaches(&database, &layout, &merged, &base, 0), None, "a walk past its budget does not guess");
    remove(&root);
}

/// A commit on top of the upstream is handed over; the same tip is not handed over twice, and an unmoved branch is not.
#[test]
fn a_branch_that_grew_ahead_of_its_upstream_is_handed_over() {
    let (root, repo) = repository("atr-grew");
    let watched = watch(&repo);
    assert_eq!(tip_to_push(&watched), None, "nothing moved");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    assert_eq!(tip_to_push(&watched), Some(oid(&repo, "HEAD")));
    remove(&root);
}

/// A branch reset to an older commit moved backwards, which is not growth.
#[test]
fn a_branch_that_moved_backwards_is_not_handed_over() {
    let (root, repo) = repository("atr-back");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    git(&repo, &["push", "--quiet", "origin", "main"]);
    let watched = watch(&repo);
    git(&repo, &["reset", "--quiet", "--hard", "HEAD~1"]);
    assert_eq!(tip_to_push(&watched), None);
    remove(&root);
}

/// A rewritten branch (the same commits replayed) has grown past the upstream but not from its old tip: a rebase.
#[test]
fn a_rewritten_branch_is_not_handed_over() {
    let (root, repo) = repository("atr-rewrite");
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=local"]);
    let watched = watch(&repo);
    git(&repo, &["commit", "--quiet", "--amend", "--allow-empty", "--message=local, rewritten"]);
    assert_eq!(tip_to_push(&watched), None);
    remove(&root);
}

/// A branch that has diverged from the tracking ref is not handed over, and neither is one without an upstream.
#[test]
fn a_diverged_branch_and_a_branch_without_an_upstream_are_not_handed_over() {
    let (root, repo) = repository("atr-diverged");
    git(&repo, &["switch", "--quiet", "--create", "loose"]);
    let loose = watch(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=more"]);
    assert_eq!(tip_to_push(&loose), None, "no upstream");
    git(&repo, &["switch", "--quiet", "main"]);
    let watched = watch(&repo);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=mine"]);
    // The tracking ref moves to a commit the branch does not contain, as a fetch would move it.
    git(&repo, &["switch", "--quiet", "--detach", "HEAD~1"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=theirs"]);
    let theirs = oid(&repo, "HEAD");
    git(&repo, &["update-ref", "refs/remotes/origin/main", &theirs]);
    git(&repo, &["switch", "--quiet", "main"]);
    assert_eq!(tip_to_push(&watched), None);
    remove(&root);
}
