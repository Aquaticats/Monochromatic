//! What:
//!  The staging step of a preparation,
//!  reachable for a control that asks it directly.
//! Why:
//!  The step decides whether a path in conflict stops the commit or is excluded from it,
//!  and that answer is what its controls assert;
//!  it lives beside the preparation it belongs to,
//!  in a file of its own so the preparation stays inside the line bound
//!  (`AGENTS.md`,
//! MXL).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = excludeOrStop(context, capture, indexes, mode, captured);
//! ```

/// The step itself,
/// which the landing's owed-index update owns.
use crate::owed_index_update::exclude_or_stop;
/// The captured invocation and the private indexes it built.
use crate::capture_invocation::InvocationCapture;
use crate::service_land_start::IndexWitness;
use crate::private_index_request::CaptureIndexes;
use crate::commit_transaction::TransactionStop;
use crate::transaction_git::GitContext;
use crate::transaction_journal::TransactionMode;

/// What:
///  The staging step of the preparation at `capture`,
///  for a control to ask on its own.
/// Why:
///   A control that checks which paths a conflicted commit excludes
///  needs that step without a whole commit through the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeOrStop(context, capture, indexes, mode, captured)
/// ```
pub(crate) fn exclude_or_stop_for_test(
    context: &GitContext,
    capture: &InvocationCapture,
    indexes: &CaptureIndexes,
    mode: TransactionMode,
    captured: &IndexWitness,
) -> Result<(Vec<String>, Option<String>), TransactionStop> {
    return exclude_or_stop(context, capture, indexes, mode, captured);
}
