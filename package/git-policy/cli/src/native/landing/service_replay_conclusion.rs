//! What:
//!  The refusal of a conclusion whose operation another command ended while the commit was prepared.
//! Why:
//!   A commit that concludes a merge,
//! a cherry-pick or a revert is made again on the branch as it is now only over the state it was prepared over:
//!  another command that concluded or abandoned the operation leaves nothing to conclude,
//!  and an ordinary commit of the same tree is not what the person asked for
//!  (`service_replay.rs`,
//!  `Captured::conclusion`).
//!  The comparison is split from the replay so neither file passes its line bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await theSameConclusion({ context, cwd, capture, expected, branch });
//! ```

use crate::commit_transaction::{TransactionStop, failed};
use crate::transaction_git::GitContext;
use std::path::Path;

/// What:
///  Refuses a conclusion the branch moved under when the operation it concluded is no longer the same one.
/// Why:
///   A conclusion's parents come from the state the worktree holds,
/// and another command that concluded or abandoned the operation leaves nothing this commit could conclude;
/// making an ordinary commit of the same tree would be a commit the person never asked for.
/// The comparison is against what the preparation saw,
/// never against a reading taken now
/// (`Captured::conclusion`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await theSameConclusion({ context, cwd, capture, expected, branch });
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` naming the operation and what to run,
/// or naming the read that failed.
pub(crate) fn the_same_conclusion(
    context: &GitContext,
    cwd: &Path,
    capture: &crate::capture_invocation::InvocationCapture,
    expected: &str,
    branch: &str,
) -> Result<(), TransactionStop> {
    let now: String =
        crate::conclusion_copy::conclusion_state_text(context, cwd, capture.layout.git_dir.as_path(), capture.layout.ref_format).map_err(the_state_read_failure)?;
    if now == expected {
        return Ok(());
    }
    let failure: crate::service_messages::ClientFailure = crate::service_messages::ClientFailure::ConclusionStateChanged { branch: branch.to_string() };
    return Err(TransactionStop::Message(crate::service_messages::render(&failure)));
}

/// The failure of reading the conclusion state,
/// as the stop that names what could not be read.
pub(crate) fn the_state_read_failure(error: crate::transaction_git::GitFailure) -> TransactionStop {
    return failed(format!("reading the state of the merge, cherry-pick or revert this commit concludes failed: {error}"));
}
