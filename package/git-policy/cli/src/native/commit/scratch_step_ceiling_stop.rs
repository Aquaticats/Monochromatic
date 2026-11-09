//! What:
//!  What the ceiling of the scratch `git commit` does when it comes:
//!  ends the step it watches and what that step started,
//!  and answers whether the step itself was signalled.
//! Why:
//!  Three answers have to stay apart:
//! the step ended by itself,
//! so the commit lands;
//! a signal was really taken,
//! so the step was stopped;
//! and nothing could be proven,
//! so nothing is decided and the ceiling stays armed.
//!  The step is named by the descriptor the kernel made the moment it started
//! (`ChildNumber::descriptor`),
//! never by a read of its number at the ceiling:
//! a read then proves the process found then,
//! and a step that ended and had its number reused would be signalled through the successor
//! (finding 1 of the second review).
//!  A step whose start gave no descriptor is not signalled here at all:
//! finding a number and opening a descriptor for it are two operations,
//! so the signal is not own-child-safe,
//! and the wait that owns the step ends it at the ceiling instead
//! (`child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`).
//!  Its children are read only while the descriptor says the step is still that process,
//! because that is the only moment the number still names it:
//! a number that cannot be reused is the one a live process holds.
//!  A record that could not be read proves nothing (finding 3),
//! a stop is recorded only when a signal was really sent (finding 5 of the first review),
//! and no signal travels to a number this command read (finding 5 of the fifth review).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const came = ceilingCame(step); // 'the-step-ended' | 'stopped' | 'unknown'
//! ```

/// The step,
/// published when it started,
/// and the descriptor that names it for good.
use crate::child_number_told_at_start::ChildNumber;
/// The processes below the step,
/// read for the stop.
use crate::process_tree::descendants_of_read;
/// One process of that tree.
use crate::process_tree::TreeMember;
/// The signal that ends the step,
/// what it came to,
/// and whether the step is still there.
use crate::member_signalled_through_its_pidfd::{Delivered, has_ended, kill_members, signal_through};

/// What:
///  What the ceiling came to when it came.
/// Why:
///  Three answers the notice must keep apart:
/// the step ended by itself,
/// so the commit lands;
/// a signal was really sent,
/// so the step was stopped;
/// and nothing could be proven or read,
/// so nothing is decided and the ceiling stays armed.
///  Treating a failed look as an empty one takes the ceiling away from a step that is still running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CeilingCame = 'the-step-ended' | 'stopped' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CeilingCame {
    /// Nothing is below this process any more:
    /// the step ended by itself.
    TheStepEnded,
    /// The kernel took a signal for the step.
    Stopped,
    /// The processes below this one could not be read at all,
    /// or nothing about them could be proven,
    /// so nothing is concluded:
    /// the ceiling stays armed and the look is made again.
    Unknown,
}

// What:
//  A control that plants what the walk below the step answers.
// Why:
//   A step that keeps a live child and ends in the same instant cannot be built from real processes:
//   a child is reparented the moment its parent exits,
//   so the walk after a real death finds nothing and the gate is never exercised.
//  A control of this process's own thread.
#[cfg(all(test, target_os = "linux"))]
thread_local! {
    // The answer the next walk gives.
    static PLANTED_WALK: std::cell::RefCell<Option<Option<Vec<TreeMember>>>> = const { std::cell::RefCell::new(None) };
}

/// Plant what the next walk below the step answers.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn plant_what_the_walk_finds(members: Vec<TreeMember>) {
    PLANTED_WALK.with(|planted| { *planted.borrow_mut() = Some(Some(members)); });
}

/// Plant a walk that could not be made at all.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn refuse_the_next_walk() {
    PLANTED_WALK.with(|planted| { *planted.borrow_mut() = Some(None); });
}

/// The processes below the step:
/// the control's answer when one is planted,
/// and the kernel's walk otherwise.
#[cfg(target_os = "linux")]
fn walk_below(number: i32) -> Option<Vec<TreeMember>> {
    #[cfg(test)]
    if let Some(planted) = PLANTED_WALK.with(|planted| { return planted.borrow_mut().take(); }) {
        return planted;
    }
    return descendants_of_read(number);
}

/// What:
///  What the ceiling came to,
/// and whether the programs below the step could be listed.
/// Why:
///   A step that is signalled while the walk below it failed leaves whatever it started to be named in the record and in the
/// stop's words,
/// so the person is told that something below may still run (finding 2 of the fourth review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheCeiling = { came: CeilingCame; belowListed: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TheCeiling {
    /// What the ceiling came to.
    pub came: CeilingCame,
    /// Whether the programs below the step were listed.
    pub below_listed: bool,
}

/// What:
///  End the step the ceiling watches and what it started,
/// and answer what that came to.
/// Why:
///   The step is signalled through the descriptor made at its start,
/// so no read of its number can name a successor.
/// Its children are read only while that descriptor says it is still there,
/// which is the only moment its number is still its own.
///  A step that ended is not signalled and its old number is not walked:
/// children it left behind are another process's business now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const came = ceilingCame(step); // an unreadable tree: 'unknown'
/// ```
pub fn ceiling_came(step: &ChildNumber) -> TheCeiling {
    let Some(number) = step.number() else {
        // A step that never started has nothing to end, and nothing is concluded about it.
        return TheCeiling { came: CeilingCame::Unknown, below_listed: true };
    };
    // What:
    //  A start the kernel refused a descriptor for names nothing here for good,
    //  so nothing is signalled and nothing is concluded.
    // Why:
    //   A number is safe only while the process is this command's own unreaped child,
    //   and proving that and opening a descriptor for it are two operations:
    //   a step that ended and was reaped in the interval leaves a number another process can hold,
    //   which is finding 5 of the fifth review.
    //  The commit stays bounded without this path:
    //  the wait that owns the step ends the child it holds (`child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`),
    //  so a ceiling is never lost with this answer.
    let Some(descriptor) = step.descriptor() else {
        return TheCeiling { came: CeilingCame::Unknown, below_listed: true };
    };
    let walked: Option<Vec<TreeMember>> = walk_below(number);
    let below_listed: bool = walked.is_some();
    if has_ended(descriptor) {
        // The step ended while its children were being read, so that read proves nothing and nothing from it is signalled.
        return TheCeiling { came: CeilingCame::TheStepEnded, below_listed };
    }
    // The step is signalled whatever the walk came to: the descriptor names it, and a walk that could not be made says
    // nothing about it (finding 2 of the fourth review).
    let answer: Delivered = signal_through(descriptor, libc::SIGKILL);
    let _ended_below: Vec<Delivered> = match walked {
        Some(below) => kill_members(below.as_slice()),
        None => Vec::new(),
    };
    let came: CeilingCame = match answer {
        Delivered::Sent => CeilingCame::Stopped,
        Delivered::Gone => CeilingCame::TheStepEnded,
        Delivered::Unproven => CeilingCame::Unknown,
    };
    return TheCeiling { came, below_listed };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_ceiling_stop_tests.rs"]
mod tests;
