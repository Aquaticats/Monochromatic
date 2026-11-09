//! What:
//!  The facts one record of a removed worktree carries,
//!  against real Git:
//!  the `HEAD` the worktree was on,
//!  the number of
//!       tracked paths that differed,
//!  and the sentence that says what was not counted.
//! Why:
//!  The owner asked for one line naming the worktree,
//!  whether `--force` was given,
//!  and what the removal took,
//!  and the line must never claim the worktree was clean when a removal takes
//!       more than it counted.

use super::{changed_paths, removal_facts, target_path, worktree_head};
use crate::discard_audit_facts::AuditFact;
use crate::test_support::{fixture, git, isolated_overlay, remove, repository};
use crate::transaction_git::GitContext;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// A context that reads a repository through `-C`.
fn context(repo: &std::path::Path) -> GitContext {
    return GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay: isolated_overlay(),
        global_prefix: vec![OsString::from("-C"), repo.as_os_str().to_os_string()],
        common_dir: None,
    };
}

/// The value of one named fact,
/// or empty text when the record carries none.
fn value(facts: &[AuditFact], name: &'static str) -> String {
    for fact in facts {
        if fact.name() != name {
            continue;
        }
        return match fact {
            AuditFact::Text(_, text) => text.clone(),
            AuditFact::Flag(_, flag) => String::from(if *flag { "true" } else { "false" }),
        };
    }
    return String::new();
}

/// Real Git:
///  a linked worktree on a branch with one tracked edit answers all four facts.
#[test]
fn real_git_answers_the_head_and_the_tracked_changes() {
    let root: PathBuf = fixture("worktree-audit");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../linked"]);
    std::fs::write(linked.join("tracked.txt"), b"the person wrote this\n").expect("edit");
    std::fs::write(linked.join("untracked.txt"), b"not counted\n").expect("untracked file");

    // The target is read before anything removes it.
    assert_eq!(worktree_head(linked.as_path()).as_deref(), Some("refs/heads/topic"));
    let reading: GitContext = context(repo.as_path());
    assert_eq!(changed_paths(&reading, linked.as_path()), Some(1));
    let facts: Vec<AuditFact> = removal_facts(&reading, linked.as_path(), OsStr::new("../linked"), true);
    assert_eq!(value(&facts, "worktree"), "../linked");
    assert_eq!(value(&facts, "force"), "true");
    assert_eq!(value(&facts, "head"), "refs/heads/topic");
    assert_eq!(value(&facts, "changed"), "1");
    assert_eq!(value(&facts, "counted"), "tracked changes only");

    // A clean worktree says zero, and a worktree Git cannot read answers nothing.
    git(linked.as_path(), &["stash", "push", "--quiet"]);
    assert_eq!(changed_paths(&reading, linked.as_path()), Some(0));
    assert_eq!(changed_paths(&reading, Path::new("/nonexistent-worktree")), None);
    assert_eq!(worktree_head(Path::new("/nonexistent-worktree")), None);
    let missing: Vec<AuditFact> = removal_facts(&reading, Path::new("/nonexistent"), OsStr::new("/nonexistent"), false);
    assert_eq!(value(&missing, "changed"), "");
    assert_eq!(value(&missing, "force"), "false");
    assert_eq!(value(&missing, "counted"), "tracked changes only");

    // A relative target is joined to the directory the command runs in, which is the repository here, and read there.
    let joined: PathBuf = target_path(OsStr::new("../linked"), repo.as_path());
    assert_eq!(std::fs::canonicalize(joined.as_path()).expect("the joined target"), linked);
    assert_eq!(changed_paths(&reading, joined.as_path()), Some(0));
    remove(root.as_path());
}
