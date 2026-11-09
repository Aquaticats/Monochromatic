//! Controls for the index update a landed batch owes:
//! the record of it in the worktree's own Git directory,
//! what recovery does with an open record that cannot be read,
//! and the repair that leaves a program's own staging alone.

use super::tests::{World, land_with_failed_index, request, world};
use super::*;
use crate::service_journal::{OPEN_EXTENSION, batch_path};
use crate::test_support::{git, git_text, remove};

/// The path of the owed record of batch `seq` in the world's worktree.
fn owed_of(world: &World, seq: u64) -> PathBuf {
    return crate::owed_index_update::path_of(world.repo.join(".git").as_path(), seq);
}

/// The file the index of the world's worktree names.
fn real_index_of(world: &World) -> PathBuf {
    return world.repo.join(".git").join("index");
}

/// The one owed record the world's worktree holds.
fn record_of(world: &World) -> crate::owed_index_update::Owed {
    let records: Vec<crate::owed_index_update::Owed> = crate::owed_index_update::records_of(world.repo.join(".git").as_path());
    assert_eq!(records.len(), 1, "one landed batch owes one index update: {records:?}");
    return records[0].clone();
}

/// Owner's rule "never discard user data":
/// an open record the journal holds that this build cannot read names the landed batch's branch values,
/// paths and index,
/// so removing it forgets the index update the landing owed and leaves the person's next commit recording the landed commit's own
///  paths against the index.
/// It is kept and named,
/// and what it owed survives in the worktree's own record of it.
#[test]
fn an_open_record_that_cannot_be_read_is_kept_and_its_owed_index_update_stays_in_the_worktree() {
    let world: World = world("service-land-unreadable-intent");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    let intent: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "D  one.txt\n?? one.txt", "the index describes the branch before the landing");
    assert!(owed_of(&world, 1).exists(), "the landing wrote what it owes in the worktree before it tried to install the index");
    assert_eq!(record_of(&world).paths, vec![String::from("one.txt")], "the record names the commit's paths");
    std::fs::write(&intent, "seq=1\nrepos").expect("the record is replaced by text this build cannot read");
    let mut reports: Vec<RecoveryReport> = Vec::new();
    let entries: Vec<crate::service_timing::capture::Captured> = crate::service_timing::capture::entries_of(|| { reports = recover(&world.context); });
    assert_eq!(reports, Vec::new(), "nothing is decided about a record that cannot be read");
    assert!(intent.exists(), "a record that cannot be read is not destroyed");
    let said: Vec<&crate::service_timing::capture::Captured> = entries.iter().filter(|entry| { return entry.priority == crate::journald_native::Priority::Error; }).collect();
    assert_eq!(said.len(), 1, "{entries:?}");
    assert!(said[0].message.contains("cannot be read") && said[0].message.contains("kept"), "{}", said[0].message);
    assert!(said[0].message.contains(intent.display().to_string().as_str()), "the record is named: {}", said[0].message);
    assert!(said[0].message.contains("batch 1"), "the batch is named: {}", said[0].message);
    assert!(owed_of(&world, 1).exists(), "what the batch owes stays where the person's next command reads it");
    remove(world.root.as_path());
}

/// The repair of an owed index update sets the entries the landing itself left to the landed commit,
/// so this commit does not record the commit's own paths against it,
/// and a path another program staged keeps that staging.
#[test]
fn the_entries_the_landing_left_are_set_to_the_landed_commit_and_a_program_staging_is_kept() {
    let world: World = world("service-land-owed-repair");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-status", "HEAD"]), "D\tone.txt", "a commit of this index would record the landed file's deletion");
    // Another program stages a path of its own after the landing.
    std::fs::write(world.repo.join("other.txt"), "other\n").expect("write");
    git(world.repo.as_path(), &["add", "other.txt"]);
    let owed: crate::owed_index_update::Owed = record_of(&world);
    let settled: Vec<Vec<u8>> = crate::owed_index_update::settle_leftovers(&world.context.git, &owed, real_index_of(&world).as_path()).expect("settled");
    assert_eq!(settled, vec![b"one.txt".to_vec()], "only the entry the landing left is set");
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-status", "HEAD"]), "A\tother.txt", "the landed commit's path is not recorded against it, and the program's staging stands");
    crate::owed_index_update::remove(world.repo.join(".git").as_path(), 1);
    remove(world.root.as_path());
}

/// A path of the landed batch that a program staged again since holds something other than what the batch started from,
/// so it is the program's own and the repair sets nothing.
#[test]
fn a_path_a_program_staged_since_the_landing_is_left_as_that_program_left_it() {
    let world: World = world("service-land-owed-program-staging");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    std::fs::write(world.repo.join("one.txt"), "mine\n").expect("write");
    git(world.repo.as_path(), &["add", "one.txt"]);
    let owed: crate::owed_index_update::Owed = record_of(&world);
    assert_eq!(owed.paths, vec![String::from("one.txt")], "the record names the commit's paths");
    let settled: Vec<Vec<u8>> = crate::owed_index_update::settle_leftovers(&world.context.git, &owed, real_index_of(&world).as_path()).expect("settled");
    assert_eq!(settled, Vec::<Vec<u8>>::new(), "the entry is not the one the batch started from, so it is the program's own");
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-status", "HEAD"]), "M\tone.txt", "the program's staging stands");
    crate::owed_index_update::remove(world.repo.join(".git").as_path(), 1);
    remove(world.root.as_path());
}

/// A batch whose index update could not be replayed because another program wrote the index since its landing is finished,
/// and what it owes is kept:
/// the next commit in the worktree sets the entries the landing left to the landed commit,
/// so it cannot record the commit's own paths against it,
/// and the entry the other program staged stands.
#[test]
fn a_batch_finished_with_its_index_written_meanwhile_keeps_what_it_owes() {
    let world: World = world("service-land-owed-written-meanwhile");
    let landed: CommitRequest = request(&world, "landed", "one.txt", "one\n");
    land_with_failed_index(&world, 1, &[landed]);
    // The program the audit names for this state: a person's `git add`, after the landing and before any recovery.
    std::fs::write(world.repo.join("other.txt"), "other\n").expect("write");
    git(world.repo.as_path(), &["add", "other.txt"]);
    let reports: Vec<RecoveryReport> = recover(&world.context);
    assert_eq!(reports, vec![RecoveryReport { seq: 1, decision: Recovery::Finish, handled: Handled::Finished }], "the batch is finished");
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-status", "HEAD"]), "D\tone.txt\nA\tother.txt", "the index still describes the branch before the landing");
    assert!(owed_of(&world, 1).exists(), "what the batch owes outlives the batch, where the next commit reads it");
    let owed: crate::owed_index_update::Owed = record_of(&world);
    let settled: Vec<Vec<u8>> = crate::owed_index_update::settle_leftovers(&world.context.git, &owed, real_index_of(&world).as_path()).expect("settled");
    assert_eq!(settled, vec![b"one.txt".to_vec()], "the entry the landing left is set to the landed commit");
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-status", "HEAD"]), "A\tother.txt", "the next commit cannot record the landed file's deletion, and the program's staging stands");
    crate::owed_index_update::remove(world.repo.join(".git").as_path(), 1);
    remove(world.root.as_path());
}
