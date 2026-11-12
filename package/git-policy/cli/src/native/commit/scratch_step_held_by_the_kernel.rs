//! What:
//!  The two lines a commit prints about a step the kernel holds in a call a signal cannot end:
//!  the one it says while the step is still waited for,
//!  and the one it says when the stop it sent did not end it.
//! Why:
//!  A `SIGKILL` decides what a process does when it is next given a processor,
//!  and a process inside an uninterruptible call makes no such transition until that call returns
//!  (`signal(7)`,
//!  `doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`).
//!  A step waiting for a disk is slow,
//!  not stuck in its own work,
//!  so its ceiling stands aside and the commit waits for it:
//!  the first line says why the ceiling the person was promised did not come,
//!  how long the step has lasted,
//!  and the bound it now runs under.
//!  When even that wait runs out,
//!  the step is stopped and the kernel may still hold it,
//!  and the commit waits for the kernel rather than for its own ceiling (the owner,
//!  2026-11-10,
//!  "Keep waiting and say so"),
//!  which the second line says once.
//!  Neither line says anything of the person's is at risk,
//!  because a person cannot ask a command that has not returned.
//!  The process each line names is the one the look found in uninterruptible sleep,
//!  which may be a program the step started and not the step:
//!  a line that named the step for a state only its helper was in would state a false fact
//!  (finding 4 of the 2026-11-10 review of the held-step notice).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! aStallIsWaitedFor({ step, waited, until, held }); heldByTheKernelLine({ step, waited, held })
//! ```

/// Time spans,
/// for the figures each line names.
use std::time::Duration;
/// Why a ceiling stood aside for a step,
/// which the line about a waiting stall says.
use crate::scratch_step_stall_decided_once::WhyTheStepWasWaitedFor;

/// What:
///  What the line is built from,
///  and what each figure is.
/// Why:
///   One stop of one step has these facts and no others,
///  and each of them is something the person checks with `ps` or waits for.
///  The process the kernel holds is stated because it is not always the step:
///  a `git commit` that started a signing program is blocked through that program,
///  and naming the step would state a false fact about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldByTheKernel = { step: number; waited: number; held: HeldProgram };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeldByTheKernel {
    /// The process number of the `git commit` the commit runs in its scratch repository.
    pub step: i32,
    /// How long that step had lasted when the ceiling sent the stop.
    pub waited: Duration,
    /// The process the kernel is holding,
    /// which is the step itself or a program it started.
    pub held: HeldProgram,
}

/// What:
///  One process the kernel is holding in a call a signal cannot end:
///  its number and the kernel's name of it.
/// Why:
///   The person reads the line with `ps` in hand,
///  so the process it names must be the one `ps` shows in state `D`,
///  and the name says what that process is
///  (`process/scratch_step_look_below.rs`,
///  `the_process_the_kernel_holds`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldProgram = { number: number; name: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeldProgram {
    /// The process number the look found in uninterruptible sleep.
    pub number: i32,
    /// The name the kernel gives that process.
    pub name: String,
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
///  A program below the step that the kernel holds is named as that program,
///  and the step is still named as the commit that was stopped.
///  The line is said once and not again:
///  a person who met it three times in one failed commit read three copies of one sentence
///  (2026-11-11,
///  the release 0.6.1-alpha.11 that failed a commit in a disk wait),
///  and the command has already said how long the step waited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ... (process ${step}) ... the kernel has not released it ...`
/// ```
pub fn held_by_the_kernel_line(held: HeldByTheKernel) -> String {
    let HeldByTheKernel { step, waited, held: the_held } = held;
    let seconds: u64 = waited.as_secs();
    if the_held.number == step {
        return format!(
            "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process {step}) was sent `SIGKILL` when its ceiling came \
             after {seconds} seconds, and the kernel has not released it: it is in uninterruptible sleep (state `D`), so it ends when the call it is inside returns. \
             Nothing was committed, and nothing of yours is at risk: your index and files are as they were. The terminal stays here until the kernel lets go.\n"
        );
    }
    let HeldProgram { number, name } = the_held;
    return format!(
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process {step}) was sent `SIGKILL` when its ceiling came \
         after {seconds} seconds, and the kernel has not released it: the program it started, `{name}` (process {number}), is in uninterruptible sleep (state `D`), \
         so the commit ends when the call that program is inside returns. Nothing was committed, and nothing of yours is at risk: your index and files are as they were. \
         The terminal stays here until the kernel lets go.\n"
    );
}

/// What:
///  What the line about a waiting stall is built from.
/// Why:
///   One stall of one step has these facts and no others,
///  and each of them is something the person checks with `ps` or waits for.
///  The bound is named because it is the one promise this wait makes:
///  the step is ended after it,
///  and the ceiling the person was told about is not what ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AStall = { step: number; waited: number; until: number; why: WhyTheStepWasWaitedFor };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AStall {
    /// The process number of the `git commit` the commit runs in its scratch repository.
    pub step: i32,
    /// How long that step had lasted when its ceiling came and stopped ending it.
    pub waited: Duration,
    /// The longest that step may last while the ceiling stands aside,
    /// counted from its start.
    pub until: Duration,
    /// Why the ceiling stood aside:
    /// the process the kernel is holding,
    /// which is the step itself or a program it started,
    /// or nothing, when nothing could be learned about the step.
    pub why: WhyTheStepWasWaitedFor,
}

/// What:
///  The line a commit prints once when its ceiling comes for a step the kernel holds in a disk wait.
/// Why:
///  The step is not stuck:
///  it is waiting for the disk,
///  which a signal cannot shorten,
///  and ending it there failed two commits on 2026-11-11.
///  So the ceiling stands aside and the commit says why it did,
///  how long the step has lasted,
///  and the bound the step now runs under,
///  which is the one moment a person can decide to abandon the command instead of waiting for it.
///  What the person's own index,
///  files and branch hold is said because the wait is long and they cannot ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ... reached its ceiling ... waiting for the disk ... waits ...`
/// ```
pub fn a_stall_is_waited_for(stall: AStall) -> String {
    let AStall { step, waited, until, why } = stall;
    let seconds: u64 = waited.as_secs();
    let bound: u64 = until.as_secs();
    // What the ceiling could not do, said as the fact the look gave:
    // a process the kernel is holding cannot act on a signal,
    // and a step whose state could not be read is one no signal can be aimed at either.
    let what_the_ceiling_could_not_do: String = match why {
        WhyTheStepWasWaitedFor::TheKernelHeldIt(the_held) if the_held.number == step => {
            String::from("it is in uninterruptible sleep (state `D`), which a signal cannot end, so the step is waiting for the disk rather than stuck in its own work")
        }
        WhyTheStepWasWaitedFor::TheKernelHeldIt(the_held) => format!(
            "the program it started, `{}` (process {}), is in uninterruptible sleep (state `D`), which a signal cannot end, so the step is waiting for the disk rather than stuck in its own work",
            the_held.name, the_held.number
        ),
        WhyTheStepWasWaitedFor::ItsStateCouldNotBeRead => {
            String::from("its state could not be read (a look at `/proc` failed), so this commit cannot tell whether the disk is holding it")
        }
    };
    return format!(
        "monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository (process {step}) reached its ceiling after {seconds} seconds and was not stopped: \
         {what_the_ceiling_could_not_do}. A step in that state is slow, not stuck, so this commit waits for it instead of failing the commit: \
         it is stopped only after {bound} seconds in all, and it lands if it ends before that. Nothing of yours is at risk while it waits: \
         it runs in a scratch repository on a copy of your index.\n"
    );
}

/// The lines and their controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_held_by_the_kernel_tests.rs"]
mod tests;
