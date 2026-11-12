//! What:
//!  Waits for a child this path owns until that child ends,
//! a stop of the service ends it,
//! or the ceiling of the step comes;
//! when the ceiling comes,
//! the child and everything it started are ended,
//! the child is reaped here,
//! and the stop is answered as a record beside the child's end.
//! Why:
//!  The wait that owns a child is the only place where its number cannot be reused:
//! the kernel keeps an unreaped child's number,
//! and this path is the only one that reaps it,
//! so the signal and the reap are one path in sequence and no reuse falls between them.
//!  A notice thread ends the step through a descriptor taken at its start when it can,
//! but that thread may never have started (the system refused it),
//! and a start the kernel refused a descriptor for leaves it naming nothing for good.
//! In both cases the command waited with no bound over the step at all,
//! which is finding 1 of the fifth review of this step and the rule the owner set on 2026-11-03,
//! "nothing should ever wait forever or indefinitely".
//!  The ceiling here is counted from the step's own start,
//! which this path is the moment of,
//! so it cannot be lost to a look that ran late.
//!  A step in an uninterruptible call is waited for here with no bound:
//! the owner chose that wait and asked for it to be said
//! (2026-11-10,
//! "Keep waiting and say so"),
//! and the notice says,
//! once and while this wait runs,
//! that the stop was sent and the kernel has not released the step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [outcome, ceilingStop] = await waitUntilItEndsTheStopOrTheCeiling(owned, stop, ceiling, 'a Git step of a commit');
//! ```

/// The child,
/// held here until it is reaped.
use crate::child_awaited_through_a_pidfd::OwnedChild;
/// The end of a child as the wrapper's exit reads it.
use crate::forwarding::{ChildOutcome, outcome_of};
/// The arm a stop of the service is told through,
/// and the error that says the stop ended the step.
use crate::stop_of_the_service_noticed_by_its_children::StopArm;
/// The processes the step started,
/// read while its number still names it.
use crate::process_tree::{TreeMember, descendants_of_read};
/// The one decision this ceiling shares with the notice's thread,
/// what it came to,
/// and why a step was waited for.
use crate::scratch_step_stall_decided_once::{TheStall, TheStallDecidedOnce, TheStallItWaitsThrough, WhyTheStepWasWaitedFor};
/// What one look at the step says about the kernel holding it,
/// and the look itself.
use crate::scratch_step_look_below::{WhatTheLookSays, what_the_look_says};
/// Ending what the step started,
/// each through a descriptor opened while its number still named it,
/// and what that came to.
use crate::member_signalled_through_its_pidfd::{Delivered, kill_members, unended_in};
/// The grace a stopped child gets before a task of the runtime collects it.
use crate::bounded_push::STOP_GRACE;
/// The queue the command's lines go through,
/// which a wait that lasts hands its line to.
use std::sync::mpsc::SyncSender;
/// The stop of a push,
/// which a stop of the service asks for.
use crate::bounded_push::stop_push;
/// The locks a stopped push leaves,
/// none of which this wait claims:
/// the step runs on a private copy of the index.
use crate::stopped_git_locks::LockLedger;
/// Waiting on the runtime's timer.
use tokio::time::sleep;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What:
///  A ceiling that came,
///  as the record states it.
/// Why:
///   The step's stop is recorded once,
///  whoever ended it,
///  so the wait answers the same shape the notice's thread does:
///  how long the step lasted,
///  and whether the programs it started could be listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CeilingStop = { waited: number; belowListed: boolean; belowUnended: number; stall?: TheStallItWaitsThrough };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CeilingStop {
    /// How long the step lasted before it was ended.
    pub waited: Duration,
    /// Whether the programs below the step were listed,
    /// so one that was not is not reported as gone.
    pub below_listed: bool,
    /// How many programs below the step were not ended:
    /// one the kernel refused a descriptor for,
    /// or one it refused the signal to.
    /// Such a program may still be running,
    /// which the record and the stop's words say.
    pub below_unended: u32,
    /// The ceiling that stood aside for this step,
    /// with the bound the step then ran under and why it was waited for,
    /// and nothing when the usual ceiling ended it.
    /// The record takes it from here when no notice thread ran,
    /// so a stop of a step that was waited for is still told from one that was not,
    /// and the words name the bound that really ended it.
    pub stall: Option<TheStallItWaitsThrough>,
}

