//! What:
//!  The answer a size read sends and the value one add holds while it runs:
//!  the read's own cost with the map it parsed,
//!  the join within the join's deadline,
//!  the settle and the abandon of a read the add will not wait for,
//!  and the Git child those last two end.
//! Why:
//!   The fence may use the answer only for the commit its source proved,
//!  so the commit travels in this one value with the answer and the child,
//!  and no other command and no other record can reach it.
//!   The join's deadline and the free of a map nobody uses are files of their own,
//!  declared here because the join and the abandon of this value are what read them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const joined = read.join(); // the map, or the sentence a failed read gives
//! read.abandon(); // ends the child of a read the add will not use
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
    note_work,
};
use crate::diagnostic_log::debug;
use crate::worktree_clone_conversion_git_reads::{
    BlobSizes,
    a_git_run_that_did_not_answer,
};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

/// Tag used for diagnostics from this size read's files.
pub(crate) const TAG: &str = "worktreeCloneConversionSizeRead";

/// The name the read's own duration is noted under in the add's record.
/// Why:
///   The read runs on another thread,
/// which the add's own steps cannot see,
///  so its time is the record's work and the join's wait is a step.
pub(crate) const WORK: &str = "fenceSizesBesideTheCheck";

/// How long the fence waits for the read started beside the check.
/// Why:
///   The read is one plumbing step,
/// which the runner ends at `PLUMBING_STEP_CEILING` and gives `SHORT_EXTRA_WAIT` to end
/// (`transaction_git_step_bounds.rs`),
///  so a read that has not answered by then has already failed inside its own runner;
/// this deadline only catches a read whose task died or is held outside it,
/// and the numbers are that step's own,
/// not a new limit.
const THE_JOIN_DEADLINE: Duration = crate::wait_ceilings::PLUMBING_STEP_CEILING
    .saturating_add(crate::wait_ceilings::SHORT_EXTRA_WAIT);

/// What the read's job answers:
/// how long it took,
/// and the map or the reason it cannot be read.
/// Why:
///   The map is parsed on the read's own thread,
/// where `main` parsed it,
/// so the parse stays off the adding thread of every add that uses the answer
/// (the integration pass's Medium 1 of 2026-11-23);
/// an add that abandons the read frees the answer beside the add (`free_the_answered_map_beside_the_add`).
pub(crate) type ReadAnswer = (Duration, Result<BlobSizes, String>);

/// What:
///  What the join got:
/// the read's own cost with its answer,
///  or the sentence for a read that answered nothing.
/// Why:
///   A read that answers and a read that does not are both facts the record states,
/// and the fence only uses the first;
/// one value carries them apart so no caller mistakes a missing answer for a map.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Joined = { spent: number; outcome: Result<BlobSizes, string> } | { silent: string };
/// ```
enum Joined {
    /// The read answered,
    /// with how long it spent.
    Answered {
        /// The read's own time.
        spent: Duration,
        /// The map,
        /// or the reason it could not be read.
        outcome: Result<BlobSizes, String>,
    },
    /// The read answered nothing,
    /// in the words a failed run of the fence gives.
    Silent(String),
}

/// What:
///  The size read one add started before its source check:
/// the commit it listed and the thread that will answer.
/// Why:
///   The fence may use the answer only for the commit its source proved,
/// so the commit travels with the thread and a different one leaves the read unused;
/// the handle is the whole of this add's own state,
/// so no other command and no other record can reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SizeReadBesideTheCheck = { commit: string; answer: Channel<Result<BlobSizes, string>> };
/// ```
pub struct SizeReadBesideTheCheck {
    /// The commit the read listed.
    pub(crate) commit: String,
    /// The one answer the read's thread sends.
    pub(crate) answer: std::sync::mpsc::Receiver<ReadAnswer>,
    /// The process id of the read's Git child while it runs,
    /// which an add that abandons the read ends.
    pub(crate) child: std::sync::Arc<std::sync::atomic::AtomicI32>,
}

impl SizeReadBesideTheCheck {
    /// What:
    ///  The read's answer,
    /// waited for within `THE_JOIN_DEADLINE`,
    /// with the wait recorded as a step of the adding thread and the read's own time as work.
    /// Why:
    ///   The wait is what the overlap saves,
    /// so it is the number the record has to show;
    /// a read that answers nothing (a deadline or a dead thread)
    /// refuses the rung here,
    /// which is the sentence a read that failed gives in the serial order.
    /// A thread the system refused never reaches this method:
    /// the start answers nothing and the fence reads where it stands.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// join() { const entered = beginStep('fenceSizesWaitingForTheRead'); const joined = this.answerWithinTheDeadline(); endStep(entered); return joined; }
    /// ```
    pub(crate) fn join(self) -> Result<BlobSizes, String> {
        let entered: bool = begin_step("fenceSizesWaitingForTheRead");
        let joined: Joined = self.answer_within_the_deadline();
        end_step(entered);
        return match joined {
            Joined::Answered {
                spent,
                outcome,
            } => {
                note_work(WORK, spent);
                match outcome {
                    Ok(sizes) => Ok(sizes),
                    Err(reason) => Err(reason),
                }
            }
            Joined::Silent(reason) => Err(reason),
        };
    }

