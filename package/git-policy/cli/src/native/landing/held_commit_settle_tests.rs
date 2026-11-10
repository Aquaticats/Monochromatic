//! What:
//!  Controls for moving the branch of a held commit once nothing stands in the way,
//!  with real Git in disposable repositories:
//!  the lock that still stands,
//!  the branch that did not move,
//!  and the branch that another session moved while the lock stood.
//! Why:
//!  These are the three states the settlement decides between,
//!  and each one is a promise about a person's commit:
//!  it is kept and named,
//!  it lands exactly once with its own bytes,
//!  and it lands on the branch as it is now without losing the other session's work.
//!  The decisions read real references,
//!  real objects and the real index,
//!  so no stand-in can show them.

use super::{Clearing, Settled, all, kept_reference, one};
use crate::held_commit_record::{Held, Holding, hold, path_of};
use crate::service_journal::{Intent, OPEN_EXTENSION, batch_path, write_intent, write_start};
use crate::service_land_start::witness_of;
use crate::test_support::{fixture, git, git_text, remove};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The file the held commit changes.
const EDITED: &str = "edited.txt";

/// The file another session changes while the lock stands.
const OTHER: &str = "other.txt";

/// The subject of the commit a held batch lands.
const HELD_SUBJECT: &str = "the held edit";

/// The subject of the commit another session lands while the lock stands.
const OTHER_SUBJECT: &str = "another session";

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

/// The subjects of the newest commits,
/// newest first.
fn subjects(repo: &Path, count: usize) -> String {
    let format: String = format!("--format=%s");
    let max: String = format!("--max-count={count}");
    return git_text(repo, &["log", format.as_str(), max.as_str(), "HEAD"]);
}

/// What:
///  A commit made on the branch and taken back off it,
///  kept on the side reference the landing names,
///  with a journal and a held record exactly as a landing that met a lock leaves them.
/// Why:
///   The state a settlement reads is what a killed or refused landing writes,
///  and this control makes it the same way:
///  the objects exist,
///  the branch is at the base,
///  the side reference holds the commit,
///  and the intent and the record name the two branch values.
fn held_batch(root: &Path, repo: &Path, context: &GitContext) -> Held {
    let base: String = head_of(repo);
    std::fs::write(repo.join(EDITED), b"the edit\n").expect("the edit");
    git(repo, &["add", "--", EDITED]);
    git(repo, &["commit", "--quiet", "--message", HELD_SUBJECT]);
    let held_commit: String = head_of(repo);
    // The branch goes back to the base, so the commit is only on the side reference, as a refused branch write leaves it.
    git(repo, &["update-ref", "--no-deref", kept_reference(1).as_str(), held_commit.as_str()]);
    git(repo, &["reset", "--hard", "--quiet", base.as_str()]);
    // The person's worktree still holds the bytes the commit captured: the commit took them and the branch has not moved.
    std::fs::write(repo.join(EDITED), b"the edit\n").expect("the edit stays in the worktree");
    let git_dir: PathBuf = repo.join(".git");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let intent: Intent = Intent {
        seq: 1,
        repository: repo.to_path_buf(),
        common_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: base.clone(),
        new: held_commit.clone(),
        ids: vec![String::from("request-1")],
        commits: vec![held_commit.clone()],
        paths: vec![EDITED.as_bytes().to_vec()],
        derived: false,
        cleanup: None,
        index: git_dir.join("index"),
        git_dir: git_dir.clone(),
    };
    write_intent(journal.as_path(), &intent).expect("the intent");
    // The landing records the index file it saw under its lock before it writes the branch;
    // a settlement replays the index update only over that file.
    write_start(journal.as_path(), 1, &witness_of(intent.index.as_path()).expect("the index file")).expect("the witness");
    let holding: Holding<'_> = Holding {
        journal: journal.as_path(),
        repository: repo,
        common_dir: git_dir.as_path(),
        git_dir: git_dir.as_path(),
        index: intent.index.as_path(),
        reference: intent.reference.as_str(),
        seq: 1,
        intent: &intent,
        locks: vec![git_dir.join("refs/heads/main.lock")],
        reflog: b"commit: the held edit".as_slice(),
    };
    let held: Held = hold(holding).expect("the held record");
    let _context = context;
    return held;
}

/// What:
///  A lock that stands keeps the branch exactly where it is,
///  and is named,
///  while the record and the commit stay.
/// Why:
///  Nothing proves that a Git process has ended,
///  so the lock is never removed and the commit waits for the person or for a proof.
#[test]
fn a_standing_lock_keeps_the_branch_without_losing_the_commit() {
    let (root, repo, context) = repository("held-settle-lock");
    let held: Held = held_batch(root.as_path(), repo.as_path(), &context);
    let base: String = head_of(repo.as_path());
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    std::fs::write(lock.as_path(), "").expect("the lock a killed Git leaves");
    let settled: Settled = one(&context, &held, Clearing::Never);
    assert_eq!(settled, Settled::WaitsForLocks(vec![lock.clone()]));
    assert_eq!(head_of(repo.as_path()), base, "the branch has not moved");
    assert!(lock.exists(), "a lock nothing proves gone is left where it is");
    assert!(path_of(held.git_dir.as_path(), 1).exists(), "the record stays while the lock does");
    let kept: String = git_text(repo.as_path(), &["rev-parse", kept_reference(1).as_str()]).trim().to_string();
    assert_eq!(kept, held.new, "the commit is kept on the side reference");
    remove(root.as_path());
}

