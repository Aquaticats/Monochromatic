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
//! and the look that came with the ceiling failed,
//! so nothing is decided and the ceiling stays armed.
//!  Reading a failed look as an empty one takes the ceiling away from a step that is still running
//! (P1 of the 2026-11-09 review),
//! and recording a stop that no signal was taken for refuses a commit that landed
//! (P2 of the same review).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const came = ceilingCame(step); // 'the-step-ended' | 'stopped' | 'unknown'
//! ```

/// The processes below the step,
/// read for the stop.
use crate::process_tree::descendants_of_read;
/// The step's own record,
/// and whether it could be read.
use crate::process_tree::{MemberRead, member_read};
/// The signal that ends the step,
/// and what it came to.
use crate::member_signalled_through_its_pidfd::{Delivered, kill_members};

/// What:
///  What the ceiling came to when it came.
/// Why:
///   Three answers the notice must keep apart:
/// the step ended by itself,
/// so the commit lands;
/// a signal was really sent,
/// so the step was stopped;
/// and the look that came with the ceiling failed,
/// so nothing is decided and the ceiling stays armed.
///  Treating a failed look as an empty one takes the ceiling away from a step that is still running,
/// which is how a commit lands after a long step.
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
    /// The kernel took a signal for the step or a program below it.
    Stopped,
    /// The processes below this one could not be read at all,
    /// so nothing is concluded:
    /// the ceiling stays armed and the look is made again.
    Unknown,
}

/// What:
///  End what the ceiling's look found below this process,
/// and answer what that came to.
/// Why:
///   A stop is recorded only when a signal was really sent:
/// a step that ended between the look and the signal landed,
/// and refusing it as stopped would fail a commit that succeeded.
///  A look that failed is not an empty tree:
/// answering "the step ended" there takes the ceiling away from a step that is still running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const came = ceilingCame(step); // an unreadable tree: 'unknown'
/// ```
pub fn ceiling_came(step: i32) -> CeilingCame {
    let Some(below) = descendants_of_read(step) else {
        return CeilingCame::Unknown;
    };
    let stopped_the_step: bool = match member_read(step) {
        MemberRead::Found(member) => match kill_members(std::slice::from_ref(&member)).first() {
            Some(Delivered::Sent) => true,
            // Nothing could be proven about the step, so nothing is concluded about it either.
            Some(Delivered::Unproven) => return CeilingCame::Unknown,
            _ => false,
        },
        MemberRead::Gone => false,
        MemberRead::Unreadable => return CeilingCame::Unknown,
    };
    // The signing program and everything else the step started goes with it,
    // whether or not the step itself was still there to be signalled.
    let _ended_below: Vec<Delivered> = kill_members(below.as_slice());
    if stopped_the_step {
        return CeilingCame::Stopped;
    }
    return CeilingCame::TheStepEnded;
}
