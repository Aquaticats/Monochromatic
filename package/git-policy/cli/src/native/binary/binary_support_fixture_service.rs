//! What:
//!  The fixture of a binary-level control and the resident service process it may start,
//! with the groups of helper processes that die with the test thread.
//! Why:
//!  The fixture and its service are one lifecycle:
//! the fixture owns the service and stops it when it is dropped,
//! so they live together in a file that depends on the limits file and on the readiness checks,
//! which say when a service is at work and when it serves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = fixture(); const service = startFixtureService(fixture);
//! ```

use crate::fixture_support_shared_by_both_test_targets::{
    KEEP_FIXTURES_VARIABLE,
    settle_fixture,
};
use crate::support_resource_limits::ServiceLaunch;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Child;

/// One disposable directory with a `bin/git` link to the wrapper.
pub struct Fixture {
    /// Canonical fixture root.
    pub root: PathBuf,
    /// The wrapper path every link and hard link in this fixture points at:
    ///  a hard link of the built executable,
    /// so no control may write to it.
    pub wrapper: PathBuf,
    /// The commit service of this fixture,
    /// started by the first wrapped command.
    pub service: std::sync::Mutex<Option<FixtureService>>,
    /// Variables the fixture's service starts with besides its fixed ones:
    /// what the user manager's environment would hold.
    pub service_environment: std::sync::Mutex<Vec<(OsString, OsString)>>,
    /// How the fixture's services are started when a fault control needs more than the plain program.
    pub service_launch: std::sync::Mutex<ServiceLaunch>,
    /// The folders this fixture's sockets stand in,
    /// named and owned here,
    /// removed when the fixture goes.
    pub socket_folders: std::sync::Mutex<crate::support_socket_folders::SocketFolders>,
}

/// How a fixture's service ended,
/// or that it did not.
/// Why an event and not a line:
///  the state a control names is that this process ended, by itself, with this status;
/// a line the service wrote about it arrives through a copy thread of its own and says only what the service claimed.
pub enum ServiceEnd {
    /// It ended by itself,
    /// with this status.
    Ended(std::process::ExitStatus),
    /// It still ran when the bound passed.
    StillRunning,
    /// The fixture held no service for this worktree,
    /// so nothing could end.
    NotServed,
}

/// What:
///  The process-group helpers this file used to define,
///  named here so every `crate::support_fixture_service::name` and `support::name` path stays the same.
/// Why:
///  Every target now reads one copy of `spawn_owned`, `own_group` and `group_of`
///  (`../testing/test_process_owner.rs`),
///  which the library's tests and both other targets include by path,
///  so a fix to either lands in all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { spawnOwned, ownGroup, groupOf } from '../testing/test_process_owner';
/// ```
pub(crate) use crate::test_process_owner::{
    group_of,
    own_group,
    spawn_owned,
};

/// The commit services of one fixture,
/// one per worktree as in production,
/// stopped with the fixture.
pub struct FixtureService {
    /// The running service of each worktree,
    /// by the worktree's own Git directory.
    ///  It is `pub(super)` so the sibling file that starts this fixture's services reads it.
    pub(super) children: std::sync::Mutex<std::collections::HashMap<PathBuf, Child>>,
    /// The end of the channel of every copy of a service's stream this fixture started,
    /// so a read of the log can wait for the copy instead of the clock.
    ///  It is `pub(super)` so the sibling file that starts this fixture's services reads it.
    pub(super) log_copies: std::sync::Mutex<Vec<std::sync::mpsc::Receiver<()>>>,
    /// The runtime directory the wrapper finds the services' sockets in.
    pub runtime: PathBuf,
}

impl Drop for FixtureService {
    fn drop(&mut self) {
        if let Ok(mut children) = self.children.lock() {
            for child in children.values_mut() {
                kill_with_everything_it_started(child);
            }
        }
        let _removed = std::fs::remove_dir_all(&self.runtime);
    }
}

