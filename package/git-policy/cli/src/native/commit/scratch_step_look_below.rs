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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = lookBelow(step); // [{ pid, name, state, cpu }], undefined when the look failed
/// ```
pub(crate) fn look_below(step: i32) -> Option<Vec<(i32, ProcessUse)>> {
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
    return Some(found);
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

/// The look and its controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_look_below_tests.rs"]
mod tests;
