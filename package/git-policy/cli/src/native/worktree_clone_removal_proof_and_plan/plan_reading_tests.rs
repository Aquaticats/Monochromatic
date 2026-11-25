//! What:
//!  The controls of the two-stage read:
//!  a record that reads,
//!  each file that must be refused before an attempt byte is read,
//!  and the folder and name rules around it.
//! Why:
//!  The reader's memory is bounded by the record's own declared length,
//!  so an oversized file,
//!  a file that ends early and a foreign entry each refuse rather than being read as far as they go
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The size bound and its basis" and "What every reader refuses").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readPlanAt(path, name).cause).toBe('oversized');
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{SAMPLE_CLONE_GUID, guid, replaced, sample_bytes, sample_plan};
use crate::worktree_clone_removal_proof_and_plan::plan_bounds::{ATTEMPT_LINE_MAXIMUM_BYTES, PLAN_HEADER_MAXIMUM_BYTES, plan_maximum_bytes};
use crate::worktree_clone_removal_proof_and_plan::plan_reading::{check_the_folder, is_a_plan_entry, read_the_plan, read_the_plan_at};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{PlanRefusal, PlanRefusalCause};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

/// The name a sample plan is filed under.
fn filed_under() -> String {
    return SAMPLE_CLONE_GUID.to_string();
}

/// One throwaway repository whose plan folder a control writes into.
fn a_repository(name: &str) -> PathBuf {
    let root: PathBuf = fixture(&format!("removal-plan-{name}"));
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    return common;
}

/// Write one file a control then reads.
fn write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the file's folder");
    }
    std::fs::write(path, bytes).expect("the planted file");
}

/// Write the sample plan under its own GUID name in a fresh repository.
fn a_written_sample(name: &str) -> (PathBuf, PathBuf) {
    let common: PathBuf = a_repository(name);
    let folder: PathBuf = plans_folder(common.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o700)).expect("a private folder");
    let path: PathBuf = folder.join(filed_under());
    write(path.as_path(), sample_bytes().as_slice());
    return (common, path);
}

#[test]
fn a_written_sample_reads_back_as_the_value_it_came_from() {
    let (_common, path) = a_written_sample("reads-back");
    let plan: DestructionPlan = read_the_plan_at(path.as_path(), Some(filed_under().as_str())).expect("the sample is a record");
    assert_eq!(plan, sample_plan());
}

#[test]
fn a_file_larger_than_its_declared_count_allows_is_refused_after_the_length_read() {
    let (_common, path) = a_written_sample("oversized");
    let mut bytes: Vec<u8> = sample_bytes();
    let declared: usize = sample_plan().attempts.len();
    let maximum: u64 = plan_maximum_bytes(declared as u64);
    assert!(bytes.len() as u64 <= maximum);
    let padding: Vec<u8> = vec![b'\n'; (maximum + 1 - bytes.len() as u64) as usize];
    bytes.extend_from_slice(padding.as_slice());
    assert_eq!(bytes.len() as u64, maximum + 1);
    write(path.as_path(), bytes.as_slice());
    let refusal: PlanRefusal = read_the_plan_at(path.as_path(), Some(filed_under().as_str())).expect_err("one byte past the bound is refused");
    assert_eq!(refusal.cause, PlanRefusalCause::Oversized);
}

#[test]
fn a_file_that_declares_more_lines_than_it_holds_is_refused() {
    let (_common, path) = a_written_sample("ends-early");
    let grown: Vec<u8> = replaced(sample_bytes().as_slice(), "attempts=2", "attempts=3");
    write(path.as_path(), grown.as_slice());
    let refusal: PlanRefusal = read_the_plan_at(path.as_path(), Some(filed_under().as_str())).expect_err("a declared line that is not there refuses");
    assert_eq!(refusal.cause, PlanRefusalCause::CountDisagrees);
}