/// Ends a service and the processes of its group:
/// the tracer in front of it,
/// if any,
/// and the Git it started.
/// A tracer that is
///  killed does not take the traced program with it,
/// so ending only the child would leave a service running.
/// The child is asked
///  to end first,
/// because a tracer that is killed outright loses the part of its log it has not written yet,
/// and a control reads
///  that log;
/// what is still there after a short wait is killed.
pub fn kill_with_everything_it_started(child: &mut Child) {
    let group: i32 = -i32::try_from(child.id()).expect("a process ID fits");
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    unsafe {
        libc::kill(
            child.id().try_into().expect("a process ID fits"),
            libc::SIGTERM,
        )
    };
    let asked: std::time::Instant = std::time::Instant::now();
    while child.try_wait().is_ok_and(|status| {
        return status.is_none();
    }) && asked.elapsed() < std::time::Duration::from_secs(2)
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    // SAFETY: as above.
    unsafe { libc::kill(group, libc::SIGKILL) };
    let _killed = child.kill();
    let _waited = child.wait();
}

/// What:
///  The executable and the runtime directory a fixture's service starts with,
///  the process-lifetime start,
///  and the manager's restart,
///  in a file of their own because the lifecycle file is over the per-file line budget.
/// Why:
///  A caller that named one of them through this module goes on working,
///  and the file that holds them describes how a service is started rather than the fixture's lifecycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './binary_support_fixture_service_start.rs';
/// ```
pub(crate) use crate::support_fixture_service_start::*;

/// What:
///  The services this fixture holds,
///  one for each worktree under it,
///  and the acts that start one,
///  stop them all and forget the ones that ended,
///  in a file of their own because the lifecycle file is over the per-file line budget.
/// Why:
///  A caller that named one of them through this module goes on working,
///  and the file that holds them describes how a service is started rather than the fixture's lifecycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './binary_support_fixture_services_of_the_worktrees.rs';
/// ```
pub(crate) use crate::support_fixture_services_of_the_worktrees::*;

impl FixtureService {
    /// What:
    ///  Waits, bounded, for every copy of a service's stream this fixture started to end,
    ///  and answers whether they all did.
    /// Why:
    ///   A copy runs on a thread of its own, and the file holds what a service wrote only once that thread has written it.
    /// The pipe closes when the last service holding it ends,
    /// so a copy whose service ended always ends,
    /// and the wait is the event a deciding read needs rather than a sleep of a guessed length.
    ///  The answer is the whole meaning of the call:
    /// `true` says every copy of the service's words has ended, so a read that decides anything may trust the file.
    /// A copy that has not ended before the bound stays here for the next wait,
    /// and so does one whose copier is gone without a word,
    /// because "nothing left to wait for" is not "everything ended";
    /// only a copy whose end was received is forgotten.
    ///  One wait runs at a time,
    /// so a second caller cannot find the copies taken out and answer for them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const settled = service.awaitCopies(10_000); // every copyEnded arrived; a give-up leaves the rest queued
    /// ```
    /// What:
    ///  Records the end of a copy of a service's stream that a control started itself,
    ///  so that one wait sees every copy of this fixture's words.
    /// Why:
    ///   A control that judges this wait holds a copy open on purpose,
    /// so it starts the pipe itself,
    /// and a wait that saw only the copies the fixture started would not be waiting for the one the control made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// service.recordACopy(copyEnded); // the wait now sees the copy the control holds open
    /// ```
    pub fn record_a_log_copy(
        &self,
        when_the_copy_ends: std::sync::mpsc::Receiver<()>,
    ) {
        self.log_copies
            .lock()
            .expect("the fixture's log copies")
            .push(when_the_copy_ends);
    }

    pub fn settle_the_service_log(
        &self,
        bound: std::time::Duration,
    ) -> bool {
        // The lock is held for the whole wait: a caller that found the copies taken out would answer that every copy
        // ended while this one is still waiting for it.
        let mut copies = self.log_copies.lock().expect("the fixture's log copies");
        let began: std::time::Instant = std::time::Instant::now();
        loop {
            let reached: Result<(), std::sync::mpsc::RecvTimeoutError> = {
                let Some(ended) = copies.first() else {
                    return true;
                };
                ended.recv_timeout(bound.saturating_sub(began.elapsed()))
            };
            match reached {
                // This copy ended, and its end is the only thing a wait may forget.
                Ok(()) => {
                    let _ended = copies.remove(0);
                }
                // A copy that has not ended, or whose copier is gone without a word, is not an end: it stays for the
                // next wait, and the answer is that not every copy ended.
                Err(_) => return false,
            }
        }
    }
}

