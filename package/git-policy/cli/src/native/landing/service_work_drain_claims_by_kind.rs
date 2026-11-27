//! What:
//!  The claimed items of one batch of the drain,
//! sorted into one list per kind of work.
//! Why:
//!  The drain runs one kind after another and each kind asks for its own list
//! (`service_work_drain_pending_claims.rs`),
//! so the sorting is one step of its own whose eight lists are one value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const byKind = claimsByKind(acceptable); // { pushes, trashes, prewarms, relinks, deletions, reviews, ahead, upkeep }
//! ```

use crate::work_item::{
    Claimed,
    WorkKind,
};

/// The claimed items of one batch,
/// one list per kind of work,
/// each in the order the items were claimed.
pub(super) struct ClaimsByKind {
    pub(super) pushes: Vec<Claimed>,
    pub(super) trashes: Vec<Claimed>,
    pub(super) prewarms: Vec<Claimed>,
    pub(super) relinks: Vec<Claimed>,
    pub(super) deletions: Vec<Claimed>,
    pub(super) reviews: Vec<Claimed>,
    pub(super) ahead: Vec<Claimed>,
    pub(super) upkeep: Vec<Claimed>,
}

/// What:
///  Sorts the items a round finds acceptable into the list of their kind.
/// Why:
///   Each kind's step of the round takes its own list,
/// and the order they were claimed in is what one worktree's items keep their turn by.
/// A kind this drain has no step for is upkeep:
///  it keeps the repository cheap to read and is started last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimsByKind = (acceptable) => { const by = { pushes: [], ... }; for (const item of acceptable) by[kindOf(item)].push(item); return by }
/// ```
pub(super) fn claims_by_kind(acceptable: Vec<Claimed>) -> ClaimsByKind {
    let mut by_kind: ClaimsByKind = ClaimsByKind {
        pushes: Vec::new(),
        trashes: Vec::new(),
        prewarms: Vec::new(),
        relinks: Vec::new(),
        deletions: Vec::new(),
        reviews: Vec::new(),
        ahead: Vec::new(),
        upkeep: Vec::new(),
    };
    for item in acceptable {
        match item.item.kind {
            WorkKind::Push => by_kind.pushes.push(item),
            WorkKind::Trash => by_kind.trashes.push(item),
            WorkKind::Prewarm => by_kind.prewarms.push(item),
            WorkKind::Relink => by_kind.relinks.push(item),
            WorkKind::SubvolumeDelete => by_kind.deletions.push(item),
            WorkKind::Review => by_kind.reviews.push(item),
            WorkKind::SnapshotAhead => by_kind.ahead.push(item),
            _ => by_kind.upkeep.push(item),
        }
    }
    return by_kind;
}
