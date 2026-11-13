//! What:
//!  The results a push returns when it sent nothing:
//!  one held back by a verdict,
//!  one skipped with a note,
//!  and one whose branch moved past the commit it read.
//! Why:
//!   Each writes the one line the person reads and says what the queue does next,
//! so each is a result of its own rather than an arm inside the push;
//! they live here,
//! and `auto_push.rs` re-exports them,
//! because the push itself would pass its own line ceiling with them beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! skippedWith(note, err); heldBackResult(branchRef, stop, err); movedPastThePin(branchRef, err);
//! ```

/// The push's result,
/// what one came to,
/// the plan a skip ends in,
/// and the tag its diagnostics carry.
use crate::auto_push::{AutoPushOutcome, AutoPushResult, Planned, TAG};
/// The verdicts that hold a push back,
/// and the lines each names.
use crate::blocked_commit_lines::{branch_name, held_back_line, undecided_line, unanswered_line};
/// Where a verdict makes the push stop.
use crate::push_held_back_by_a_blocked_commit::WhereThePushStops;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// `Write` is the stderr sink the terminal lines go to.
use std::io::Write;

/// What:
///  The result of a push that sent nothing because a verdict holds the branch back.
/// Why:
///   The line the queue records for the person and the line every command prints are the same words,
/// and the outcome tells
///  the service to try the branch again rather than to report a failure or to settle the push as done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heldBack = (branchRef, stop, err) => ({ outcome: 'held-back', exitCode: 0, shown: line });
/// ```
pub(crate) fn held_back_result(branch_ref: &str, stop: &WhereThePushStops, err: &mut dyn Write) -> AutoPushResult {
    let line: String = match stop {
        WhereThePushStops::Nothing => String::new(),
        WhereThePushStops::Before(stopped) => held_back_line(branch_ref, stopped),
        WhereThePushStops::Undecided(stopped) => undecided_line(branch_ref, stopped),
        WhereThePushStops::Unanswered(stopped) => unanswered_line(branch_ref, stopped),
    };
    debug(TAG, format!("nothing is pushed of {branch_ref}: {line}").as_str());
    // Terminal output error: the line is the only diagnostic of a push that sent nothing.
    let _written = writeln!(err, "{line}");
    return AutoPushResult { outcome: AutoPushOutcome::HeldBack, exit_code: 0, shown: line, push_addresses: Vec::new() };
}

/// What:
///  The plan that skips a branch with a note,
/// after writing the note to the terminal sink.
/// Why:
///   The note is the only diagnostic of a push that never ran,
/// so the result carries it as well,
/// for the caller that
///  reports it elsewhere than the sink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skippedWith(note: string, err: Writable): Planned;
/// ```
pub(crate) fn skipped_with(note: String, err: &mut dyn Write) -> Planned {
    // Terminal output error: the note itself is the only diagnostic; a failed write to stderr is not a push failure.
    let _written = writeln!(err, "{note}");
    return Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: note, push_addresses: Vec::new() });
}

/// What:
///  The result of a matching push that sent nothing because the branch moved past the commit it may send.
/// Why:
///   A push under `push.default=matching` gives Git no refspec,
///  so Git resolves the branch itself and would send a commit that landed after the reading,
///  whose own gate may still keep it local.
///  Nothing is sent,
///  so nothing is lost,
///  and the retry plan the service keeps tries the branch again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const movedPastThePin = (branchRef, err) => ({ outcome: 'held-back', exitCode: 0, shown: line });
/// ```
pub(crate) fn moved_past_the_pin(branch_ref: &str, err: &mut dyn Write) -> AutoPushResult {
    let line: String = format!(
        "{} moved after this push read it, and `push.default=matching` gives Git no refspec, so nothing was sent; the commit service tries the branch again.",
        branch_name(branch_ref)
    );
    debug(TAG, line.as_str());
    // Terminal output error: the line is the only diagnostic of a push that sent nothing.
    let _written = writeln!(err, "{line}");
    return AutoPushResult { outcome: AutoPushOutcome::HeldBack, exit_code: 0, shown: line, push_addresses: Vec::new() };
}
