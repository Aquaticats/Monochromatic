//! What:
//!  Controls for a restart that finds an open batch whose commit a reference lock held off the branch,
//!  while the branch another session made has moved:
//!  the settlement replays the commit,
//!  rewrites the batch's intent to name the commit it put on the branch,
//!  and leaves what the batch still owes.
//! Why:
//!  The restart decides what to do with an open batch from the branch,
//!  and the branch it reads says nothing about a commit the settlement replayed a moment before:
//!  reading the intent it parsed *before* the settlement therefore decides about a commit the
//!  branch does not name,
//!  drops the batch,
//!  and with it deletes the owed index repair of a commit that is on the branch
//!  (a Kimi K3.1 Pro audit of every wait since `0.6.1-alpha.8`,
//! its second finding).
//!  The owner's rule is "never discard user data",
//!  so the control asks for the two records by name:
//!  the repaid commit's intent and the record of the index update it still owes.

use crate::held_commit_record::{Held, Holding, hold};
use crate::owed_index_update;
use crate::service_fault::hooks;
use crate::service_journal::{Intent, OPEN_EXTENSION, batch_path, parse_intent, write_intent, write_start};
use crate::service_land::{LandContext, recover};
use crate::service_land_finish::RecoveryReport;
use crate::service_land_start::witness_of;
use crate::test_support::{fixture, git, git_text, remove, repository};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The file the held commit changes.
const THE_HELD_FILE: &str = "held.txt";

/// The subject of the commit a held batch lands.
const THE_HELD_SUBJECT: &str = "the held edit";

/// The file another session changes while the reference lock stood.
const THE_OTHER_FILE: &str = "other.txt";

/// The subject of the commit another session lands while the lock stood.
const THE_OTHER_SUBJECT: &str = "another session";

/// The batch number every record of this control belongs to.
const SEQ: u64 = 1;

/// A repository of this control's own,
/// with one commit,
/// and the journal its batches are recorded in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const world = { root, repo, journal, context };
/// ```
struct World {
    /// This control's fixture directory.
    root: PathBuf,
    /// The worktree the batch lands in.
    repo: PathBuf,
    /// The context the recovery runs in.
    context: LandContext,
}

impl Drop for World {
    fn drop(&mut self) {
        remove(self.root.as_path());
    }
}

/// The repository,
/// journal and recovery context of one control.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal");
    let git_dir: PathBuf = repo.join(".git");
    let context: LandContext = LandContext {
        git: GitContext { common_dir: Some(git_dir), ..crate::test_support::git_context() },
        journal,
        on_objects_written: None,
        lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
    };
    return World { root, repo, context };
}

/// The commit the branch names.
fn head_of(repo: &Path) -> String {
    return git_text(repo, &["rev-parse", "HEAD"]).trim().to_owned();
}

/// The subjects of the newest commits on the branch,
/// newest first.
fn subjects(repo: &Path) -> String {
    return git_text(repo, &["log", "--format=%s", "HEAD"]);
}

/// What:
///  A commit made on the branch and taken back off it,
///  with the batch's open intent,
///  the index witness its landing recorded,
///  the record of the index update it owes,
///  and the held record a refused reference lock leaves.
/// Why:
///  This is the state a restart reads,
/// and this control makes it the way a landing that met a lock makes it,
///  so no stand-in stands for the record the settlement rewrites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldBatch(world); owedIndex.write(ofIntent(intent));
/// ```
fn held_batch(world: &World) -> Held {
    let repo: PathBuf = world.repo.clone();
    let git_dir: PathBuf = repo.join(".git");
    let base: String = head_of(repo.as_path());
    std::fs::write(repo.join(THE_HELD_FILE), b"the held edit\n").expect("the held edit");
    git(repo.as_path(), &["add", "--", THE_HELD_FILE]);
    git(repo.as_path(), &["commit", "--quiet", "--message", THE_HELD_SUBJECT]);
    let held_commit: String = head_of(repo.as_path());
    // The branch write was refused, so the commit is only on the batch's own side reference and the branch stayed where it was.
    git(repo.as_path(), &["update-ref", "--no-deref", crate::held_commit_settle::kept_reference(SEQ).as_str(), held_commit.as_str()]);
    git(repo.as_path(), &["reset", "--hard", "--quiet", base.as_str()]);
    // The person's worktree still holds the bytes the commit captured.
    std::fs::write(repo.join(THE_HELD_FILE), b"the held edit\n").expect("the edit stays in the worktree");
    let intent: Intent = Intent {
        seq: SEQ,
        repository: repo.clone(),
        common_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: base,
        new: held_commit.clone(),
        ids: vec![String::from("request-1")],
        commits: vec![held_commit],
        paths: vec![THE_HELD_FILE.as_bytes().to_vec()],
        derived: false,
        cleanup: None,
        index: git_dir.join("index"),
        git_dir: git_dir.clone(),
    };
    write_intent(world.context.journal.as_path(), &intent).expect("the intent");
    write_start(world.context.journal.as_path(), SEQ, &witness_of(intent.index.as_path()).expect("the index file")).expect("the witness");
    // The landing writes the record of the index update it will owe before it moves the branch, so it is here whether or not the
    //  branch write was refused.
    owed_index_update::write(&owed_index_update::of_intent(&intent));
    let holding: Holding<'_> = Holding {
        journal: world.context.journal.as_path(),
        repository: repo.as_path(),
        common_dir: git_dir.as_path(),
        git_dir: git_dir.as_path(),
        index: intent.index.as_path(),
        reference: intent.reference.as_str(),
        seq: SEQ,
        intent: &intent,
        locks: vec![git_dir.join("refs/heads/main.lock")],
        reflog: b"commit: the held edit".as_slice(),
    };
    return hold(holding).expect("the held record");
}

