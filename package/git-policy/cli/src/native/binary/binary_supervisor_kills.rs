//! What:
//!  The record of a command the monitor ended,
//!  from the moment it is published (before the first signal goes out) to the moment its test claims it,
//!  and the abort and the exit status a record nobody claims earns.
//! Why:
//!   A kill is never silent:
//!   the record is published before the first signal,
//!   the test that ran the command claims it right after it waited,
//!   and a record nobody claims aborts the process three seconds later,
//!   or exits it with a failing status when the process leaves first.
//!  The grace counts from the moment the command is gone,
//!  not from the kill:
//!  a tree a starved host takes seconds to end must not use up the time its test needs to claim the record.
//!  So a kill is judged together with the hold that says a thread is occupied with the command or its ending is still
//!  going on,
//!  and that hold and the registry are `binary_supervisor_registry.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorded = kill(pid); const claimed = claim(pid); if (!claimed && now - publishedAt >= grace) abort();
//! ```
use super::supervisor_limits::{
    UNCLAIMED_EXIT_STATUS,
    UNCLAIMED_GRACE,
};
use super::supervisor_pin::Pin;
use super::supervisor_registry::{
    Kill,
    Occupied,
    ending_the_command_of,
    with_registry,
};
use std::time::{
    Duration,
    Instant,
};

/// Take the message of the kill of the command `pid`,
/// if the monitor ended it;
/// the test that ran the command reports it.
///  The record is published before the first signal of the kill,
/// so a command the monitor ended has its record by the time
///  the test sees the command end.
pub fn take_kill(pid: u32) -> Option<String> {
    let ended: i32 = i32::try_from(pid).ok()?;
    return with_registry(|registry| {
        let position: usize = registry.kills.iter().rposition(|kill| {
            return kill.pid == ended;
        })?;
        return Some(registry.kills.remove(position).message);
    });
}

/// Record the kill of the command `pin` names,
/// and only then end its tree with `end`:
/// the test that waits for the command
///  sees it end after the first signal,
/// so the record has to be there before.
pub fn end_command(
    pin: &Pin,
    message: &str,
    end: &dyn Fn(&Pin),
) {
    with_registry(|registry| {
        registry.kills.push(Kill {
            pid: pin.pid(),
            message: message.to_string(),
            at: Instant::now(),
        });
    });
    // What: the mark saying this command's ending is still going on, held until the tree is ended.
    // Why:  the record above cannot be claimed until the command is gone, and a starved host can take seconds to end a
    //       tree, so the monitor must not abort over a record no test could have claimed yet.
    let _ending: Occupied = ending_the_command_of(pin.pid());
    end(pin);
    drop(_ending);
    // The test has the whole grace to claim the record from the moment the command is gone: an ending that a starved host
    //  makes slow must not use the grace up before the test has had a chance to run (a gate saw the process abort that way).
    with_registry(|registry| {
        if let Some(kill) = registry.kills.iter_mut().rev().find(|kill| {
            return kill.pid == pin.pid();
        }) {
            kill.at = Instant::now();
        }
    });
}

/// Whether a kill of the command `pid` is recorded and not claimed.
pub fn kill_recorded(pid: i32) -> bool {
    return with_registry(|registry| {
        return registry.kills.iter().any(|kill| {
            return kill.pid == pid;
        });
    });
}

/// The message of a kill nobody has claimed within `grace` of `now`,
/// if there is one.
pub fn unclaimed_kill(
    now: Instant,
    grace: Duration,
) -> Option<String> {
    return with_registry(|registry| {
        return registry
            .kills
            .iter()
            .find(|kill| {
                return now.saturating_duration_since(kill.at) >= grace;
            })
            .map(|kill| {
                return kill.message.clone();
            });
    });
}

/// Record a kill as the monitor would,
/// for the control that reads what happens to one nobody claims.
pub fn plant_kill(
    pid: i32,
    message: &str,
    at: Instant,
) {
    with_registry(|registry| {
        registry.kills.push(Kill {
            pid,
            message: message.to_string(),
            at,
        });
    });
}

/// What:
///  The message of a kill past `grace` that no thread of this process is occupied with,
///  if there is one.
/// Why:
///  This is what the monitor aborts on:
///  a kill whose command no thread of the test is waiting for or ending,
///  which no amount of time will make anybody claim.
///  A command a thread is occupied with is left to that thread,
///  because its claim happens as soon as this host schedules it
///  (issue 6469).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killToAbort = (now, grace) => kills.find((k) => now - k.at >= grace && !occupied.has(k.pid));
/// ```
pub fn kill_to_abort(
    now: Instant,
    grace: Duration,
) -> Option<String> {
    return with_registry(|registry| {
        return registry
            .kills
            .iter()
            .find(|kill| {
                return now.saturating_duration_since(kill.at) >= grace
                    && !registry.occupied.contains_key(&kill.pid);
            })
            .map(|kill| {
                return kill.message.clone();
            });
    });
}

/// A kill nobody claims and no thread is occupied with ends the process,
/// so a command ended by the supervisor always fails its test.
pub(super) fn abort_on_unclaimed(now: Instant) {
    if let Some(message) = kill_to_abort(now, UNCLAIMED_GRACE) {
        eprintln!(
            "a command the supervisor ended was never reported by its test, so the process aborts:\n{message}"
        );
        std::process::abort();
    }
}

/// The exit handler:
/// a kill nobody claimed ends the process with a failing status,
/// whatever the test body concluded.
pub(super) extern "C" fn fail_unclaimed_at_exit() {
    if let Some(message) = unclaimed_kill(Instant::now(), Duration::ZERO) {
        eprintln!(
            "a command the supervisor ended was never reported by its test, so the process exits with a failure:\n{message}"
        );
        // SAFETY:
        // `_exit` ends the process at once and touches no memory of it;
        // an exit handler must not call `exit` again.
        unsafe { libc::_exit(UNCLAIMED_EXIT_STATUS) };
    }
}
