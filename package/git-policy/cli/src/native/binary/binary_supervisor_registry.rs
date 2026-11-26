//! What:
//!  The one registry of watched commands this process keeps,
//!  the short-lived holds a test and a wait take on a command's entry,
//!  and the lock that makes them safe to share between the test thread and the monitor thread.
//! Why:
//!   The monitor is one thread for the whole test process,
//!   and every test thread asks it the same questions (is this command watched,
//!   was it ended,
//!   how many looks has it had),
//!  so the answers live in one value,
//!  guarded by one lock,
//!  rather than in a copy each caller would have to find.
//!  The holds are the two states a kill is judged by:
//!  a thread is occupied with a command (it will claim its kill as soon as this host schedules it),
//!  or its ending is still going on (so the grace has not begun).
//!  Neither is a duration:
//!  what a hold records is a state,
//!  which is why a starved host cannot make the guard abort over a record nobody could claim yet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registry = { watched: new Map(), kills: [], holds: new Map(), diskWaitLooks: new Map(), looks: new Map(), occupied: new Map() };
//! ```
use super::supervisor_judge::Judge;
use super::supervisor_pin::Pin;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// A command the monitor ended.
#[derive(Clone, Debug)]
pub(super) struct Kill {
    /// The process ID of the command.
    pub(super) pid: i32,
    /// The message.
    pub(super) message: String,
    /// When it was recorded,
    /// which is before the first signal.
    pub(super) at: Instant,
}

/// One command the monitor watches.
pub(super) struct Watched {
    /// The judge with its limits.
    pub(super) judge: Judge,
    /// When the command was first seen.
    pub(super) began: Instant,
    /// The command's process,
    /// pinned.
    pub(super) pin: Pin,
}

/// What the monitor shares with the tests.
#[derive(Default)]
pub(super) struct Registry {
    /// The commands watched,
    /// by process ID.
    pub(super) watched: HashMap<i32, Watched>,
    /// The commands ended and not yet claimed.
    pub(super) kills: Vec<Kill>,
    /// How many tests hold the idle judgment of a command,
    /// by process ID.
    pub(super) holds: HashMap<i32, u32>,
    /// How many looks found a task of the command waiting for the disk,
    /// by process ID,
    /// kept after the command ends.
    pub(super) disk_wait_looks: HashMap<i32, u32>,
    /// How many looks the monitor took of the command,
    /// by process ID,
    /// kept after the command ends.
    pub(super) looks: HashMap<i32, u32>,
    /// The commands a thread of this process is occupied with,
    /// by process ID,
    /// each with how many threads hold it.
    /// Why:
    ///  A kill that is not claimed aborts the process only when nothing here is occupied with its command:
    ///  a wait that is running claims it as soon as this host schedules that thread,
    ///  and a kill whose ending is still going on cannot be claimed at all until the ending is over.
    ///  What this map holds is a state,
    ///  never how much time has passed.
    pub(super) occupied: HashMap<i32, u32>,
}

/// The registry of this process.
static REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);

/// Runs the registry's closure under its lock.
pub(super) fn with_registry<T>(action: impl FnOnce(&mut Registry) -> T) -> T {
    let mut guard = REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    return action(guard.get_or_insert_with(Registry::default));
}

/// A test's hold on the idle judgment of a command,
/// released when it is dropped.
pub struct IdleHold {
    /// The command's process ID.
    pid: i32,
}

impl Drop for IdleHold {
    fn drop(&mut self) {
        with_registry(|registry| {
            if let Some(count) = registry.holds.get_mut(&self.pid) {
                *count -= 1;
                if *count == 0 {
                    registry.holds.remove(&self.pid);
                }
            }
        });
    }
}

/// Suspend the idle rule for the command `pid` while the returned hold lives:
/// for a wait the test itself controls (it will
///  drain the pipe,
/// release the lock or send the signal once its own work is done).
/// The processor budget and the backstop
///  keep judging.
/// A test that holds a command and never lets it go ends at the backstop.
pub fn hold_idle_judgment(pid: u32) -> IdleHold {
    let held: i32 = i32::try_from(pid).expect("a process ID fits");
    with_registry(|registry| {
        *registry.holds.entry(held).or_default() += 1;
    });
    return IdleHold {
        pid: held,
    };
}

/// What:
///  A thread's hold on one command's entry in the registry of the commands this process is occupied with,
///  released when the thread is done with it,
///  whether it returned or unwound.
/// Why:
///  One value carries both the mark and its removal,
///  so no path of a wait or of an ending can leave a command marked after the thread stopped working on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Occupied { constructor(pid) { markOccupied(pid); } [Symbol.dispose]() { stopOccupying(pid); } }
/// ```
pub struct Occupied {
    /// The command this hold marks.
    pid: i32,
}

/// A thread ends its hold here,
/// however it ended.
impl Drop for Occupied {
    fn drop(&mut self) {
        with_registry(|registry| {
            if let Some(count) = registry.occupied.get_mut(&self.pid) {
                *count -= 1;
                if *count == 0 {
                    registry.occupied.remove(&self.pid);
                }
            }
        });
    }
}

/// What:
///  Mark the command `pid` as one this thread is waiting for,
///  until the returned hold is dropped.
/// Why:
///  `support_observation_bound::wait_under_bound` is the only place a kill is claimed,
///  so a kill of a command a wait is running for is a claim that is on its way,
///  not a kill nobody reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using _waiting = waitingFor(child.pid); // cleared when the wait returns or throws
/// ```
pub fn waiting_for(pid: u32) -> Occupied {
    return occupy(pid);
}

/// What:
///  Mark the command `pid` as one whose ending is still going on,
///  until the returned hold is dropped.
/// Why:
///  The kill of a command is recorded before its tree is ended,
///  and a tree a starved host takes seconds to end leaves a record nobody can claim yet:
///  the grace counts from the moment the command is gone
///  (`end_command`).
///  So the abort must know an ending is in progress,
///  or it overruns the very grace it grants.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using _ending = endingTheCommandOf(pin.pid()); // cleared when the tree is ended
/// ```
pub fn ending_the_command_of(pid: i32) -> Occupied {
    let marked: u32 = u32::try_from(pid).expect("a command's number is positive");
    return occupy(marked);
}

/// Add one thread's mark to the command `pid`,
/// answering the hold that removes it again.
fn occupy(pid: u32) -> Occupied {
    let marked: i32 = i32::try_from(pid).expect("a process ID fits");
    with_registry(|registry| {
        *registry.occupied.entry(marked).or_default() += 1;
    });
    return Occupied {
        pid: marked,
    };
}
