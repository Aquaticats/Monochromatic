//! Controls for the pass a prune runs over the wrapper's own records:
//! a pointer restored from the record,
//! a pointer that already stands,
//! and the slots whose registration nothing here can protect.

use super::*;
use crate::testing::fixture_zfs_binding_record::{common_directory_in, publish_under, record_naming};
use crate::testing::mounted_dataset_table::{ScriptedMountTable, row};
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_prune_pointer::pointer_state::PointerSite;
use crate::test_support::{fixture, git, remove};
use std::path::PathBuf;

/// One slot whose registration stands,
/// made by real Git so its `gitdir` is Git's own.
struct Bound {
    /// The fixture root.
    root: PathBuf,
    /// The Git common directory the record lives under.
    common: PathBuf,
    /// The slot's path.
    slot: PathBuf,
    /// The registration's path.
    registration: PathBuf,
    /// The bytes Git wrote into the slot.
    pointer: Vec<u8>,
}

/// What:
///  One real registration for one slot under a fixture's common directory.
/// Why:
///  The guard reads the registration's `gitdir`,
///  and only real Git writes one the guard compares.
fn bound_slot(name: &str, slot_name: &str) -> Bound {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(repository.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "base"]);
    let slot: PathBuf = root.join(slot_name);
    git(repository.as_path(), &["worktree", "add", "--quiet", "--no-checkout", slot.to_str().expect("utf-8"), "HEAD"]);
    let common: PathBuf = repository.join(".git");
    let registration: PathBuf = common.join("worktrees").join(slot_name);
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    return Bound { root, common, slot, registration, pointer };
}

/// One mount table in which nothing stands at the slots.
fn nothing_mounted() -> ScriptedMountTable {
    return ScriptedMountTable::holding(&[row(1, 0, "/", "/", "zfs", "tank/ROOT")]);
}

