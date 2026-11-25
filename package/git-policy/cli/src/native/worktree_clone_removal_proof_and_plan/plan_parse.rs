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

use crate::worktree_clone_binding_record::binding_encoding::{hexadecimal_as_bytes, path_from_hexadecimal};
use crate::worktree_clone_binding_record::plan_folder_read::is_canonical_guid_name;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::canonical_decimal::{is_canonical_decimal, parse_canonical_u64};
use crate::worktree_clone_removal_proof_and_plan::plan_bounds::{ATTEMPT_COUNT_MAXIMUM_DIGITS, NUMBER_MAXIMUM_DIGITS};
use crate::worktree_clone_removal_proof_and_plan::plan_attempt_parse::parse_attempts;
use crate::worktree_clone_removal_proof_and_plan::plan_lines::take_line;
use crate::worktree_clone_removal_proof_and_plan::plan_name::{is_a_snapshot_of, is_dataset_name, is_snapshot_name};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanAttempt, PlanFacts};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{PlanRefusal, PlanRefusalCause};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, NO_MOUNT_ID, PLAN_HEADER, PLAN_KEYS_IN_ORDER};
use crate::worktree_clone_removal_proof_and_plan::started_moment::is_seconds_with_offset;
use crate::subvolume::fs_beneath::ObjectId;

/// The bytes of the header line's own prefix,
/// which stay the same while the version after them may not.
const HEADER_PREFIX: &str = "# monochromatic-git-policy-cli zfs worktree clone destruction plan, version ";

/// The fields one header region holds while they are parsed.
struct HeaderValues {
    run: Option<RequestNonce>,
    owner: Option<String>,
    started: Option<String>,
    clone_dataset: Option<String>,
    clone_guid: Option<DatasetGuid>,
    clone_createtxg: Option<u64>,
    origin_dataset: Option<String>,
    origin_dataset_guid: Option<DatasetGuid>,
    origin_snapshot: Option<String>,
    origin_snapshot_guid: Option<DatasetGuid>,
    origin_snapshot_createtxg: Option<u64>,
    registration_id: Option<Vec<u8>>,
    admin: Option<String>,
    target: Option<std::path::PathBuf>,
    unmount_nonce: Option<RequestNonce>,
    unmount_mount_id: Option<Option<u64>>,
    head: Option<HeadWord>,
    attempts: Option<u64>,
}

impl HeaderValues {
    fn empty() -> HeaderValues {
        return HeaderValues {
            run: None,
            owner: None,
            started: None,
            clone_dataset: None,
            clone_guid: None,
            clone_createtxg: None,
            origin_dataset: None,
            origin_dataset_guid: None,
            origin_snapshot: None,
            origin_snapshot_guid: None,
            origin_snapshot_createtxg: None,
            registration_id: None,
            admin: None,
            target: None,
            unmount_nonce: None,
            unmount_mount_id: None,
            head: None,
            attempts: None,
        };
    }
}

/// Whether one `owner` value is shaped `<pid>:linux:<start time>`.
fn owner_is_shaped(value: &str) -> bool {
    let Some((pid, identity)) = value.split_once(':') else {
        return false;
    };
    let Some(start) = identity.strip_prefix("linux:") else {
        return false;
    };
    return is_canonical_decimal(pid, ATTEMPT_COUNT_MAXIMUM_DIGITS) && is_canonical_decimal(start, NUMBER_MAXIMUM_DIGITS);
}

