//! What:
//!  The thread that watches the commit Git builds in the scratch repository:
//!  one look at the step and the programs below it,
//!  the one decision it shares with the wait that owns the step,
//!  and the line a person reads while the step runs.
//! Why:
//!  The watching is one job of its own,
//!  and it is the longest one in this notice:
//!  it looks,
//!  decides the ceiling,
//!  stops the step,
//!  and says what it sees,
//!  once per look,
//!  until the step ends or the command is over.
//!  The window a control plants is declared here because it is this thread's own state,
//!  and `native_commit_waiting_notice.rs` re-exports it so a control reaches it as it always did.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watched = spawn(() => watchTheStep({ cadence, ceiling, signs, sending, ended, toldAnswers, watched, theStall, began }));
//! ```

use crate::child_number_told_at_start::ChildNumber;
use crate::process_tree::ProcessUse;
use crate::scratch_commit_step::ScratchStepFacts;
use crate::scratch_step_ceiling_stop::{
    CeilingCame,
    TheCeiling,
    ceiling_came,
};
use crate::scratch_step_look_below::{
    look_below,
    process_cpu,
    process_is_blocked,
};
use crate::scratch_step_notice_line::{
    notice_line,
    other_programs,
};
use crate::scratch_step_programs_below::{
    SeenBelow,
    credit_between_looks,
    seen_below,
};
use crate::scratch_step_stall_decided_once::{
    HeldProgram,
    TheStall,
    TheStallDecidedOnce,
    TheStallItWaitsThrough,
    WhyTheStepWasWaitedFor,
};
use crate::wait_progress::{
    Cadence,
    Ticker,
};
/// A decision two enforcers of one ceiling share,
/// kept across this thread.
use std::sync::Arc;
/// The channel that ends this thread,
/// and the queue it hands its lines to.
use std::sync::mpsc::{
    RecvTimeoutError,
    Sender,
    SyncSender,
};
/// Time spans and the monotonic clock.
use std::time::{
    Duration,
    Instant,
};

/// The shortest pause between two looks of the notice thread,
/// so a very short interval does not spin.
const LOOK_PAUSE: Duration = Duration::from_millis(1);

/// The longest the notice goes between two looks at what runs below this process.
/// Why:
///   The ceiling is counted from the first look that found the step,
/// so a start this host held up (a full process table,
/// memory pressure,
/// a stall in starting a process) cannot take the ceiling away:
/// a look that finds nothing is a statement about the start,
/// never about the step.
///  That reading is the one the record could not make for a commit whose step ran 36,651 ms under a
///  3,006 ms ceiling on 2026-11-08:
/// it landed,
/// so the ceiling never ended it.
///  The same look is what the record's `scratchSteps` reports,
/// so this is also how finely the time of a program below the step is known,
/// and how soon a look that failed is tried again.
///  A look reads the kernel's per-thread child lists and one `/proc/<pid>/stat` per descendant
/// (`look_below`),
/// which cost 19.6 microseconds on 2026-11-09 on this host (1,210 entries,
/// load average 44 to 91 on 6 threads around the runs);
/// the pass over `/proc` it falls back to where the kernel has no child lists took 17.6 ms at the median and 19.7 ms at most over 30 looks.
/// A quarter of a second keeps a step that is started late seen within that much of its start,
/// and a step that ends sooner than that is ended by its own channel before a second look.
pub(crate) const LOOK_INTERVAL: Duration = Duration::from_millis(250);

/// What:
///  A control's window on the notice:
///  the ceiling it plants and the look it waits for.
/// Why:
///  A control of the ceiling must make the ceiling come,
///  and must not race a clock to do it.
///  Test-only,
/// so a release build reads no atomic here.
#[cfg(all(test, target_os = "linux"))]
#[path = "native_commit_waiting_notice_control_window.rs"]
pub(crate) mod the_controls_window;

/// What:
///  The ceiling this look counts from,
///  which a control may have planted,
///  or `ceiling`.
/// Why:
///   The ceiling is read at the top of every look,
///  so a control that plants one makes it come at the look it chose,
///  and no sleep decides the order.
///  A release build reads no such state and uses the ceiling it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ceilingOfThisLook = (ceiling) => theControlsWindow.planted() ?? ceiling;
/// ```
fn ceiling_of_this_look(ceiling: Duration) -> Duration {
    #[cfg(all(test, target_os = "linux"))]
    if let Some(planted) = the_controls_window::the_ceiling_the_control_planted() {
        return planted;
    }
    return ceiling;
}