// What:
//  A control that ends the step in the interval between the ceiling and the signal.
// Why:
//   That interval is a few instructions wide on a real host,
//  so a control that races a real exit proves nothing;
//  here the wait itself holds the step until the kernel says it ended,
//  and the status waits for the reap below.
#[cfg(all(test, target_os = "linux"))]
thread_local! {
    /// Whether the next ceiling holds its step until that step has ended by itself.
    static A_STEP_ENDS_BEFORE_THE_SIGNAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The longest a control's step is waited for once the ceiling has come.
///  Longer than any step a control starts takes to end,
/// so a starved runtime delays the control rather than failing it,
/// and a control whose step never ended fails rather than passing on a window it never made.
#[cfg(all(test, target_os = "linux"))]
const A_CONTROLS_BOUND: Duration = Duration::from_secs(30);

/// Arm the next ceiling to hold its step until that step has ended by itself.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn a_control_ends_the_step_here() {
    A_STEP_ENDS_BEFORE_THE_SIGNAL.with(|armed| { armed.set(true); });
}

/// What:
///  Hold the step at the moment between the ceiling and the signal,
///  when a control asked for it,
///  until the kernel says the step ended.
/// Why:
///   A step that finishes there was ended by its own exit,
///  and the reap below still collects that status,
///  which is what the caller must answer with.
///  The wait is bounded so a control's mistake ends the run with the wait's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await aControlHoldsTheStepUntilItEnded(owned);
/// ```
#[cfg(all(test, target_os = "linux"))]
async fn a_control_holds_the_step_until_it_ended(owned: &OwnedChild) {
    if !A_STEP_ENDS_BEFORE_THE_SIGNAL.with(|armed| { return armed.replace(false); }) {
        return;
    }
    let deadline: Instant = Instant::now() + A_CONTROLS_BOUND;
    while !owned.has_ended_without_reaping() {
        if Instant::now() >= deadline {
            return;
        }
        sleep(Duration::from_millis(1)).await;
    }
    return;
}

/// What:
///  The two moments that end a step this wait owns.
/// Why:
///   A ceiling is not one number any more:
///  a step waiting for a disk is slow and not stuck,
///  so the bound it is ended at is later than the bound it is noticed at
///  (`wait_ceilings::SCRATCH_COMMIT_STEP_STALL_CEILING`),
///  and both are counted from the step's own start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ceilings = { usually: number; whileTheKernelHoldsIt: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ceilings {
    /// The longest the step may last when nothing holds it.
    pub usually: Duration,
    /// The longest it may last while the kernel holds it in a call a signal cannot end,
    /// and never sooner than `usually`.
    pub while_the_kernel_holds_it: Duration,
}

/// What:
///  What the ceiling of one step is told about the command it runs in.
/// Why:
///   The wait that owns the child needs the one decision it shares with the notice's thread,
///  so the two cannot disagree about whether the step is waited for,
///  and it needs the queue the command's lines go through,
///  which is the notice's writer thread.
///  Both are the command's and neither is a property of the ceiling,
///  so they travel together and the wait keeps one argument for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheCeilingIsTold = { stall?: TheStallDecidedOnce; lines?: Queue };
/// ```
pub struct WhatTheCeilingIsTold<'a> {
    /// The one decision this ceiling shares with the notice's thread,
    /// and nothing when this ceiling is the only enforcer of its bound:
    /// it then makes the decision itself, once, exactly as a shared one is made.
    pub stall: Option<&'a TheStallDecidedOnce>,
    /// Where the command's lines go,
    /// and nothing when it has no writer to hand one to.
    pub lines: Option<&'a SyncSender<String>>,
}

impl WhatTheCeilingIsTold<'_> {
    /// What:
    ///  A ceiling that is the only enforcer of its bound and has no writer of its own.
    /// Why:
    ///   A wait no notice thread watches still stands aside for a step the kernel holds,
    ///  and still says nothing while it waits,
    ///  so it makes the one decision itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// WhatTheCeilingIsTold.alone();
    /// ```
    pub fn alone() -> Self {
        return Self { stall: None, lines: None };
    }
}

impl Ceilings {
    /// What:
    ///  The two bounds of a step whose ceiling is `ceiling`.
    /// Why:
    ///   The callers name one ceiling,
    ///  and the longer bound is the one this package measured,
    ///  or the ceiling itself when a person set one that is later.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ceilings = Ceilings.of(ceiling);
    /// ```
    pub fn of(ceiling: Duration) -> Self {
        return Self { usually: ceiling, while_the_kernel_holds_it: crate::wait_ceilings::stall_ceiling_of(ceiling) };
    }
}

