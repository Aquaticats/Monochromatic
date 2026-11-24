//! What:
//!  The byte bounds a destruction plan's reader enforces and its writer proves:
//!  the header's 10,130 bytes,
//!  the attempt line's 361 bytes,
//!  and the widest value each header field may hold,
//!  derived from the kernel bounds each one rests on.
//! Why:
//!  The record's reader reads in two stages and refuses an oversized file before it reads an
//!  attempt byte,
//!  so the bound is what keeps its memory to the record's own length with no count ceiling
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The size bound and its basis" and facts `F1`,
//! `F11` and `F14`).
//!  The per-field widths are this build's own reading of those facts:
//! they sum below the design's
//!  10,130,
//!  so the design's number stays the reader's cap and every field still has one bound a writer can
//!  refuse a value past.
//!  A ZFS name is shorter than 256 bytes,
//!  the slot path is at most the kernel's 4,095,
//!  a canonical `uint64` takes at most twenty digits,
//!  and an identity is four numbers and three colons.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const maximum = (attempts: number) => HEADER + ATTEMPT_LINE * attempts;
//! ```

use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    ATTEMPT_KEY, ATTEMPT_NAME_KEY, PLAN_HEADER, SNAPSHOT_KIND,
};

/// The largest header a plan may carry,
/// which is the version line and eighteen fields at their widest.
///
/// Why:
///  The design's own computed figure (`F11`),
///  kept as the reader's stage-one cap.
pub const PLAN_HEADER_MAXIMUM_BYTES: u64 = 10_130;

/// The largest one attempt line may carry.
///
/// Why:
///  The design's own sum:
/// the key,
///  the kind,
///  an attempt number,
///  an outcome word,
///  two canonical `uint64` values,
///  an errno,
///  a name key and a 255-byte name,
///  six separators and the newline.
pub const ATTEMPT_LINE_MAXIMUM_BYTES: u64 = 361;

/// The most digits a canonical `uint64` takes.
pub const NUMBER_MAXIMUM_DIGITS: usize = 20;

/// The most digits a declared attempt count takes,
/// which is the grammar's own ceiling rather than a workload number.
pub const ATTEMPT_COUNT_MAXIMUM_DIGITS: usize = 7;

/// The most digits an errno takes.
pub const ERRNO_MAXIMUM_DIGITS: usize = 11;

/// The longest one ZFS name the kernel accepts,
/// less its terminator.
pub const ZFS_NAME_MAXIMUM_BYTES: usize = 255;

/// The longest slot path a plan records:
/// the kernel's own path bound.
pub const SLOT_PATH_MAXIMUM_BYTES: usize = 4_095;

/// The digits of a request nonce,
/// spelled as the route spells it.
pub const NONCE_MAXIMUM_DIGITS: usize = 32;

/// The longest head word,
/// which is `requested`.
pub const HEAD_WORD_MAXIMUM_BYTES: usize = 9;

/// The longest snapshot outcome word,
/// which is `destroyed`.
pub const OUTCOME_WORD_MAXIMUM_BYTES: usize = 9;

/// The longest folder identity,
/// which is four `uint64` values and the three colons between them.
pub const IDENTITY_MAXIMUM_BYTES: usize = 4 * NUMBER_MAXIMUM_DIGITS + 3;

/// The longest `owner` value:
/// a seven-digit process id,
/// one colon,
/// the birth identity's prefix,
/// and a canonical `uint64`.
pub const OWNER_MAXIMUM_BYTES: usize = 8 + 1 + 6 + NUMBER_MAXIMUM_DIGITS;

/// The longest `started` value,
/// which is an ISO 8601 second with a six-byte offset.
pub const STARTED_MAXIMUM_BYTES: usize = 26;

/// The separators one attempt line carries between its fields,
/// which are the six single spaces.
pub const ATTEMPT_SEPARATORS_BYTES: u64 = 6;

