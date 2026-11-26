//! Controls for the live link report and for the paths one
//! `git worktree repair` names.

use super::*;
use crate::test_support::{
    fixture,
    git,
    remove,
};
use crate::testing::fixture_zfs_binding_record::{
    common_directory_in,
    publish_under,
    record_naming,
};
use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
};
use crate::worktree_clone_binding_record::binding_words::Phase;
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  A record under a fixture's common directory is reported from the machine,
///  and a slot with no record is not this route's to report on.
/// Why:
///  The report is printed by a command that met a slot,
///  so its answer must be nothing when the route holds nothing there.
#[test]
fn a_record_is_reported_and_a_slot_without_one_is_not() {
    let root: PathBuf = fixture("live-link-report");
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(
        repository.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message", "base"],
    );
    let slot: PathBuf = root.join("one");
    git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            slot.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let common: PathBuf = repository.join(".git");
    let registration: PathBuf = common.join("worktrees").join("one");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    publish_under(
        common.as_path(),
        &record_naming(
            Phase::Filled,
            slot.as_path(),
            registration.as_path(),
            Some(pointer.clone()),
        ),
        "one",
    );
    let table: ScriptedMountTable =
        ScriptedMountTable::holding(&[row(1, 0, "/", "/", "zfs", "tank/ROOT")]);
    let report = the_links_report(common.as_path(), slot.as_path(), &table)
        .expect("the record's slot is reported");
    assert_eq!(report.slot, "one");
    assert_eq!(report.state, "intact");
    let missing: PathBuf = root.join("two");
    std::fs::create_dir(missing.as_path()).expect("second slot");
    assert!(the_links_report(common.as_path(), missing.as_path(), &table).is_none());
    let _unused: PathBuf = common_directory_in(root.as_path());
    remove(root.as_path());
}

/// What:
///  The paths one repair command names are read the way Git reads them,
///  and every other command names none.
/// Why:
///  The notice must be about the worktree the person named,
///  and reading another command's operands would report on the wrong path.
#[test]
fn the_repair_paths_are_the_ones_the_command_names() {
    let root: PathBuf = fixture("live-link-report-paths");
    std::fs::create_dir(root.join("one")).expect("slot");
    let values: Vec<OsString> = vec![
        OsString::from("worktree"),
        OsString::from("repair"),
        OsString::from("one"),
    ];
    let paths: Vec<PathBuf> = repair_paths(values.as_slice(), 0, root.as_path(), None);
    assert_eq!(
        paths,
        vec![std::fs::canonicalize(root.join("one")).expect("canonical")]
    );
    let aside: Vec<OsString> = vec![
        OsString::from("worktree"),
        OsString::from("repair"),
        OsString::from("--"),
        OsString::from("one"),
    ];
    assert_eq!(
        repair_paths(aside.as_slice(), 0, root.as_path(), None).len(),
        1
    );
    let moved: Vec<OsString> = vec![
        OsString::from("worktree"),
        OsString::from("move"),
        OsString::from("one"),
        OsString::from("two"),
    ];
    assert!(repair_paths(moved.as_slice(), 0, root.as_path(), None).is_empty());
    let absent: Vec<OsString> = vec![
        OsString::from("worktree"),
        OsString::from("repair"),
        OsString::from("absent"),
    ];
    assert!(repair_paths(absent.as_slice(), 0, root.as_path(), None).is_empty());
    remove(root.as_path());
}