    /// What:
    ///  Take the read this add is not going to use only if it has already answered;
    /// never wait for it.
    /// Why:
    ///   An add the source check refuses,
    ///   or the fence declines,
    ///  starts this read before the check and then ends it (`abandon`);
    /// an add that reaches this method with the read still held is one that served the add
    /// (the decision `doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
    /// its section "The overlap,
    /// answered 2026-11-22"),
    ///  and it never waits for it:
    /// a read longer than the add's own Git run would otherwise make the add slower than on `main`,
    ///  which is the integration pass's High 1 of 2026-11-22.
    /// A read that has answered is stated as the record's work;
    /// one that has not is stated as started and not joined,
    /// and the add ends without it.
    /// The read's thread ends with this process,
    /// and its child ends either through the kernel's parent-death signal
    /// or,
    ///  where that signal is cleared (an exec into an AppArmor-enforced profile),
    /// through the `Drop` below.
    /// Nothing here changes the add's answer:
    ///  a read that failed is only a debug line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// settle() { const answer = this.answer.tryRecv(); if (answer) noteWork('fenceSizesBesideTheCheck', answer.spent); else note('fenceSizesBesideTheCheck', 'started and not joined'); }
    /// ```
    pub fn settle(self) {
        match self.answer.try_recv() {
            Ok((spent, outcome)) => {
                note_work(WORK, spent);
                if let Err(reason) = outcome {
                    debug(TAG, format!("the size read started beside the check answers {reason}, and the add no longer waits for it").as_str());
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                crate::add_timing::note(WORK, "started and not joined");
                debug(
                    TAG,
                    "the size read started beside the check has not answered, so the add states it as started and not joined; this value's end settles its child",
                );
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                crate::add_timing::note(WORK, "started and not joined");
                debug(
                    TAG,
                    "the size read started beside the check ended without answering, so the add states it as started and not joined",
                );
            }
        }
    }

    /// What:
    ///  Stop the read this add is not going to use:
    ///  state it as work when it already answered and as started and not joined otherwise,
    ///  and end its Git child here,
    ///  or at the moment that child is published.
    /// Why:
    ///   The source check can refuse the source before the fence,
    ///  the fence can decline,
    ///  and a fence with no path to size or one whose read listed another commit will not use the answer;
    ///  any of them that leaves the read running makes the add slower than `main`
    ///  (the integration pass's cone shape and the recheck's Low 4 of 2026-11-22),
    ///  so the child is ended as soon as the add knows it will not use it.
    /// The abandon often comes before the read's thread has reached its spawn
    ///  (a source check that refuses at once takes a few microseconds),
    ///  so the cell takes `ABANDONED_CHILD` and the runner ends the child the moment it publishes it
    ///  (`child_watch_one_shot_run.rs`);
    /// a child already published is signalled here.
    /// A read whose child was reaped has zero here within the time the runner takes to finish reading the child's pipes;
    /// between the kernel's reap and that clear the cell still names an id no process has,
    /// and between this swap and its signal the id can be reaped,
    /// so either signal reaches nothing unless the id is reused in that window
    /// (about four million forks in under a second at this host's `pid_max`,
    /// the integration recheck's Low 3 of 2026-11-22).
    /// The thread answers into a receiver this value drops,
    /// and its send is ignored.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// abandon() { const answer = this.answer.tryRecv(); if (answer) noteWork(work, answer.spent); else note(work, 'started and not joined'); const published = this.child.swap(ABANDONED_CHILD); if (published > 0) kill(published, 'SIGKILL'); }
    /// ```
    pub fn abandon(self) {
        let take_entered: bool = begin_step("abandonTakeTheAnswer");
        let answered: Option<BlobSizes> = match self.answer.try_recv() {
            Ok((spent, outcome)) => {
                note_work(WORK, spent);
                match outcome {
                    Ok(sizes) => Some(sizes),
                    Err(reason) => {
                        debug(TAG, format!("the size read started beside the check answers {reason}, and the add no longer waits for it").as_str());
                        None
                    }
                }
            }
            Err(
                std::sync::mpsc::TryRecvError::Empty | std::sync::mpsc::TryRecvError::Disconnected,
            ) => {
                crate::add_timing::note(WORK, "started and not joined");
                None
            }
        };
        free_the_answered_map_beside_the_add(answered);
        end_step(take_entered);
        let child_entered: bool = begin_step("abandonEndTheChild");
        if self.child.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            debug(
                TAG,
                "the size read started beside the check is not used before its child exists, so the runner ends that child the moment it publishes it",
            );
        }
        self.end_the_child();
        end_step(child_entered);
    }

