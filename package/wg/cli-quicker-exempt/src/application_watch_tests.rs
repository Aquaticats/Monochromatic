//! Verifies that the watch setup blocks the shutdown signals before any thread exists.
//! The failure reporter's notification worker is a thread,
//! and a thread created before the signals are blocked inherits them unblocked,
//! so a process-directed `SIGTERM` can be delivered to it and end the process with status 143
//! while the signal descriptor never sees it.
//! The check reads the signal mask of every thread the setup created from the kernel's view of the process.
//! Sending a real `SIGTERM` is not used,
//! because the test harness keeps a main thread that never blocks it.

/// The setup under test.
use crate::application_watch::{prepare_application_watch, rescan_due, run_application_watch, ReporterIdentity};
/// Fake scan roots.
use crate::application_targets::ScanRoots;
/// A move the fake manager never has to make.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter, Joined, LimitRequest};
/// The policy engine over fake trees.
use crate::update_routing::{RoutingSetup, UpdateRouting};
/// Threads before and after the setup.
use std::collections::BTreeSet;
/// Fixture paths.
use std::path::{Path, PathBuf};

/// Bit of `SIGINT` in the signal mask of a thread.
const SIGINT_BIT: u64 = 1 << (libc::SIGINT - 1);
/// Bit of `SIGTERM` in the signal mask of a thread.
const SIGTERM_BIT: u64 = 1 << (libc::SIGTERM - 1);

/// A manager that creates every slice and moves nothing.
struct IdleAdopter;

/// The idle manager.
impl Adopter for IdleAdopter {
    /// The slice always exists.
    fn ensure_slice(&self, _uid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    /// No scope of an origin runs in this fixture.
    fn existing_scope_name(&self, _uid: u32, _slice: &str, _origin: &str) -> Option<String> {
        return None;
    }

    /// No process matches in the fake tree,
    /// so no move is requested.
    fn adopt(&self, _request: &AdoptRequest<'_>) -> Result<Joined, AdoptError> {
        return Err(AdoptError::Failed("this fixture moves nothing".to_string()));
    }

    /// No scope exists in this fixture.
    fn apply_limits(&self, _request: &LimitRequest<'_>) -> Result<(), AdoptError> {
        return Err(AdoptError::ScopeGone);
    }
}

/// The ids of the threads of this process now.
fn thread_ids() -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for entry in std::fs::read_dir("/proc/self/task").expect("list the threads").flatten() {
        ids.insert(entry.file_name().to_string_lossy().into_owned());
    }
    return ids;
}

/// The mask of blocked signals of one thread,
/// from the line `SigBlk` of its status file.
fn blocked_signals(thread_id: &str) -> u64 {
    let status = std::fs::read_to_string(format!("/proc/self/task/{thread_id}/status")).expect("read the thread status");
    for line in status.lines() {
        if let Some(mask) = line.strip_prefix("SigBlk:") {
            return u64::from_str_radix(mask.trim(), 16).expect("a hexadecimal mask");
        }
    }
    panic!("the status of thread {thread_id} has no SigBlk line");
}

/// Every thread the setup creates,
/// the reporter's notification worker among them,
/// starts with `SIGTERM` and `SIGINT` blocked.
/// A thread that started with them unblocked could receive the signal itself and end the process.
#[test]
fn every_thread_the_setup_creates_starts_with_the_shutdown_signals_blocked() {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-watch-setup-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let app_slice = base.join("app.slice");
    let proc_root = base.join("proc");
    let cgroup_root = base.join("cgroup");
    for directory in [&app_slice, &proc_root, &cgroup_root] {
        std::fs::create_dir_all(directory).expect("make a fake tree");
    }
    let routing = UpdateRouting::new(RoutingSetup {
        uid: 1000,
        install_roots: Vec::<PathBuf>::new(),
        proc_root: proc_root.clone(),
        cgroup_root: cgroup_root.clone(),
        adopter: Box::new(IdleAdopter),
    });
    let roots = ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    };
    let before = thread_ids();
    let watch = prepare_application_watch(roots, 8888, routing, ReporterIdentity { uid: 1000, gid: 1000 }, None).expect("prepare the watch");
    let created: Vec<String> = thread_ids().difference(&before).cloned().collect();
    assert!(!created.is_empty(), "the setup started no thread, so this check would pass without checking anything");
    for thread_id in &created {
        let blocked = blocked_signals(thread_id);
        assert_eq!(blocked & (SIGTERM_BIT | SIGINT_BIT), SIGTERM_BIT | SIGINT_BIT, "thread {thread_id} started with a shutdown signal unblocked: mask {blocked:x}");
    }
    drop(watch);
}

/// The periodic scan is due after its interval,
/// and a drop of process events brings the next scan forward only to the overrun spacing,
/// so a storm of dropped events cannot make the watcher scan every process on every wakeup.
#[test]
fn a_dropped_event_brings_the_scan_forward_only_to_the_spacing() {
    let at = std::time::Duration::from_millis;
    assert!(!rescan_due(false, at(0)));
    assert!(!rescan_due(false, at(1_999)));
    assert!(rescan_due(false, at(2_000)));
    assert!(!rescan_due(true, at(0)));
    assert!(!rescan_due(true, at(999)));
    assert!(rescan_due(true, at(1_000)));
    assert!(rescan_due(true, at(10_000)));
}

/// When the watcher is told to stop it ends its subscription to process events and drops the socket,
/// so the kernel does not go on counting a listener that is gone.
/// The signal goes to the test thread alone,
/// which blocks it since the setup,
/// like the watcher thread;
/// a signal sent to the process could end it through a thread that does not.
/// It needs the kernel connector,
/// which this kernel allows without privilege and a container may not.
#[test]
#[ignore = "needs the kernel process-event connector"]
fn the_watcher_ends_its_process_event_subscription_when_it_stops() {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-watch-stop-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let app_slice = base.join("app.slice");
    let proc_root = base.join("proc");
    let cgroup_root = base.join("cgroup");
    for directory in [&app_slice, &proc_root, &cgroup_root] {
        std::fs::create_dir_all(directory).expect("make a fake tree");
    }
    let routing = UpdateRouting::new(RoutingSetup {
        uid: 1000,
        install_roots: Vec::<PathBuf>::new(),
        proc_root: proc_root.clone(),
        cgroup_root: cgroup_root.clone(),
        adopter: Box::new(IdleAdopter),
    });
    let roots = ScanRoots {
        app_slice: &app_slice,
        proc_root: &proc_root,
        cgroup_root: &cgroup_root,
    };
    let mut watch = prepare_application_watch(roots, 8888, routing, ReporterIdentity { uid: 1000, gid: 1000 }, None).expect("prepare the watch");
    assert!(watch.has_process_events(), "the watch holds no subscription, so this check would pass without checking anything");
    // SAFETY: pthread_self and pthread_kill have no preconditions beyond a valid signal number.
    let sent = unsafe { libc::pthread_kill(libc::pthread_self(), libc::SIGTERM) };
    assert_eq!(sent, 0);
    run_application_watch(&mut watch).expect("the watcher stops on the signal");
    assert!(!watch.has_process_events(), "the watcher kept its subscription after it stopped");
}
