//! Controls for the ending a commit has when the service made it and held it behind a reference lock,
//! and only the answer was lost:
//! the command ends with success,
//! no branch names the commit,
//! and no push of it is released.

use super::the_held_ending;
use crate::held_commit_record::{Held, write};
use crate::test_support::{fixture, git, git_text, remove};
use crate::transaction_git::GitContext;
use std::path::PathBuf;

/// The file the held commit changes.
const EDITED: &str = "edited.txt";

/// The subject of the held commit.
const SUBJECT: &str = "the held edit";

/// The lock the commit waited behind.
const LOCK: &str = "refs/heads/main.lock";

/// A repository with the branch at its base and the commit the branch does not name,
/// a journal,
/// and the record of the hold,
/// which is the durable state the service leaves before it answers.
fn held_state(label: &str) -> (PathBuf, PathBuf, PathBuf, Held) {
    let root: PathBuf = fixture(label);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    std::fs::write(repo.join(EDITED), b"the edit\n").expect("the edit");
    git(repo.as_path(), &["add", "--", EDITED]);
    git(repo.as_path(), &["commit", "--quiet", "--message", SUBJECT]);
    let held_commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    // The branch has not moved: the reference lock refused its write.
    git(repo.as_path(), &["update-ref", "--no-deref", "refs/heads/main", base.as_str()]);
    let git_dir: PathBuf = repo.join(".git");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let held: Held = Held {
        seq: 1,
        journal: journal.clone(),
        repository: repo.clone(),
        common_dir: git_dir.clone(),
        git_dir: git_dir.clone(),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: base,
        new: held_commit.clone(),
        commits: vec![held_commit],
        index: git_dir.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![git_dir.join(LOCK)],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write(git_dir.as_path(), &held).expect("the record of the hold");
    return (root, repo, journal, held);
}

/// What:
///  A commit the service held behind a reference lock ends the command with exit `0`,
///  names no landed commit,
///  and releases no push.
/// Why:
///  A received `held` answer ends exactly this way,
///  and the rule is that commits never fail,
///  so a command that learned the same thing from the record of the hold ends the same way.
#[test]
fn a_held_commit_ends_the_command_with_success_and_no_landed_commit() {
    let (root, repo, _journal, held) = held_state("landed-aftermath-held");
    let context: GitContext = GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(held.common_dir.clone()) };
    let before: String = git_text(repo.as_path(), &["status", "--porcelain=v1"]);
    let ending: crate::service_commit::ServiceCommit = the_held_ending(&context, repo.as_path(), &held);
    assert_eq!(ending.result.exit_code, 0, "the commit is made and held, so the command succeeds");
    assert_eq!(ending.result.landed_oid, None, "no branch names the commit yet");
    assert_eq!(ending.result.events, Vec::new(), "the commit ran no policy on this command");
    assert!(ending.handoff.is_none(), "no push is released for a commit no branch names");
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1"]), before, "the ending changes nothing in the repository");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim(), held.old, "the branch has not moved");
    remove(root.as_path());
}