/// What:
///  Parse the header region of one plan.
/// Why:
///  The reader parses this region before it reads the file's length,
///  so a count with more digits than the grammar allows is refused within the header bound.
///
/// # Throws
/// The key,
/// value or line that is not this grammar's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const header = parseHeader(headerBytes);
/// ```
fn parse_header(header: &[u8]) -> Result<(HeaderValues, HeadWord, u64), PlanRefusal> {
    let text: &str = std::str::from_utf8(header).map_err(|_| return PlanRefusal::at(PlanRefusalCause::NotCanonical, "header", "bytes that are not text"))?;
    let mut region: &str = text;
    let first: &str = take_line(&mut region, PlanRefusalCause::EndsEarly)?;
    if first != PLAN_HEADER {
        let found: String = match first.strip_prefix(HEADER_PREFIX) {
            Some(version) => {
                return Err(PlanRefusal::at(PlanRefusalCause::UnknownVersion, "version", version));
            }
            None => String::from(first),
        };
        return Err(PlanRefusal::at(PlanRefusalCause::NotThisRecord, "header", found));
    }
    let mut values: HeaderValues = HeaderValues::empty();
    for (place, expected) in PLAN_KEYS_IN_ORDER.iter().enumerate() {
        let line: &str = take_line(&mut region, PlanRefusalCause::EndsEarly)?;
        let Some((key, value)) = line.split_once('=') else {
            return Err(PlanRefusal::at(PlanRefusalCause::NotCanonical, "line", line));
        };
        if key != *expected {
            let seen: PlanRefusalCause = match PLAN_KEYS_IN_ORDER.iter().position(|known| return known == &key) {
                Some(at) if at < place => PlanRefusalCause::RepeatedKey,
                Some(_) => PlanRefusalCause::OutOfOrderKey,
                None => PlanRefusalCause::UnknownKey,
            };
            return Err(PlanRefusal::at(seen, key, value));
        }
        assign_field(&mut values, *expected, value)?;
    }
    if !region.is_empty() {
        return Err(PlanRefusal::at(PlanRefusalCause::UnknownKey, "line", region.lines().next().unwrap_or(region)));
    }
    let head: HeadWord = values.head.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "head"))?;
    let count: u64 = values.attempts.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "attempts"))?;
    return Ok((values, head, count));
}

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

