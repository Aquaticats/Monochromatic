//! What:
//!  The line a commit prints once when the scratch step it stopped was not released:
//!  the `git commit` of the scratch repository,
//!  its process,
//!  the state that shows the kernel still holds it,
//!  what became of the person's work,
//!  and that the terminal stays here.
//! Why:
//!  A `SIGKILL` decides what a process does when it is next given a processor,
//!  and a process inside an uninterruptible call makes no such transition until that call returns
//!  (`signal(7)`,
//!  `doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`).
//!  So the ceiling's stop ends nothing while the kernel holds the step,
//!  and the commit waits for the kernel rather than for its own ceiling.
//!  The owner's rule is that nothing waits for ever,
//!  and his answer on this one case (2026-11-10,
//!  "Keep waiting and say so") is that the wait stays and the command says why it is still here:
//!  the words of a stop come only after the kernel lets go,
//!  and a step it never releases never reaches them.
//!  The line also says what is not at risk,
//!  because a person cannot ask a command that has not returned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! heldByTheKernelLine({ step, waited })
//! ```

/// Time spans,
/// for the one figure the line names.
use std::time::Duration;

/// What:
///  What the line is built from,
///  and what each figure is.
/// Why:
///   One stop of one step has these facts and no others,
///  and each of them is something the person checks with `ps` or waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldByTheKernel = { step: number; waited: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeldByTheKernel {
    /// The process number of the `git commit` the commit runs in its scratch repository.
    pub step: i32,
    /// How long that step had lasted when the ceiling sent the stop.
    pub waited: Duration,
}

/// What:
///  The line a commit prints once when the step it stopped is still held by the kernel.
/// Why:
///   The step was killed at its ceiling and cannot act on that signal,
///  so nothing is committed from it,
///  and the command says so before it says anything about waiting:
///  a person reads this line while their terminal is unusable and their commit unfinished.
///  The state letter is named because it is what the person sees in `ps`,
///  and the patience is named as the kernel's,
///  because nothing in this command can end that call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ... (process ${step}) ... the kernel has not released it ...`
/// ```
pub fn held_by_the_kernel_line(held: HeldByTheKernel) -> String {
    let HeldByTheKernel { step, waited } = held;
    return format!(
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process {step}) was sent `SIGKILL` when its ceiling came \
         after {} seconds, and the kernel has not released it: it is in uninterruptible sleep (state `D`), so it ends when the call it is inside returns. \
         Nothing was committed, and nothing of yours is at risk: your index and files are as they were. The terminal stays here until the kernel lets go.\n",
        waited.as_secs()
    );
}

/// The line and its controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_held_by_the_kernel_tests.rs"]
mod tests;