/// What:
///  Wait for `owned` until it ends,
///  a stop of the service ends it,
///  or the ceiling of the step comes;
///  and answer how it ended,
///  with the stop when the ceiling came first.
/// Why:
///  This is the one wait that can name the step it owns and send its signal without a number a reuse can reach,
///  so a ceiling that no notice thread enforces is still a ceiling.
///  The step is signalled and reaped here,
///  never by a number read at the ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = now(); const ceilingCame = await race([owned.exited, stop.arm, sleep(ceiling)]); if (!ceilingCame) return [outcome, undefined]; killOwned(owned); return [{ signal: 9 }, { waited: now() - began, belowListed }];
/// ```
///
/// # Errors
///
/// The error of a wait the system failed,
/// or `EndedByStop` when the service's stop ended the child.
pub async fn wait_until_it_exits_the_stop_or_the_ceiling(
    mut owned: OwnedChild,
    mut stop: Option<StopArm>,
    ceilings: Ceilings,
    what: &str,
    told: WhatTheCeilingIsTold<'_>,
) -> std::io::Result<(ChildOutcome, Option<CeilingStop>)> {
    let WhatTheCeilingIsTold { stall: shared, lines } = told;
    // A ceiling with no other enforcer makes the one decision itself, and it lives as long as this wait.
    let own_decision: TheStallDecidedOnce = TheStallDecidedOnce::new();
    let the_stall: &TheStallDecidedOnce = shared.unwrap_or(&own_decision);
    let began: Instant = Instant::now();
    // The child's own end wins over the ceiling when both are ready:
    // a step that finished by itself landed, and stopped after its ceiling is a different record.
    let ended: Option<std::process::ExitStatus> = match stop.as_mut() {
        Some(arm) => tokio::select! {
            biased;
            () = arm.stop_told() => {
                stop_push(owned, STOP_GRACE, LockLedger::none()).await;
                return Err(crate::child_watch::ended_by_the_stop(what));
            }
            finished = owned.wait() => Some(finished?),
            () = sleep(ceilings.usually) => None,
        },
        None => tokio::select! {
            biased;
            finished = owned.wait() => Some(finished?),
            () = sleep(ceilings.usually) => None,
        },
    };
    if let Some(status) = ended {
        return Ok((outcome_of(status), None));
    }
    let mut waited: Duration = began.elapsed();
    // What: The step reached its ceiling while the kernel held it in a disk wait, or while nothing could be learned about it.
    // Why:  Such a step is waiting for the disk and not stuck in its own work,
    //       and a signal cannot shorten that call;
    //       ending it there is what failed a commit on 2026-11-11 whose step had landed a moment later.
    //       So the ceiling stands aside, the commit says so once, and the step is waited for to the bound the package
    //       measured, which an ordinary slow disk never reaches.
    //       The stand-aside is the one decision this ceiling shares with the notice's thread
    //       (`scratch_step_stall_decided_once.rs`), so an enforcer that found the step held is honoured here
    //       even when this look found nothing, and the step is never ended at the usual ceiling after the other stood aside.
    //       A look that could not be made decides nothing and is made again, up to that bound:
    //       reading it as "not held" is how a held step was killed at the usual ceiling (finding 1 of the 2026-11-12 review).
    let stall: Option<TheStallItWaitsThrough> = match what_the_ceiling_decides(&mut owned, ceilings, began, the_stall).await? {
        // A step that ended while nothing could be read about it landed, and its own status decides.
        HowTheCeilingWasDecided::ItEnded(status) => return Ok((outcome_of(status), None)),
        HowTheCeilingWasDecided::TheUsualCeiling => None,
        HowTheCeilingWasDecided::StoodAside(stood) => {
            let stalled: AStalledStep<'_> = AStalledStep {
                began,
                waited: stood.stood_aside_after,
                until: stood.bound,
                why: stood.why.clone(),
                lines,
            };
            match until_it_ends_or_the_hold_bound(&mut owned, &mut stop, stalled).await? {
                HowTheStallEnded::ItEnded(status) => return Ok((outcome_of(status), None)),
                // The bound came and the kernel still held it: the step is ended here as any step at its ceiling is.
                HowTheStallEnded::TheBound => waited = began.elapsed(),
                HowTheStallEnded::TheStopOfTheService => {
                    stop_push(owned, STOP_GRACE, LockLedger::none()).await;
                    return Err(crate::child_watch::ended_by_the_stop(what));
                }
            }
            Some(stood)
        }
    };
    // What the step started is read now, while the step is alive:
    // a number cannot be reused while its process holds it, and everything below the step is found by that number only
    // then.
    let walked: Option<Vec<TreeMember>> = i32::try_from(owned.id()).ok().and_then(descendants_of_read);
    // What: The interval between the ceiling and the signal.
    // Why:  A step that finishes there was ended by its own exit, and the status the reap collects says so;
    //       the control holds the step here so that order is its own and not a clock's.
    #[cfg(all(test, target_os = "linux"))]
    a_control_holds_the_step_until_it_ended(&owned).await;
    // The step itself is killed through the child this path still owns and has not reaped,
    // so no process that took its number can be reached.
    let _killed: std::io::Result<()> = owned.kill();
    let ended_below: Vec<Delivered> = kill_members(walked.as_deref().unwrap_or_default());
    // What: The programs below the step that this wait could not end.
    // Why:  A signal the kernel refused leaves the program running, and the stop must say so rather than report a walk
    //       that ended everything it listed.
    let below_unended: u32 = unended_in(ended_below.as_slice());
    // A killed step is reaped here, and this wait ends when the kernel releases it:
    // a step in an uninterruptible call cannot act on `SIGKILL` until that call returns,
    // and the owner chose that this commit waits the kernel's time and says so
    // (2026-11-10, "Keep waiting and say so"), an exception to his rule that nothing waits for ever.
    // The line that says why comes from this wait and never from the notice's thread, which a system at its thread
    //  limit never started (`wait_after_the_stop_saying_the_kernel_holds_it.rs`).
    let reaped: Option<std::process::ExitStatus> = match crate::wait_after_the_stop_saying_the_kernel_holds_it::until_reaped(&mut owned, waited, lines)
        .await
    {
        Ok(collected) => Some(collected),
        // What: A status the wait could not collect.
        // Why:  The step was signalled here and the wait knows nothing of what ended it, which is what the answer says;
        //       a commit is not failed over the reading of a status it already ended the step for
        //       (owner: "Commits should never fail.").
        Err(_not_collected) => None,
    };
    // A status the reap collected decides what ended the step: one that is not the signal this wait sent proves the
    //  step finished by itself in the interval before the signal, and its exit is what the caller must answer.
    if let Some(status) = reaped
        && it_ended_by_itself(status)
    {
        return Ok((outcome_of(status), None));
    }
    return Ok((ChildOutcome::Signaled(libc::SIGKILL), Some(CeilingStop { waited, below_listed: walked.is_some(), below_unended, stall })));
}

