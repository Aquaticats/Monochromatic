//! The entries of a trash removed side by side:
//! the removals overlap,
//! the reasons come in entry order,
//! an entry that stays does
//!  not stop the others,
//! and the host budget bounds the overlap.

use super::*;
use crate::fs_beneath::{identity_at, open_root};
use crate::parallel_test_support::{PATIENCE, Rendezvous, SERIAL_PATIENCE};
use crate::test_support::{fixture, remove};
use crate::worktree_trash_remove::remove_tree_beneath;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to;

/// How many entries the trash of these controls holds.
const ENTRIES: usize = 4;

/// How many places of the process's bound on blocking jobs the place control holds.
const ONE_PLACE: usize = 1;

/// How long the job of the place control holds the only place,
/// so the work it measures has time to reach its own job.
const HELD_BY_ANOTHER_JOB: std::time::Duration = std::time::Duration::from_millis(300);

/// A host large enough that neither the CPUs nor the memory bound a removal of `ENTRIES` entries.
const LARGE_HOST: HostResources = HostResources { parallelism: ENTRIES * 8, memory_available: None };

/// A trash directory `dir` holding `ENTRIES` entries with a file each,
/// and the manifest that records them.
fn trash_with_entries(name: &str) -> (PathBuf, Arc<File>, Arc<TrashManifest>) {
    let root: PathBuf = fixture(name);
    let trash_path: PathBuf = root.join("trash");
    for index in 0..ENTRIES {
        std::fs::create_dir_all(trash_path.join(entry_name(index)).join("deep")).expect("entry");
        std::fs::write(trash_path.join(entry_name(index)).join("deep/file"), b"left out").expect("file");
    }
    let held: File = open_root(trash_path.as_path()).expect("trash");
    let entries: Vec<_> = (0..ENTRIES).map(|index: usize| { return identity_at(&held, std::ffi::OsStr::new(entry_name(index).as_str())).expect("identity"); }).collect();
    let trash = identity_at(&open_root(root.as_path()).expect("root"), std::ffi::OsStr::new("trash")).expect("trash identity");
    return (root, Arc::new(held), Arc::new(TrashManifest { name: "a".repeat(40), trash, entries }));
}

/// The removal of the entries of one trash on `host` with `remove`,
/// as the call site states it.
fn request<'a>(root: &Path, held: &'a Arc<File>, manifest: &'a Arc<TrashManifest>, host: &HostResources, remove: &'a Arc<RemoveOne>) -> EntryRemoval {
    return EntryRemoval { inside: root.join("trash"), trash: Arc::clone(held), manifest: Arc::clone(manifest), host: *host, remove: Arc::clone(remove) };
}

/// Remove the entries with `remove` on `host` and return the reasons.
fn remove_with(held: &Arc<File>, root: &Path, manifest: &Arc<TrashManifest>, host: &HostResources, remove: &Arc<RemoveOne>) -> Vec<String> {
    return remove_entries(&request(root, held, manifest, host, remove));
}

/// The removals of all the entries start before any has finished:
/// a point that needs all of them at once is passed only by
///  parallel work.
#[test]
fn the_entries_are_removed_side_by_side() {
    let (root, held, manifest) = trash_with_entries("trash-entries-overlap");
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::new(ENTRIES));
    let met: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (at_the_point, met_it) = (Arc::clone(&point), Arc::clone(&met));
    let one_removal: Arc<RemoveOne> = Arc::new(move |label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        if at_the_point.arrive() {
            met_it.fetch_add(1, Ordering::SeqCst);
        }
        return remove_tree_beneath(label, parent, name, expected);
    });
    let left = remove_with(&held, root.as_path(), &manifest, &LARGE_HOST, &one_removal);
    assert!(left.is_empty(), "{left:?}");
    assert_eq!(met.load(Ordering::SeqCst), ENTRIES);
    for index in 0..ENTRIES {
        assert!(!root.join("trash").join(entry_name(index)).exists(), "d{index} is gone");
    }
    remove(root.as_path());
}

/// Entries that stay give their reasons in entry order,
/// whichever finishes first,
/// and the others are removed.
#[test]
fn the_reasons_of_entries_that_stay_come_in_entry_order_and_the_others_go() {
    let (root, held, manifest) = trash_with_entries("trash-entries-reasons");
    let later_done: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(ENTRIES - 1, PATIENCE));
    let others = Arc::clone(&later_done);
    let one_removal: Arc<RemoveOne> = Arc::new(move |label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        let own: bool = name == std::ffi::OsStr::new(entry_name(0).as_str());
        if own {
            assert!(others.wait_for_target(), "the other entries were handled while the first waited");
            return Err(String::from("d0 stays"));
        }
        let outcome = remove_tree_beneath(label, parent, name, expected);
        others.arrive_without_waiting();
        return if name == std::ffi::OsStr::new(entry_name(1).as_str()) { Err(String::from("d1 stays")) } else { outcome };
    });
    let left = remove_with(&held, root.as_path(), &manifest, &LARGE_HOST, &one_removal);
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
    let running: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let busiest: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let point: Arc<Rendezvous> = Arc::new(Rendezvous::impatient(allowed, SERIAL_PATIENCE * 10));
    let (live, most, at_the_point) = (Arc::clone(&running), Arc::clone(&busiest), Arc::clone(&point));
    let one_removal: Arc<RemoveOne> = Arc::new(move |label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        let now: usize = live.fetch_add(1, Ordering::SeqCst) + 1;
        most.fetch_max(now, Ordering::SeqCst);
        at_the_point.arrive();
        let outcome = remove_tree_beneath(label, parent, name, expected);
        live.fetch_sub(1, Ordering::SeqCst);
        return outcome;
    });
    let left = remove_with(&held, root.as_path(), &manifest, &host, &one_removal);
    assert!(left.is_empty(), "{left:?}");
    assert_eq!(busiest.load(Ordering::SeqCst), allowed);
    remove(root.as_path());
}

