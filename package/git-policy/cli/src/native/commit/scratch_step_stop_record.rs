//! What:
//!  The record of one stop of the scratch `git commit`,
//!  taken whole from the enforcer of its ceiling that ended the step:
//!  how long the step lasted,
//!  the bound that ended it,
//!  and what became of the programs below it.
//! Why:
//!   Two enforcers watch one ceiling, and each answers a stop of its own.
//!  Taking one figure from one of them and one from the other is how a stop named the longer bound
//!  and a wait that never happened,
//!  for a step the other enforcer had ended at the usual ceiling
//!  (finding 2 of the 2026-11-12 review of the stand-aside).
//!  So a stop is taken from one enforcer as a whole,
//!  and that enforcer is the one that ended the step:
//!  the notice's thread when it recorded the stop,
//!  and the wait that owns the child otherwise.
//!  The bound itself is not either enforcer's:
//!  it is the one decision they share (`scratch_step_stall_decided_once.rs`),
//!  so the words name the bound the step really ran under
//!  and the reason it was waited for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stop = theStopOfTheStep({ facts, theWaitsStop, theStall, theUsualCeiling, signs }); // StopStop | undefined
//! ```

/// What the notice's thread saw of the step,
/// and the stop the wait that owns the child answered.
use crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop;
use crate::scratch_commit_step::ScratchStepFacts;
/// The words of a stop,
/// the bound that ended it,
/// and what a step was waited for.
use crate::scratch_step_stop_words::{ScratchStepStop, TheBoundThatEndedACommit};
use crate::scratch_step_stall_decided_once::TheStall;
/// Time spans,
/// for the usual ceiling a stop names.
use std::time::Duration;

/// What:
///  What the two enforcers of one step's ceiling answered about its stop.
/// Why:
///  The notice's facts and the wait's stop are two records of one step,
///  and the one decision between them says which bound really ended it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheEnforcersOfAStep = { facts: ScratchStepFacts; theWaitsStop?: CeilingStop; theStall: TheStall; theUsualCeiling: number; signs: boolean };
/// ```
#[derive(Clone, Copy)]
pub struct TheEnforcersOfAStep<'a> {
    /// What the notice's thread saw while the step ran,
    /// and the stop it recorded when it ended the step.
    pub facts: &'a ScratchStepFacts,
    /// The stop the wait that owns the child answered,
    /// and nothing when that wait ended no step.
    pub the_waits_stop: Option<&'a CeilingStop>,
    /// The one decision both enforcers read:
    /// whether the ceiling stood aside, and the bound the step then ran under.
    pub the_stall: &'a TheStall,
    /// The usual ceiling of a commit whose message came with the command,
    /// which the words name when that is the bound that ended the step.
    pub the_usual_ceiling: Duration,
    /// Whether the commit asks for a signature,
    /// which makes the signing program the first thing to check.
    pub signs: bool,
}

/// What:
///  The stop of one step,
///  and nothing when nothing stopped it.
/// Why:
///   A commit that landed has no stop to word,
///  and a stop is taken from the enforcer that ended the step,
///  with the bound read from the one decision both enforcers share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (facts.stoppedAfter) return { waited: facts.stoppedAfter, bound: theBound(theStall, theUsualCeiling), below: facts };
/// if (!theWaitsStop) return undefined; return { waited: theWaitsStop.waited, bound: theBound(theStall, theUsualCeiling), below: theWaitsStop };
/// ```
pub fn the_stop_of_the_step(enforcers: TheEnforcersOfAStep<'_>) -> Option<ScratchStepStop> {
    let TheEnforcersOfAStep { facts, the_waits_stop, the_stall, the_usual_ceiling, signs } = enforcers;
    let bound_that_ended_it: TheBoundThatEndedACommit = match the_stall {
        TheStall::StoodAside(stall) => TheBoundThatEndedACommit::TheStallBound { bound: stall.bound, why: stall.why.clone() },
        // A ceiling that ended a step never stood aside for it, and one still deciding ended nothing:
        // the usual ceiling is the bound such a step ran under.
        TheStall::TheUsualCeilingEndsIt | TheStall::Undecided => TheBoundThatEndedACommit::TheUsualCeiling(the_usual_ceiling),
    };
    if let Some(waited) = facts.stopped_after {
        return Some(ScratchStepStop {
            waited,
            bound_that_ended_it,
            signs,
            below_unlisted: facts.below_unlisted,
            below_unended: facts.below_unended,
        });
    }
    let stop: &CeilingStop = the_waits_stop?;
    return Some(ScratchStepStop {
        waited: stop.waited,
        bound_that_ended_it,
        signs,
        below_unlisted: !stop.below_listed,
        below_unended: stop.below_unended,
    });
}

/// The record and its words stay in the release executable;
/// their controls sit beside them.
#[cfg(test)]
#[path = "scratch_step_stop_record_tests.rs"]
mod tests;