/// What:
///  What the ceiling does now that it has come:
///  stands aside for this step, or ends it at the usual ceiling.
/// Why:
///   The stand-aside is one decision shared with the notice's thread,
///  so a step the other enforcer found held is waited for here even when this look found nothing.
///  A look that found the kernel holding a process of the step stands aside,
///  a look that found none has the usual ceiling end the step,
///  and a look that could not be made decides nothing at all:
///  the ceiling stays armed and the look is made again until the bound the step is waited for comes,
///  which is where a step nothing can be learned about is ended.
///  Reading a failed look as "not held" is how a held step was killed at the usual ceiling,
///  and a commit that would have landed failed (finding 1 of the 2026-11-12 review).
///  Every wait here is bounded by that same bound,
///  and a step that ends by itself is noticed on the way to the next look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (stall.state() === 'undecided') { const look = whatTheLookSays(step); if (look.held) return stall.standAside(...); if (look.notHeld) return stall.theUsualCeilingEndsIt() ? 'the-usual-ceiling' : stall.state(); if (began.elapsed() >= bound) return stall.standAside({ why: 'its-state-could-not-be-read' }); await sleep(250) }
/// ```
async fn what_the_ceiling_decides(owned: &mut OwnedChild, ceilings: Ceilings, began: Instant, the_stall: &TheStallDecidedOnce) -> std::io::Result<HowTheCeilingWasDecided> {
    loop {
        match the_stall.state() {
            TheStall::StoodAside(stood) => return Ok(HowTheCeilingWasDecided::StoodAside(stood)),
            TheStall::TheUsualCeilingEndsIt => return Ok(HowTheCeilingWasDecided::TheUsualCeiling),
            TheStall::Undecided => {}
        }
        // What: The later bound is a stand-aside only when it really is later.
        // Why:  A person may set a ceiling above the bound this release waits a step for,
        //       and a control may plant one: standing aside for a bound that is not later would end the step at its ceiling
        //       and say it was waited for.
        if ceilings.while_the_kernel_holds_it <= ceilings.usually {
            let _claimed: bool = the_stall.the_usual_ceiling_ends_it();
            return Ok(HowTheCeilingWasDecided::TheUsualCeiling);
        }
        let what_it_says: WhatTheLookSays = what_the_look_says(owned.id());
        // What: A step whose state could not be read is waited for to the bound this ceiling gives a held step,
        //       and the wait is never the usual ceiling's.
        // Why:  Nothing is known about it, so ending it at the usual ceiling is ending a step that may be waiting for the disk;
        //       the bound is the one the package measured and nothing waits past it.
        let nothing_is_known_and_the_bound_came: bool = matches!(what_it_says, WhatTheLookSays::CouldNotLook) && began.elapsed() >= ceilings.while_the_kernel_holds_it;
        let decided: TheStall = match what_it_says {
            WhatTheLookSays::TheKernelHoldsIt(held) => the_stall.stand_aside(TheStallItWaitsThrough {
                stood_aside_after: began.elapsed(),
                bound: ceilings.while_the_kernel_holds_it,
                why: WhyTheStepWasWaitedFor::TheKernelHeldIt(held),
            }),
            WhatTheLookSays::NotHeld => {
                let _claimed: bool = the_stall.the_usual_ceiling_ends_it();
                the_stall.state()
            }
            WhatTheLookSays::CouldNotLook if nothing_is_known_and_the_bound_came => the_stall.stand_aside(TheStallItWaitsThrough {
                stood_aside_after: began.elapsed(),
                bound: ceilings.while_the_kernel_holds_it,
                why: WhyTheStepWasWaitedFor::ItsStateCouldNotBeRead,
            }),
            WhatTheLookSays::CouldNotLook => {
                // What: A look that failed decides nothing: it is made again at the pace a failed look is retried at.
                // Why:  A `/proc` read refused at a descriptor or memory limit is the documented cause,
                //       and it is a fact about this look and never about the step.
                //       The step ending by itself is noticed here too, so a commit whose step landed is never waited for to that bound.
                tokio::select! {
                    biased;
                    status = owned.wait() => return Ok(HowTheCeilingWasDecided::ItEnded(status?)),
                    () = sleep(crate::wait_after_the_stop_saying_the_kernel_holds_it::AFTER_THE_STOP) => continue,
                }
            }
        };
        return Ok(match decided {
            TheStall::StoodAside(stood) => HowTheCeilingWasDecided::StoodAside(stood),
            // The other enforcer had the usual ceiling end this step before this look could stand aside for it.
            TheStall::TheUsualCeilingEndsIt | TheStall::Undecided => HowTheCeilingWasDecided::TheUsualCeiling,
        });
    }
}

