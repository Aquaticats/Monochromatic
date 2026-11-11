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
        holders: vec![String::new()],
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

/// The commit the branch names,
/// in a repository whose Git directory is not `<work_tree>/.git`.
fn head_beside(git_dir: &Path, work_tree: &Path) -> String {
    let git_dir_argument: String = format!("--git-dir={}", git_dir.display());
    let work_tree_argument: String = format!("--work-tree={}", work_tree.display());
    return git_text(
        work_tree,
        &[git_dir_argument.as_str(), work_tree_argument.as_str(), "rev-parse", "HEAD"],
    )
    .trim()
    .to_string();
}

/// What:
///  A repository whose Git directory lies beside its work tree,
///  with one commit,
///  and the two paths a held record names.
/// Why:
///  A commit honours `--git-dir` and `--work-tree`,
///  and a record written for such a repository names both.
///  Git finds no repository from the work tree at all in this layout,
///  which is what shows that a settlement names the record's repository instead of hoping the
///  directory finds it (`repository_selection.rs`,
/// `of_paths`).
fn repository_beside_its_work_tree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git");
    let work_tree: PathBuf = root.join("work");
    std::fs::create_dir_all(work_tree.as_path()).expect("the work tree");
    let git_dir_text: String = git_dir.display().to_string();
    git(root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", git_dir_text.as_str()]);
    git(git_dir.as_path(), &["config", "core.bare", "false"]);
    let git_dir_argument: String = format!("--git-dir={git_dir_text}");
    let work_tree_argument: String = format!("--work-tree={}", work_tree.display());
    std::fs::write(work_tree.join("base.txt"), b"base\n").expect("the base file");
    git(root.as_path(), &[git_dir_argument.as_str(), work_tree_argument.as_str(), "add", "--", "base.txt"]);
    git(root.as_path(), &[git_dir_argument.as_str(), work_tree_argument.as_str(), "commit", "--quiet", "--message=base"]);
    return (root, git_dir, work_tree);
}

/// What:
///  A held commit lands in a repository whose Git directory is not `<work_tree>/.git`,
///  and so does one whose branch moved.
/// Why:
///  The record names the repository its batch was prepared in,
///  and a settlement that let the current directory find the repository would act on nothing here.
///  The context it is given carries no selection of its own,
///  as a plain `git status` in that work tree carries none.
#[test]
fn a_held_commit_lands_in_a_repository_whose_git_directory_is_beside_it() {
    let (root, git_dir, work_tree) = repository_beside_its_work_tree("held-settle-beside");
    let git_dir_argument: String = format!("--git-dir={}", git_dir.display());
    let work_tree_argument: String = format!("--work-tree={}", work_tree.display());
    let beside = |arguments: &[&str]| {
        let mut words: Vec<String> = vec![git_dir_argument.clone(), work_tree_argument.clone()];
        words.extend(arguments.iter().map(|word| { return (*word).to_string(); }));
        let borrowed: Vec<&str> = words.iter().map(String::as_str).collect();
        return git_text(root.as_path(), borrowed.as_slice());
    };
    let base: String = head_beside(git_dir.as_path(), work_tree.as_path());
    std::fs::write(work_tree.join(EDITED), b"the edit\n").expect("the edit");
    beside(&["add", "--", EDITED]);
    beside(&["commit", "--quiet", "--message", HELD_SUBJECT]);
    let held_commit: String = head_beside(git_dir.as_path(), work_tree.as_path());
    beside(&["update-ref", "--no-deref", kept_reference(1).as_str(), held_commit.as_str()]);
    beside(&["reset", "--hard", "--quiet", base.as_str()]);
    std::fs::write(work_tree.join(EDITED), b"the edit\n").expect("the edit stays in the work tree");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let intent: Intent = Intent {
        seq: 1,
        repository: work_tree.clone(),
        common_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: base.clone(),
        new: held_commit.clone(),
        ids: vec![String::from("request-1")],
        commits: vec![held_commit.clone()],
        holders: vec![String::new()],
        paths: vec![EDITED.as_bytes().to_vec()],
        derived: false,
        cleanup: None,
        index: git_dir.join("index"),
        git_dir: git_dir.clone(),
    };
    write_intent(journal.as_path(), &intent).expect("the intent");
    write_start(journal.as_path(), 1, &witness_of(intent.index.as_path()).expect("the index file")).expect("the witness");
    let holding: Holding<'_> = Holding {
        journal: journal.as_path(),
        repository: work_tree.as_path(),
        common_dir: git_dir.as_path(),
        git_dir: git_dir.as_path(),
        index: intent.index.as_path(),
        reference: intent.reference.as_str(),
        seq: 1,
        intent: &intent,
        locks: vec![],
        reflog: b"commit: the held edit".as_slice(),
    };
    let held: Held = hold(holding).expect("the held record");
    // No global option names the repository here, exactly as a plain command in that work tree carries none.
    let context: GitContext = GitContext { real_git: PathBuf::from("/usr/bin/git"), overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(git_dir.clone()) };
    let settled: Settled = one(&context, &held, Clearing::Never);
    assert_eq!(settled, Settled::Landed { commit: held_commit.clone() });
    assert_eq!(head_beside(git_dir.as_path(), work_tree.as_path()), held_commit, "the branch names the commit the batch made");
    let shown: String = beside(&["show", format!("HEAD:{EDITED}").as_str()]);
    assert_eq!(shown, "the edit", "the bytes it captured are what landed");
    let status: String = beside(&["status", "--porcelain=v1"]);
    assert_eq!(status, "", "the index caught up in the repository the record names");
    assert!(!path_of(held.git_dir.as_path(), 1).exists(), "the record goes with the settlement");
    remove(root.as_path());
}

