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

use super::{changed_paths, removal_facts, removal_target, worktree_head};
use crate::discard_audit_facts::AuditFact;
use crate::command_test_support::git_status;
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
    let joined: PathBuf = removal_target(&reading, OsStr::new("../linked"), repo.as_path());
    assert_eq!(std::fs::canonicalize(joined.as_path()).expect("the joined target"), linked);
    assert_eq!(changed_paths(&reading, joined.as_path()), Some(0));
    remove(root.as_path());
}

/// A `.git` that is not a regular file is never opened:
/// a FIFO with no writer would hold the removal for ever.
#[test]
fn a_pointer_that_is_not_a_regular_file_is_never_opened() {
    let root: PathBuf = fixture("worktree-audit-fifo");
    let linked: PathBuf = root.join("linked");
    std::fs::create_dir(&linked).expect("linked directory");
    let made: std::process::Output = std::process::Command::new("mkfifo")
        .arg(linked.join(".git"))
        .output()
        .expect("mkfifo runs");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(made.stderr.as_slice()));
    // The question runs on another thread, so a read that waits cannot hold this control for ever.
    let (sender, receiver) = std::sync::mpsc::channel::<Option<String>>();
    let asked: PathBuf = linked.clone();
    let _asker = std::thread::spawn(move || {
        // A receiver that is gone means the control already failed; nobody is left to tell.
        let _sent = sender.send(worktree_head(asked.as_path()));
    });
    let answer: Option<String> = match receiver.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(found) => found,
        Err(_) => panic!("the audit opened the FIFO and never returned"),
    };
    assert_eq!(answer, None);
    // A directory named `.git` is a main worktree, which a removal never names.
    std::fs::remove_file(linked.join(".git")).expect("drop the FIFO");
    std::fs::create_dir(linked.join(".git")).expect("main worktree directory");
    assert_eq!(worktree_head(linked.as_path()), None);
    remove(root.as_path());
}

/// A relative `gitdir:` pointer is resolved against the worktree it sits in,
/// not against the directory the
///  wrapper happens to run in.
#[test]
fn a_relative_pointer_is_resolved_against_its_worktree() {
    let root: PathBuf = fixture("worktree-audit-relative");
    let linked: PathBuf = root.join("linked");
    let own: PathBuf = linked.join("own-git-dir");
    std::fs::create_dir_all(&own).expect("own Git directory");
    std::fs::write(own.join("HEAD"), b"ref: refs/heads/topic\n").expect("HEAD");
    std::fs::write(linked.join(".git"), b"gitdir: ./own-git-dir\n").expect("pointer file");
    assert_eq!(worktree_head(linked.as_path()), Some(String::from("refs/heads/topic")));
    // An absolute pointer is read as it is written.
    std::fs::write(
        linked.join(".git"),
        format!("gitdir: {}\n", own.display()).as_bytes(),
    )
    .expect("pointer file");
    assert_eq!(worktree_head(linked.as_path()), Some(String::from("refs/heads/topic")));
    // A pointer Git would refuse names no Git directory.
    std::fs::write(linked.join(".git"), b"not a pointer\n").expect("pointer file");
    assert_eq!(worktree_head(linked.as_path()), None);
    remove(root.as_path());
}

/// Real Git:
///  a removal names the worktree by its path or by a unique path suffix,
///  and the resolution names the
///       same worktree Git removes in each spelling,
///  or the named path where no worktree matches.
#[test]
fn real_git_resolves_the_worktree_a_removal_names() {
    let root: PathBuf = fixture("worktree-audit-suffix");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let parent: PathBuf = root.join("worktrees");
    std::fs::create_dir(&parent).expect("worktrees directory");
    let linked: PathBuf = parent.join("topic");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../worktrees/topic"]);
    let reading: GitContext = context(repo.as_path());
    // The exact path, in both spellings Git accepts for it.
    let name: &OsStr = linked.file_name().expect("a name");
    assert_eq!(removal_target(&reading, name, repo.as_path()), linked);
    assert_eq!(removal_target(&reading, OsStr::new("../worktrees/topic"), repo.as_path()), linked);
    // A unique suffix of the registered path, where the command's own directory holds no such worktree.
    std::fs::create_dir(repo.join("topic")).expect("a directory of the same name");
    assert_eq!(removal_target(&reading, OsStr::new("topic"), repo.as_path()), linked);
    // A name no worktree answers is the path the command named, so the record reads that and says nothing about
    //  another directory.
    assert_eq!(
        removal_target(&reading, OsStr::new("no-such-worktree"), repo.as_path()),
        repo.join("no-such-worktree")
    );
    // The control: Git removes that worktree under exactly the suffix this resolution names.
    let removed: std::process::Output = git_status(repo.as_path(), &["worktree", "remove", "--force", "topic"]);
    assert!(removed.status.success(), "{}", String::from_utf8_lossy(removed.stderr.as_slice()));
    assert!(!linked.exists());
    remove(root.as_path());
}

/// Real Git:
///  a worktree whose path holds a line break is listed whole,
///  so a removal by its unique suffix
///       resolves to that worktree and not to a directory beside the command.
#[test]
fn real_git_resolves_a_worktree_whose_path_holds_a_line_break() {
    let root: PathBuf = fixture("worktree-audit-line-break");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let parent: PathBuf = root.join("worktrees");
    std::fs::create_dir(&parent).expect("worktrees directory");
    let linked: PathBuf = parent.join("line\nbreak/chapter");
    git(
        repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "chapter", "../worktrees/line\nbreak/chapter"],
    );
    assert!(linked.is_dir(), "the control needs a worktree whose path holds a line break");
    let reading: GitContext = context(repo.as_path());
    // The listed paths keep the line break, so the suffix names this worktree.
    assert_eq!(removal_target(&reading, OsStr::new("chapter"), repo.as_path()), linked);
    // The control: Git removes that worktree under exactly that suffix.
    std::fs::create_dir(repo.join("chapter")).expect("a directory of the same name");
    let removed: std::process::Output = git_status(repo.as_path(), &["worktree", "remove", "--force", "chapter"]);
    assert!(removed.status.success(), "{}", String::from_utf8_lossy(removed.stderr.as_slice()));
    assert!(!linked.exists());
    remove(root.as_path());
}