/// How long a copy of a service's stream may take to end once the services holding it have ended:
/// the pipe closes with the last of them,
/// so a copy that has not ended by this bound means something still holds the stream,
/// and the read that waited says the log it sees may be incomplete.
pub(crate) const SERVICE_LOG_END_BOUND: std::time::Duration = std::time::Duration::from_secs(10);

/// What:
///  Waits, bounded, for every copy of a service's stream this fixture started to end,
///  and answers whether they all did.
/// Why:
///  A copy runs on a thread of its own, so `SERVICE_LOG_FILE` holds what a service wrote only once that thread has
/// written it;
/// a read that decides anything calls this first,
/// which is the event and not a sleep of a guessed length.
/// The services whose streams are waited for must have ended:
/// the copy of a running service's stream ends only when it does.
pub fn settle_the_service_log(
    fixture: &Fixture,
    bound: std::time::Duration,
) -> bool {
    let service = fixture.service.lock().expect("the fixture's service lock");
    return service.as_ref().is_none_or(|services| {
        return services.settle_the_service_log(bound);
    });
}

/// What:
///  Waits, bounded, for the service of a worktree or of its own Git directory to end, and answers how it ended.
/// Why:
///   A control whose subject is a service that ends reads that end as the event it is:
///  the child's own exit status,
/// never a line the service wrote about itself,
/// which arrives through a copy thread and says only what the service claimed.
///  The bound is the control's, and a service still running when it passes is one that did not end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const end = await serviceEndOf(fixture, repo, 10_000); // { kind: 'ended', status } | 'still-running' | 'not-served'
/// ```
pub fn wait_for_the_service_to_end(
    fixture: &Fixture,
    worktree_or_its_git_dir: &Path,
    bound: std::time::Duration,
) -> ServiceEnd {
    // The services are recorded by the worktree's own Git directory, which is what `serve` keys them by.
    let named_git_dir: PathBuf = worktree_or_its_git_dir.join(".git");
    let own_git_dir: PathBuf = if named_git_dir.exists() {
        named_git_dir
    } else {
        worktree_or_its_git_dir.to_path_buf()
    };
    let key: PathBuf = std::fs::canonicalize(own_git_dir.as_path()).unwrap_or(own_git_dir);
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        let reached: Option<Option<std::process::ExitStatus>> = {
            let service = fixture.service.lock().expect("the fixture's service lock");
            let Some(services) = service.as_ref() else {
                return ServiceEnd::NotServed;
            };
            let mut children = services.children.lock().expect("the fixture's services");
            children.get_mut(&key).map(|child| {
                return child.try_wait().ok().flatten();
            })
        };
        match reached {
            None => return ServiceEnd::NotServed,
            Some(Some(status)) => return ServiceEnd::Ended(status),
            Some(None) => {}
        }
        if began.elapsed() > bound {
            return ServiceEnd::StillRunning;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
/// A fixture goes away however its test ends:
/// a panic unwinds through this too.
impl Drop for Fixture {
    fn drop(&mut self) {
        // The service goes first, and it is stopped here rather than at the end of this function:
        // it writes into the fixture, and a look of it that outlived the tree removal would report a repository that is
        // gone, which is what the two service lines of issue 6345's journal were.
        // A binding that begins with an underscore lives to the end of the function, which is after the tree goes.
        let stopped: Option<FixtureService> = match self.service.lock() {
            Ok(mut service) => service.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        drop(stopped);
        // Then whatever product code started here and a failed control never ended (a hook that waits with one `sleep`).
        end_processes_working_in(self.root.as_path());
        settle_fixture(
            self.root.as_path(),
            std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some(),
        );
        // Then the folders this control's sockets stood in, which are this fixture's own and nobody else's.
        if let Ok(folders) = self.socket_folders.lock() {
            for folder in folders.made_folders() {
                settle_fixture(
                    folder.as_path(),
                    std::env::var_os(KEEP_FIXTURES_VARIABLE).is_some(),
                );
            }
        }
    }
}

/// The shell condition that holds while the test that started a fixture script is still there.
#[path = "../testing/test_alive.rs"]
mod test_alive;

pub use test_alive::{
    end_processes_working_in,
    while_test_lives,
};
