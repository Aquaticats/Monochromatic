//! What:
//!  The removal-side half of the ZFS worktree plan that needs no command to reach it yet:
//!  the lineage proof a removal makes before its first delete,
//!  the inventory and bookmark rule that proof applies,
//!  the persisted destruction plan it writes,
//!  the removal's `unmount` intent and the verified answer step 9's code carries,
//!  and the list of names it prints as lost.
//! Why:
//!  Plan step 14 builds steps 1 and 2 of "Removal and recovery":
//! a removal proves the live pool,
//!  clone and origin identities,
//!  proves its lineage in a fixed order,
//!  refuses before its first delete when anything is unproved,
//!  and only then records the removal intent and the destruction plan
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! "Step 14";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Removal and recovery").
//!  The record's shapes are the accepted ones:
//!  one `key=value` format for the three removal-side records and one line per attempt
//!  (`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`),
//!  the plan in the account-owned Git common directory
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`),
//!  and the decided bookmark rule `A: Live or planned`
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Bookmarks on a clone being removed").
//!  This folder is built ahead of the plan's order and is wired by no command:
//!  step 14's own wiring calls it,
//!  and no `git` command,
//! `doctor`,
//! mount handler or boot unit reaches it
//!  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`,
//!  "Building later pieces ahead of the plan's order").
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `canonical_decimal.rs`:
//!   the one spelling a plan's numbers take.
//! - `loss_names.rs`:
//!   the list of names a removal prints as lost.
//! - `plan_refusal.rs`:
//!   why bytes are not a plan,
//!   and the sentence a person reads.
//! - `plan_lines.rs`:
//!   the one line reader both halves of the parse share.
//! - `plan_words.rs`:
//!   the header line,
//!   the eighteen keys in order,
//!   the five snapshot outcomes and the five head words.
//! - `plan_bounds.rs`:
//!   the header and line bounds and the per-field widths they rest on.
//! - `plan_name.rs`:
//!   the file's GUID name,
//!   a dataset's name and a snapshot's.
//! - `removal_stop.rs`:
//!   every way the removal stops.
//! - `removal_stop_sentences.rs`:
//!   the sentence each way of stopping prints.
//! - `fresh_holder_scan.rs`:
//!   the fresh all-namespace scan each destroy is preceded by.
//! - `removal_identities.rs`:
//!   the identities the add recorded,
//!   the origin-side inventory it recorded,
//!   and the registration's own identity.
//! - `removal_settle.rs`:
//!   the Git settle of one removal.
//! - `removal_slot_claim.rs`:
//!   the per-slot claim the whole order takes first and releases last,
//!   and what a release did.
//! - `started_moment.rs`:
//!   the `started` field's shape and value.
//! - `plan_record.rs`:
//!   the plan's fields and its first write.
//! - `plan_record_bytes.rs`:
//!   the bytes a plan publishes.
//! - `plan_attempt_parse.rs`:
//!   one attempt region read into one line per attempt.
//! - `plan_header_fields.rs`:
//!   the header region read into one plan's fields.
//! - `plan_parse.rs`:
//!   the header completed into the plan's facts and the two halves joined.
//! - `plan_reading.rs`:
//!   the two-stage file read and the folder and owner checks.
//! - `plan_publication.rs`:
//!   the folder's creation and the one write a removal makes.
//! - `removal_inventory.rs`:
//!   the clone's live snapshots and bookmarks by kind,
//!   name,
//!   GUID,
//!   owning dataset and `createtxg`.
//! - `bookmark_rule.rs`:
//!   the decided rule `A: Live or planned`.
//! - `removal_snapshot_settlement.rs`:
//!   the names settled by their GUIDs and the amended plan a new own snapshot takes.
//! - `removal_head_destroy.rs`:
//!   the head destroy with its absence proof.
//! - `removal_reads.rs`:
//!   the pool reads the proof makes,
//!   as a seam with one live implementation over the binding.
//! - `lineage_proof.rs`:
//!   the three checks,
//!   the origin-side comparison and the refusal each makes.
//! - `removal_records_end.rs`:
//!   the removal's own records,
//!   removed as the last witness.
//! - `removal_report.rs`:
//!   the lines a finished removal prints.
//! - `unmount_request.rs`:
//!   the removal intent and the verified answer.
//! - `removal_clock.rs`:
//!   the three process facts a removal's plan records.
//! - `removal_entry.rs`:
//!   the one order a removal's proof,
//!   intent,
//!   unmount and plan take,
//!   and every step of it that changes nothing.
//! - `removal_completion_inputs.rs`:
//!   everything one whole removal drives,
//!   as named seams a control answers without a pool.
//! - `removal_origin_step.rs`:
//!   the answers removal origin step holds.
//! - `removal_settle_step.rs`:
//!   the Git settle of one removal,
//!   called only with the proof that this slot's clone is gone.
//! - `removal_destruction.rs`:
//!   the deferred-origin check before every snapshot request,
//!   the batch those requests destroy,
//!   the fresh enumeration and the check before the head destroy.
//! - `removal_reading.rs`:
//!   the removal's read-only reading:
//!   what a removal would prove and lose,
//!   and the lines a command prints instead of writing.
//! - `removal_resume.rs`:
//!   where a removal starts:
//!   a fresh entry,
//!   an adopted kept plan,
//!   or the head-gone path.
//! - `plan_names_and_states.rs`:
//!   the plan's names and the state each one's history ends on.
//! - `removal_completion_head_present.rs`:
//!   the steps one removal takes while its clone still stands.
//! - `removal_completion_head_gone.rs`:
//!   the steps a removal takes when its plan's head word is already `gone`.
//! - `removal_completion.rs`:
//!   the whole order from the plan to the report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proven = proveTheLineage(reads, recorded, requested);
//! const plan = proven.firstWrite(facts);
//! return enterTheRemoval(folder, plan);
//! ```

