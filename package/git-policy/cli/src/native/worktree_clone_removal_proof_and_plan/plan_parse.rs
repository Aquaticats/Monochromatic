//! What:
//!  The parse a plan's two-stage read calls:
//!  the header region into the plan's fields,
//!  and the attempt region into one line per attempt,
//!  with every ordering rule the grammar holds.
//! Why:
//!  Nothing is read as a partial record:
//!  a key outside the fixed order,
//!  a repeated key,
//!  a missing key,
//!  a count that disagrees with the lines,
//!  an attempt number used twice,
//!  and a line after a `destroyed` one each refuse the whole record
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The grammar" and "Every transition,
//! and the rule that allows it").
//!  A gap in one name's numbers is not an error:
//!  it is the trace of a write that was lost before it landed,
//!  and the highest number still stands as that name's current state.
//!  The regions arrive separately because the reader has read the header before it may ask the
//!  file's length,
//!  which is what bounds its memory with no count ceiling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parsePlan(headerBytes, attemptBytes, fileName);
//! ```

use crate::worktree_clone_binding_record::plan_folder_read::is_canonical_guid_name;
use crate::worktree_clone_removal_proof_and_plan::plan_attempt_parse::parse_attempts;
use crate::worktree_clone_removal_proof_and_plan::plan_header_fields::parse_header;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
    PlanFacts,
};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;

/// What:
///  The header of one plan,
///  parsed before any attempt byte is read:
///  its identities,
///  its head word and its declared attempt count.
/// Why:
///  The reader's first stage needs the count to know how large a file its own grammar allows,
///  and it must refuse a malformed field before it asks the file's length rather than after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanHeader = { facts: PlanFacts; head: HeadWord; count: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanHeader {
    /// The identities the removal recorded.
    pub facts: PlanFacts,
    /// The head destroy's state.
    pub head: HeadWord,
    /// How many attempt lines the record declares.
    pub count: u64,
}

/// What:
///  One plan's header region,
///  its fields and its declared count,
///  read whole.
/// Why:
///  The two stages of the file reader each take one half of this parse,
///  and nothing else in the route reads a plan's header alone.
///
/// # Throws
/// The field or line that refuses the header.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const header = parsePlanHeader(headerBytes);
/// ```
pub fn parse_plan_header(header: &[u8]) -> Result<PlanHeader, PlanRefusal> {
    let (values, head, count) = parse_header(header)?;
    let facts: PlanFacts = PlanFacts {
        run: values
            .run
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "run"))?,
        owner: values
            .owner
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "owner"))?,
        started: values
            .started
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "started"))?,
        clone_dataset: values
            .clone_dataset
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "cloneDataset"))?,
        clone_guid: values
            .clone_guid
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "cloneGuid"))?,
        clone_createtxg: values.clone_createtxg.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "cloneCreatetxg");
        })?,
        origin_dataset: values
            .origin_dataset
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "originDataset"))?,
        origin_dataset_guid: values.origin_dataset_guid.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "originDatasetGuid");
        })?,
        origin_snapshot: values.origin_snapshot.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "originSnapshot");
        })?,
        origin_snapshot_guid: values.origin_snapshot_guid.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "originSnapshotGuid");
        })?,
        origin_snapshot_createtxg: values.origin_snapshot_createtxg.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "originSnapshotCreatetxg");
        })?,
        registration_id: values.registration_id.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "registrationIdHex");
        })?,
        admin: values
            .admin
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "admin"))?,
        target: values
            .target
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "targetHex"))?,
        unmount_nonce: values
            .unmount_nonce
            .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "unmountNonce"))?,
        unmount_mount_id: values.unmount_mount_id.ok_or_else(|| {
            return PlanRefusal::of(PlanRefusalCause::MissingKey, "unmountMountId");
        })?,
    };
    return Ok(PlanHeader {
        facts,
        head,
        count,
    });
}

/// What:
///  One whole plan from its two regions and the name it is filed under.
/// Why:
///  Every reader the design names calls this one parse,
///  so the bookmark rule,
///  the snapshot batch and the deferred-origin check cannot disagree about what a record says.
///
/// # Throws
/// The field,
/// line or ordering rule that refuses the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = parsePlan(header, attempts, name);
/// ```
pub fn parse_plan(
    header: &[u8],
    attempt_lines: &[u8],
    filed_under: Option<&str>,
) -> Result<DestructionPlan, PlanRefusal> {
    let parsed: PlanHeader = parse_plan_header(header)?;
    let facts: PlanFacts = parsed.facts;
    if let Some(name) = filed_under {
        if !is_canonical_guid_name(name) {
            return Err(PlanRefusal::at(
                PlanRefusalCause::ForeignName,
                "file name",
                name,
            ));
        }
        if name != facts.clone_guid.file_name() {
            return Err(PlanRefusal::at(
                PlanRefusalCause::NameDisagreesWithTheRecord,
                "cloneGuid",
                facts.clone_guid.file_name(),
            ));
        }
    }
    let attempts: Vec<PlanAttempt> =
        parse_attempts(attempt_lines, parsed.count, facts.clone_dataset.as_str())?;
    return Ok(DestructionPlan {
        facts,
        head: parsed.head,
        attempts,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_parse_tests.rs"]
mod tests;
