//! What:
//!  What the ceiling of the scratch `git commit` does when it comes:
//!  ends the step it watches and what that step started,
//!  and answers whether the step itself was signalled.
//! Why:
//!  Three answers have to stay apart:
//! the step ended by itself, so the commit lands;
//! a signal was really taken, so the step was stopped;
//! and nothing could be proven, so nothing is decided and the ceiling stays armed.
//!  The step is named by the descriptor the kernel made the moment it started
//! (`ChildNumber::descriptor`),
//! never by a read of its number at the ceiling:
//! a read then proves the process found then,
//! and a step that ended and had its number reused would be signalled through the successor
//! (finding 1 of the second review).
//!  Its children are read only while the descriptor says the step is still that process,
//! because that is the only moment the number still names it:
//! a number that cannot be reused is the one a live process holds.
//!  A record that could not be read proves nothing (finding 3),
//! and a stop is recorded only when a signal was really sent (finding 5 of the first review).
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
/// The step's own record,
/// and whether it could be read.
use crate::process_tree::{MemberRead, member_read};
/// The signal that ends the step,
/// what it came to,
/// and whether the step is still there.
use crate::member_signalled_through_its_pidfd::{Delivered, has_ended, kill_members, signal_through};

/// What:
///  What the ceiling came to when it came.
/// Why:
///  Three answers the notice must keep apart:
/// the step ended by itself, so the commit lands;
/// a signal was really sent, so the step was stopped;
/// and nothing could be proven or read, so nothing is decided and the ceiling stays armed.
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
pub fn ceiling_came(step: &ChildNumber) -> CeilingCame {
    let Some(number) = step.number() else {
        // A step that never started has nothing to end, and nothing is concluded about it.
        return CeilingCame::Unknown;
    };
    let stopped_the_step: bool = match step.descriptor() {
        Some(descriptor) => {
            if has_ended(descriptor) {
                return CeilingCame::TheStepEnded;
            }
            // The step is still that process, so its number still names it and its children can be read below it.
            let Some(below) = descendants_of_read(number) else {
                return CeilingCame::Unknown;
            };
            let answer: Delivered = signal_through(descriptor, libc::SIGKILL);
            let _ended_below: Vec<Delivered> = kill_members(below.as_slice());
            match answer {
                Delivered::Sent => true,
                Delivered::Gone => false,
                Delivered::Unproven => return CeilingCame::Unknown,
            }
        }
        // A start whose descriptor the kernel refused: the step is proven by its record and then pinned,
        // and a step nothing can be proven about keeps the ceiling armed.
        None => match member_read(number) {
            MemberRead::Found(member) => match kill_members(std::slice::from_ref(&member)).first() {
                Some(Delivered::Sent) => true,
                Some(Delivered::Unproven) => return CeilingCame::Unknown,
                _ => false,
            },
            MemberRead::Gone => false,
            MemberRead::Unreadable => return CeilingCame::Unknown,
        },
    };
    if stopped_the_step {
        return CeilingCame::Stopped;
    }
    return CeilingCame::TheStepEnded;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_ceiling_stop_tests.rs"]
mod tests;
