//! What:
//!  Why a source cannot be copied,
//! and which kind of check said so.
//! Why:
//!   The reason is for the log and for the sentence the person reads;
//! the kind lets the sentence be chosen without reading the reason's words,
//! so the ladder can tell the person the cause in fitting words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { reason: string; kind: 'unclean' | 'unsupported' | 'status-failed' };
//! ```

use crate::worktree_clean::{
    CleanRefusal,
    CleanRefusalKind,
};

/// Which kind of check refused a source,
/// so the person can be told the cause in words that fit it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalKind {
    /// The source could not be proven clean,
    /// for the kind of reason the clean check gives.
    Unclean(CleanRefusalKind),
    /// A feature a copy of the files does not reproduce,
    /// or a state the checks cannot read.
    Unsupported,
    /// The stat data could not vouch for the files,
    /// and the Git run that settles it by content failed.
    StatusFailed,
}

/// What:
///  Why a source cannot be copied,
/// and which kind of check said so.
/// Why:
///   The reason is for the log and for the sentence the person reads;
/// the kind lets the sentence be chosen without reading
///  the reason's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { reason: string; kind: 'unclean' | 'unsupported' | 'status-failed' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// The reason,
    /// in words.
    pub reason: String,
    /// The kind of check that refused.
    pub kind: RefusalKind,
}

impl Refusal {
    /// A refusal by a feature or a state the checks cannot read.
    pub(crate) fn unsupported(reason: impl Into<String>) -> Refusal {
        return Refusal {
            reason: reason.into(),
            kind: RefusalKind::Unsupported,
        };
    }
}

/// What:
///  A refusal for a source that is not clean,
/// carrying the reason and kind of the clean check's refusal.
/// Why:
///   The ladder reports a refusal of its own kind,
/// and an unclean source is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Refusal(why.reason, { unclean: why.kind })
/// ```
pub(crate) fn unclean(why: CleanRefusal) -> Refusal {
    return Refusal {
        reason: why.reason,
        kind: RefusalKind::Unclean(why.kind),
    };
}

/// What:
///  A refusal for a source whose changed paths Git could not list.
/// Why:
///   A source with changes is snapshotted only when its changes can be listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Refusal(reason, 'statusFailed')
/// ```
pub(crate) fn status_failed(reason: String) -> Refusal {
    return Refusal {
        reason,
        kind: RefusalKind::StatusFailed,
    };
}
