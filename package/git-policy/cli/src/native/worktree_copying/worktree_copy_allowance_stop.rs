//! What:
//!  What becomes of one add whose own steps have spent the allowance for them:
//!  what the ladder prepared is
//!  settled,
//!  the record the command wrote is released,
//!  the empty destination it reserved is taken away,
//!  and the add
//!  says what stopped it.
//! Why:
//!  The allowance itself is `worktree/worktree_add_own_work_deadline.rs`;
//!  what an add that ends at it does is
//!  this file,
//!  so the lifecycle keeps its own size and a reader of one subject finds all of it in one place.
//!  Nothing here is
//!  ever removed that this command did not make:
//!  `settle_created_destination` checks the identity of the
//!  destination first,
//!  so a folder another program wrote into stays and is named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const failure = abandonTheAdd({ invocation, ladder, spent, held, madeHere, reserved });
//! ```

use crate::diagnostic_log::warn;
use crate::worktree_copy::{GitRunner, GitStatus, Invocation};
use crate::worktree_copy_after_refusal::settle_refused_destination;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_ladder::{Ladder, LadderResult};
use crate::worktree_subvolume::Reservation;
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCopyAllowanceStop";

/// What:
///  Give up the add's durable record when the add made no worktree that owes a copy.
/// Why:
///   The record exists so a command killed before its copy can be reported and finished;
///  an add that made nothing owes
///  nothing,
///  and a record left behind is exactly what the next command would trip on:
///  the recovery would report a
///  worktree that is not there and keep the record while it stands.
///  A record that cannot be removed is named rather than failing the add,
///  as every other release of a claim is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function releaseTheRecord(claim?: StageClaim) { claim?.settle(); }
/// ```
pub(crate) fn release_the_record(claim: Option<StageClaim>) {
    let Some(unused) = claim else {
        return;
    };
    if let Err(error) = unused.settle() {
        warn(TAG, format!("the record of the worktree add could not be removed ({})", error.0).as_str());
    }
}

/// What:
///  Ends an add whose own steps have spent the allowance for them,
///  before real Git makes a worktree the wrapper could no longer
///  finish:
///  settles what the ladder prepared,
///  takes away the empty folder this command reserved,
///  and says what stopped the add.
/// Why:
///   The person's command is Git's own and would wait as long as Git waits,
///  so the wrapper starts it only when it can finish what it
///  owes after it:
///  a worktree Git made and the wrapper could not fill with the source's ignored state is a worktree the
///  next command has to report and a person has to finish.
///  An add stopped here made nothing,
///  so nothing of it is left:
///  the
///  subvolume or folder the ladder prepared goes,
///  the claim on the stage name goes,
///  and no registration exists to trip the next
///  command.
///  Everything removed is this command's own and is checked before it goes
///  (`settle_created_destination`),
///  so a
///  folder another program wrote into stays and is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function abandonTheAdd({ ladder, spent, madeHere, reserved }): WorktreeCopyError;
/// ```
pub(crate) fn abandon_the_add(
    invocation: &Invocation,
    ladder: Ladder,
    spent: &crate::worktree_add_own_work_deadline::Spent,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
    made_here: bool,
    reserved: &Reservation,
    held: &mut Option<StageClaim>,
) -> WorktreeCopyError {
    // Nothing ran in the worktree, so the ladder settles only what it made before Git: the empty subvolume Git would have
    // filled, or the claim on a stage name. Its status is the one a Git that did not run reports.
    let settled: LadderResult = ladder.finish(GitStatus { code: None }, run_git, &invocation.context, &mut std::io::sink(), stderr);
    release_the_record(settled.claim);
    // A claim no rung took is this command's until the copy owns it, and there is no copy here.
    release_the_record(held.take());
    if made_here
        && let Reservation::Mine(made) = reserved
        && let Some(line) = settle_refused_destination(made)
    {
        // The person's stream carries this line, and a closed one keeps the failure's own status.
        let _written = stderr.write_all(line.as_bytes());
    }
    return WorktreeCopyError(spent.the_add_stopped_line(invocation.destination.as_deref()));
}
