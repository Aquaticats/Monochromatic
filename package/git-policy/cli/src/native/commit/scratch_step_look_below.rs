//! What:
//!  One look at the step the commit waits for and at the processes below it:
//!  their numbers,
//!  names,
//!  state letters and processor times.
//! Why:
//!  The notice asks this same question many times while a `git commit` runs,
//!  so a look is one read of one small file per process,
//!  from the kernel's per-thread child lists,
//!  and it falls back to a pass over `/proc` only where the kernel has no child lists.
//!  It answers nothing at all when `/proc` could not be read:
//!  a look that failed is a different fact from a step that ended,
//!  and a failed look must leave the ceiling armed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const below = lookBelow(step); // [{ pid, name, state, cpu }], undefined when the look failed
//! ```

/// One process of a tree.
use crate::process_tree::TreeMember;
/// The kernel's name,
/// state and processor time of one process.
use crate::process_tree::ProcessUse;
/// What:
///   the process numbers below this one,
/// from the kernel's per-thread child lists.
use crate::process_tree::descendant_numbers_from_thread_lists;
/// The processes below a process,
/// read from `/proc` where the kernel has no child lists.
use crate::process_tree::descendants_of_read;
/// What:
///   one process's name,
/// state and processor time,
/// from one `/proc/<pid>/stat`.
use crate::process_tree::use_of;
/// Time spans,
/// for the processor time a look reports.
use std::time::Duration;

/// The step and its programs,
/// read the way a control may answer for them.
#[cfg(all(test, target_os = "linux"))]
#[path = "scratch_step_look_below_control_window.rs"]
pub(crate) mod the_controls_window;

/// What:
///  The step and every living process below it,
/// with their names,
/// states and processor times.
/// Why:
///   The step's own read comes first,
/// so a look that finds it and nothing else still arms the ceiling.
///  One read of one small file per process,
/// from the kernel's per-thread child lists,
/// so a notice that looks many times stays cheap:
/// a walk costs 19.6 microseconds and a pass over `/proc` 17.6 ms on 2026-11-09 (`process_tree.rs`).
///  `/proc` itself is read only where the kernel has no child lists.
///  A process the kernel holds as a zombie is not below the step any more,
/// as `descendants_of` counts it.
///  Nothing is answered when `/proc` could not be read at all:
/// that is a look that failed,
/// never a step that ended.
///  A control may state that one process is inside a call a signal cannot end,
/// or that a look cannot be made,
/// because neither can be produced on this host (`the_controls_window`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = lookBelow(step); // [{ pid, name, state, cpu }], undefined when the look failed
/// ```
pub(crate) fn look_below(step: i32) -> Option<Vec<(i32, ProcessUse)>> {
    #[cfg(all(test, target_os = "linux"))]
    if the_controls_window::a_control_says_this_look_cannot_be_made() {
        return None;
    }
    let numbers: Vec<i32> = match descendant_numbers_from_thread_lists(step) {
        Some(found) => found,
        None => descendants_of_read(step)?.iter().map(member_number).collect(),
    };
    let mut found: Vec<(i32, ProcessUse)> = Vec::new();
    // The step itself, so a look that finds it and nothing else still arms the ceiling.
    if let Some(used) = use_of(step)
        && used.state != 'Z'
    {
        found.push((step, used));
    }
    let mut read_below: bool = false;
    for pid in numbers.iter().copied() {
        if let Some(used) = use_of(pid) {
            read_below = true;
            if used.state != 'Z' {
                found.push((pid, used));
            }
        }
    }
    // A walk that named processes and let none of them be read is a look that failed,
    // never a step with nothing below it.
    if !numbers.is_empty() && !read_below {
        return None;
    }
    // What: The state a control states for one process of this look.
    // Why:  The kernel's answer about a process in an uninterruptible call is what such a control states,
    //       so a look answers the step the way it answers every other fact it reads.
    #[cfg(all(test, target_os = "linux"))]
    if let Some((held, name)) = the_controls_window::a_control_says_the_kernel_holds() {
        the_state_a_control_states(&mut found, step, (held, name));
    }
    return Some(found);
}