/// What:
///  The pause this look waits before the next one,
///  with the stretch a control planted between this step's looks added to it.
/// Why:
///   A host under load woke this thread late and stretched each look interval,
///   so a control's fixed window held one look fewer than its figures assumed (issue 6592);
///   a control states that stretch so the record is asked for its promise under the schedule that lost it a look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waiting = pause + aDelayAControlPlanted(step.number());
/// ```
#[cfg(all(test, target_os = "linux"))]
fn pause_before_the_next_look(
    pause: Duration,
    watched: &ChildNumber,
) -> Duration {
    let planted: Option<Duration> =
        the_controls_window::the_delay_between_looks_of(watched.number());
    return match planted {
        Some(delay) => pause + delay,
        // No control stated a stretch for this step, so the pause is this thread's own.
        None => pause,
    };
}

/// What:
///  The pause this look waits before the next one,
///  which no build that ships lengthens.
/// Why:
///  Only a control states a stretch between two looks,
///  so this build reads no window and waits the pause it computed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waiting = pause;
/// ```
#[cfg(not(all(test, target_os = "linux")))]
fn pause_before_the_next_look(
    pause: Duration,
    _unread_step: &ChildNumber,
) -> Duration {
    return pause;
}

/// What:
///  What one watching thread is given.
/// Why:
///   The thread reads nothing of the stack of the command that started it,
///  so every fact it needs travels here as one value:
///  the cadence of the lines,
///  the ceiling this step is under,
///  whether the commit is signed,
///  the queue its lines go to,
///  the channel that ends it,
///  the sender its answer leaves through,
///  the number of the step it watches,
///  the decision it shares with the wait that owns the step,
///  and when the notice started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheWatch = { cadence: Cadence; ceiling: Duration; signs: boolean; sending: SyncSender<string>; ended: Receiver<void>; toldAnswers: Sender<ScratchStepFacts>; watched: ChildNumber; theStall: Arc<TheStallDecidedOnce>; began: Instant };
/// ```
pub(crate) struct TheWatch {
    /// The cadence of the lines.
    pub(crate) cadence: Cadence,
    /// How long the step lasts before it is stopped.
    pub(crate) ceiling: Duration,
    /// Whether the commit is signed.
    pub(crate) signs: bool,
    /// The queue the lines go to,
    /// and a send that never waits for a reader.
    pub(crate) sending: SyncSender<String>,
    /// The channel that ends this thread when it is sent to or dropped.
    pub(crate) ended: std::sync::mpsc::Receiver<()>,
    /// Where the facts this thread saw leave,
    /// so a thread that cannot answer never holds the commit.
    pub(crate) told_answers: Sender<ScratchStepFacts>,
    /// The step this thread watches.
    pub(crate) watched: ChildNumber,
    /// The one decision this ceiling shares with the wait that owns the step.
    pub(crate) the_stall: Arc<TheStallDecidedOnce>,
    /// When the notice started,
    /// which the ceiling is counted from the first look that found the step.
    pub(crate) began: Instant,
}

