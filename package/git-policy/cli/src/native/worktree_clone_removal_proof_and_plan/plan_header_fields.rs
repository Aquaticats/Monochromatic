//! What:
//!  The header region of one plan read into its fields:
//!  the fixed eighteen keys in the order the grammar writes them,
//!  and each value parsed into its own type.
//! Why:
//!  The reader parses this region before it asks the file's length,
//!  so a count with more digits than the grammar allows is refused within the header bound,
//!  and a key outside the order,
//!  a repeated key or an unknown one refuses the whole record rather than one field of it.
//!  A field stays unfilled here until the region ends,
//!  which is how an absent key is known only at the end,
//!  and `plan_parse.rs` completes them into the plan's facts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [values, head, count] = parseHeader(headerBytes);
//! ```

use crate::subvolume::fs_beneath::ObjectId;
use crate::worktree_clone_binding_record::binding_encoding::{
    hexadecimal_as_bytes,
    path_from_hexadecimal,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::canonical_decimal::{
    is_canonical_decimal,
    parse_canonical_u64,
};
use crate::worktree_clone_removal_proof_and_plan::plan_bounds::{
    ATTEMPT_COUNT_MAXIMUM_DIGITS,
    NUMBER_MAXIMUM_DIGITS,
};
use crate::worktree_clone_removal_proof_and_plan::plan_lines::take_line;
use crate::worktree_clone_removal_proof_and_plan::plan_name::{
    is_a_snapshot_of,
    is_dataset_name,
    is_snapshot_name,
};
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{
    PlanRefusal,
    PlanRefusalCause,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    NO_MOUNT_ID,
    PLAN_HEADER,
    PLAN_KEYS_IN_ORDER,
};
use crate::worktree_clone_removal_proof_and_plan::started_moment::is_seconds_with_offset;

/// The bytes of the header line's own prefix,
/// which stay the same while the version after them may not.
const HEADER_PREFIX: &str =
    "# monochromatic-git-policy-cli zfs worktree clone destruction plan, version ";

/// The fields one header region holds while they are parsed.
pub(super) struct HeaderValues {
    pub(super) run: Option<RequestNonce>,
    pub(super) owner: Option<String>,
    pub(super) started: Option<String>,
    pub(super) clone_dataset: Option<String>,
    pub(super) clone_guid: Option<DatasetGuid>,
    pub(super) clone_createtxg: Option<u64>,
    pub(super) origin_dataset: Option<String>,
    pub(super) origin_dataset_guid: Option<DatasetGuid>,
    pub(super) origin_snapshot: Option<String>,
    pub(super) origin_snapshot_guid: Option<DatasetGuid>,
    pub(super) origin_snapshot_createtxg: Option<u64>,
    pub(super) registration_id: Option<Vec<u8>>,
    pub(super) admin: Option<String>,
    pub(super) target: Option<std::path::PathBuf>,
    pub(super) unmount_nonce: Option<RequestNonce>,
    pub(super) unmount_mount_id: Option<Option<u64>>,
    pub(super) head: Option<HeadWord>,
    pub(super) attempts: Option<u64>,
}

impl HeaderValues {
    pub(super) fn empty() -> HeaderValues {
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
    return is_canonical_decimal(pid, ATTEMPT_COUNT_MAXIMUM_DIGITS)
        && is_canonical_decimal(start, NUMBER_MAXIMUM_DIGITS);
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
pub(super) fn parse_header(header: &[u8]) -> Result<(HeaderValues, HeadWord, u64), PlanRefusal> {
    let text: &str = std::str::from_utf8(header).map_err(|_| {
        return PlanRefusal::at(
            PlanRefusalCause::NotCanonical,
            "header",
            "bytes that are not text",
        );
    })?;
    let mut region: &str = text;
    let first: &str = take_line(&mut region, PlanRefusalCause::EndsEarly)?;
    if first != PLAN_HEADER {
        let found: String = match first.strip_prefix(HEADER_PREFIX) {
            Some(version) => {
                return Err(PlanRefusal::at(
                    PlanRefusalCause::UnknownVersion,
                    "version",
                    version,
                ));
            }
            None => String::from(first),
        };
        return Err(PlanRefusal::at(
            PlanRefusalCause::NotThisRecord,
            "header",
            found,
        ));
    }
    let mut values: HeaderValues = HeaderValues::empty();
    for (place, expected) in PLAN_KEYS_IN_ORDER.iter().enumerate() {
        let line: &str = take_line(&mut region, PlanRefusalCause::EndsEarly)?;
        let Some((key, value)) = line.split_once('=') else {
            return Err(PlanRefusal::at(
                PlanRefusalCause::NotCanonical,
                "line",
                line,
            ));
        };
        if key != *expected {
            let seen: PlanRefusalCause = match PLAN_KEYS_IN_ORDER
                .iter()
                .position(|known| return known == &key)
            {
                Some(at) if at < place => PlanRefusalCause::RepeatedKey,
                Some(_) => PlanRefusalCause::OutOfOrderKey,
                None => PlanRefusalCause::UnknownKey,
            };
            return Err(PlanRefusal::at(seen, key, value));
        }
        assign_field(&mut values, *expected, value)?;
    }
    if !region.is_empty() {
        return Err(PlanRefusal::at(
            PlanRefusalCause::UnknownKey,
            "line",
            region.lines().next().unwrap_or(region),
        ));
    }
    let head: HeadWord = values
        .head
        .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "head"))?;
    let count: u64 = values
        .attempts
        .ok_or_else(|| return PlanRefusal::of(PlanRefusalCause::MissingKey, "attempts"))?;
    return Ok((values, head, count));
}

/// One field's value,
/// parsed into its own type and refused when it is not that type's canonical spelling.
fn assign_field(
    values: &mut HeaderValues,
    key: &str,
    value: &str,
) -> Result<(), PlanRefusal> {
    let not_canonical =
        |found: &str| return PlanRefusal::at(PlanRefusalCause::NotCanonical, key, found);
    match key {
        "run" => {
            values.run = Some(RequestNonce::parse(value).map_err(|_| return not_canonical(value))?)
        }
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
        "cloneGuid" => {
            values.clone_guid =
                Some(DatasetGuid::parse(value).map_err(|_| return not_canonical(value))?)
        }
        "cloneCreatetxg" => {
            values.clone_createtxg = Some(
                parse_canonical_u64(value, NUMBER_MAXIMUM_DIGITS)
                    .ok_or_else(|| return not_canonical(value))?,
            )
        }
        "originDataset" => {
            if !is_dataset_name(value) {
                return Err(not_canonical(value));
            }
            values.origin_dataset = Some(String::from(value));
        }
        "originDatasetGuid" => {
            values.origin_dataset_guid =
                Some(DatasetGuid::parse(value).map_err(|_| return not_canonical(value))?)
        }
        "originSnapshot" => {
            let Some(dataset) = values.origin_dataset.as_deref() else {
                return Err(not_canonical(value));
            };
            if !is_snapshot_name(value) || !is_a_snapshot_of(dataset, value) {
                return Err(not_canonical(value));
            }
            values.origin_snapshot = Some(String::from(value));
        }
        "originSnapshotGuid" => {
            values.origin_snapshot_guid =
                Some(DatasetGuid::parse(value).map_err(|_| return not_canonical(value))?)
        }
        "originSnapshotCreatetxg" => {
            values.origin_snapshot_createtxg = Some(
                parse_canonical_u64(value, NUMBER_MAXIMUM_DIGITS)
                    .ok_or_else(|| return not_canonical(value))?,
            );
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
        "targetHex" => {
            values.target =
                Some(path_from_hexadecimal(value).ok_or_else(|| return not_canonical(value))?)
        }
        "unmountNonce" => {
            values.unmount_nonce =
                Some(RequestNonce::parse(value).map_err(|_| return not_canonical(value))?)
        }
        "unmountMountId" => {
            if value == NO_MOUNT_ID {
                values.unmount_mount_id = Some(None);
            } else {
                values.unmount_mount_id = Some(Some(
                    parse_canonical_u64(value, NUMBER_MAXIMUM_DIGITS)
                        .ok_or_else(|| return not_canonical(value))?,
                ));
            }
        }
        "head" => {
            values.head = Some(HeadWord::parse(value).ok_or_else(|| {
                return PlanRefusal::at(PlanRefusalCause::UnknownHead, key, value);
            })?)
        }
        "attempts" => {
            values.attempts = Some(
                parse_canonical_u64(value, ATTEMPT_COUNT_MAXIMUM_DIGITS)
                    .ok_or_else(|| return not_canonical(value))?,
            );
        }
        other => return Err(PlanRefusal::at(PlanRefusalCause::UnknownKey, other, value)),
    }
    return Ok(());
}
