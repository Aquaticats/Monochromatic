//! What:
//!  The one decision two enforcers of a step's ceiling make together:
//!  whether that ceiling stands aside for a step the kernel holds in a disk wait,
//!  the bound the step then runs under,
//!  and why it was waited for.
//! Why:
//!  The scratch `git commit` has two enforcers of one ceiling:
//!  the notice's thread, which prints the lines and can end the step through the descriptor it took at its start,
//!  and the wait that owns the child, which needs no thread and no number a reuse can reach.
//!  A step inside an uninterruptible call is slow and not stuck,
//!  and a signal cannot shorten that call,
//!  so both enforcers stand aside for it instead of ending it
//!  (`wait_ceilings::SCRATCH_COMMIT_STEP_STALL_CEILING`).
//!  When each enforcer decided that from its own look at the kernel,
//!  one could stand aside while the other ended the step at the usual ceiling:
//!  the commit failed at the bound the stand-aside exists to pass,
//!  and the words named a bound and a wait that never happened
//!  (findings 1 and 2 of the 2026-11-12 review of the stand-aside).
//!  So the stand-aside is one property of the step,
//!  decided here once and read by both enforcers.
//!  The first decision is the one the whole step keeps:
//!  the bound the words name and the wait the stop records are that decision's,
//!  whoever made it,
//!  and neither enforcer can end the step at the usual ceiling after the other has stood aside.
//!  A look that failed is a third answer and never "not held":
//!  it decides nothing,
//!  the ceiling stays armed,
//!  and the look is made again until the bound the step is waited for comes,
//!  which is the same answer `process/wait_after_the_stop_saying_the_kernel_holds_it.rs` gives
//!  and the one the notice's blind looks already gave.
//!  The same shape is used for the bound of every other Git step
//!  (`push/child_watch.rs`, `process/step_output_read_whole_through_pipes.rs`, `TheWatchOfAStep`):
//!  one value, read once, that says who may end the step and how much later.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stall = new TheStallDecidedOnce(); stall.state(); stall.standAside({ waited, bound, why }); stall.theUsualCeilingEndsIt();
//! ```

/// The process the kernel is holding,
/// which is the step itself or a program it started.
use crate::scratch_step_held_by_the_kernel::HeldProgram;
/// The one lock around the decision,
/// taken only to read or write one small value and never held across a look or a wait.
use std::sync::Mutex;
/// Time spans,
/// for the moment the ceiling stood aside and the bound it named.
use std::time::Duration;

/// What:
///  Why a step's ceiling stopped ending the step.
/// Why:
///   The two reasons give the person different things to check and different words:
///  a step the kernel holds is waiting for the disk,
///  and one whose state could not be read is one the wrapper knew nothing about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyTheStepWasWaitedFor = { theKernelHeldIt: HeldProgram } | 'its-state-could-not-be-read';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhyTheStepWasWaitedFor {
    /// The kernel held a process of the step in a call a signal cannot end.
    TheKernelHeldIt(HeldProgram),
    /// Nothing could be learned about the step:
    /// the look at `/proc` could not be made.
    ItsStateCouldNotBeRead,
}

/// What:
///  A ceiling that stood aside for a step,
///  and the bound the step then runs under.
/// Why:
///   One stand-aside has these facts and no others,
///  and every one of them is something the person or the record needs:
///  when the ceiling stopped ending the step,
///  the bound that now ends it,
///  and why it was waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStallItWaitsThrough = { stoodAsideAfter: number; bound: number; why: WhyTheStepWasWaitedFor };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheStallItWaitsThrough {
    /// How long the step had lasted when its ceiling came and stood aside.
    pub stood_aside_after: Duration,
    /// The longest the step may last while the ceiling stands aside,
    /// counted from the step's start.
    pub bound: Duration,
    /// Why the ceiling stood aside.
    pub why: WhyTheStepWasWaitedFor,
}