/// The widest value each header key may carry,
/// in the order the header writes them.
///
/// Why:
///  A writer refuses a value wider than its key's bound,
///  so one rule proves a plan cannot carry a field its reader would refuse.
pub const HEADER_FIELD_WIDEST_BYTES: [(&str, usize); 18] = [
    ("run", NONCE_MAXIMUM_DIGITS),
    ("owner", OWNER_MAXIMUM_BYTES),
    ("started", STARTED_MAXIMUM_BYTES),
    ("cloneDataset", ZFS_NAME_MAXIMUM_BYTES),
    ("cloneGuid", NUMBER_MAXIMUM_DIGITS),
    ("cloneCreatetxg", NUMBER_MAXIMUM_DIGITS),
    ("originDataset", ZFS_NAME_MAXIMUM_BYTES),
    ("originDatasetGuid", NUMBER_MAXIMUM_DIGITS),
    ("originSnapshot", ZFS_NAME_MAXIMUM_BYTES),
    ("originSnapshotGuid", NUMBER_MAXIMUM_DIGITS),
    ("originSnapshotCreatetxg", NUMBER_MAXIMUM_DIGITS),
    ("registrationIdHex", 2 * ZFS_NAME_MAXIMUM_BYTES),
    ("admin", IDENTITY_MAXIMUM_BYTES),
    ("targetHex", 2 * SLOT_PATH_MAXIMUM_BYTES),
    ("unmountNonce", NONCE_MAXIMUM_DIGITS),
    ("unmountMountId", NUMBER_MAXIMUM_DIGITS),
    ("head", HEAD_WORD_MAXIMUM_BYTES),
    ("attempts", ATTEMPT_COUNT_MAXIMUM_DIGITS),
];

/// The bytes one key and one value take as a line:
/// the key,
/// the separator,
/// the value and the newline.
pub fn line_bytes(key: &str, value_bytes: usize) -> u64 {
    return key.len() as u64 + 1 + value_bytes as u64 + 1;
}

/// The bytes one line takes when it is text and no value:
/// the text and the newline,
/// which is how the version line is written.
pub fn bare_line_bytes(text: &str) -> u64 {
    return text.len() as u64 + 1;
}

/// The widest one header line may carry,
/// or nothing when the key is not the plan's.
pub fn header_field_maximum_bytes(key: &str) -> Option<u64> {
    for (known, widest) in HEADER_FIELD_WIDEST_BYTES {
        if known == key {
            return Some(line_bytes(known, widest));
        }
    }
    return None;
}

/// The widest header this build writes:
/// the version line and eighteen fields at the bounds above.
///
/// Why:
///  The reader's cap is the design's 10,130 and this sum is what a writer can reach,
///  so a control compares the two rather than restating either.
pub fn widest_header_bytes() -> u64 {
    let mut total: u64 = bare_line_bytes(PLAN_HEADER);
    for (key, widest) in HEADER_FIELD_WIDEST_BYTES {
        total += line_bytes(key, widest);
    }
    return total;
}

/// The widest attempt line this build writes,
/// computed from the words and bounds it holds.
pub fn widest_attempt_line_bytes() -> u64 {
    // `tuple=` is a prefix and takes no separator after it, so the six separators sit between the
    // kind, the attempt, the outcome, the GUID, the group, the errno and `name=`.
    let pieces: usize = ATTEMPT_KEY.len()
        + 1
        + SNAPSHOT_KIND.len()
        + NUMBER_MAXIMUM_DIGITS
        + OUTCOME_WORD_MAXIMUM_BYTES
        + NUMBER_MAXIMUM_DIGITS
        + NUMBER_MAXIMUM_DIGITS
        + ERRNO_MAXIMUM_DIGITS
        + ATTEMPT_NAME_KEY.len()
        + 1
        + ZFS_NAME_MAXIMUM_BYTES;
    return pieces as u64 + ATTEMPT_SEPARATORS_BYTES + 1;
}

/// The largest file a plan declaring `attempt_lines` attempt lines may be.
pub fn plan_maximum_bytes(attempt_lines: u64) -> u64 {
    return PLAN_HEADER_MAXIMUM_BYTES + ATTEMPT_LINE_MAXIMUM_BYTES * attempt_lines;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_bounds_tests.rs"]
mod tests;