pub mod bookmark_rule;
pub mod canonical_decimal;
pub mod fresh_holder_scan;
pub mod lineage_proof;
pub mod loss_names;
pub mod plan_attempt_parse;
pub mod plan_bounds;
pub mod plan_header_fields;
pub mod plan_lines;
pub mod plan_name;
pub mod plan_names_and_states;
pub mod plan_parse;
pub mod plan_publication;
pub mod plan_reading;
pub mod plan_record;
pub mod plan_record_bytes;
pub mod plan_refusal;
pub mod plan_words;
pub mod removal_clock;
pub mod removal_completion;
pub mod removal_completion_head_gone;
pub mod removal_completion_head_present;
pub mod removal_completion_inputs;
pub mod removal_destruction;
pub mod removal_entry;
pub mod removal_head_destroy;
pub mod removal_identities;
pub mod removal_inventory;
pub mod removal_origin_step;
pub mod removal_reading;
pub mod removal_reads;
pub mod removal_records_end;
pub mod removal_report;
pub mod removal_resume;
pub mod removal_settle;
pub mod removal_settle_step;
pub mod removal_slot_claim;
pub mod removal_snapshot_settlement;
pub mod removal_stop;
pub mod removal_stop_sentences;
pub mod started_moment;
pub mod unmount_request;