/// What:
///  What is decided about one step's ceiling.
/// Why:
///  Three states stay apart:
/// nothing is decided yet, so an enforcer may still stand aside;
/// the ceiling stood aside, and the step is waited for to the bound it names;
/// and the usual ceiling ends the step,
/// so no enforcer may stand aside for it afterwards and no words may claim a wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStall = 'undecided' | { stoodAside: TheStallItWaitsThrough } | 'the-usual-ceiling-ends-it';
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheStall {
    /// No enforcer has decided anything yet.
    Undecided,
    /// The ceiling stands aside,
    /// and the step is waited for to the bound this names.
    StoodAside(TheStallItWaitsThrough),
    /// The usual ceiling ends this step.
    TheUsualCeilingEndsIt,
}

/// What:
///  The one decision of one step's ceiling,
///  written once and read by both enforcers of that ceiling.
/// Why:
///   One step has one ceiling,
///  and two answers to "did it stand aside" is how a step was ended at the usual ceiling
///  while the person was told it was waited for to the later bound.
///  The lock covers one value of three states and one clone of it,
///  never a look and never a wait,
///  so a look that costs a millisecond never holds an enforcer that ends a step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheStallDecidedOnce { #decided = 'undecided'; state() {} standAside(stall) {} theUsualCeilingEndsIt() {} }
/// ```
pub struct TheStallDecidedOnce {
    /// The decision as it stands.
    decided: Mutex<TheStall>,
}

impl Default for TheStallDecidedOnce {
    /// A ceiling nothing has been decided about.
    fn default() -> Self {
        return Self::new();
    }
}

impl TheStallDecidedOnce {
    /// What:
    ///  A ceiling no enforcer has decided anything about.
    /// Why:
    ///   The commit makes one of these for the step it runs,
    ///  and hands it to the notice's thread and to the wait that owns the child,
    ///  so both ask the same value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new TheStallDecidedOnce();
    /// ```
    pub fn new() -> Self {
        return Self { decided: Mutex::new(TheStall::Undecided) };
    }

    /// What:
    ///  The decision as it stands,
    ///  without making one.
    /// Why:
    ///  An enforcer that meets a step whose ceiling already stood aside must honour that bound
    ///  rather than decide from its own look,
    ///  and one that meets a ceiling already ending the step must not stand aside.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stall.state(); // 'undecided' | { stoodAside } | 'the-usual-ceiling-ends-it'
    /// ```
    pub fn state(&self) -> TheStall {
        // A poisoned lock means an enforcer panicked while deciding; the value it left is still one of the three states.
        let kept = self.decided.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        return kept.clone();
    }

    /// What:
    ///  Stand aside for this step,
    ///  unless an enforcer has already had the usual ceiling end it.
    /// Why:
    ///   A step the kernel holds must be waited for whichever enforcer saw it first,
    ///  and the bound the first look found is the bound the step keeps,
    ///  so the words and the stop name one wait and not two.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const after = stall.standAside({ stoodAsideAfter, bound, why }); // { stoodAside } | 'the-usual-ceiling-ends-it'
    /// ```
    pub fn stand_aside(&self, stall: TheStallItWaitsThrough) -> TheStall {
        let mut kept = self.decided.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let TheStall::Undecided = &*kept {
            *kept = TheStall::StoodAside(stall);
        }
        return kept.clone();
    }

    /// What:
    ///  Have the usual ceiling end this step,
    ///  unless the ceiling already stood aside for it.
    /// Why:
    ///   An enforcer that found no process of the step in the kernel's hold is about to end it at the usual ceiling,
    ///  and a stand-aside decided afterwards would print and record a wait that never happened.
    ///  Answering false says the other enforcer stood aside first,
    ///  so this one must wait too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (!stall.theUsualCeilingEndsIt()) { /* the ceiling stood aside: wait instead */ }
    /// ```
    pub fn the_usual_ceiling_ends_it(&self) -> bool {
        let mut kept = self.decided.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let TheStall::Undecided = &*kept {
            *kept = TheStall::TheUsualCeilingEndsIt;
        }
        return matches!(&*kept, TheStall::TheUsualCeilingEndsIt);
    }
}

/// The decision and its controls stay in the release executable only as the type above.
#[cfg(test)]
#[path = "scratch_step_stall_decided_once_tests.rs"]
mod tests;
