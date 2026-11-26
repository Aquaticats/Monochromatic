//! What:
//!  The one name the drains,
//!  the rounds and the other folders' steps use for
//!  what the work of the commit service is made of:
//! the kinds it runs,
//! the handlers that run each kind,
//! which records are acceptable work,
//! how claims are settled,
//! what a drain returned and how a push that failed is planned again.
//!  Each of those roles is a sibling file of its own,
//!  and this file re-exports them,
//!  so a caller keeps one path for all of them.
//! Why:
//!  The drain of pending claims (`service_work_drain_pending_claims.rs`),
//! the drains of directories (`service_work.rs`) and the
//!  rounds (`service_work_run_round.rs`) all speak in these,
//!  and the repair jobs name a verdict from here;
//!  the pieces sit in files that use none of those loops,
//!  and each of those files holds one role,
//!  so a reader takes the vocabulary,
//!  the handlers,
//!  the acceptance rules,
//!  the given-up words,
//!  the drain's answer,
//!  the retry's words
//!  and the steps for a repository that is gone or unreachable
//!  one file at a time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { Verdict, WorkHandlers, isAcceptable, plannedRetry } from './service_work';
//! ```

/// The work directory of a repository,
/// which items are acceptable work,
/// and what becomes of the claims this instance holds.
pub use super::service_work_acceptable_records_and_claims::{
    is_acceptable,
    work_directory,
};
/// Settling every claim to a state,
/// and keeping only the claims a renewal still holds.
pub(crate) use super::service_work_acceptable_records_and_claims::{
    settle_all,
    still_held,
};
/// What one drain of a directory hands back,
/// and the retry state it works with.
pub use super::service_work_drained_and_replanning::Drained;
/// The retry state one drain works with.
pub(crate) use super::service_work_drained_and_replanning::Replanning;
/// The notice the recovery hands each record it gave up on.
pub(crate) use super::service_work_given_up_notices::SkipNotice;
/// The words a person reads when the service gave up on a push or a relink,
/// which the controls of this folder and of `repair/` ask for by this name.
#[cfg(test)]
pub(crate) use super::service_work_given_up_notices::tell_about_given_up;
/// What a handler came to and what runs each kind of work.
pub use super::service_work_handlers_and_verdicts::{
    Verdict,
    WorkHandlers,
};
/// The kinds of work the service runs,
/// the kinds one round claims,
/// the kinds the slow lane claims,
/// how many times an item is tried,
/// and the repository and review credential a wake names.
pub use super::service_work_kinds_and_a_wake::{
    MAX_TRIES,
    PUSH_KINDS,
    SERVICE_KINDS,
    SLOW_KINDS,
    Wake,
};
/// The words and the plan of a push that is tried again.
pub(crate) use super::service_work_planned_retry::{
    PlannedRetry,
    planned_retry,
};
/// The plans of a repository the file system proves gone,
/// and those of a repository that cannot be reached.
pub(crate) use super::service_work_unreachable_repository_plans::{
    end_the_plans_of_a_gone_repository_awaiting,
    keep_plans_while_unreachable,
};
