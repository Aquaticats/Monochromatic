//! The entries of a trash removed side by side: the removals overlap, the reasons come in entry order, an entry that stays does
//!  not stop the others, and the host budget bounds the overlap.

use super::*;
use crate::worktree_trash_remove::remove_tree_beneath;
use crate::fs_beneath::{identity_at, open_root};
use crate::parallel_test_support::{PATIENCE, Rendezvous, SERIAL_PATIENCE};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many entries the trash of these controls holds.
const ENTRIES: usize = 4;

/// A host large enough that neither the CPUs nor the memory bound a removal of `ENTRIES` entries.
const LARGE_HOST: HostResources = HostResources { parallelism: ENTRIES * 8, memory_available: None };

/// A trash directory `dir` holding `ENTRIES` entries with a file each, and the manifest that records them.
fn trash_with_entries(name: &str) -> (PathBuf, File, TrashManifest) {
    let root: PathBuf = fixture(name);
    let trash_path: PathBuf = root.join("trash");
    for index in 0..ENTRIES {
        std::fs::create_dir_all(trash_path.join(entry_name(index)).join("deep")).expect("entry");
        std::fs::write(trash_path.join(entry_name(index)).join("deep/file"), b"left out").expect("file");
    }
    let held: File = open_root(trash_path.as_path()).expect("trash");
    let entries: Vec<_> = (0..ENTRIES).map(|index: usize| { return identity_at(&held, std::ffi::OsStr::new(entry_name(index).as_str())).expect("identity"); }).collect();
    let trash = identity_at(&open_root(root.as_path()).expect("root"), std::ffi::OsStr::new("trash")).expect("trash identity");
    return (root, held, TrashManifest { name: "a".repeat(40), trash, entries });
}

/// Remove the entries with `remove` on `host` and return the reasons.
fn remove_with(held: &File, root: &Path, manifest: &TrashManifest, host: &HostResources, remove: &RemoveOne<'_>) -> Vec<String> {
    return remove_entries(&EntryRemoval { inside: root.join("trash").as_path(), trash: held, manifest, host, remove });
}

/// The removals of all the entries start before any has finished: a point that needs all of them at once is passed only by
///  parallel work.
#[test]
fn the_entries_are_removed_side_by_side() {
    let (root, held, manifest) = trash_with_entries("trash-entries-overlap");
    let point: Rendezvous = Rendezvous::new(ENTRIES);
    let met: AtomicUsize = AtomicUsize::new(0);
    let left = remove_with(&held, root.as_path(), &manifest, &LARGE_HOST, &|label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        if point.arrive() {
            met.fetch_add(1, Ordering::SeqCst);
        }
        return remove_tree_beneath(label, parent, name, expected);
    });
    assert!(left.is_empty(), "{left:?}");
    assert_eq!(met.load(Ordering::SeqCst), ENTRIES);
    for index in 0..ENTRIES {
        assert!(!root.join("trash").join(entry_name(index)).exists(), "d{index} is gone");
    }
    remove(root.as_path());
}

/// Entries that stay give their reasons in entry order, whichever finishes first, and the others are removed.
#[test]
fn the_reasons_of_entries_that_stay_come_in_entry_order_and_the_others_go() {
    let (root, held, manifest) = trash_with_entries("trash-entries-reasons");
    let later_done: Rendezvous = Rendezvous::impatient(ENTRIES - 1, PATIENCE);
    let left = remove_with(&held, root.as_path(), &manifest, &LARGE_HOST, &|label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        let own: bool = name == std::ffi::OsStr::new(entry_name(0).as_str());
        if own {
            assert!(later_done.wait_for_target(), "the other entries were handled while the first waited");
            return Err(String::from("d0 stays"));
        }
        let outcome = remove_tree_beneath(label, parent, name, expected);
        later_done.arrive_without_waiting();
        return if name == std::ffi::OsStr::new(entry_name(1).as_str()) { Err(String::from("d1 stays")) } else { outcome };
    });
    assert_eq!(left, vec![String::from("d0 stays"), String::from("d1 stays")]);
    assert!(root.join("trash").join(entry_name(0)).exists(), "the entry that stays is untouched");
    for index in 2..ENTRIES {
        assert!(!root.join("trash").join(entry_name(index)).exists(), "d{index} is gone");
    }
    remove(root.as_path());
}

/// The host's budget bounds the overlap.
#[test]
fn the_budget_of_the_host_bounds_the_overlap() {
    let (root, held, manifest) = trash_with_entries("trash-entries-budget");
    let host: HostResources = HostResources { parallelism: 3, memory_available: None };
    let allowed: usize = worker_budget(&BudgetRequest { items: ENTRIES, per_item_memory: ENTRY_MEMORY, ceiling: None, cap: None, host: &host });
    assert!(allowed > 1 && allowed < ENTRIES, "the control needs a bound between one and the entries: {allowed}");
    let running: AtomicUsize = AtomicUsize::new(0);
    let busiest: AtomicUsize = AtomicUsize::new(0);
    let point: Rendezvous = Rendezvous::impatient(allowed, SERIAL_PATIENCE * 10);
    let left = remove_with(&held, root.as_path(), &manifest, &host, &|label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        let now: usize = running.fetch_add(1, Ordering::SeqCst) + 1;
        busiest.fetch_max(now, Ordering::SeqCst);
        point.arrive();
        let outcome = remove_tree_beneath(label, parent, name, expected);
        running.fetch_sub(1, Ordering::SeqCst);
        return outcome;
    });
    assert!(left.is_empty(), "{left:?}");
    assert_eq!(busiest.load(Ordering::SeqCst), allowed);
    remove(root.as_path());
}

/// The estimate covers the measured peak of removing a flat directory of a million names.
#[test]
fn the_entry_estimate_covers_the_measured_peak() {
    const { assert!(ENTRY_MEMORY >= MEASURED_MILLION_NAME_PEAK, "the entry estimate must cover the measured peak") };
}

/// The estimate is the measurement rounded up to a power of two and not more.
#[test]
fn the_entry_estimate_is_the_measurement_rounded_up_to_a_power_of_two() {
    assert_eq!(ENTRY_MEMORY, MEASURED_MILLION_NAME_PEAK.next_power_of_two());
}
