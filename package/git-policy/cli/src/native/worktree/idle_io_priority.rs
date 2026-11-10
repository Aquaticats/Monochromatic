//! What:
//!  Run one stretch of storage work at the lowest I/O priority Linux offers,
//! and put the calling thread's own priority back afterwards,
//! whether or not the work returns.
//! Why:
//!  Emptying a snapshot's trash unlinks hundreds of thousands of entries,
//! and every `unlink` starts a btrfs
//!  transaction.
//! Measured on this host on 2026-11-10
//! (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`),
//!  concurrent adds spent 2,182 of 2,796 blocked samples inside `unlink` or `rmdir` with the
//!  drive 99% busy.
//! The person's next command is what must not wait for that,
//! so the emptying asks the kernel to serve
//!  it only when nothing else wants the disk.
//! The class is I/O priority class `idle`,
//! the lowest of the three Linux offers
//! (`ioprio_set(2)`,
//!  `IOPRIO_CLASS_IDLE`),
//! and it is set on the thread that does the unlinking rather than on the whole process,
//!  because the commit service pushes a branch and reads the repository on other threads while the emptying
//!  runs.
//! A thread that cannot be lowered is left as it is:
//! the emptying is still correct,
//! only  faster,
//! and a filesystem that has no per-thread priority (a network mount,
//! a FUSE mount)  answers the same.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! withIdleIoPriority(() => removeTree(...)); // ioprio_set(thread, IDLE) ... ioprio_set(thread, before)
//! ```

/// The `ioprio_set(2)` selector for one thread:
/// `IOPRIO_WHO_PROCESS` with id `0` names the caller.
const WHO_PROCESS: libc::c_int = 1;

/// What:
///  The priority value of I/O priority class `idle`,
/// which the kernel reads as `(class << IOPRIO_CLASS_SHIFT) | data`.
/// Why:
///   The value is built here rather than imported,
/// so the class and the shift are named once and a reader can see
///  that the class is the lowest one
/// (`IOPRIO_CLASS_RT` is 1,
/// `IOPRIO_CLASS_BE` 2,
/// `IOPRIO_CLASS_IDLE` 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDLE = (3 << 13) | 0; // IOPRIO_CLASS_IDLE
/// ```
const IDLE: libc::c_int = (3 << 13) | 0;

/// What:
///  The I/O priority this thread has now,
/// or nothing when the kernel will not say.
/// Why:
///   The value is read before the work and restored after it,
/// so a thread the pool reuses for another task is left
///  exactly as it was found,
/// and one that cannot be read is never written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = ioprioGet(thread); // undefined when the call fails
/// ```
fn priority_here() -> Option<libc::c_int> {
    // The call answers the priority itself, or -1 with the reason in `errno`; nothing else is written.
    let found: libc::c_long = unsafe { libc::syscall(libc::SYS_ioprio_get, WHO_PROCESS, 0 as libc::c_int) };
    return libc::c_int::try_from(found).ok().filter(|value: &libc::c_int| return *value >= 0);
}

/// What:
///  Ask the kernel for this thread's I/O priority to be `value`;
/// whether it did.
/// Why:
///   A refusal is not an error of the emptying:
/// a filesystem or a kernel that has no per-thread priority answers
///  `EINVAL` or `ENOSYS`,
/// and the removal runs at the priority it already had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lowered = ioprioSet(thread, IDLE) === 0;
/// ```
fn set_priority_here(value: libc::c_int) -> bool {
    let answered: libc::c_long = unsafe { libc::syscall(libc::SYS_ioprio_set, WHO_PROCESS, 0 as libc::c_int, value) };
    return answered == 0;
}

/// What:
///  Run `work` with this thread's I/O priority at the lowest class,
/// and give the thread back the priority it had.
/// Why:
///   The emptying's removals run as jobs of one blocking pool,
/// so the thread that unlinks is the thread whose
///  priority matters;
/// setting it here covers every removal that goes through this call,
/// and restoring it keeps the pool's
///  other work (a push reading a pack,
/// a review reading a commit) at the priority it expects.
/// A value that cannot be read or set leaves the thread alone in that direction:
/// no write without a
///  reading to restore,
/// and no promise that the class changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withIdleIoPriority(work) { const before = priorityHere(); const lowered = setPriorityHere(IDLE); try { return work(); } finally { if (lowered && before !== undefined) setPriorityHere(before); } }
/// ```
pub fn with_idle_io_priority<T>(work: impl FnOnce() -> T) -> T {
    let before: Option<libc::c_int> = priority_here();
    let lowered: bool = set_priority_here(IDLE);
    let outcome: T = work();
    if lowered
        && let Some(value) = before
    {
        let _restored: bool = set_priority_here(value);
    }
    return outcome;
}

/// The class this thread's I/O priority names,
/// as the kernel numbers it,
/// for the controls that tell an emptying apart from ordinary work;
/// nothing when the kernel will not say.
#[cfg(test)]
pub(crate) fn class_here() -> Option<libc::c_int> {
    return priority_here().map(|value: libc::c_int| return value >> 13);
}

/// Priority controls stay out of the release executable.
#[cfg(test)]
#[path = "idle_io_priority_tests.rs"]
mod tests;
