//! What:
//!  The kinds of work the commit service runs,
//!  and what a wake message brings:
//!  the kinds it runs itself,
//!  the kinds one round claims,
//!  the kinds the slow lane claims,
//!  how many times an item is tried,
//!  and the repository and the review credential a wake names.
//! Why:
//!  The round (`service_work_run_round.rs`) and the slow lane
//!  (`service_work_slow_lane.rs`) each claim their own kinds,
//!  and a wake hands the service one repository at a time;
//!  a file of its own lets a reader meet the vocabulary before the loops
//!  that speak it,
//!  while every drain and every step reads these names from here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const SERVICE_KINDS = ['push', 'maintenance', 'review', 'trash', 'prewarm', 'relink', 'subvolume-delete', 'snapshot-ahead'];
//! const PUSH_KINDS = ['push'];
//! const MAX_TRIES = 3;
//! type Wake = [repository: string, reviewKey: ReviewKey | null];
//! ```

use crate::service_review_key::ReviewKey;
use crate::work_item::WorkKind;
use std::path::PathBuf;

/// The kinds of work the service runs itself:
/// the push and every slow kind.
pub const SERVICE_KINDS: &[WorkKind] = &[
    WorkKind::Push,
    WorkKind::Maintenance,
    WorkKind::Review,
    WorkKind::Trash,
    WorkKind::Prewarm,
    WorkKind::Relink,
    WorkKind::SubvolumeDelete,
    WorkKind::SnapshotAhead,
];

/// The kinds the round that answers a wake claims:
/// the push,
/// which is the person's backup.
/// Why:
///   A round holds the service until it ends,
/// so nothing else may be claimed beside a push:
/// on `2026-11-10` six landings of `refs/heads/main` waited 18 minutes for one round that was
/// emptying trashes of removed worktrees,
/// and every round that held a push that day was measured the same way
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// The other kinds are the slow lane's (`landing/service_work_slow_lane.rs`).
pub const PUSH_KINDS: &[WorkKind] = &[WorkKind::Push];

/// The kinds whose steps can take minutes:
/// the upkeep,
/// the prewarm of a new worktree's build directories,
/// the relink of a new worktree's copied package state,
/// the delete of a
/// removed worktree's subvolume,
/// the emptying of a trash,
/// the snapshot made ahead of an add,
/// and the shadow review.
/// Why:
///   No command waits for these steps to end:
/// a person's add that wrote a snapshot ahead looks for one and makes its own when none is ready in time
/// (`worktree/worktree_snapshot_ahead.rs`),
/// and the whole of `2026-11-10` shows that a step of these kinds that ran for minutes held every push of
/// `refs/heads/main` of the round it ran in
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// They are claimed by the slow lane (`landing/service_work_slow_lane.rs`),
/// one at a time,
/// and never by the round.
pub const SLOW_KINDS: &[WorkKind] = &[
    WorkKind::Maintenance,
    WorkKind::Prewarm,
    WorkKind::Relink,
    WorkKind::SubvolumeDelete,
    WorkKind::Trash,
    WorkKind::SnapshotAhead,
    WorkKind::Review,
];

/// How many times an item is tried before it is given up on.
pub const MAX_TRIES: u32 = 3;

/// What a wake message brings:
/// the repository to work on and the review credential that came with it,
/// if any.
pub type Wake = (PathBuf, Option<ReviewKey>);
