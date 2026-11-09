//! What:
//!  What became of one request of a batch,
//!  and the announcement a landing makes when none of its clients is told anything.
//! Why:
//!  A batch answers with one outcome per request,
//! and a caller that has no client to tell states that in a value instead of a null announcer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Outcome = { kind: 'landed', commit: string } | { kind: 'retry-prepare', base: string } | { kind: 'branch-moved' };
//! const ignoreAnnouncements = { announce: (_note: Announce) => {} };
//! ```

use crate::service_index_lock::{Announce, Announcer};

/// What became of one request of a batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The commit is on the branch.
    Landed { commit: String },
    /// A named path changed on the branch since the request's base.
    RetryPrepare { base: String },
    /// An amend,
    /// whose parents are the base's parents,
    /// found the branch somewhere else than at its base.
    BranchMoved,
}

/// What:
///  An announcement that nobody is told.
/// Why:
///  `land_batch` is `land_batch_announcing` for a caller that has no clients to tell what the landing waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignoreAnnouncements = { announce: (_note: Announce) => {} };
/// ```
pub(crate) struct IgnoreAnnouncements;

impl Announcer for IgnoreAnnouncements {
    fn announce(&self, _note: Announce) {}
}

/// What:
///  Whether any outcome of a batch is a landed commit.
/// Why:
///  A batch that landed something hands its syncs to the background;
/// one that landed nothing syncs them before it returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcomes.some((outcome) => outcome.kind === 'landed')
/// ```
pub(crate) fn any_landed(outcomes: &[Outcome]) -> bool {
    for outcome in outcomes {
        if matches!(outcome, Outcome::Landed { .. }) {
            return true;
        }
    }
    return false;
}
