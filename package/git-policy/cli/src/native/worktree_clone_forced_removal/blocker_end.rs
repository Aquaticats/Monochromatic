//! What:
//!  The signal stage:
//!  `SIGTERM` to every blocker,
//!  a wait for it to leave,
//!  then `SIGKILL` to the survivors,
//!  with the caller's own liveness checked before the kill.
//! Why:
//!  The owner chose "ask,
//!  wait,
//!  then kill",
//!  and the design places the wait in the signal stage,
//!  after every namespace's structural stage and before every unmount,
//!  sending each signal through the descriptor its pin was made with
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "The signal";
//!  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The signal").
//!  Two rules are why this is code and not prose:
//!  a signal travels through a descriptor opened while the number still named
//!  the process,
//!  so a reused number cannot receive it,
//!  and a wrapper that died during the wait sends no `SIGKILL` at all,
//!  which the act files as `caller-gone`.
//!  The wait's length is the window the constants give,
//!  which ships with no value,
//!  so a control hands its own short window and the release has none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const endings = endTheBlockers(blockers, window, wait, () => callerStillLives());
//! endings.killed; endings.callerGone;
//! ```

use crate::process::member_signalled_through_its_pidfd::{Delivered, Pinned, PinnedMember};
use crate::process::process_tree::{ProcessUse, TreeMember, member_of, use_of};
use crate::worktree_clone_forced_removal::forced_arm_numbers::TimeoutWindow;

/// How many blockers one signal stage handles.
/// Why:
///  The holder scan lists at most this many findings,
///  and a namespaces' blockers come from that report,
///  so a longer list is a list this arm was never handed
///  (`clone_holder_scan::holder_scan::MAXIMUM_DETAIL_LINES`).
pub const MOST_BLOCKERS_ONE_ACT_HOLDS: usize = 1024;

/// The signals the stage sends,
/// named rather than numbered so a reader does not have to know the numbers.
pub const ASK: i32 = 15;

/// The kill the stage sends to a survivor.
pub const KILL: i32 = 9;

/// What:
///  One task the act means to end:
///  its number and the start time that proves later that the number still
///  names it.
/// Why:
///  A number alone is a number another process can take,
///  so the pair is what a pin proves before any signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Blocker = { pid: number; started: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blocker {
    /// The task's number.
    pub pid: i32,
    /// The start time the kernel recorded for it,
    /// in clock ticks since boot.
    pub started: u64,
}

impl Blocker {
    /// A blocker from one member the crate's own tree read answered.
    pub fn from_member(member: &TreeMember) -> Blocker {
        return Blocker { pid: member.pid, started: member.started };
    }

    /// What:
    ///  A blocker read from `/proc` now,
    ///  which is what a control and the handler's own scan-to-act hand-over do.
    /// Why:
    ///  The pair must come from one read of one record,
    ///  because two reads could describe two processes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const blocker = Blocker.read(pid);   // undefined when the task has gone
    /// ```
    pub fn read(pid: i32) -> Option<Blocker> {
        let member: TreeMember = member_of(pid)?;
        return Some(Blocker { pid: member.pid, started: member.started });
    }
}

/// What:
///  What became of one blocker.
/// Why:
///  "exited after `SIGTERM`" and "was killed" are different lines in the
///  journal and in the report,
///  and a blocker that went before the stage touched it costs nothing and is
///  its own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EndResult = 'exited-after-ask' | 'killed' | 'gone' | 'unproven' | 'survived';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndResult {
    /// It left after `SIGTERM`,
    /// and no `SIGKILL` was sent.
    ExitedAfterTheAsk,
    /// It ignored `SIGTERM` and was killed.
    Killed,
    /// No process had its number when the stage pinned it.
    Gone,
    /// The stage could not prove it,
    /// so nothing was sent and nothing is concluded.
    Unproven,
}

/// What:
///  What the stage came to.
/// Why:
///  A wrapper that died during the wait stops the stage with no `SIGKILL`,
///  and the receipt says so with its own word,
///  so the report has to keep the two apart from a stage that finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SignalStage = { endings: [Blocker, EndResult][]; callerGone: boolean };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SignalStage {
    /// Each blocker and what became of it,
    /// in the order the stage was handed them.
    pub endings: Vec<(Blocker, EndResult)>,
    /// Whether the caller stopped during the wait,
    /// which is what makes the act file `caller-gone`.
    pub caller_gone: bool,
}

impl SignalStage {
    /// How many blockers were killed rather than asked out.
    pub fn killed(&self) -> usize {
        return self.endings.iter().filter(|(_blocker, result)| return *result == EndResult::Killed).count();
    }

    /// How many could not be proven,
    /// which every caller reads as a blocker still there.
    pub fn unproven(&self) -> usize {
        return self.endings.iter().filter(|(_blocker, result)| return *result == EndResult::Unproven).count();
    }
}

