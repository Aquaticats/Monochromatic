//! What:
//!  The controls of the folder and the one write a removal makes:
//!  the folder's mode,
//!  the record's name and mode,
//!  the replacement a resume makes,
//!  and the writer that refuses bytes its own reader would refuse.
//! Why:
//!  The plan is the last witness of what a removal destroyed,
//!  so a write that published bytes no reader reads back would leave a removal with no record at
//!  all
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The folder,
//! its owner and its modes",
//! "One writer,
//! one publication" and section 8).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const path = PlanFolder.openOrCreate(commonDir).publish(plan);
//! expect(mode(path)).toBe(0o600);
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{SAMPLE_CLONE_GUID, sample_plan};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan_at;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanAttempt};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::PlanRefusalCause;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::PathBuf;

/// One throwaway repository.
fn a_repository(name: &str) -> PathBuf {
    let root: PathBuf = fixture(&format!("removal-plan-publication-{name}"));
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    return common;
}

#[test]
fn the_first_write_makes_a_private_folder_and_publishes_under_the_clones_guid() {
    let common: PathBuf = a_repository("first-write");
    let folder: PlanFolder = PlanFolder::open_or_create(common.as_path()).expect("the folder is made");
    let path: PathBuf = folder.publish(&sample_plan()).expect("the sample is published");
    assert_eq!(path, plans_folder(common.as_path()).join(SAMPLE_CLONE_GUID.to_string()));
    let folder_mode: u32 = std::fs::metadata(folder.path()).expect("the folder exists").mode() & 0o777;
    assert_eq!(folder_mode, 0o700, "the plan folder is private");
    let file_mode: u32 = std::fs::metadata(path.as_path()).expect("the record exists").mode() & 0o777;
    assert_eq!(file_mode, 0o600, "the record is private");
    let read_back: DestructionPlan = read_the_plan_at(path.as_path(), Some(SAMPLE_CLONE_GUID.to_string().as_str())).expect("this build reads its own write");
    assert_eq!(read_back, sample_plan());
}

#[test]
fn a_second_write_replaces_the_record_whole_and_never_mixes_two() {
    let common: PathBuf = a_repository("second-write");
    let folder: PlanFolder = PlanFolder::open_or_create(common.as_path()).expect("the folder is made");
    folder.publish(&sample_plan()).expect("the first write");
    let mut second: DestructionPlan = sample_plan();
    second.record_line(PlanAttempt { snapshot: second.attempts[0].snapshot.clone(), attempt: 1, outcome: SnapshotOutcome::Destroyed, errno: None }).expect("a later attempt is recorded");
    let path: PathBuf = folder.publish(&second).expect("the second write");
    let read_back: DestructionPlan = read_the_plan_at(path.as_path(), Some(SAMPLE_CLONE_GUID.to_string().as_str())).expect("the second write reads back");
    assert_eq!(read_back, second, "the file holds the second record whole");
    let entries: Vec<PathBuf> = std::fs::read_dir(folder.path())
        .expect("the folder lists")
        .map(|entry| return entry.expect("one entry").path())
        .collect();
    assert_eq!(entries.len(), 1, "the staging file went with the rename: {entries:?}");
}

#[test]
fn a_writer_refuses_bytes_its_own_reader_would_refuse() {
    let common: PathBuf = a_repository("writer-refuses");
    let folder: PlanFolder = PlanFolder::open_or_create(common.as_path()).expect("the folder is made");
    let mut broken: DestructionPlan = sample_plan();
    broken.facts.admin = String::from("not-an-identity");
    let refusal = folder.publish(&broken).expect_err("an identity this build does not read is not published");
    assert_eq!(refusal.cause, PlanRefusalCause::NotCanonical);
    assert_eq!(std::fs::read_dir(folder.path()).expect("the folder lists").count(), 0, "nothing was left behind");
}

#[test]
fn a_folder_whose_mode_was_widened_is_refused_before_the_first_write() {
    let common: PathBuf = a_repository("widened");
    let folder: PathBuf = plans_folder(common.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o775)).expect("a widened folder");
    let refusal = PlanFolder::open_or_create(common.as_path()).expect_err("a widened folder is refused");
    assert_eq!(refusal.cause, PlanRefusalCause::FolderMode);
}