/// What:
///  Put the state a control stated for one process into a look.
/// Why:
///   The step comes first in a look,
///  so a control that names the step keeps that place,
///  and one that names a program below it keeps the place a program has.
#[cfg(all(test, target_os = "linux"))]
fn the_state_a_control_states(found: &mut Vec<(i32, ProcessUse)>, step: i32, held: (i32, String)) {
    let (number, name) = held;
    let stated: ProcessUse = ProcessUse { name, started: 0, state: 'D', cpu: Duration::ZERO };
    // What: A control marks a process the look found, and never adds one.
    // Why:  A control's words are the kernel's answer about a process of the step's own tree,
    //       and a process a control named that no look found is not one this step started:
    //       a plant that added it would let one control's statement into another test's look.
    match found.iter().position(|entry| { return entry.0 == number; }) {
        Some(at) => found[at] = (number, stated),
        None if number == step => found.insert(0, (number, stated)),
        None => {}
    }
}

/// What:
///  The process the kernel holds in a disk wait,
///  below a step,
///  read in one look.
/// Why:
///  A ceiling that comes for a step the kernel holds must stand aside instead of ending it,
///  and a step inside an uninterruptible call is the one fact that tells those two apart.
///  The look is the one the notice also makes,
///  and a control may state its answer
///  (`the_controls_window`).
///  A look that could not be made is answered as itself and never as a step with nothing held:
///  the two are different facts, and reading one as the other is how a held step was killed
///  (finding 1 of the 2026-11-12 review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = whatTheLookSays(pid); // { held } | 'not-held' | 'could-not-look'
/// ```
pub(crate) fn what_the_look_says(step: u32) -> WhatTheLookSays {
    let number: i32 = match i32::try_from(step) {
        Ok(number) => number,
        // A number no look can name is a look that proved nothing about the step.
        Err(_) => return WhatTheLookSays::CouldNotLook,
    };
    let below: Vec<(i32, ProcessUse)> = match look_below(number) {
        Some(found) => found,
        None => return WhatTheLookSays::CouldNotLook,
    };
    return match the_process_the_kernel_holds(below.as_slice()) {
        Some((found, used)) => WhatTheLookSays::TheKernelHoldsIt(crate::scratch_step_held_by_the_kernel::HeldProgram { number: *found, name: used.name.clone() }),
        None => WhatTheLookSays::NotHeld,
    };
}

/// What:
///  What one look at a step answered about the kernel holding it.
/// Why:
///  A process of the step the kernel holds,
///  a look that found none of them in that state,
///  and a look that could not be made at all
///  are three different facts,
///  and a ceiling must treat the last one as nothing known:
///  reading it as a step with nothing held is what ended a held step at the usual ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheLookSays = { held: HeldProgram } | 'not-held' | 'could-not-look';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WhatTheLookSays {
    /// The process the kernel is holding,
    /// which is the step itself or a program it started.
    TheKernelHoldsIt(crate::scratch_step_held_by_the_kernel::HeldProgram),
    /// No process of the step is in an uninterruptible call.
    NotHeld,
    /// `/proc` could not be read,
    /// so nothing about the step is known.
    CouldNotLook,
}

/// The number of one member of a tree.
fn member_number(member: &TreeMember) -> i32 {
    return member.pid;
}

/// The processor time one process of a look has used.
pub(crate) fn process_cpu(entry: &(i32, ProcessUse)) -> Duration {
    return entry.1.cpu;
}

/// What:
///  Whether a process of a look is inside a call a signal cannot end.
/// Why:
///   A process in uninterruptible sleep is in a kernel path it has not returned from,
///  so `SIGKILL` stays pending until it does,
///  and a step in this state is the one wait a ceiling cannot end.
pub(crate) fn process_is_blocked(entry: &(i32, ProcessUse)) -> bool {
    return entry.1.state == 'D';
}

/// What:
///  The process of a look that the kernel is holding in a call a signal cannot end,
///  and never a process the look found in any other state.
/// Why:
///   A `git commit` that started a signing program can be blocked through that program rather than itself,
///  and a line that named the step for a state the step was never seen in states a false fact about it
///  (finding 4 of the 2026-11-10 review of the held-step notice).
///  The step comes first in a look,
///  so a look that holds both names the step,
///  which is the process the person started the wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = below.find(processIsBlocked); // { pid, name, state: 'D' } | undefined
/// ```
pub(crate) fn the_process_the_kernel_holds(below: &[(i32, ProcessUse)]) -> Option<&(i32, ProcessUse)> {
    for entry in below {
        if process_is_blocked(entry) {
            return Some(entry);
        }
    }
    return None;
}

/// The look and its controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_look_below_tests.rs"]
mod tests;