/// What:
///  How the ceiling of a step came out once it had come.
/// Why:
///   The two answers end the step differently:
///  a ceiling that stood aside leaves the step running under the later bound,
///  and one that ends it at the usual ceiling does so at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HowTheCeilingWasDecided = 'the-usual-ceiling' | { stoodAside: TheStallItWaitsThrough } | { itEnded: number };
/// ```
enum HowTheCeilingWasDecided {
    /// The usual ceiling ends the step here.
    TheUsualCeiling,
    /// The ceiling stood aside, and the step is waited for to the bound this names.
    StoodAside(TheStallItWaitsThrough),
    /// The step ended by itself while nothing could be learned about it.
    ItEnded(std::process::ExitStatus),
}

/// What:
///  One wait for a step the kernel holds in a disk wait is given.
/// Why:
///   The bound is counted from the step's own start,
///  the line names the process the kernel is holding,
///  and the wait that lasts is the one that says it,
///  because a notice thread the system refused says nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AStalledStep = { began: number; waited: number; until: number; why: WhyTheStepWasWaitedFor; lines?: Queue };
/// ```
struct AStalledStep<'a> {
    /// When the step started.
    began: Instant,
    /// How long it had lasted when its ceiling came.
    waited: Duration,
    /// The longest it may last while the ceiling stands aside.
    until: Duration,
    /// Why the ceiling stood aside:
    /// the process the kernel is holding,
    /// or nothing, when nothing could be learned about the step.
    why: WhyTheStepWasWaitedFor,
    /// Where the command's lines go,
    /// and nothing when it has no writer to hand one to.
    lines: Option<&'a SyncSender<String>>,
}