    /// What:
    ///  End the Git child this read published,
    /// or record that the runner must end it at the moment it publishes one.
    /// Why:
    ///   `abandon` and the `Drop` below both end the child,
    /// and the cell's swap is what makes the ending happen once:
    /// a second call finds `ABANDONED_CHILD` and signals nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// endTheChild() { const published = this.child.swap(ABANDONED_CHILD); if (published > 0) kill(published, 'SIGKILL'); }
    /// ```
    fn end_the_child(&self) {
        let published: i32 = self.child.swap(
            crate::transaction_git_request_and_reply_types::ABANDONED_CHILD,
            std::sync::atomic::Ordering::SeqCst,
        );
        if published > 0 {
            // SAFETY: `published` is this add's own Git child, published by its runner while it ran;
            // a signal to a process that ended in between reaches nothing.
            let _ = unsafe { libc::kill(published, libc::SIGKILL) };
            debug(TAG, format!("the size read started beside the check is not used, so its child {published} is ended and the add does not wait for it").as_str());
        }
    }

    /// What:
    ///  The read's answer within the deadline,
    ///  or the sentence for a read that answered nothing.
    /// Why:
    ///   One wait answers both facts:
    /// a task that died and a read that outlived the deadline are both "no output to decide with",
    ///  and the sentence is the one a failed run of the fence gives,
    /// not a new one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answerWithinTheDeadline() { const waited = await timeout(deadline, this.answer); return read(waited); }
    /// ```
    fn answer_within_the_deadline(self) -> Joined {
        let deadline: Duration = deadline::the_deadline(THE_JOIN_DEADLINE);
        return match self.answer.recv_timeout(deadline) {
            Ok((spent, outcome)) => Joined::Answered {
                spent,
                outcome,
            },
            Err(RecvTimeoutError::Timeout) => {
                Joined::Silent(a_git_run_that_did_not_answer(&std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("`git ls-tree` did not answer within {deadline:?}"),
                )))
            }
            Err(RecvTimeoutError::Disconnected) => Joined::Silent(a_git_run_that_did_not_answer(
                &std::io::Error::other("the read ended without answering"),
            )),
        };
    }

    /// What:
    ///  The child number the cell holds now,
    /// for a control that shows the cell is cleared once the child is gone.
    /// Why:
    ///   Nothing in the shipped command reads the cell for an answer;
    /// a control has to,
    /// to show the window the integration recheck's Low 1 of 2026-11-22 found is closed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// publishedChildForAControl() { return this.child.load(); }
    /// ```
    #[cfg(test)]
    pub(crate) fn published_child_for_a_control(&self) -> i32 {
        return self.child.load(std::sync::atomic::Ordering::SeqCst);
    }

    /// What:
    ///  A handle on the cell that holds the read's child,
    /// for a control that reads it after `abandon` took the read.
    /// Why:
    ///   `abandon` consumes the read,
    /// and a control of the cell's value needs the cell itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// childCellForAControl() { return this.child; }
    /// ```
    #[cfg(test)]
    pub(crate) fn child_cell_for_a_control(&self) -> std::sync::Arc<std::sync::atomic::AtomicI32> {
        return self.child.clone();
    }
}

/// What:
///  The end of an add's read that nobody joined ends its Git child too.
/// Why:
///   The child is spawned with `PR_SET_PDEATHSIG`,
/// and the kernel clears that signal when a process's exec changes its security context
/// (`fs/exec.c`,
///  `begin_new_exec`:
///  `if (bprm->secureexec) me->pdeath_signal = 0;`),
/// which AppArmor does for every exec that attaches a profile
/// (`security/apparmor/domain.c` sets `bprm->secureexec` for a transition):
/// on this machine `/usr/bin/sleep` and `/usr/bin/git` are under enforced profiles,
/// so an add that ended without settling its read left the read's `git ls-tree` running to its own end
/// (the release pass over alpha.45's finding L1).
/// A read the add joined or abandoned already swapped the cell,
/// so this drop signals nothing;
///  a read the add ends by returning,
///  by an error or by a panic ends its child here.
/// An add the kernel kills outright runs no destructor,
/// and that limit is recorded in `doc/troubleshooting/pdeathsig-is-cleared-by-a-confined-exec.md`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// onDrop() { this.endTheChild(); }
/// ```
impl Drop for SizeReadBesideTheCheck {
    fn drop(&mut self) {
        self.end_the_child();
    }
}

/// The join's shipped deadline and the control that shortens it.
#[path = "worktree_clone_conversion_size_read_deadline.rs"]
pub(crate) mod deadline;

/// The answered map an add will not use,
/// and the thread that frees it,
/// with the control seams that show where the free happened.
#[path = "worktree_clone_conversion_size_read_free.rs"]
pub(crate) mod free;

use self::free::free_the_answered_map_beside_the_add;
