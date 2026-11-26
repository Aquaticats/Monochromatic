//! What:
//!  The one shape a test's helper process is started with:
//!  a group of its own that dies with the test thread and with this process,
//!  and the group written down so the thread's fixtures can end it.
//! Why:
//!  `Child` does not kill on drop,
//!  so a child a failing assertion never reached the end of (a service whose idle exit is hours away,
//!  a shell loop) kept running after its test:
//!  a forced-removal fixture's child outlived its run by 45 minutes (issue 6463),
//!  and a test binary ended by `SIGKILL` or by the watchdog's `abort` runs no guard at all.
//!  The group kill is the guard, and the kernel's parent-death signal is what still runs when a
//!  process is killed outright.
//!  Written once here, every target that starts such a child answers the same way:
//!  the library's tests, the binary controls' crate and the candidate consumer's crate include this file by path.
//! Gotcha:
//!  A child that must lead a session of its own cannot be started here, because a group leader cannot start a session;
//!  `child_test_process` starts that one with `own_group` alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const child = spawnOwned(command);   // its own group, killed with the test thread and with the process
//! ```

/// What:
///  `Command` is the recipe a helper is started from and `Child` the running helper.
/// Why:
///  This file sets the group and the parent-death signal on the recipe, then hands back the child it started.
use std::process::{Child, Command};

/// What:
///  The process groups of the helper processes this thread started,
///  killed when the thread's fixtures go and when the thread ends.
///  The siblings a reader might expect are a `Mutex` around one list (which two threads would share) and a list per
///  process (which one thread's failing test would not end).
/// Why:
///  A group belongs to the thread that started it, so a failing test ends its own helpers and no other test's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ownedGroups = new ThreadLocal<{ groups: number[] }>(() => ({ groups: [] }));
/// ```
struct OwnedGroups {
    /// The group number of each helper this thread started.
    groups: std::cell::RefCell<Vec<libc::pid_t>>,
}

/// What:
///  A thread that ended runs this, so the helpers one target starts are ended even where nothing else of its fixtures
///  runs.
/// Why:
///  A test binary is many tests as threads of one process, and the thread that started a helper is gone when the next
///  test runs; a helper still going would outlive the test it was started for (issue 6463).
impl Drop for OwnedGroups {
    fn drop(&mut self) {
        end_groups(self.groups.borrow().as_slice());
    }
}

thread_local! {
    /// What:
    ///  The groups of the helpers this thread started.
    /// Why:
    ///  Every helper of one thread is ended at the same two moments: the thread's fixtures going, and the thread ending.
    static OWNED_GROUPS: OwnedGroups = const { OwnedGroups { groups: std::cell::RefCell::new(Vec::new()) } };
}

/// What:
///  `killpg` every group in `groups`, with `SIGKILL`, and read no memory of theirs.
/// Why:
///  A helper leads a group of its own, so this reaches it and everything it started;
///  a group with no member left answers `ESRCH`, which is the state this wants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const group of groups) process.kill(-group, 'SIGKILL');
/// ```
fn end_groups(groups: &[libc::pid_t]) {
    for group in groups {
        // SAFETY: `killpg` takes a group number and a signal and touches no memory; a group with no member is `ESRCH`.
        unsafe {
            libc::killpg(*group, libc::SIGKILL);
        }
    }
}

/// What:
///  End every helper group this thread started, now,
///  which is what the thread's fixtures do before they remove the directories those helpers work in.
/// Why:
///  A helper still running holds its fixture directory open, and a directory removed under a live process is a
///  failure the next control reads as its own;
///  the fixtures call this first, so the helpers die before their directories go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endOwnedGroups(): void { endGroups(ownedGroups.get().groups); ownedGroups.get().groups.length = 0; }
/// ```
#[allow(dead_code, reason = "the library's thread fixtures are what call this before their directories go; a target that keeps no fixtures ends its groups with the thread's own end")]
pub(crate) fn end_owned_groups() {
    // The list is taken first and emptied after, so a group ended twice is not an error and no group is missed.
    // A thread whose own list was destroyed before its fixtures is the case `try_with` answers: that list's `Drop`
    //  ended its groups already, so there is nothing here to end.
    let groups: Vec<libc::pid_t> = OWNED_GROUPS
        .try_with(|owned| {
            return std::mem::take(&mut *owned.groups.borrow_mut());
        })
        .unwrap_or_default();
    end_groups(groups.as_slice());
}

/// What:
///  Start a helper process that dies with the test thread that started it,
///  whether the test passes, fails, panics, or the whole test process is killed.
/// Why:
///   A shell loop that a failing push-progress test never reached the end of ran for 87 minutes in a deleted fixture.
/// The
///  helper leads a process group of its own,
/// so what it starts (a `sleep` of the loop) is killed with it,
///  and the group is
///  killed when the thread's fixtures are dropped,
/// the same moment the fixture directories go.
///  The group kill is a guard this process runs,
///  and a process ended by `SIGKILL` or by `run_progress`'s `abort` runs no guard,
///  so the helper also asks the kernel for a parent-death signal,
///  which ends it when the thread that started it is gone however the process ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spawnOwned(command: Command): Child { command.group = 0; command.onFork = () => prctl(PR_SET_PDEATHSIG, SIGKILL); const child = command.spawn(); ownedGroups.get().groups.push(child.pid); return child; }
/// ```
pub(crate) fn spawn_owned(command: &mut Command) -> std::io::Result<Child> {
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
    // SAFETY: between `fork` and `exec` only `prctl`, an async-signal-safe call, runs, and it touches no memory of this process.
    //  A test binary ended by `SIGKILL`, or by the `abort`s of `run_progress`, runs no guard at all, so `own_group`'s kill
    //  never happens; the kernel's parent-death signal is what ends the child then.
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
            return Ok(());
        });
    }
    let child: Child = command.spawn()?;
    own_group(&child);
    return Ok(child);
}

/// What:
///  Make the process group a child leads die with the test thread,
/// as `spawn_owned` does for the children it starts.
/// Why:
///   A helper that must become a session leader (a terminal helper calls `setsid` between `fork` and `exec`) cannot also be put in
///  a group by `spawn_owned`,
/// because a group leader cannot start a session.
/// Such a child leads a group of its own already,
///  so only the thread's list of groups to end needs its number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownGroup(child: Child): void { ownedGroups.get().groups.push(child.pid); }
/// ```
pub(crate) fn own_group(child: &Child) {
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    OWNED_GROUPS.with(|owned| { owned.groups.borrow_mut().push(group); });
}