/// What:
///  End every blocker outside the protected set,
///  by the order the owner chose.
/// Why:
///  Every blocker is asked first and only the survivors are killed,
///  the caller's liveness is read after the wait and before the kill,
///  a blocker that went during the wait costs no signal,
///  and a blocker whose proof failed is reported and never signalled,
///  because a signal by number after a proof is still a signal to a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const blocker of blockers) pin(blocker)?.signal('TERM');
/// wait(window.termination());
/// if (!callerAlive()) return { callerGone: true };
/// for (const survivor of survivors) survivor.signal('KILL');
/// wait(window.termination());
/// ```
pub fn end_the_blockers<Wait, CallerAlive>(
    blockers: &[Blocker],
    window: TimeoutWindow,
    mut wait: Wait,
    caller_alive: CallerAlive,
) -> SignalStage
where
    Wait: FnMut(std::time::Duration),
    CallerAlive: Fn() -> bool,
{
    let mut stage: SignalStage = SignalStage::default();
    let mut pins: Vec<(Blocker, PinnedMember)> = Vec::new();
    let kept: &[Blocker] = if blockers.len() > MOST_BLOCKERS_ONE_ACT_HOLDS {
        // A list this long was never one scan's answer; the overflow is reported by the caller, not signalled here.
        &blockers[..MOST_BLOCKERS_ONE_ACT_HOLDS]
    } else {
        blockers
    };
    for blocker in kept {
        let member: TreeMember = TreeMember { pid: blocker.pid, started: blocker.started };
        match PinnedMember::open(&member) {
            Pinned::Held(pin) => match pin.signal(ASK) {
                Delivered::Sent => pins.push((*blocker, pin)),
                Delivered::Gone => {
                    stage.endings.push((*blocker, EndResult::Gone));
                }
                Delivered::Unproven => {
                    stage.endings.push((*blocker, EndResult::Unproven));
                }
            },
            Pinned::Gone => {
                stage.endings.push((*blocker, EndResult::Gone));
            }
            Pinned::Unproven => {
                stage.endings.push((*blocker, EndResult::Unproven));
            }
        }
    }
    wait(window.termination());
    if !caller_alive() {
        // A wrapper that ended during the wait sends no kill at all; every pin is dropped with its descriptor.
        for (blocker, _pin) in pins {
            stage.endings.push((blocker, EndResult::Unproven));
        }
        stage.caller_gone = true;
        return stage;
    }
    let mut survivors: Vec<(Blocker, PinnedMember)> = Vec::new();
    for (blocker, pin) in pins {
        if pin.has_ended() {
            stage.endings.push((blocker, EndResult::ExitedAfterTheAsk));
        } else {
            survivors.push((blocker, pin));
        }
    }
    if survivors.is_empty() {
        return stage;
    }
    for (blocker, pin) in survivors {
        match pin.signal(KILL) {
            Delivered::Sent => {
                stage.endings.push((blocker, EndResult::Killed));
            }
            Delivered::Gone => {
                stage.endings.push((blocker, EndResult::ExitedAfterTheAsk));
            }
            Delivered::Unproven => {
                stage.endings.push((blocker, EndResult::Unproven));
            }
        }
    }
    wait(window.termination());
    return stage;
}

/// What:
///  The blockers one process record names as a path holder,
///  when the record's state says it is an ordinary task.
/// Why:
///  A task in `Z`, `X`, `D` or with a number the kernel holds as a thread is not
///  a task this stage can end:
///  a zombie holds nothing,
///  and an uninterruptible task is reported rather than waited on forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endable = state !== 'Z' && state !== 'X' && state !== 'D';
/// ```
pub fn endable_from(state: char) -> bool {
    return !matches!(state, 'Z' | 'X' | 'D');
}

/// What:
///  One record the endability read names,
///  with the answer this build gives before any signal.
/// Why:
///  The preflight refuses an unendable holder before anything is ended,
///  and this read is where "cannot be ended" is decided.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = endabilityOf(pid);   // { blocker } or { unendable }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Endability {
    /// The task can be pinned and signalled.
    Endable(Blocker),
    /// It cannot,
    /// with the reason the refusal prints.
    Unendable(String),
    /// Its number names no task any more,
    /// which is not an unendable holder.
    Gone,
}

/// What:
///  Read one holder's endability from `/proc`.
/// Why:
///  A task in an uninterruptible sleep is reported rather than waited on,
///  and a number that names no task is gone rather than unendable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = endabilityOf(pid);
/// ```
pub fn endability_of(pid: i32) -> Endability {
    let Some(record): Option<ProcessUse> = use_of(pid) else {
        return Endability::Gone;
    };
    if !endable_from(record.state) {
        return Endability::Unendable(format!(
            "task {pid} is in state `{}`, which this act does not end",
            record.state
        ));
    }
    let Some(blocker): Option<Blocker> = Blocker::read(pid) else {
        return Endability::Gone;
    };
    return Endability::Endable(blocker);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "blocker_end_tests.rs"]
mod tests;