#[test]
fn a_header_that_never_completes_is_refused_before_the_length_is_asked_for() {
    let (_common, path) = a_written_sample("short-header");
    let bytes: Vec<u8> = sample_bytes();
    let first_newline: usize = bytes.iter().position(|byte| return *byte == b'\n').expect("the header line ends");
    write(path.as_path(), &bytes[..first_newline + 1]);
    let refusal: PlanRefusal = read_the_plan_at(path.as_path(), Some(filed_under().as_str())).expect_err("a header of one line is no record");
    assert_eq!(refusal.cause, PlanRefusalCause::EndsEarly);
}

#[test]
fn a_directory_or_a_link_under_a_plans_name_is_refused_rather_than_read() {
    let common: PathBuf = a_repository("not-a-file");
    let folder: PathBuf = plans_folder(common.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    let as_a_directory: PathBuf = folder.join(filed_under());
    std::fs::create_dir_all(as_a_directory.as_path()).expect("a planted directory");
    let refusal: PlanRefusal = read_the_plan_at(as_a_directory.as_path(), Some(filed_under().as_str())).expect_err("a directory is no plan");
    assert_eq!(refusal.cause, PlanRefusalCause::NotARegularFile);
    std::fs::remove_dir(as_a_directory.as_path()).expect("the planted directory goes");
    let target: PathBuf = folder.join("elsewhere");
    write(target.as_path(), sample_bytes().as_slice());
    let as_a_link: PathBuf = folder.join(filed_under());
    std::os::unix::fs::symlink(target.as_path(), as_a_link.as_path()).expect("a planted link");
    let through_a_link: PlanRefusal = read_the_plan_at(as_a_link.as_path(), Some(filed_under().as_str())).expect_err("a link is read by nobody");
    assert_eq!(through_a_link.cause, PlanRefusalCause::NotARegularFile);
}

#[test]
fn a_record_filed_under_another_name_than_its_own_guid_refuses() {
    let (_common, path) = a_written_sample("another-name");
    let refusal: PlanRefusal = read_the_plan_at(path.as_path(), Some("7828944690444299988")).expect_err("a record names its own clone");
    assert_eq!(refusal.cause, PlanRefusalCause::NameDisagreesWithTheRecord);
    let foreign: PlanRefusal = read_the_plan_at(path.as_path(), Some("plan")).expect_err("a plan is named by a GUID");
    assert_eq!(foreign.cause, PlanRefusalCause::ForeignName);
}

#[test]
fn a_folder_whose_mode_is_not_0700_is_refused_and_a_private_one_is_read() {
    let (common, _path) = a_written_sample("folder-mode");
    let folder: PathBuf = plans_folder(common.as_path());
    check_the_folder(folder.as_path(), common_owner(common.as_path())).expect("a private folder is read");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o775)).expect("a widened folder");
    let refusal: PlanRefusal = check_the_folder(folder.as_path(), common_owner(common.as_path())).expect_err("a widened folder is refused");
    assert_eq!(refusal.cause, PlanRefusalCause::FolderMode);
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the private folder comes back");
    read_the_plan(common.as_path(), guid(SAMPLE_CLONE_GUID)).expect("the record reads once its folder is private again");
}

/// The account that owns one folder.
fn common_owner(path: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    return std::fs::metadata(path).expect("the folder exists").uid();
}

#[test]
fn a_staging_entry_is_no_plans_name_and_no_plan_is_read_through_it() {
    assert!(is_a_plan_entry(filed_under().as_str()));
    assert!(!is_a_plan_entry(format!(".{}.a1b2.partial", filed_under()).as_str()));
    assert!(!is_a_plan_entry("unreadable"));
}

#[test]
fn the_readers_two_bounds_are_the_designs_own() {
    assert_eq!(PLAN_HEADER_MAXIMUM_BYTES, 10_130);
    assert_eq!(ATTEMPT_LINE_MAXIMUM_BYTES, 361);
}
