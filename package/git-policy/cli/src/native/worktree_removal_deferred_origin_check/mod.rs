//! What:
//!  The deferred-origin checks of a ZFS worktree clone's removal:
//!  one verdict function a removal calls before the first snapshot request and again before the
//!  head destroy,
//!  the look the lineage proof makes before anything is taken down,
//!  and the words a refusal at either point prints.
//! Why:
//!  An origin snapshot somebody marked with `zfs destroy -d` is destroyed by ZFS itself once its
//!  last clone is gone,
//!  so destroying the last clone when that mark stands destroys history another program asked to
//!  keep
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Reading the mark").
//!  The project placed the check before the first snapshot delete as well as before the head
//!  destroy,
//!  and added a look before the unmount
//!  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`).
//!  This folder is plan step 15's piece, built ahead of the plan's order and called by no
//!  person's command yet:
//!  its own step wires it in, and its tests and its guest arm are its only callers
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "Building later pieces ahead of the plan's order").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `origin_mark.rs`:
//!   the mark's property,
//!   its three states and the one read of it the origin-cleanup design assumes.
//! - `origin_refusals.rs`:
//!   the fields a plan must carry,
//!   the object each cause names,
//!   the three message forms and what a refusal leaves.
//! - `origin_refusal.rs`:
//!   each cause as a typed value,
//!   with the fields its message names.
//! - `origin_refusal_causes.rs`:
//!   the part of a message that names the cause,
//!   its object,
//!   its field and the mark's two counts.
//! - `planned_origin.rs`:
//!   the identities a removal planned, and the refusal of a plan that lacks one.
//! - `origin_verdict.rs`:
//!   the check at both call sites, the look before the unmount, and the three reads.
//! - `removal_history.rs`:
//!   the loss a plan already records and the one sentence a refusal prints from it.
//! - `origin_refusal_words.rs`:
//!   the three forms of message and the ways forward.
//! - `origin_refusal_answers.rs`:
//!   each cause's step word,
//!   form,
//!   record rule,
//!   the clone and slot it names,
//!   and its message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = checkPlannedOrigin(reader, planned);
//! if (verdict.kind === 'refused') throw new Error(verdict.cause.message(history));
//! ```

/// The mark's property, its states and the one read of it.
pub mod origin_mark;

/// The loss a plan records and the sentence a refusal prints from it.
pub mod removal_history;

/// The fields a plan must carry, the objects and forms a message names, and what a refusal leaves.
pub mod origin_refusals;

/// The identities a removal planned, and the refusal of a plan that lacks one.
pub mod planned_origin;

/// Each cause as a typed value, with the fields its message names.
pub mod origin_refusal;

/// Each cause's step word, form, record rule and message.
pub mod origin_refusal_answers;

/// The cause sentence a message prints, with the mark's two counts.
pub mod origin_refusal_causes;

/// The three forms of message and the ways forward.
pub mod origin_refusal_words;

/// The check, the look and the three reads.
pub mod origin_verdict;

/// The mark as a reply carries it, and the one read a caller outside this folder makes.
pub use origin_mark::{
    DEFER_DESTROY_PROPERTY,
    MARK_OFF,
    MARK_ON,
    OriginMark,
    USER_HOLDS_PROPERTY,
    UserHolds,
    mark_of_reply,
    origin_mark,
};
/// One cause, with the word, the form and the record rule it answers.
pub use origin_refusal::OriginRefusal;
/// The fields, objects, forms and record rule a refusal's message rests on.
pub use origin_refusals::{
    JudgedObject,
    PlanField,
    RefusalForm,
    RefusalPoint,
    WhatARefusalLeaves,
};
/// The check, the look and the verdict they answer.
pub use origin_verdict::{
    Verdict,
    check_planned_origin,
    judge_the_look,
};
/// The identities a removal planned and the fields a plan must carry.
pub use planned_origin::{
    MissingField,
    PlannedOrigin,
    PlannedOriginFields,
};
/// The loss a plan records.
pub use removal_history::RemovalHistory;

/// The scripted pool every control of this folder drives. Never part of the release executable.
#[cfg(test)]
#[path = "origin_fixtures_tests.rs"]
pub(crate) mod fixtures;