/// One field's value,
/// parsed into its own type and refused when it is not that type's canonical spelling.
fn assign_field(values: &mut HeaderValues, key: &str, value: &str) -> Result<(), PlanRefusal> {
    let not_canonical = |found: &str| return PlanRefusal::at(PlanRefusalCause::NotCanonical, key, found);
    match key {
        "run" => values.run = Some(RequestNonce::parse(value).map_err(|_| return not_canonical(value))?),
        "owner" => {
            if !owner_is_shaped(value) {
                return Err(not_canonical(value));
            }
            values.owner = Some(String::from(value));
        }
        "started" => {
            if !is_seconds_with_offset(value) {
                return Err(not_canonical(value));
            }
            values.started = Some(String::from(value));
        }
        "cloneDataset" => {
            if !is_dataset_name(value) {
                return Err(not_canonical(value));
            }
            values.clone_dataset = Some(String::from(value));
        }
        "cloneGuid" => values.clone_guid = Some(DatasetGuid::parse(value).map_err(|_| return not_canonical(value))?),
        "cloneCreatetxg" => values.clone_createtxg = Some(parse_canonical_u64(value, NUMBER_MAXIMUM_DIGITS).ok_or_else(|| return not_canonical(value))?),
        "originDataset" => {
            if !is_dataset_name(value) {
                return Err(not_canonical(value));
            }
            values.origin_dataset = Some(String::from(value));
        }
        "originDatasetGuid" => values.origin_dataset_guid = Some(DatasetGuid::parse(value).map_err(|_| return not_canonical(value))?),
        "originSnapshot" => {
            let Some(dataset) = values.origin_dataset.as_deref() else {
                return Err(not_canonical(value));
            };
            if !is_snapshot_name(value) || !is_a_snapshot_of(dataset, value) {
                return Err(not_canonical(value));
            }
            values.origin_snapshot = Some(String::from(value));
        }
        "originSnapshotGuid" => values.origin_snapshot_guid = Some(DatasetGuid::parse(value).map_err(|_| return not_canonical(value))?),
        "originSnapshotCreatetxg" => {
            values.origin_snapshot_createtxg = Some(parse_canonical_u64(value, NUMBER_MAXIMUM_DIGITS).ok_or_else(|| return not_canonical(value))?);
        }
        "registrationIdHex" => {
            let Some(bytes) = hexadecimal_as_bytes(value) else {
                return Err(not_canonical(value));
            };
            values.registration_id = Some(bytes);
        }
        "admin" => {
            let Some(identity) = ObjectId::decode(value) else {
                return Err(not_canonical(value));
            };
            values.admin = Some(identity.encode());
        }
        "targetHex" => values.target = Some(path_from_hexadecimal(value).ok_or_else(|| return not_canonical(value))?),
        "unmountNonce" => values.unmount_nonce = Some(RequestNonce::parse(value).map_err(|_| return not_canonical(value))?),
        "unmountMountId" => {
            if value == NO_MOUNT_ID {
                values.unmount_mount_id = Some(None);
            } else {
                values.unmount_mount_id = Some(Some(parse_canonical_u64(value, NUMBER_MAXIMUM_DIGITS).ok_or_else(|| return not_canonical(value))?));
            }
        }
        "head" => values.head = Some(HeadWord::parse(value).ok_or_else(|| return PlanRefusal::at(PlanRefusalCause::UnknownHead, key, value))?),
        "attempts" => {
            values.attempts = Some(parse_canonical_u64(value, ATTEMPT_COUNT_MAXIMUM_DIGITS).ok_or_else(|| return not_canonical(value))?);
        }
        other => return Err(PlanRefusal::at(PlanRefusalCause::UnknownKey, other, value)),
    }
    return Ok(());
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
        run: values.run.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "run"))?,
        owner: values.owner.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "owner"))?,
        started: values.started.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "started"))?,
        clone_dataset: values.clone_dataset.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "cloneDataset"))?,
        clone_guid: values.clone_guid.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "cloneGuid"))?,
        clone_createtxg: values.clone_createtxg.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "cloneCreatetxg"))?,
        origin_dataset: values.origin_dataset.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "originDataset"))?,
        origin_dataset_guid: values.origin_dataset_guid.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "originDatasetGuid"))?,
        origin_snapshot: values.origin_snapshot.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "originSnapshot"))?,
        origin_snapshot_guid: values.origin_snapshot_guid.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "originSnapshotGuid"))?,
        origin_snapshot_createtxg: values.origin_snapshot_createtxg.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "originSnapshotCreatetxg"))?,
        registration_id: values.registration_id.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "registrationIdHex"))?,
        admin: values.admin.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "admin"))?,
        target: values.target.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "targetHex"))?,
        unmount_nonce: values.unmount_nonce.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "unmountNonce"))?,
        unmount_mount_id: values.unmount_mount_id.ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "unmountMountId"))?,
    };
    return Ok(PlanHeader { facts, head, count });
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
pub fn parse_plan(header: &[u8], attempt_lines: &[u8], filed_under: Option<&str>) -> Result<DestructionPlan, PlanRefusal> {
    let parsed: PlanHeader = parse_plan_header(header)?;
    let facts: PlanFacts = parsed.facts;
    if let Some(name) = filed_under {
        if !is_canonical_guid_name(name) {
            return Err(PlanRefusal::at(PlanRefusalCause::ForeignName, "file name", name));
        }
        if name != facts.clone_guid.file_name() {
            return Err(PlanRefusal::at(PlanRefusalCause::NameDisagreesWithTheRecord, "cloneGuid", facts.clone_guid.file_name()));
        }
    }
    let attempts: Vec<PlanAttempt> = parse_attempts(attempt_lines, parsed.count, facts.clone_dataset.as_str())?;
    return Ok(DestructionPlan { facts, head: parsed.head, attempts });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_parse_tests.rs"]
mod tests;