/// The intent the journal holds open for batch `seq`.
fn open_intent(world: &World) -> Option<Intent> {
    let text: String = std::fs::read_to_string(batch_path(world.context.journal.as_path(), SEQ, OPEN_EXTENSION)).ok()?;
    return parse_intent(text.as_str());
}

/// What:
///  A restart whose settlement replays a held commit onto a branch another session moved keeps the
///  intent that now names the replayed commit and the record of the index update the batch still owes,
///  and the next restart finishes them.
/// Why:
///  The commit is on the branch,
///  so its index update is owed,
///  and deleting the record of it makes the next commit record the landed change's paths in reverse
///  (`owed_index_update.rs`,
/// its first paragraphs).
///  The settlement cannot install the index here (the same step a second program holding `index.lock`
///  leaves unfinished),
///  so what the record is for is exactly the state this control makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reports = recover(world.context); expect(owedRecord).toBeExisting(); expect(openIntent().new).toBe(replayed); hooks.clearFailures(); recover(world.context); expect(status()).toEqual('');
/// ```
#[test]
fn a_restart_that_replayed_a_held_commit_keeps_the_index_update_it_owes() {
    let world: World = world("land-finish-held-replayed");
    let held: Held = held_batch(&world);
    // The branch moved while the reference lock stood, and the replayed commit has to go on top of it.
    std::fs::write(world.repo.join(THE_OTHER_FILE), b"the other session's file\n").expect("the other session's file");
    git(world.repo.as_path(), &["add", "--", THE_OTHER_FILE]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message", THE_OTHER_SUBJECT, "--", THE_OTHER_FILE]);
    let moved: String = head_of(world.repo.as_path());
    // The index install is the step another program holding `index.lock` leaves unfinished, and Git cannot be made to fail
    //  there on demand, so a control names the point (`service_fault.rs`).
    hooks::fail_at("build-index");
    let reports: Vec<RecoveryReport> = recover(&world.context);
    hooks::clear_failures();
    let owed: PathBuf = owed_index_update::path_of(world.repo.join(".git").as_path(), SEQ);
    let fresh: Option<Intent> = open_intent(&world);
    let said: String = format!("{reports:?}");
    assert!(owed.exists(), "the index update the landed commit owes is kept: {} ({said})", owed.display());
    let named: Option<String> = fresh.as_ref().map(|intent| { return intent.new.clone(); });
    assert!(named.is_some(), "the batch's intent stays open and names the commit the branch took: {said}");
    assert_ne!(named.as_deref(), Some(held.new.as_str()), "the intent names the commit the settlement replayed, not the one it started from");
    assert_eq!(subjects(world.repo.as_path()), format!("{THE_HELD_SUBJECT}\n{THE_OTHER_SUBJECT}\ninitial"), "the held commit landed on the branch as it moved");
    assert_ne!(head_of(world.repo.as_path()), moved, "the branch names the replayed commit");
    // The next restart finds the batch where the settlement left it, finishes the index update it owes, and marks it done.
    let afterwards: Vec<RecoveryReport> = recover(&world.context);
    assert!(!owed.exists(), "the owed record goes once its update is installed: {afterwards:?}");
    assert!(open_intent(&world).is_none(), "the intent is marked done once nothing is owed: {afterwards:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--porcelain=v1"]), "", "the index shows the landing");
}
