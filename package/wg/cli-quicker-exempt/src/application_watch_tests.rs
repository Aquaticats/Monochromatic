//! Verifies that the watch setup blocks the shutdown signals before any thread exists.
//! The failure reporter's notification worker is a thread,
//! and a thread created before the signals are blocked inherits them unblocked,
//! so a process-directed `SIGTERM` can be delivered to it and end the process with status 143
//! while the signal descriptor never sees it.
//! The check reads the signal mask of every thread the setup created from the kernel's view of the process.
//! Sending a real `SIGTERM` is not used,
//! because the test harness keeps a main thread that never blocks it.

/// The setup under test.
use crate::application_watch::{prepare_application_watch, ReporterIdentity};
/// Fake scan roots.
use crate::application_targets::ScanRoots;
/// A move the fake manager never has to make.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter};
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
    fn ensure_slice(&self, _uid: u32, _gid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    /// No process matches in the fake tree,
    /// so no move is requested.
    fn adopt(&self, _request: &AdoptRequest<'_>) -> Result<(), AdoptError> {
        return Err(AdoptError::Failed("this fixture moves nothing".to_string()));
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
        gid: 1000,
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
    let watch = prepare_application_watch(roots, 8888, routing, ReporterIdentity { uid: 1000, gid: 1000 }).expect("prepare the watch");
    let created: Vec<String> = thread_ids().difference(&before).cloned().collect();
    assert!(!created.is_empty(), "the setup started no thread, so this check would pass without checking anything");
    for thread_id in &created {
        let blocked = blocked_signals(thread_id);
        assert_eq!(blocked & (SIGTERM_BIT | SIGINT_BIT), SIGTERM_BIT | SIGINT_BIT, "thread {thread_id} started with a shutdown signal unblocked: mask {blocked:x}");
    }
    drop(watch);
}