/// What:
///  How a wait for a step the kernel holds ended.
/// Why:
///   The three answers end the command differently:
///  a step that ended by itself lands,
///  a bound that came stops the step,
///  and a stop of the service is reported as itself and never as a ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HowTheStallEnded = { itEnded: number } | { theBound: true } | { theStopOfTheService: true };
/// ```
enum HowTheStallEnded {
    /// The step ended by itself while the kernel held it.
    ItEnded(std::process::ExitStatus),
    /// The bound came and the kernel still held it.
    TheBound,
    /// The service's stop was told while the step waited.
    TheStopOfTheService,
}

/// What:
///  Wait for a step the kernel holds until it ends,
///  the bound comes,
///  or the service's stop ends it,
///  saying once why the wait lasts.
/// Why:
///  A step inside an uninterruptible call is slow and not stuck,
///  so ending it at the ceiling it reached fails a commit that would have landed.
///  The wait stays and says so once:
///  a person who read the same sentence three times in one failed commit learned nothing the first two times.
///  A line is dropped rather than waited for when the writer cannot take it,
///  because a reader that stopped reading must never hold this wait,
///  and the line is handed over again at the next look while nothing was taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = await untilItEndsOrTheHoldBound(owned, stop, stalled); if (ended.itEnded) return [ended.itEnded, undefined];
/// ```
async fn until_it_ends_or_the_hold_bound(owned: &mut OwnedChild, stop: &mut Option<StopArm>, stalled: AStalledStep<'_>) -> std::io::Result<HowTheStallEnded> {
    let AStalledStep { began, waited, until, why, lines } = stalled;
    let mut said: bool = false;
    let mut pause: Duration = Duration::ZERO;
    loop {
        if began.elapsed() >= until {
            return Ok(HowTheStallEnded::TheBound);
        }
        // What: The service's stop is waited for beside the step.
        // Why:  A shutdown must not wait the bound this wait keeps;
        //       a step it stops is reported as the stop and never as a ceiling.
        let told = async {
            return match stop.as_mut() {
                Some(arm) => arm.stop_told().await,
                None => std::future::pending().await,
            };
        };
        tokio::select! {
            biased;
            () = told => return Ok(HowTheStallEnded::TheStopOfTheService),
            status = owned.wait() => return Ok(HowTheStallEnded::ItEnded(status?)),
            () = sleep(pause) => {}
        }
        if !said {
            // What: The line that says why the ceiling did not come, and the process the kernel is holding.
            // Why:  A step nothing could be learned about has no such process to name,
            //       and a line that named one would state a fact no look found (`scratch_step_held_by_the_kernel.rs`).
            let line: String = crate::scratch_step_held_by_the_kernel::a_stall_is_waited_for(crate::scratch_step_held_by_the_kernel::AStall {
                step: i32::try_from(owned.id()).unwrap_or(0),
                waited,
                until,
                why: why.clone(),
            });
            said = match lines {
                Some(queue) => queue.try_send(line).is_ok(),
                None => true,
            };
        }
        pause = crate::wait_after_the_stop_saying_the_kernel_holds_it::AFTER_THE_STOP;
    }
}

/// What:
///  Whether a reaped status proves the step ended by itself,
///  rather than by the signal this wait sent.
/// Why:
///   `SIGKILL` is the only signal sent here,
///  so a status that is not it is the step's own end,
///  and a status with a code is an exit that the kill could not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itEndedByItself = (status) => status.code !== null || status.signal !== 'SIGKILL';
/// ```
fn it_ended_by_itself(status: std::process::ExitStatus) -> bool {
    // `if let Some(code) = ...` runs only when the child exited normally, and a kill cannot turn an exit into a code.
    if status.code().is_some() {
        return true;
    }
    #[cfg(unix)]
    {
        // The trait adds `.signal()` to exit statuses on Unix.
        use std::os::unix::process::ExitStatusExt;
        return status.signal() != Some(libc::SIGKILL);
    }
    #[cfg(not(unix))]
    return false;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "child_awaited_until_it_exits_the_stop_or_the_ceiling_tests.rs"]
mod tests;
