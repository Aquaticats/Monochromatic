//! Controls for the look for the commit a request that was already sent made:
//! a finished batch names it,
//! an open batch names it and the branch says whether it is made,
//! and a look that finds nothing waits no longer than a look that has nothing to wait for.

use super::*;
use crate::service_journal::{DONE_EXTENSION, batch_path, intent_text, write_intent};
use crate::service_units_of_removed_worktrees::JOURNAL_FOLDER;
use crate::test_support::{git, git_text, remove, repository};

/// The identity of the request the controls send.
const ID: &str = "0d1e2f3a-4b5c-6d7e-8f90-a1b2c3d4e5f6";

/// A journal folder of this control's own.
fn a_journal(label: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture(label);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    return journal;
}

/// A record of a batch that named `ids` and the `commits` they became.
fn a_record(seq: u64, reference: &str, old: &str, new: &str, ids: &[&str], commits: &[&str]) -> Intent {
    return Intent {
        seq,
        repository: PathBuf::from("/nowhere/repo"),
        common_dir: PathBuf::from("/nowhere/repo/.git"),
        reference: String::from(reference),
        old: String::from(old),
        new: String::from(new),
        ids: ids.iter().map(|id| { return String::from(*id); }).collect(),
        commits: commits.iter().map(|commit| { return String::from(*commit); }).collect(),
        paths: Vec::new(),
        derived: true,
        cleanup: None,
        index: PathBuf::from("/nowhere/repo/.git/index"),
        git_dir: PathBuf::from("/nowhere/repo/.git"),
    };
}

/// What:
///  A finished batch names the commit its request became,
/// so a lost answer is the commit that was made.
/// Why:
///   The record is written after the branch moved,
/// so the commit it names is a fact and no branch read is needed.
#[test]
fn a_finished_batch_names_the_commit_the_request_became() {
    let journal: PathBuf = a_journal("looking-finished");
    let commit: String = "1".repeat(40);
    let record: Intent = a_record(7, "refs/heads/main", &"2".repeat(40), commit.as_str(), &[ID], &[commit.as_str()]);
    std::fs::write(batch_path(journal.as_path(), 7, DONE_EXTENSION), intent_text(&record)).expect("the finished record");
    let context: GitContext = crate::test_support::git_context();
    let looked: Option<String> = looking_for_what_landed(LookForWhatLanded {
        context: &context,
        cwd: journal.as_path(),
        journal: Some(journal.as_path()),
        id: ID,
        wait: Duration::from_millis(200),
    });
    remove(journal.parent().expect("the control's root"));
    assert_eq!(looked, Some(commit), "the record names the commit this request became");
}

/// What:
///  An open batch that names the request is the landed commit once the branch holds the commit it is making.
/// Why:
///   The service was killed after the branch moved and before its record was finished,
/// which is the route of the audit of service restarts
/// (`f3 ... after-ref`):
/// the commit is on the branch,
/// and only the open record says which commit it is.
#[test]
fn an_open_batch_names_the_commit_the_branch_holds() {
    let root: PathBuf = crate::test_support::fixture("looking-open-on-the-branch");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let before: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=landed"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    let record: Intent = a_record(9, "refs/heads/main", before.as_str(), landed.as_str(), &[ID], &[landed.as_str()]);
    write_intent(journal.as_path(), &record).expect("the open record");
    let context: GitContext = crate::test_support::git_context();
    let looked: Option<String> = looking_for_what_landed(LookForWhatLanded {
        context: &context,
        cwd: repo.as_path(),
        journal: Some(journal.as_path()),
        id: ID,
        wait: Duration::from_millis(200),
    });
    remove(root.as_path());
    assert_eq!(looked, Some(landed), "the branch holds the commit the open batch is making");
}

/// What:
///  An open batch whose branch has not moved yet is looked at until it moves,
/// within the bound.
/// Why:
///   The landing writes the record before it moves the branch,
/// so a look that reads once would say the commit is not there a moment before it is.
/// The audit saw the commit land seconds after the client gave up.
#[test]
fn an_open_batch_is_looked_at_until_its_branch_moves() {
    let root: PathBuf = crate::test_support::fixture("looking-open-later");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let before: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=landed"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    // The branch is put back, so the commit exists and the branch does not hold it yet.
    git(repo.as_path(), &["reset", "--quiet", "--hard", before.as_str()]);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    let record: Intent = a_record(11, "refs/heads/main", before.as_str(), landed.as_str(), &[ID], &[landed.as_str()]);
    write_intent(journal.as_path(), &record).expect("the open record");
    let moving_repo: PathBuf = repo.clone();
    let moving_commit: String = landed.clone();
    let moving = std::thread::spawn(move || {
        std::thread::sleep(LOOK_EVERY * 2);
        git(moving_repo.as_path(), &["update-ref", "refs/heads/main", moving_commit.as_str()]);
    });
    let context: GitContext = crate::test_support::git_context();
    let looked: Option<String> = looking_for_what_landed(LookForWhatLanded {
        context: &context,
        cwd: repo.as_path(),
        journal: Some(journal.as_path()),
        id: ID,
        wait: Duration::from_secs(20),
    });
    moving.join().expect("the branch moves");
    remove(root.as_path());
    assert_eq!(looked, Some(landed), "the look waits for the branch the open batch is about to move");
}

