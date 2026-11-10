//! Controls for a service whose own worktree is deleted while it serves:
//! it asks for its own socket to be stopped and ends,
//! so the socket unit does not listen for a directory that is gone.
//! A worktree that is there,
//! and one whose repository is on a device that is not mounted,
//! keep the service serving.
//! The stop is a control's own function,
//! so no manager is asked anything.

use super::*;
use crate::test_support::remove;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The instance of the controls.
const INSTANCE: &str = "0000000000000000000000000000000c";

/// The file a control's stop writes,
/// beside the instance it was asked about.
const STOPPED_FILE: &str = "stopped-instance";

/// The most a control waits for the service to end:
/// far beyond what the loop needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How often the controls' service looks at its worktree:
/// short,
/// so a control ends in moments.
const CHECK: Duration = Duration::from_millis(20);

/// A stop that names no manager and writes who it was asked about instead,
/// so a control reads it from the journal it shares with the service.
fn record_the_stop(instance: &str, journal: &Path) -> bool {
    std::fs::write(journal.join(STOPPED_FILE), instance).expect("the control's stop is recorded");
    return true;
}

/// One disposable world:
/// a checkout whose Git directory exists,
/// a journal directory,
/// and a socket for the service to listen on.
struct World {
    root: PathBuf,
    git_dir: PathBuf,
    journal: PathBuf,
    socket: PathBuf,
}

fn world(name: &str) -> World {
    let home: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory"));
    // The socket's path must fit the kernel's 108 bytes, which the fixture tree's depth does not allow.
    let root: PathBuf = home.join("temp/agent").join(format!("ow-{}-{name}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("short directory");
    let git_dir: PathBuf = root.join("checkout/.git");
    std::fs::create_dir_all(&git_dir).expect("the worktree's Git directory");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    return World { socket: root.join("s"), root, git_dir, journal };
}

impl World {
    /// The settings of a service of this world whose idle exit cannot be the reason it ends.
    fn settings(&self) -> ServeSettings {
        let mut settings: ServeSettings = ServeSettings::new(IDLE_EXIT);
        settings.instance = Some(String::from(INSTANCE));
        settings.own_git_dir = Some(self.git_dir.clone());
        settings.own_units_stop = record_the_stop;
        settings.own_worktree_check = CHECK;
        return settings;
    }

    /// Starts the service on a thread of its own and gives the thread.
    fn serve(&self) -> std::thread::JoinHandle<()> {
        let listener: UnixListener = UnixListener::bind(&self.socket).expect("bind");
        let context: LandContext = LandContext {
            git: crate::test_support::git_context(),
            journal: self.journal.clone(),
            on_objects_written: None,
            lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
        };
        let settings: ServeSettings = self.settings();
        return std::thread::spawn(move || { serve(listener, &context, &settings); });
    }

    /// The instance the control's stop was asked about,
    /// nothing when it was not asked.
    fn stopped(&self) -> Option<String> {
        return std::fs::read_to_string(self.journal.join(STOPPED_FILE)).ok();
    }
}

/// Waits at most `PATIENCE` for a serving thread to end.
fn ended_within(thread: std::thread::JoinHandle<()>) -> bool {
    let began: Instant = Instant::now();
    while !thread.is_finished() {
        if began.elapsed() >= PATIENCE {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _joined = thread.join();
    return true;
}

/// The Git directory is deleted under a serving service:
/// the service asks for its own socket to be stopped and ends,
/// rather than listen on for a directory that is gone.
#[test]
fn a_service_whose_worktree_is_deleted_while_it_serves_ends_and_asks_for_its_socket_to_be_stopped() {
    let world: World = world("gone");
    let thread: std::thread::JoinHandle<()> = world.serve();
    // Long enough for several checks with the worktree still there: nothing may be stopped for it.
    std::thread::sleep(CHECK * 5);
    assert_eq!(world.stopped(), None, "a worktree that is there is not stopped");
    std::fs::remove_dir_all(world.git_dir.as_path()).expect("the worktree's Git directory is gone");
    assert!(ended_within(thread), "the service kept serving a worktree that is gone");
    assert_eq!(world.stopped().as_deref(), Some(INSTANCE), "the service asks for its own socket to be stopped");
    remove(world.root.as_path());
}

/// A worktree that is still there keeps the service serving:
/// every check finds it and nothing is stopped.
#[test]
fn a_service_whose_worktree_is_there_keeps_serving() {
    let world: World = world("present");
    let thread: std::thread::JoinHandle<()> = world.serve();
    std::thread::sleep(CHECK * 10);
    assert!(!thread.is_finished(), "a worktree that is there keeps its service");
    assert_eq!(world.stopped(), None);
    remove(world.root.as_path());
}

/// A repository on a device that is not mounted is not gone:
/// its mount point is there and empty,
/// the checkout that held the Git directory is not,
/// and the service keeps serving.
#[test]
fn a_service_whose_worktree_is_on_a_device_that_is_not_mounted_keeps_serving() {
    let world: World = world("unmounted");
    let mount_point: PathBuf = world.root.join("drive");
    std::fs::create_dir_all(mount_point.as_path()).expect("mount point");
    let mut settings: ServeSettings = world.settings();
    settings.own_git_dir = Some(mount_point.join("repository/.git"));
    let listener: UnixListener = UnixListener::bind(&world.socket).expect("bind");
    let context: LandContext = LandContext {
        git: crate::test_support::git_context(),
        journal: world.journal.clone(),
        on_objects_written: None,
        lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
    };
    let thread: std::thread::JoinHandle<()> = std::thread::spawn(move || { serve(listener, &context, &settings); });
    std::thread::sleep(CHECK * 10);
    assert!(!thread.is_finished(), "a worktree on a device that is not mounted is not gone");
    assert_eq!(world.stopped(), None);
    remove(world.root.as_path());
}
