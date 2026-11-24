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
//! each using only the files listed before it:
//!
//! - `plan_words.rs`:
//!   the header line,
//!   the eighteen keys in order,
//!   the five snapshot outcomes and the five head words.
//! - `canonical_decimal.rs`:
//!   the one spelling a plan's numbers take.
//! - `plan_bounds.rs`:
//!   the header and line bounds and the per-field widths they rest on.
//! - `plan_refusal.rs`:
//!   why bytes are not a plan,
//!   and the sentence a person reads.
//! - `plan_name.rs`:
//!   the file's GUID name,
//!   a dataset's name and a snapshot's.
//! - `started_moment.rs`:
//!   the `started` field's shape and value.
//! - `plan_record.rs`:
//!   the plan's fields,
//!   its first write and its bytes.
//! - `plan_lines.rs`:
//!   the one line reader both halves of the parse share.
//! - `plan_attempt_parse.rs`:
//!   one attempt region read into one line per attempt.
//! - `plan_parse.rs`:
//!   the header read into the plan's fields and the two halves joined.
//! - `plan_reading.rs`:
//!   the two-stage file read and the folder and owner checks.
//! - `plan_publication.rs`:
//!   the folder's creation and the one write a removal makes.
//! - `removal_identities.rs`:
//!   the identities the add recorded,
//!   the origin-side inventory it recorded,
//!   and the registration's own identity.
//! - `removal_inventory.rs`:
//!   the clone's live snapshots and bookmarks by kind,
//!   name,
//!   GUID,
//!   owning dataset and `createtxg`.
//! - `removal_reads.rs`:
//!   the pool reads the proof makes,
//!   as a seam with one live implementation over the binding.
//! - `removal_stop.rs`:
//!   every way the removal stops,
//!   and the sentence each prints.
//! - `lineage_proof.rs`:
//!   the three checks,
//!   the origin-side comparison and the refusal each makes.
//! - `bookmark_rule.rs`:
//!   the decided rule `A: Live or planned`.
//! - `unmount_request.rs`:
//!   the removal intent and the verified answer.
//! - `loss_names.rs`:
//!   the list of names a removal prints as lost.
//! - `removal_entry.rs`:
//!   the one order a removal's proof,
//!   intent,
//!   unmount and plan take.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proven = proveTheLineage(reads, recorded, requested);
//! const plan = proven.firstWrite(facts);
//! return enterTheRemoval(folder, plan);
//! ```

pub mod plan_words;
pub mod canonical_decimal;
pub mod plan_bounds;
pub mod plan_refusal;
pub mod plan_name;
pub mod started_moment;
pub mod plan_record;
pub mod plan_lines;
pub mod plan_attempt_parse;
pub mod plan_parse;
pub mod plan_reading;
pub mod plan_publication;
pub mod removal_inventory;
pub mod removal_reads;
pub mod removal_stop;
pub mod removal_identities;
pub mod lineage_proof;
pub mod bookmark_rule;
pub mod loss_names;
pub mod unmount_request;
pub mod removal_entry;

pub use canonical_decimal::{decimal_digits, is_canonical_decimal, parse_canonical_u64};
pub use plan_bounds::{
    ATTEMPT_COUNT_MAXIMUM_DIGITS, ATTEMPT_LINE_MAXIMUM_BYTES, ERRNO_MAXIMUM_DIGITS, NUMBER_MAXIMUM_DIGITS, PLAN_HEADER_MAXIMUM_BYTES, SLOT_PATH_MAXIMUM_BYTES, ZFS_NAME_MAXIMUM_BYTES, header_field_maximum_bytes, line_bytes,
    plan_maximum_bytes, widest_attempt_line_bytes, widest_header_bytes,
};
pub use plan_name::{is_a_snapshot_of, is_canonical_plan_name, is_dataset_name, is_snapshot_name};
pub use plan_parse::{PlanHeader, parse_plan, parse_plan_header};
pub use plan_publication::PlanFolder;
pub use plan_reading::{HEADER_LINE_COUNT, check_the_folder, check_the_record_owner, is_a_plan_entry, read_an_existing_plan, read_the_plan, read_the_plan_at, split_the_header};
pub use plan_record::{DestructionPlan, PlanAttempt, PlanFacts, PlanSnapshot, owner_text, target_hexadecimal};
pub use plan_refusal::{PlanRefusal, PlanRefusalCause};
pub use plan_words::{ATTEMPT_KEY, ATTEMPT_NAME_KEY, HeadWord, NO_ERRNO, NO_MOUNT_ID, PLAN_HEADER, PLAN_KEYS_IN_ORDER, SNAPSHOT_KIND, SnapshotOutcome};
pub use started_moment::{is_seconds_with_offset, now_with_offset};
pub use removal_inventory::{LiveInventory, LiveObject};
pub use removal_reads::{LiveRemovalReads, RemovalPoolReads, whole_inventory};
pub use removal_stop::{BookmarkRefusalReason, RefusedBookmark, RemovalStop};
pub use removal_identities::{RecordedBinding, prove_the_adds_registration, read_the_origin_inventory, read_the_recorded_binding};
pub use lineage_proof::{ObjectIdentity, ProvenIdentities, ProvenRemoval, prove_the_lineage};
pub use bookmark_rule::{Explanations, explanations_of, refuse_the_unexplained_bookmarks};
pub use loss_names::NamesToBeLost;
pub use unmount_request::{ExpectedUnmount, LiveRemovalUnmount, RemovalIntent, RemovalUnmount, UnmountAnswer, UnmountProof, verify_the_answer};
pub use removal_entry::{EnteredRemoval, RemovalClock, RemovalEntry, TheLiveClock, enter_the_removal};

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