/// What:
///  The removal of every entry runs inside the process's runtime,
/// as a job of its blocking pool,
/// and not on a thread of its own.
/// Why:
///   This is the whole of the conversion:
/// the removal used to run on scoped threads,
/// where the process's runtime is not entered at all,
///  and the runtime is what the owner asked every wait to sit on.
#[test]
fn the_removals_run_on_the_processs_runtime() {
    let (root, held, manifest) = trash_with_entries("trash-entries-on-the-runtime");
    // The tiniest answer that says "the removal ran inside the process's runtime".
    let probe: Arc<std::sync::Mutex<Vec<bool>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let inside = Arc::clone(&probe);
    let one_removal: Arc<RemoveOne> = Arc::new(move |label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        inside.lock().expect("lock").push(tokio::runtime::Handle::try_current().is_ok());
        return remove_tree_beneath(label, parent, name, expected);
    });
    let left = remove_with(&held, root.as_path(), &manifest, &LARGE_HOST, &one_removal);
    assert!(left.is_empty(), "{left:?}");
    let seen: Vec<bool> = probe.lock().expect("lock").clone();
    assert_eq!(seen, vec![true; ENTRIES], "a removal ran outside the process's runtime");
    remove(root.as_path());
}

/// A removal waits for the one place the process's blocking pool has:
/// only a job of the pool takes one,
/// and work that ran on a worker of the runtime instead would start at once.
#[test]
fn the_removals_take_places_of_the_processs_bound_on_blocking_jobs() {
    let (root, held, manifest) = trash_with_entries("trash-entries-a-place-of-the-bound");
    let (given_back, finished) = hold_the_one_place();
    let started_after: Arc<std::sync::Mutex<Vec<bool>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let (began, given) = (Arc::clone(&started_after), Arc::clone(&given_back));
    let one_removal: Arc<RemoveOne> = Arc::new(move |label: &Path, parent: &File, name: &std::ffi::OsStr, expected| {
        began.lock().expect("lock").push(given.load(Ordering::SeqCst));
        return remove_tree_beneath(label, parent, name, expected);
    });
    let left = remove_with(&held, root.as_path(), &manifest, &LARGE_HOST, &one_removal);
    assert!(left.is_empty(), "{left:?}");
    let seen: Vec<bool> = started_after.lock().expect("lock").clone();
    assert_eq!(seen, vec![true; ENTRIES], "a removal started while another job held the only place of the process's bound, so it did not take one");
    assert!(finished.recv_timeout(PATIENCE).is_ok(), "the job that held the place never gave it back");
    remove(root.as_path());
}

/// What:
///  Holds the process's one place on blocking jobs on a task of the runtime,
///  and gives it back after `HELD_BY_ANOTHER_JOB`.
/// Why:
///   A control can then show that the work it measures waits for a place,
///  which only a job of the blocking pool does;
///  work that ran on a worker of the runtime instead would start at once.
/// The place is known to be taken before this returns,
///  so the work that follows either waits for it or did not take one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [givenBack, finished] = holdTheOnePlace();
/// ```
fn hold_the_one_place() -> (Arc<std::sync::atomic::AtomicBool>, std::sync::mpsc::Receiver<()>) {
    hold_blocking_jobs_to(ONE_PLACE);
    let given_back: Arc<std::sync::atomic::AtomicBool> = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (taken, has_taken) = std::sync::mpsc::channel::<()>();
    let (done, finished) = std::sync::mpsc::channel::<()>();
    let flag: Arc<std::sync::atomic::AtomicBool> = Arc::clone(&given_back);
    let holder = crate::lazy_runtime::spawn_on_runtime(async move {
        let _outcome = crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = taken.send(());
            std::thread::sleep(HELD_BY_ANOTHER_JOB);
            flag.store(true, Ordering::SeqCst);
            let _sent = done.send(());
        })
        .await;
    });
    assert!(holder.is_some(), "the runtime has workers");
    assert!(has_taken.recv_timeout(PATIENCE).is_ok(), "the job that holds the place never started");
    return (given_back, finished);
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