/// What:
///  A pointer an account deleted while the clone is unmounted is written back
///  from the record,
///  and the prune goes on with its registration kept.
/// Why:
///  This is the underlay pointer plan step 12 keeps for prune,
///  and an explicit prune drops the registration at once without it.
#[test]
fn a_deleted_pointer_is_restored_before_the_prune() {
    let bound: Bound = bound_slot("prune-guard-restore", "one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    let guard: PruneGuard = guard_the_prune(bound.common.as_path(), &nothing_mounted());
    assert!(guard.refused.is_empty(), "{:?}", guard.refused);
    assert_eq!(guard.lines.len(), 1, "{:?}", guard.lines);
    assert!(guard.lines[0].contains("restored the pointer"), "{:?}", guard.lines);
    assert_eq!(std::fs::read(bound.slot.join(".git")).expect("pointer reads"), bound.pointer);
    // Real Git's own prune now keeps the registration, which is the whole point of the restore.
    let pruned: std::process::Output = git(bound.root.join("repository").as_path(), &["worktree", "prune", "--verbose"]);
    assert!(pruned.status.success(), "prune ran: {}", String::from_utf8_lossy(pruned.stderr.as_slice()));
    assert!(bound.registration.is_dir(), "the registration stands after the prune");
    remove(bound.root.as_path());
}

/// What:
///  A pointer that stands is left alone and prints nothing.
/// Why:
///  Every prune of this repository runs the pass,
///  and a line for each intact slot would bury the one that matters.
#[test]
fn an_intact_pointer_prints_nothing() {
    let bound: Bound = bound_slot("prune-guard-intact", "one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    let guard: PruneGuard = guard_the_prune(bound.common.as_path(), &nothing_mounted());
    assert!(guard.lines.is_empty(), "{:?}", guard.lines);
    assert!(guard.refused.is_empty());
    remove(bound.root.as_path());
}

/// What:
///  A pointer nothing can compare or write stops the prune,
///  and the report names the slot and the link.
/// Why:
///  Git's prune would drop the registration and the clone dataset would then
///  be reachable by nobody,
///  so the refusal is what keeps the work,
///  and a link at `.git` is never written through.
#[test]
fn a_pointer_nothing_restores_stops_the_prune() {
    let bound: Bound = bound_slot("prune-guard-unrestorable", "one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    std::os::unix::fs::symlink(bound.root.join("elsewhere"), bound.slot.join(".git")).expect("planted link");
    let guard: PruneGuard = guard_the_prune(bound.common.as_path(), &nothing_mounted());
    assert_eq!(guard.refused.len(), 1, "{:?}", guard.lines);
    match &guard.refused[0] {
        PruneRefusal::Link(facts) => {
            assert_eq!(facts.slot, "one");
            assert!(matches!(facts.pointer, PointerState::Unprovable { .. }), "{:?}", facts.pointer);
        }
        other => panic!("a link at the pointer is a link's refusal: {other:?}"),
    }
    assert!(std::fs::symlink_metadata(bound.slot.join(".git")).expect("the link stands").file_type().is_symlink(), "nothing was written through the link");
    remove(bound.root.as_path());
}

/// What:
///  A record no reader may act on stops the prune,
///  because it may name a registration the prune would drop.
/// Why:
///  The record is the only place its slot path and registration are written,
///  so nothing about them is proved when it cannot be read.
#[test]
fn an_unreadable_record_stops_the_prune() {
    let bound: Bound = bound_slot("prune-guard-unreadable", "one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), None), "one");
    let guard: PruneGuard = guard_the_prune(bound.common.as_path(), &nothing_mounted());
    assert_eq!(guard.refused.len(), 1, "{:?}", guard.lines);
    assert!(matches!(&guard.refused[0], PruneRefusal::Record { slot, .. } if slot == "one"), "{:?}", guard.refused);
    assert!(guard.lines.join("\n").contains("cannot be read"), "{:?}", guard.lines);
    remove(bound.root.as_path());
}

/// What:
///  A slot another mount covers is not written and stops the prune.
/// Why:
///  The `.git` at the slot would then not be either copy this route wrote,
///  and a write there could land in another dataset.
#[test]
fn another_mount_at_the_slot_stops_the_prune() {
    let bound: Bound = bound_slot("prune-guard-foreign-mount", "one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    let table: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(1, 0, "/", "/", "zfs", "tank/ROOT"),
        row(41, 1, "/", bound.slot.to_str().expect("utf-8"), "zfs", "tank/other"),
    ]);
    let guard: PruneGuard = guard_the_prune(bound.common.as_path(), &table);
    assert_eq!(guard.refused.len(), 1, "{:?}", guard.lines);
    match &guard.refused[0] {
        PruneRefusal::Link(facts) => {
            assert_eq!(facts.site, PointerSite::AnotherMount { described: String::from("`tank/other` at the slot, mounted as `zfs`") });
        }
        other => panic!("a foreign mount is a link's refusal: {other:?}"),
    }
    assert!(!bound.slot.join(".git").exists(), "nothing was written");
    remove(bound.root.as_path());
}

/// What:
///  A record under a slot name whose path names another slot is not written,
///  and says so.
/// Why:
///  A write would land in a folder the record does not describe,
///  and a person must know the record disagrees with itself.
#[test]
fn a_record_naming_another_path_is_not_written() {
    let bound: Bound = bound_slot("prune-guard-other-path", "one");
    let elsewhere: PathBuf = bound.root.join("two");
    std::fs::create_dir(elsewhere.as_path()).expect("other slot");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, elsewhere.as_path(), bound.registration.as_path(), Some(bound.pointer.clone())), "one");
    let guard: PruneGuard = guard_the_prune(bound.common.as_path(), &nothing_mounted());
    assert!(guard.refused.is_empty());
    assert!(guard.lines.join("\n").contains("names the slot at"), "{:?}", guard.lines);
    assert!(!elsewhere.join(".git").exists(), "nothing was written");
    remove(bound.root.as_path());
}

/// What:
///  A record home that holds no binding name prints nothing and refuses
///  nothing.
/// Why:
///  Every repository's prune runs this pass,
///  and a common directory with no wrapper record is the ordinary case.
#[test]
fn a_home_with_no_record_does_nothing() {
    let root: PathBuf = fixture("prune-guard-empty-home");
    let common: PathBuf = common_directory_in(root.as_path());
    let guard: PruneGuard = guard_the_prune(common.as_path(), &nothing_mounted());
    assert!(guard.lines.is_empty(), "{:?}", guard.lines);
    assert!(guard.refused.is_empty());
    remove(root.as_path());
}

/// A mount table no control may read: asking it for one fails the test.
/// Why:
///  A prune in a repository with no record folder has no bound slot to check,
///  and a table that fails when read proves the pass never looked.
struct ATableNoControlReads;

impl crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets for ATableNoControlReads {
    fn mounted_datasets(&self) -> Result<Vec<crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance>, String> {
        panic!("the prune pass read the mount table before it listed the records");
    }

    fn mount_holding(&self, _path: &Path) -> Result<Option<crate::zfs_mounted_dataset::mounted_dataset_instance::MountHoldingAPath>, String> {
        panic!("the prune pass read the mount at a slot before it listed the records");
    }
}

/// What:
///  A repository with no record folder is guarded against nothing without
///  reading the mount table.
/// Why:
///  Every prune in an ordinary repository is this case,
///  and the mount table is a file read per command that no record can need.
#[test]
fn a_repository_without_records_reads_no_mount_table() {
    let root: PathBuf = fixture("prune-guard-no-records");
    let common: PathBuf = common_directory_in(root.as_path());
    let guard: PruneGuard = guard_the_prune(common.as_path(), &ATableNoControlReads);
    assert_eq!(guard, PruneGuard::default(), "nothing is restored and nothing is refused");
    remove(root.as_path());
}
