//! What:
//!  The durable work-item record:
//! a small file in the Git directory that says "this landed object needs this
//!  asynchronous work",
//! written by the wrapper and drained by a separate resident consumer.
//! Why:
//!  A wrapper process exits when its command ends,
//! and it may start no program,
//! so work that must not delay a landing
//!  (the standards review,
//! later auto-push and maintenance) is left as a record instead.
//! The record is generic:
//! a kind,
//!  the landed object and an id derived from both,
//! so a retry cannot duplicate it.
//! The design,
//! its failure cases and
//!  the consumer contract are in `doc/planning/cli-git-standards-review-port.md`.
//! Nothing calls this yet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, { kind: 'review', oid, ... }); const item = await claimNext(workDir); await finish(workDir, item);
//! ```

pub use crate::work_item_claim_taking::{
    claim_all_of,
    claim_next,
    claim_next_of,
    find_claim,
};
/// Controls reach the claim step with a stamp they can refuse.
#[cfg(test)]
pub(crate) use crate::work_item_claim_taking::{
    claim_next_stamped,
    stamp_modification_time,
};
pub use crate::work_item_enqueue::{
    Enqueued,
    enqueue,
    enqueue_all,
};
pub use crate::work_item_holding::{
    block,
    forget_held,
    hold,
    promote,
    record_pass,
    record_refusal,
    record_verdict,
    requeue_after_giving_up,
};
pub use crate::work_item_kinds_and_their_names::WorkKind;
pub use crate::work_item_queued_ids::queued_ids;
pub use crate::work_item_record::{
    Claimed,
    VERSION,
    WORK_DIRECTORY,
    WorkItem,
};
pub use crate::work_item_settling::{
    oldest_pending_age,
    renew_claim,
    settle,
    settle_named,
};
pub use crate::work_item_state_directories_and_claim_names::{
    CLAIMED,
    DONE,
    FAILED,
    HELD,
    PENDING,
    RECOVERING,
    claim_file_name,
    id_of_entry,
    state_directory,
};
pub(crate) use crate::work_item_state_directories_and_claim_names::{
    entries_oldest_first,
    new_generation,
    parse_claim_name,
};

/// Controls reach the settlement step with a stamp they can refuse.
#[cfg(test)]
pub(crate) use crate::work_item_settling::renew_claim_stamped;

#[cfg(test)]
pub(crate) use crate::work_item_ids_and_names::MAX_OBJECT_NAME_LENGTH;
/// The shape of an id,
/// and the length an object name may have,
/// beside the digest they work with.
pub(crate) use crate::work_item_ids_and_names::is_valid_id;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_item_tests.rs"]
mod tests;