/// What:
///  A second settlement of one record puts nothing on the branch,
///  when the first one landed the change in the instant between the record being read and the branch being decided from.
/// Why:
///  Every command of a worktree settles the records it finds,
///  so two commands can settle one record at once,
///  and the record is what each of them decides from.
///  A settlement that carries the change onto a moved branch rewrites the batch's record before it moves the branch
///  (`carry`),
///  so a decision taken from the value read before that rewrite puts the same change on the branch twice,
///  as two commits with the same tree
///  (a review of `aa7f601c6d`,
///  finding 3).
#[test]
fn a_second_settlement_of_one_record_lands_the_change_once() {
    /// Where the second settlement of the same record runs.
    const POINT: &str = "held-before-a-settlement-decides";
    let (root, repo, context) = repository("held-settle-two-settlers");
    let held: Held = held_batch(root.as_path(), repo.as_path(), &context);
    std::fs::write(repo.join(OTHER), b"the other edit\n").expect("the other session's file");
    git(repo.as_path(), &["add", "--", OTHER]);
    git(repo.as_path(), &["commit", "--quiet", "--message", OTHER_SUBJECT]);
    let inside: GitContext = context.clone();
    let record: Held = held.clone();
    let ran: std::rc::Rc<std::cell::Cell<bool>> = std::rc::Rc::new(std::cell::Cell::new(false));
    crate::service_fault::hooks::set(
        POINT,
        std::rc::Rc::new(move || {
            // Once: the settlement this one starts reaches the same instant, and settling that record again is what the
            //  first settlement would do on another command.
            if ran.replace(true) {
                return;
            }
            let other: Held = record.clone();
            let _settled: Settled = one(&inside, &other, Clearing::Never);
        }),
    );
    let settled: Settled = one(&context, &held, Clearing::Never);
    crate::service_fault::hooks::clear();
    assert_eq!(subjects(repo.as_path(), 3), format!("{HELD_SUBJECT}\n{OTHER_SUBJECT}\nbase"), "the change is on the branch once: {settled:?}");
    assert_eq!(git_text(repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3", "the branch carries the base, the other session's commit and the held change, and nothing else");
    assert_eq!(git_text(repo.as_path(), &["show", format!("HEAD:{EDITED}").as_str()]), "the edit", "the change landed with its bytes");
    assert!(!path_of(held.git_dir.as_path(), 1).exists(), "no settlement leaves the record behind");
    remove(root.as_path());
}

/// What:
///  A record a kill leaves after its batch landed on a branch that moved is recognized,
///  reported as there,
///  and removed,
///  instead of being said for ever to name a commit the batch never landed.
/// Why:
///  The settlement writes the commit it made in the batch's record before it moves the branch
///  (`carry`),
///  and the finished record is written after the move:
///  a kill between them leaves a record whose `new` is the commit the batch started with,
///  while the branch names the commit made from it.
///  Comparing only the first value would keep the record and its warning for ever,
///  and the side reference it tells the person about is already gone
///  (a review of `aa7f601c6d`,
///  finding 4).
#[test]
fn a_record_left_by_a_kill_after_its_replayed_landing_is_recognized_and_removed() {
    let (root, repo, context) = repository("held-settle-killed-after-finish");
    let held: Held = held_batch(root.as_path(), repo.as_path(), &context);
    std::fs::write(repo.join(OTHER), b"the other edit\n").expect("the other session's file");
    git(repo.as_path(), &["add", "--", OTHER]);
    git(repo.as_path(), &["commit", "--quiet", "--message", OTHER_SUBJECT]);
    let base_of_the_replay: String = head_of(repo.as_path());
    let replayed = crate::held_commit_replay::onto(&context, &held, base_of_the_replay.as_str(), repo.join(".git").as_path());
    let crate::held_commit_replay::Replay::Replayed { commits } = replayed else { panic!("the change combines with the branch: {replayed:?}") };
    let landed: String = commits.last().cloned().expect("the replayed commit");
    let journal: &Path = held.journal.as_path();
    let open: std::path::PathBuf = batch_path(journal, 1, OPEN_EXTENSION);
    let mut written: Intent = crate::service_journal::parse_intent(std::fs::read_to_string(open.as_path()).expect("the record the landing wrote").as_str()).expect("the record reads");
    written.new = landed.clone();
    written.commits = commits;
    write_intent(journal, &written).expect("the record names the commit made from the change");
    // The branch move, and the finished record written after it: the kill is after both, and before the record of the hold goes.
    git(repo.as_path(), &["update-ref", "--no-deref", "refs/heads/main", landed.as_str(), base_of_the_replay.as_str()]);
    git(repo.as_path(), &["update-ref", "-d", "--no-deref", kept_reference(1).as_str()]);
    crate::service_journal::mark_done(journal, 1).expect("the finished record");
    assert!(path_of(held.git_dir.as_path(), 1).exists(), "the kill left the record of the hold");
    let settled: Settled = one(&context, &held, Clearing::Never);
    assert_eq!(settled, Settled::AlreadyThere { commit: landed.clone() }, "the branch is shown to hold the commit the batch became");
    assert!(!path_of(held.git_dir.as_path(), 1).exists(), "the record goes, so no command says it waits any more");
    assert_eq!(head_of(repo.as_path()), landed, "the branch was not moved again");
    remove(root.as_path());
}