pub use bookmark_rule::{
    Explanations,
    explanations_of,
    refuse_the_unexplained_bookmarks,
};
pub use canonical_decimal::{
    decimal_digits,
    is_canonical_decimal,
    parse_canonical_u64,
};
pub use fresh_holder_scan::{
    FreshHolderScan,
    TheLiveHolderScan,
    the_holder_lines,
};
pub use lineage_proof::{
    ObjectIdentity,
    ProvenIdentities,
    ProvenRemoval,
    prove_the_lineage,
};
pub use loss_names::NamesToBeLost;
pub use plan_bounds::{
    ATTEMPT_COUNT_MAXIMUM_DIGITS,
    ATTEMPT_LINE_MAXIMUM_BYTES,
    ERRNO_MAXIMUM_DIGITS,
    NUMBER_MAXIMUM_DIGITS,
    PLAN_HEADER_MAXIMUM_BYTES,
    SLOT_PATH_MAXIMUM_BYTES,
    ZFS_NAME_MAXIMUM_BYTES,
    header_field_maximum_bytes,
    line_bytes,
    plan_maximum_bytes,
    widest_attempt_line_bytes,
    widest_header_bytes,
};
pub use plan_name::{
    is_a_snapshot_of,
    is_dataset_name,
    is_snapshot_name,
};
pub use plan_parse::{
    PlanHeader,
    parse_plan,
    parse_plan_header,
};
pub use plan_publication::PlanFolder;
pub use plan_reading::{
    HEADER_LINE_COUNT,
    check_the_folder,
    check_the_record_owner,
    is_a_plan_entry,
    read_an_existing_plan,
    read_the_plan,
    read_the_plan_at,
    split_the_header,
};
pub use plan_record::{
    DestructionPlan,
    PlanAttempt,
    PlanFacts,
    PlanSnapshot,
    owner_text,
    target_hexadecimal,
};
pub use plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
pub use plan_words::{
    ATTEMPT_KEY,
    ATTEMPT_NAME_KEY,
    HeadWord,
    NO_ERRNO,
    NO_MOUNT_ID,
    PLAN_HEADER,
    PLAN_KEYS_IN_ORDER,
    SNAPSHOT_KIND,
    SnapshotOutcome,
};
pub use removal_clock::{
    RemovalClock,
    TheLiveClock,
};
pub use removal_completion::complete_the_removal;
pub use removal_completion_inputs::RemovalCompletion;
pub use removal_destruction::{
    DestructionUnderway,
    RemovalDestruction,
    destroy_the_removal,
};
pub use removal_entry::{
    EnteredRemoval,
    RemovalEntry,
    enter_the_removal,
};
pub use removal_head_destroy::{
    CloneAbsence,
    DatasetDestroyDevice,
    HeadDestroyed,
    the_clone_absence,
};
pub use removal_identities::{
    RecordedBinding,
    prove_the_adds_registration,
    read_the_origin_inventory,
    read_the_recorded_binding,
};
pub use removal_inventory::{
    LiveInventory,
    LiveObject,
};
pub use removal_reading::{
    RemovalReading,
    read_the_removal,
};
pub use removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
    whole_inventory,
};
pub use removal_records_end::{
    RecordsGone,
    remove_the_removal_records,
};
pub use removal_report::{
    OriginCleanupDone,
    TheRemovalDone,
};
pub use removal_resume::{
    TheStart,
    start_the_removal,
};
pub use removal_settle::{
    RegistrationSettle,
    SettleAnswer,
    SettleRequest,
    TheLiveSettle,
};
pub use removal_slot_claim::{
    ClaimEnd,
    RemovalClaim,
    release_at_the_last_step,
    take_the_removals_claim,
};
pub use removal_stop::{
    BookmarkRefusalReason,
    RefusedBookmark,
    RemovalStop,
};
pub use started_moment::{
    is_seconds_with_offset,
    now_with_offset,
};
pub use unmount_request::{
    ExpectedUnmount,
    LiveRemovalUnmount,
    RemovalIntent,
    RemovalUnmount,
    UnmountAnswer,
    UnmountProof,
    verify_the_answer,
};

/// The plan every control of this folder starts from.
/// Never part of the release executable.
#[cfg(test)]
#[path = "plan_fixtures_tests.rs"]
pub(crate) mod fixtures;

/// The repository,
/// binding,
/// origin inventory and scripted pool the removal's controls drive.
/// Never part of the release executable.
#[cfg(test)]
#[path = "removal_fixtures_tests.rs"]
pub(crate) mod removal_fixtures;

/// The guest arm of this folder's piece,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "removal_guest_arm_tests.rs"]
mod removal_guest_arm_tests;

/// The live pool and repository the guest arms build,
/// shared by both of them.
/// Never part of the release executable.
#[cfg(test)]
#[path = "removal_live_fixtures_tests.rs"]
pub(crate) mod removal_live_fixtures;

/// The guest arm of the destruction step,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "removal_destruction_guest_arm_tests.rs"]
mod removal_destruction_guest_arm_tests;

/// The guest arm of the whole removal,
/// compiled for tests only and run in the throwaway guest.
#[cfg(test)]
#[path = "removal_completion_guest_arm_tests.rs"]
mod removal_completion_guest_arm_tests;

/// The seams a whole-removal control answers,
/// and the one call that runs the completion over the fixture.
/// Never part of the release executable.
#[cfg(test)]
#[path = "removal_completion_fixtures_tests.rs"]
pub(crate) mod removal_completion_fixtures;
