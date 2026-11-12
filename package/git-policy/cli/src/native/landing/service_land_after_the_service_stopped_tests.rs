//! What:
//!  Controls for the commit a command makes for itself when the commit service stopped answering it,
//!  with real Git in disposable repositories:
//!  the commit that lands,
//!  the same request made again,
//!  the branch another session moved,
//!  and the reference lock that keeps the branch where it is.
//! Why:
//!  These are the states the rule "a commit never fails where native Git would succeed" is kept in,
//!  and each one is a promise about a person's commit:
//!  it lands with its own bytes,
//!  it lands exactly once even when the service makes the same request too,
//!  it lands on the branch as it is now,
//!  and it is kept and named when nothing can move the branch.
//!  The decisions read real references and real objects,
//!  so no stand-in can show them.

use super::*;
use crate::service_protocol::Change;
use crate::test_support::{fixture, git, git_text, remove};
use std::path::{Path, PathBuf};

/// The file a commit of these controls changes.
const EDITED: &str = "edited.txt";

/// A repository of this control's own,
/// with one commit,
/// and the Git context that runs Git in it.
fn repository(name: &str) -> (PathBuf, PathBuf, GitContext) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(repo.as_path()).expect("the repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("base.txt"), b"base\n").expect("the base file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let git_dir: PathBuf = repo.join(".git");
    let context: GitContext = GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(git_dir.clone()) };
    return (root, repo, context);
}

/// The commit the branch names.
fn head_of(repo: &Path) -> String {
    return git_text(repo, &["rev-parse", "HEAD"]).trim().to_string();
}

/// How many commits the branch holds.
fn commits_in(repo: &Path) -> usize {
    return git_text(repo, &["rev-list", "--count", "HEAD"]).trim().parse().expect("a count");
}

/// The bytes a commit recorded at `path`.
fn recorded_bytes(repo: &Path, commit: &str, path: &str) -> String {
    return git_text(repo, &["cat-file", "blob", format!("{commit}:{path}").as_str()]);
}