/// What:
///  Watch the step until it ends,
///  the ceiling comes,
///  or the command is over:
///  look at the step and the programs below it,
///  decide the ceiling,
///  and hand the person a line while the step runs.
/// Why:
///   The watch runs on a thread of its own,
///  so a reader that stopped reading holds the writer and never this look,
///  and the ceiling comes even when no line can be written.
///  The ceiling is counted from the first look that found the step,
///  never from when the notice started,
///  and a look that could not be made decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchTheStep(theWatch); // nothing returns here: the facts leave through toldAnswers
/// ```
pub(crate) fn watch_the_step(the_watch: TheWatch) {
    let TheWatch {
        cadence,
        ceiling,
        signs,
        sending,
        ended,
        told_answers,
        watched,
        the_stall,
        began,
    } = the_watch;
    let mut ticker: Ticker = Ticker::new(cadence);
    let mut first: bool = true;
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(ceiling));
    facts.armed = true;
    // When the step was first found below this process; the ceiling is counted from then.
    let mut seen_at: Option<Duration> = None;
    // The processes the previous look saw below the step, and when that look was.
    let mut before: Vec<SeenBelow> = Vec::new();
    let mut last_look: Duration = Duration::ZERO;
    // What: The bound the step runs under once a look finds the kernel holding it in a disk wait.
    // Why:  A step inside an uninterruptible call is waiting for the disk and not stuck in its own work,
    //       so ending it at the ceiling it reached fails a commit that would have landed
    //       (`scratch_step_held_by_the_kernel.rs`).
    //       A step that may open an editor keeps no ceiling at all and never reaches here.
    let stall_bound: Duration = crate::wait_ceilings::stall_ceiling_of(ceiling);
    // What: The later bound is a stand-aside only when it really is later.
    // Why:  A person may set a ceiling above the bound this release waits a step for,
    //       and a control may plant one: standing aside for a bound that is not later would end the step at its ceiling
    //       and say it was waited for.
    let the_later_bound_is_later: bool = stall_bound > ceiling;
    // One label for every way the loop ends, so the facts leave through the channel and not through the join.
    'notice: loop {
        let waited: Duration = began.elapsed();
        // What: The one decision this ceiling shares with the wait that owns the step.
        // Why:  Two enforcers of one ceiling must give one answer (`scratch_step_stall_decided_once.rs`),
        //       and the bound the person is told about is that answer's.
        let decided: TheStall = the_stall.state();
        // What: The ceiling this look counts from.
        // Why:  A control may plant one, so the ceiling comes at the look it chose instead of a slept window;
        //       and a step the kernel holds runs under the bound it is waited for, not the one it reached.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const bound = decided.stoodAside ? decided.stoodAside.bound : ceilingOfThisLook(ceiling);
        // ```
        let bound: Duration = match &decided {
            TheStall::StoodAside(stood) => stood.bound,
            TheStall::TheUsualCeilingEndsIt | TheStall::Undecided => ceiling_of_this_look(ceiling),
        };
        // One look: the step, the programs below it, and what they are using.
        // A step that has not started yet is nothing to look at, which is not a look that failed.
        let look: Option<Vec<(i32, ProcessUse)>> = match watched.number() {
            Some(number) => look_below(number),
            None => Some(Vec::new()),
        };
        let below: &[(i32, ProcessUse)] = look.as_deref().unwrap_or_default();
        facts.looks = facts.looks.saturating_add(1);
        if !below.is_empty() {
            facts.looks_found = facts.looks_found.saturating_add(1);
        }
        // What: Note every look that found the step, for a control that waits for such looks.
        // Why:  A control's figure rests on how many looks found the step,
        //       so every one of them is noted and not only the first (issue 6592).
        #[cfg(all(test, target_os = "linux"))]
        if !below.is_empty() {
            the_controls_window::noted_a_look_that_found_the_step();
        }
        if seen_at.is_none() && !below.is_empty() {
            seen_at = Some(waited);
        }
        credit_between_looks(
            &mut facts,
            before.as_slice(),
            below,
            waited.saturating_sub(last_look),
        );
        before = seen_below(below);
        last_look = waited;
        facts.below_cpu = below.iter().map(process_cpu).sum();
        if below.iter().any(process_is_blocked) {
            facts.blocked_looks = facts.blocked_looks.saturating_add(1);
        }
        // How long the step has been running, and whether the ceiling has come, once the step has been seen.
        let since_seen: Duration = match seen_at {
            Some(seen) => waited.saturating_sub(seen),
            None => Duration::ZERO,
        };
        // The ceiling can only have been counted from a look that found the step, so its number is known here.
        if watched.number().is_some() && seen_at.is_some() && since_seen >= bound {
            // What: The one decision about this ceiling, and what this look adds to it.
            // Why:  A step the kernel holds in a disk wait is slow, not stuck: it is waiting for a call of the kernel's,
            //       which a signal cannot shorten,
            //       and ending it there fails a commit that would have landed (`scratch_step_held_by_the_kernel.rs`).
            //       The wait that owns the step stands aside for the same reason through the same decision
            //       (`scratch_step_stall_decided_once.rs`), so a stand-aside either enforcer made is honoured by both,
            //       and neither ends the step at the usual ceiling afterwards (finding 2 of the 2026-11-12 review).
            //       A look that could not be made decides nothing:
            //       the ceiling stays armed and the step is looked at again,
            //       until the bound it is waited for comes,
            //       because reading a failed look as "not held" killed a held step (finding 1).
            //       The later bound is only a stand-aside when it really is later.
            let mut decided_now: TheStall = the_stall.state();
            if let TheStall::Undecided = decided_now {
                decided_now = if !the_later_bound_is_later {
                    let _claimed: bool = the_stall.the_usual_ceiling_ends_it();
                    the_stall.state()
                } else if look.is_none() {
                    if since_seen < stall_bound {
                        TheStall::Undecided
                    } else {
                        the_stall.stand_aside(TheStallItWaitsThrough {
                            stood_aside_after: waited,
                            bound: stall_bound,
                            why: WhyTheStepWasWaitedFor::ItsStateCouldNotBeRead,
                        })
                    }
                } else {
                    match crate::scratch_step_look_below::the_process_the_kernel_holds(below) {
                        Some((found, used)) => the_stall.stand_aside(TheStallItWaitsThrough {
                            stood_aside_after: waited,
                            bound: stall_bound,
                            why: WhyTheStepWasWaitedFor::TheKernelHeldIt(HeldProgram {
                                number: *found,
                                name: used.name.clone(),
                            }),
                        }),
                        None => {
                            let _claimed: bool = the_stall.the_usual_ceiling_ends_it();
                            the_stall.state()
                        }
                    }
                };
            }
            // What: The record of a ceiling that stood aside, whoever decided it.
            // Why:  A notice thread the system refused leaves the wait's own stop to say it,
            //       and a record with no bound would leave the next reading of the log unable to tell a slow step from a stopped one.
            if let TheStall::StoodAside(stood) = &decided_now {
                facts.stall_ceiling = Some(stood.bound);
                if matches!(stood.why, WhyTheStepWasWaitedFor::TheKernelHeldIt(_))
                    && facts.held_after_ceiling.is_none()
                {
                    facts.held_after_ceiling = Some(stood.stood_aside_after);
                }
            }
            match decided_now {
                TheStall::StoodAside(_stood) => {
                    // What: The step is waited for, and this thread says nothing of its own.
                    // Why:  The wait that owns the step is the one that lasts and says why
                    //       (`process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`),
                    //       and one sentence about one stand-aside is said once (finding 3).
                }
                TheStall::TheUsualCeilingEndsIt => {
                    let the_ceiling: TheCeiling = ceiling_came(&watched);
                    // A step signalled while the walk below it failed leaves whatever it started unnamed.
                    facts.below_unlisted = !the_ceiling.below_listed;
                    // A program this ceiling could not end may still be running, and the record says so.
                    facts.below_unended = the_ceiling.below_unended;
                    match the_ceiling.came {
                        CeilingCame::Stopped => {
                            facts.stopped_after = Some(waited);
                            // What: The step was sent the stop.
                            // Why:  Whether the kernel has released it is said by the wait that keeps waiting for it, and not
                            //       from this thread, which the system may never have started
                            //       (`process/wait_after_the_stop_saying_the_kernel_holds_it.rs`).
                            break 'notice;
                        }
                        CeilingCame::TheStepEnded => break 'notice,
                        // The signal proved nothing: nothing is known about the step, and the ceiling stays armed so a
                        //  later look can stop it.
                        CeilingCame::Unknown => {
                            facts.blind_looks = facts.blind_looks.saturating_add(1);
                        }
                    }
                }
                // Nothing is known about the step yet: this look decided nothing, and the ceiling stays armed.
                TheStall::Undecided => {
                    facts.blind_looks = facts.blind_looks.saturating_add(1);
                }
            }
        }
        if ticker.due(waited) {
            let line: String = notice_line(
                signs,
                waited,
                first,
                other_programs(below).as_slice(),
                bound,
            );
            first = false;
            // A closed terminal is not a reason to stop the commit, and a reader that stopped reading is not one either:
            // the line is handed to the writer, and dropped and counted while the writer still holds the one before it.
            if sending.try_send(line).is_err() {
                facts.lines_dropped = facts.lines_dropped.saturating_add(1);
            }
        }
        // Wake for the sooner of the next line, the look, and the ceiling.
        let to_next: Duration = ticker.next_in(waited);
        let pause: Duration = match seen_at {
            Some(_seen) => to_next
                .min(bound.saturating_sub(since_seen))
                .min(LOOK_INTERVAL),
            None => to_next.min(LOOK_INTERVAL),
        };
        // A ceiling that came and decided nothing is looked at no sooner than the look's own interval,
        // so a step whose processes cannot be read costs one look per interval rather than one per pause.
        let floor: Duration = if since_seen >= bound {
            LOOK_INTERVAL
        } else {
            LOOK_PAUSE
        };
        match ended.recv_timeout(pause_before_the_next_look(pause.max(floor), &watched)) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => break 'notice,
        }
    }
    // The facts leave through the channel `finish` waits on, so a thread that cannot answer never holds the commit:
    // returning them through the join would make the join the wait.
    let _answered = told_answers.send(facts);
    return;
}