/// What:
///  The lock gone,
///  the settlement moves the branch to the commit the batch made,
///  installs the index entries,
///  and takes the record and the side reference away.
/// Why:
///  This is the whole promise of the owner's "hold until lock goes":
///  the commit lands once,
///  with the bytes it captured,
///  and nothing of the settlement is left behind.
#[test]
fn a_held_commit_lands_once_the_lock_is_gone() {
    let (root, repo, context) = repository("held-settle-lands");
    let held: Held = held_batch(root.as_path(), repo.as_path(), &context);
    let settled: Vec<(Held, Settled)> = all(&context, held.git_dir.as_path(), Clearing::Never).0;
    assert_eq!(settled.len(), 1);
    assert_eq!(settled[0].1, Settled::Landed { commit: held.new.clone() });
    assert_eq!(head_of(repo.as_path()), held.new, "the branch names the commit the batch made");
    let shown: String = git_text(repo.as_path(), &["show", format!("HEAD:{EDITED}").as_str()]);
    assert_eq!(shown, "the edit", "the bytes it captured are what landed");
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1"]), "", "the index caught up");
    assert!(!path_of(held.git_dir.as_path(), 1).exists(), "the record goes with the settlement");
    let side: String = git_text(repo.as_path(), &["for-each-ref", kept_reference(1).as_str()]);
    assert_eq!(side, "", "the side reference goes with the branch move");
    assert!(!batch_path(held.journal.as_path(), 1, OPEN_EXTENSION).exists(), "the intent is marked done");
    remove(root.as_path());
}

/// What:
///  A branch another session moved while the lock stood takes the held commit on top of it,
///  and both changes are there.
/// Why:
///  No client is left to prepare the commit again,
///  so the change is merged onto the branch as it is now,
///  which is the rule a commit that lands on a moved branch follows
///  (`doc/decision/cli-git-concurrent-commits.md`).
#[test]
fn a_held_commit_lands_on_the_branch_another_session_moved() {
    let (root, repo, context) = repository("held-settle-moved");
    let held: Held = held_batch(root.as_path(), repo.as_path(), &context);
    std::fs::write(repo.join(OTHER), b"the other edit\n").expect("the other session's file");
    git(repo.as_path(), &["add", "--", OTHER]);
    git(repo.as_path(), &["commit", "--quiet", "--message", OTHER_SUBJECT]);
    let settled: Vec<(Held, Settled)> = all(&context, held.git_dir.as_path(), Clearing::Never).0;
    assert!(matches!(settled[0].1, Settled::Landed { .. }), "the held commit landed: {:?}", settled[0].1);
    assert_eq!(subjects(repo.as_path(), 3), format!("{HELD_SUBJECT}\n{OTHER_SUBJECT}\nbase"), "it landed on top of the other session's commit");
    assert_eq!(git_text(repo.as_path(), &["show", format!("HEAD:{EDITED}").as_str()]), "the edit");
    assert_eq!(git_text(repo.as_path(), &["show", format!("HEAD:{OTHER}").as_str()]), "the other edit", "the other session's work is kept");
    // The other session's commit rewrote the index after the landing saw it, so its entries are left as that program made them and
    //  the next commit in the worktree sets them (`service_land_finish.rs`, `Gate::Leave`): nothing is staged over another's work.
    assert!(matches!(settled[0].1, Settled::Landed { .. }), "the branch landed and what it owed is recorded: {:?}", settled[0].1);
    remove(root.as_path());
}

/// What:
///  A held commit whose change cannot be combined with the moved branch keeps its record and names the paths.
/// Why:
///  Nothing decides a conflict for the person,
///  and the commit must not be lost over one.
#[test]
fn a_held_commit_that_conflicts_keeps_its_record_and_names_the_paths() {
    let (root, repo, context) = repository("held-settle-conflict");
    let held: Held = held_batch(root.as_path(), repo.as_path(), &context);
    // The other session rewrites the line the held commit wrote, from the same base.
    std::fs::write(repo.join(EDITED), b"a different edit\n").expect("the other session's edit");
    git(repo.as_path(), &["add", "--", EDITED]);
    git(repo.as_path(), &["commit", "--quiet", "--message", OTHER_SUBJECT]);
    let settled: Settled = one(&context, &held, Clearing::Never);
    match settled {
        Settled::Conflicts { paths, .. } => assert_eq!(paths, vec![EDITED.as_bytes().to_vec()]),
        other => panic!("a change that does not combine is a conflict, and this was {other:?}"),
    }
    assert!(path_of(held.git_dir.as_path(), 1).exists(), "the record stays, so the commit is found again");
    assert_eq!(subjects(repo.as_path(), 2), format!("{OTHER_SUBJECT}\nbase"), "the branch is left as the other session made it");
    remove(root.as_path());
}