/// What:
///  A request that writes `bytes` to `path`,
///  as the client prepares one:
///  the content is in the object store,
///  and the request names it.
/// Why:
///  The landing reads only the request,
///  so a control that sends one has to make the objects it names first,
///  exactly as a preparation does.
fn a_request_writing(repo: &Path, path: &str, bytes: &[u8], base: &str, subject: &str) -> CommitRequest {
    std::fs::write(repo.join(path), bytes).expect("the file");
    let blob: String = git_text(repo, &["hash-object", "-w", "--", path]).trim().to_string();
    return CommitRequest {
        id: String::from("request-of-a-stopped-service"),
        repository: repo.to_path_buf(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: base.to_string(),
        changes: vec![Change { path: PathBuf::from(path), mode: 0o100644, blob: Some(blob) }],
        message: format!("{subject}\n"),
        author: None,
        committer: String::from("Fixture <fixture@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: repo.join(".git").join("index"),
        scratch: repo.join(".git"),
        captured: 0,
        holder: String::new(),
    };
}

/// What:
///  A commit the service did not answer is made here,
///  and the branch names it with the bytes the person wrote.
/// Why:
///  Native Git makes this commit on the same repository with no service at all,
///  and the rule is that a commit never fails where native Git would succeed.
#[test]
fn a_commit_the_service_did_not_answer_is_made_here_and_the_branch_names_it() {
    let (root, repo, context) = repository("land-without-service");
    let base: String = head_of(repo.as_path());
    let request: CommitRequest = a_request_writing(repo.as_path(), EDITED, b"the edit\n", base.as_str(), "made without the service");
    let journal: PathBuf = root.join("journal");
    let outcome: WithoutTheService = land_after_the_service_stopped(&context, &request, Some(journal.as_path()));
    let WithoutTheService::Landed { commit, the_index_waits } = outcome else {
        panic!("the commit is made here: {outcome:?}");
    };
    assert!(!the_index_waits, "no other program holds the index, so the update is installed");
    assert_eq!(head_of(repo.as_path()), commit, "the branch names the commit this command made");
    assert_eq!(commits_in(repo.as_path()), 2, "one commit was added to the one the repository had");
    assert_eq!(recorded_bytes(repo.as_path(), commit.as_str(), EDITED), "the edit", "the commit holds the person's bytes, which Git prints without the final newline");
    assert_eq!(git_text(repo.as_path(), &["log", "--format=%s", "-1", "HEAD"]).trim(), "made without the service");
    let staged: String = git_text(repo.as_path(), &["ls-files", "--stage", EDITED]);
    let committed: String = git_text(repo.as_path(), &["rev-parse", format!("{commit}:{EDITED}").as_str()]);
    assert!(staged.contains(committed.trim()), "the index holds the entry of the commit: {staged} against {committed}");
    assert!(crate::held_commit_record::list(repo.join(".git").as_path()).0.is_empty(), "the record of the hold is gone once the branch moved");
    remove(root.as_path());
}

/// What:
///  The same request made a second time lands nothing new:
///  the branch already names the commit this request makes.
/// Why:
///  This is the answer to a service that wakes and lands the same request after this command made it:
///  the commit's identity is a function of the request,
///  so the compare-and-swap of the second one finds its own commit already there.
#[test]
fn the_same_request_made_twice_is_one_commit_on_the_branch() {
    let (root, repo, context) = repository("land-without-service-twice");
    let base: String = head_of(repo.as_path());
    let request: CommitRequest = a_request_writing(repo.as_path(), EDITED, b"the edit\n", base.as_str(), "made twice");
    let first: WithoutTheService = land_after_the_service_stopped(&context, &request, Some(root.join("journal").as_path()));
    let WithoutTheService::Landed { commit, .. } = first else {
        panic!("the commit is made here: {first:?}");
    };
    let second: WithoutTheService = land_after_the_service_stopped(&context, &request, Some(root.join("journal").as_path()));
    let WithoutTheService::Landed { commit: again, .. } = second else {
        panic!("the same request made again says the commit is there: {second:?}");
    };
    assert_eq!(again, commit, "the same request makes the same commit");
    assert_eq!(commits_in(repo.as_path()), 2, "the branch holds one commit more than it began with");
    remove(root.as_path());
}

/// What:
///  A commit made here on a branch another session has moved is carried onto that branch,
///  with its change and without losing the other session's work.
/// Why:
///  The batch was prepared on a value the branch no longer names,
///  so the same rule a commit that lands on a moved branch follows applies here,
///  and nothing of either session is discarded.
#[test]
fn a_commit_on_a_branch_that_moved_is_carried_onto_it() {
    let (root, repo, context) = repository("land-without-service-moved-branch");
    let base: String = head_of(repo.as_path());
    let request: CommitRequest = a_request_writing(repo.as_path(), EDITED, b"the edit\n", base.as_str(), "carried onto a moved branch");
    // Another session lands a commit of its own while this command is not looking.
    std::fs::write(repo.join("theirs.txt"), b"their work\n").expect("their file");
    git(repo.as_path(), &["add", "--", "theirs.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=another session"]);
    let theirs: String = head_of(repo.as_path());
    let outcome: WithoutTheService = land_after_the_service_stopped(&context, &request, Some(root.join("journal").as_path()));
    let WithoutTheService::Landed { commit, .. } = outcome else {
        panic!("the change is carried onto the branch as it is now: {outcome:?}");
    };
    assert_eq!(git_text(repo.as_path(), &["rev-parse", format!("{commit}^").as_str()]).trim(), theirs, "the commit sits on the other session's commit");
    assert_eq!(recorded_bytes(repo.as_path(), commit.as_str(), EDITED), "the edit");
    assert_eq!(recorded_bytes(repo.as_path(), commit.as_str(), "theirs.txt"), "their work", "the other session's work is not lost");
    remove(root.as_path());
}

/// What:
///  A reference lock that a live Git may hold leaves the commit written and kept,
///  with the record that names it,
///  and the command says where it is.
/// Why:
///  A commit that cannot be put on the branch must never be discarded:
///  the record is what the next command settles,
///  and the words name the commit and the lock.
#[test]
fn a_reference_lock_that_stands_keeps_the_commit_and_says_where_it_is() {
    let (root, repo, context) = repository("land-without-service-held");
    let base: String = head_of(repo.as_path());
    let request: CommitRequest = a_request_writing(repo.as_path(), EDITED, b"the edit\n", base.as_str(), "kept behind a lock");
    let lock: PathBuf = repo.join(".git").join("refs").join("heads").join("main.lock");
    std::fs::write(lock.as_path(), b"").expect("a lock that may be a live Git's");
    let outcome: WithoutTheService = land_after_the_service_stopped(&context, &request, Some(root.join("journal").as_path()));
    let WithoutTheService::Held { commit, words } = outcome else {
        panic!("a standing lock keeps the commit: {outcome:?}");
    };
    assert_eq!(head_of(repo.as_path()), base, "the branch has not moved while the lock stands");
    let records = crate::held_commit_record::list(repo.join(".git").as_path()).0;
    assert_eq!(records.len(), 1, "the record of the hold is there for the next command");
    assert_eq!(records[0].new, commit);
    assert!(words.contains(commit.get(..7).unwrap_or(commit.as_str())) && words.contains("main.lock"), "{words}");
    assert!(words.contains("has not moved yet") && words.contains("is kept at"), "{words}");
    assert!(records[0].locks.iter().any(|lock| lock.ends_with("main.lock")), "the record names the lock that stands: {:?}", records[0].locks);
    remove(root.as_path());
}

/// What:
///  A commit with no folder to record the batch in is not made,
///  and the person is told that nothing changed.
/// Why:
///  A batch recorded nowhere is one nothing settles,
///  and the words must not claim a commit that no record and no branch names.
#[test]
fn a_commit_with_no_journal_to_record_it_is_not_made_and_says_so() {
    let (root, repo, context) = repository("land-without-service-no-journal");
    let base: String = head_of(repo.as_path());
    let request: CommitRequest = a_request_writing(repo.as_path(), EDITED, b"the edit\n", base.as_str(), "no journal");
    let outcome: WithoutTheService = land_after_the_service_stopped(&context, &request, None);
    let WithoutTheService::NotMade { commit, words } = outcome else {
        panic!("no record can be written, so no commit is made: {outcome:?}");
    };
    assert_eq!(commit, None);
    assert!(words.contains("was not made") && words.contains("as they were"), "{words}");
    assert_eq!(head_of(repo.as_path()), base, "nothing moved");
    remove(root.as_path());
}