/// What:
///  A look with nothing to look at says nothing,
/// and does not wait for the bound.
/// Why:
///   A request that no batch names is one nothing is landing,
/// so waiting cannot turn the answer into a commit;
/// the words the failure already had are the honest ones,
/// and the command must not be held for them.
#[test]
fn a_look_that_finds_no_batch_does_not_wait() {
    let journal: PathBuf = a_journal("looking-nothing");
    let context: GitContext = crate::test_support::git_context();
    let began: Instant = Instant::now();
    let looked: Option<String> = looking_for_what_landed(LookForWhatLanded {
        context: &context,
        cwd: journal.as_path(),
        journal: Some(journal.as_path()),
        id: ID,
        wait: Duration::from_secs(10),
    });
    let waited: Duration = began.elapsed();
    remove(journal.parent().expect("the control's root"));
    assert_eq!(looked, None, "no batch names this request");
    assert!(waited < Duration::from_secs(2), "a look with nothing to wait for returned after {waited:?}");
}

/// What:
///  A look with no journal folder says nothing and runs no Git.
/// Why:
///   A command whose environment names no state folder,
/// or whose Git directory names no instance,
/// has no journal to read,
/// and the look must not guess one.
#[test]
fn a_look_with_no_journal_says_nothing() {
    let context: GitContext = crate::test_support::git_context();
    let looked: Option<String> = looking_for_what_landed(LookForWhatLanded {
        context: &context,
        cwd: std::path::Path::new("/nowhere"),
        journal: None,
        id: ID,
        wait: Duration::from_millis(200),
    });
    assert_eq!(looked, None, "there is no journal to look in");
}

/// What:
///  The journal folder of a request is the one the unit gives the worktree's service.
/// Why:
///   The client that wrote the unit reads the same folder back,
/// so the two spellings of the path must be one:
/// `<state>/monochromatic-git-policy-cli/service/<instance>`.
#[test]
fn the_journal_of_a_request_is_the_folder_the_unit_gives_it() {
    let root: PathBuf = crate::test_support::fixture("looking-journal-path");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut request: CommitRequest = a_request_of(&repo);
    request.git_dir = repo.join(".git");
    let journal: Option<PathBuf> = journal_of_request(&request);
    let instance: String = crate::service_instance::instance_of(repo.join(".git").as_path());
    remove(root.as_path());
    let found: PathBuf = journal.expect("this process's environment names a state folder");
    assert_eq!(found.file_name().and_then(std::ffi::OsStr::to_str), Some(instance.as_str()), "the folder is the instance's");
    assert_eq!(found.parent().and_then(std::path::Path::file_name), Some(std::ffi::OsStr::new(JOURNAL_FOLDER)), "the folder sits in the journal folder");
}

/// A request of this control's own,
/// whose Git directory the caller sets.
fn a_request_of(repo: &Path) -> CommitRequest {
    return CommitRequest {
        id: String::from(ID),
        repository: repo.to_path_buf(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        reference: String::from("refs/heads/main"),
        base: "0".repeat(40),
        changes: Vec::new(),
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: repo.join(".git/index"),
        scratch: repo.join(".git/scratch"),
        captured: 0,
    };
}

/// What:
///  The record of a commit the open batch of this request left held is found,
///  and a record of another batch or another journal is not.
/// Why:
///  A service that answered `held` into a connection that broke left the commit made and kept,
///  and only the record of the hold says so:
///  a command that looked only at the branch would report a commit that was made as a failure.
#[test]
fn the_held_record_of_the_open_batch_of_a_request_is_found() {
    let journal: PathBuf = a_journal("looking-held");
    let git_dir: PathBuf = journal.parent().expect("the control's root").join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the worktree's Git directory");
    let held_commit: String = "3".repeat(40);
    let record: Intent = a_record(9, "refs/heads/main", &"4".repeat(40), held_commit.as_str(), &[ID], &[held_commit.as_str()]);
    write_intent(journal.as_path(), &record).expect("the open record");
    let held: crate::held_commit_record::Held = crate::held_commit_record::Held {
        seq: 9,
        journal: journal.clone(),
        repository: git_dir.parent().expect("the worktree").to_path_buf(),
        common_dir: git_dir.clone(),
        git_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: record.old.clone(),
        new: held_commit.clone(),
        commits: vec![held_commit.clone()],
        index: git_dir.join("index"),
        reflog: b"commit: held".to_vec(),
        locks: vec![git_dir.join("refs/heads/main.lock")],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    crate::held_commit_record::write(git_dir.as_path(), &held).expect("the record of the hold");
    assert_eq!(a_held_batch_of(journal.as_path(), git_dir.as_path(), ID).map(|found| { return found.new; }), Some(held_commit), "the record of this batch is the one found");
    let elsewhere: PathBuf = journal.parent().expect("the control's root").join("other-journal");
    std::fs::create_dir_all(elsewhere.as_path()).expect("another journal");
    assert_eq!(a_held_batch_of(elsewhere.as_path(), git_dir.as_path(), ID), None, "a record of another journal is not this request's");
    assert_eq!(a_held_batch_of(journal.as_path(), git_dir.as_path(), "another-request"), None, "a record of another request is not this one's");
    remove(journal.parent().expect("the control's root"));
}
