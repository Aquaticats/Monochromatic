//! What:
//!  How a stop reaches the child it started and never another command:
//! the pin of a step's root taken while that child is this process's own unreaped child,
//! the signals sent through that pin,
//! and the processes below the root read through it.
//! Why:
//!  A process number is a safe address only while the process it names is this process's own unreaped child,
//! and the thread that waits for the child reaps it as soon as it ends,
//! so a number read at stopping time can name another command
//! (the third review's finding 1).
//!  The pin is therefore taken at the start and names that child for good,
//! and this file holds that rule alone;
//! `child_watch_clock_marks_and_stopping.rs` ends a child through it and re-exports what the watch and its callers name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pin = pinAtStart(child.pid); // held only if the record read then named the child
//! const delivered = sendToTheRoot(pin, pid, signal);
//! const tree = stillThere(pin) ? descendantsOf(pid) : [];
//! ```

/// The answers a signal comes to,
/// the descriptor that names one process for good,
/// and a record proven against that descriptor.
use crate::member_signalled_through_its_pidfd::{
    Delivered,
    Pinned,
    PinnedMember,
};
/// The record read of one process,
/// the processes below it,
/// and the reading of both.
use crate::process_tree::{
    MemberRead,
    TreeMember,
    descendants_of,
    member_read,
};

/// The name of `signal` as the person knows it,
/// for a line that says which signal was not sent.
#[cfg(unix)]
fn signal_name(signal: libc::c_int) -> &'static str {
    return match signal {
        libc::SIGTERM => "SIGTERM",
        libc::SIGKILL => "SIGKILL",
        _ => "a signal",
    };
}

/// What:
///  The pin of a step's root,
/// taken when the child started,
/// while it was this process's own unreaped child.
/// Why:
///  A number is a safe address only while the process it names is this process's own unreaped child,
/// and the thread
///  that waits for the child reaps it as soon as it ends.
/// A pin taken at stopping time therefore pins whatever process
///  holds the number *then*:
/// the step can have ended and been reaped while its grace ran,
/// the number can
///  have passed to another command,
/// and that command would receive the stop
/// (the third review's finding 1).
///  Taken at the start,
/// the number certainly names the child,
/// and the pin names that child for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = pinAtStart(child.pid); // held only if the record read then named the child
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) type RootPin = Pinned;

/// Without a descriptor to take a pin with,
/// nothing is ever signalled to a step's root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
#[derive(Debug)]
pub(crate) struct RootPin;

/// What:
///  The pin of the child that has just started,
/// taken while its number is certainly its own.
/// Why:
///   The child is this process's own unreaped child right after the start,
/// so the record read here describes it and
///  no other process;
/// the descriptor that record is proven against then names that child for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = pinAtStart(child.pid);
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn pin_at_start(pid: u32) -> RootPin {
    let Ok(raw) = i32::try_from(pid) else {
        return Pinned::Unproven;
    };
    let MemberRead::Found {
        member,
        ..
    } = member_read(raw)
    else {
        return Pinned::Unproven;
    };
    return PinnedMember::open(&member);
}

/// Without `/proc` nothing is known about the child's number,
/// so no pin is made and nothing is signalled.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn pin_at_start(_pid: u32) -> RootPin {
    return RootPin;
}

/// What:
///  Send `signal` to the step's root through its pin,
/// and answer what the signal came to.
/// Why:
///   A record that could not be read proves nothing about the process,
/// and a number that was not pinned may
///  name an unrelated process by now,
/// so no number is signalled;
/// the step is still waited for,
/// and one line says it
///  was not stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sendToTheRoot(pinned: Pinned, pid: number, signal: number): Delivered;
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn send_to_the_root(
    pinned: &RootPin,
    raw: i32,
    signal: libc::c_int,
) -> Delivered {
    let delivered: Delivered = match pinned {
        Pinned::Held(member) => member.signal(signal),
        Pinned::Gone => Delivered::Gone,
        Pinned::Unproven => Delivered::Unproven,
    };
    if delivered == Delivered::Unproven {
        crate::diagnostic_log::warn(
            "childWatch",
            format!(
                "the step {raw} was not sent {}: nothing proved which process its number named, so no number was signalled. The \
                 step is still waited for; check with `ps` whether it runs and what it waits for.",
                signal_name(signal)
            )
            .as_str(),
        );
    }
    return delivered;
}

/// A platform without a descriptor cannot signal the step's root;
/// the wait it announces is all it has.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn send_to_the_root(
    _pinned: &RootPin,
    raw: i32,
    signal: libc::c_int,
) -> Delivered {
    crate::diagnostic_log::warn(
        "childWatch",
        format!(
            "the step {raw} was not sent {}: this platform gives no descriptor to name a process with, so no number is signalled. \
             Check with `ps` whether it runs and what it waits for.",
            signal_name(signal)
        )
        .as_str(),
    );
    return Delivered::Unproven;
}

/// What:
///  Whether the step's root is still there,
/// as the pin taken when the child started tells it.
/// Why:
///   The pin names the child for good,
/// so a pin that reports the process gone means the step itself ended;
/// the status its wait
///  collected is then the answer,
/// and nothing of a stop may replace it
/// (the fourth review's finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillThere = (pin) => pin.kind === 'held' || pin.kind === 'unproven';
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn the_root_is_still_there(pin: &RootPin) -> bool {
    return match pin {
        // The descriptor becomes readable when the process it names exits, which answers this without a signal.
        Pinned::Held(member) => !member.has_ended(),
        Pinned::Gone => false,
        // Nothing proved which process the number names, so the stop goes on as if the root were there: a bound the step keeps
        //  is never lost to a record that could not be read.
        Pinned::Unproven => true,
    };
}

/// What:
///  Whether the pin of a step's root proves which process to signal.
/// Why:
///   A stop signals the root only through the descriptor taken when the child started,
/// and a pin that proves nothing signals
///  nothing:
/// the watch then cannot end the step at its ceiling,
/// and the wait for the step's output is what does,
///  at the ceiling itself rather than after the watch has had its ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = (pin) => pin.kind === 'held';
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn the_root_is_proven(pin: &RootPin) -> bool {
    // A pin that reports the child gone names no process either, so nothing is signalled for it and the step is the wait's to end.
    return matches!(pin, Pinned::Held(_));
}

/// Without a descriptor nothing is proven,
/// so no watch can end the step it watches.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn the_root_is_proven(_pin: &RootPin) -> bool {
    return false;
}

/// What:
///  The processes below a step's root,
/// read only while its pin proves the root is still the process it names.
/// Why:
///  Once the root is reaped its number may name another process,
/// and the children of that process are not this
///  step's:
/// enumerating them would signal a command of the person's that has nothing to do with the stop
///  (the fourth review's finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = stillThere(pin) ? descendantsOf(pid) : [];
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn descendants_of_a_root_that_still_runs(
    pin: &RootPin,
    raw: i32,
) -> Vec<TreeMember> {
    if !the_root_is_still_there(pin) {
        return Vec::new();
    }
    return descendants_of(raw);
}

/// Without a descriptor nothing is known about the root,
/// so the stop goes on as if it were there.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn the_root_is_still_there(_pin: &RootPin) -> bool {
    return true;
}

/// Without `/proc` no process is listed,
/// so there is no tree to read.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn descendants_of_a_root_that_still_runs(
    _pin: &RootPin,
    _raw: i32,
) -> Vec<TreeMember> {
    return Vec::new();
}
