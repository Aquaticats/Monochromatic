//! What:
//!  The byte bounds of the binding and the inventory records,
//!  each computed from the kernel limits the design's "The size bound" derives them from.
//! Why:
//!  A reader refuses a record by size before it parses one,
//!  so a runaway or replaced file is named instead of read into memory;
//!  rule `LQM` asks every shipped limit to cite its basis,
//!  and the design's own sum is 43,027 bytes for the binding,
//!  computed here from the terms rather than written as a number.
//!  Nothing bounds an origin's history,
//!  so no inventory carries a total cap:
//!  its own `entries=` count refuses a truncated file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bindingBound = 69 + widest('claim', 20) + widest('nonce', 32) + ... ;
//! const widestEntry = 'entry='.length + 'snapshot'.length + 3 + 20 + 20 + 255 + 1;
//! ```

use super::binding_words::{
    BINDING_HEADER,
    INVENTORY_HEADER,
};

/// The kernel's `PATH_MAX`,
/// measured as 4096 on this host (`F14` of the design's facts).
const PATH_MAX: u64 = 4_096;

/// Git's `gitdir: ` prefix inside a pointer file,
/// which the pointer bound counts.
const GITDIR_POINTER_PREFIX: u64 = 8;

/// `ZFS_MAX_DATASET_NAME_LEN` less its terminating NUL (`F9`).
const ZFS_NAME_WIDEST: u64 = 255;

/// The widest decimal form of an unsigned 64-bit number (`F11`).
const DECIMAL_WIDEST: u64 = 20;

/// The widest folder identity:
/// inode,
/// birth seconds and birth nanoseconds,
/// each at its widest,
/// and their two colons.
const IDENTITY_WIDEST: u64 = 20 + 1 + 20 + 1 + 10;

/// The nonce's fixed width,
/// in lowercase hexadecimal digits.
const NONCE_WIDEST: u64 = 32;

/// The widest phase word,
/// `snapshotted`.
const PHASE_WIDEST: u64 = 11;

/// The widest settle reason,
/// `checkout`.
const SETTLE_REASON_WIDEST: u64 = 8;

/// The widest request form word:
/// `branch`,
/// `commit` or `detach`.
const REQUEST_FORM_WIDEST: u64 = 6;

/// The most generations one slot's claim may take,
/// as the widest decimal.
const CLAIM_GENERATION_WIDEST: u64 = DECIMAL_WIDEST;

/// One path in hexadecimal:
/// twice `PATH_MAX`.
const PATH_HEXADECIMAL_WIDEST: u64 = PATH_MAX * 2;

/// One pointer in hexadecimal:
/// `gitdir: `,
/// a path and a newline,
/// each byte doubled.
const POINTER_HEXADECIMAL_WIDEST: u64 = (GITDIR_POINTER_PREFIX + PATH_MAX + 1) * 2;

/// The widest request value:
/// its form word,
/// a space,
/// and a doubled `PATH_MAX` argument.
const REQUEST_WIDEST: u64 = REQUEST_FORM_WIDEST + 1 + PATH_HEXADECIMAL_WIDEST;

/// One `key=value` line's bytes,
/// its newline included.
const fn line(
    key: &str,
    value: u64,
) -> u64 {
    return key.len() as u64 + 1 + value + 1;
}

/// The binding's header line and its newline.
const fn header_bytes() -> u64 {
    return BINDING_HEADER.len() as u64 + 1;
}

/// The widest line a binding may hold,
/// which is the `pointer=` line.
pub const BINDING_MAXIMUM_LINE_BYTES: u64 = line("pointer", POINTER_HEXADECIMAL_WIDEST);

/// The largest one binding file may hold,
/// which is every line of the grammar at its widest.
pub const BINDING_MAXIMUM_BYTES: u64 = header_bytes()
    + line("claim", CLAIM_GENERATION_WIDEST)
    + line("nonce", NONCE_WIDEST)
    + line("request", REQUEST_WIDEST)
    + line("phase", PHASE_WIDEST)
    + line("settleReason", SETTLE_REASON_WIDEST)
    + line("checkoutDataset", ZFS_NAME_WIDEST)
    + line("checkoutDatasetGuid", DECIMAL_WIDEST)
    + line("repository", PATH_HEXADECIMAL_WIDEST)
    + line("repositoryIdentity", IDENTITY_WIDEST)
    + line("slot", PATH_HEXADECIMAL_WIDEST)
    + line("slotIdentity", IDENTITY_WIDEST)
    + line("registration", PATH_HEXADECIMAL_WIDEST)
    + line("registrationIdentity", IDENTITY_WIDEST)
    + line("pool", ZFS_NAME_WIDEST)
    + line("poolGuid", DECIMAL_WIDEST)
    + line("originSnapshot", ZFS_NAME_WIDEST)
    + line("originSnapshotGuid", DECIMAL_WIDEST)
    + line("originSnapshotCreatetxg", DECIMAL_WIDEST)
    + line("originDataset", ZFS_NAME_WIDEST)
    + line("originDatasetGuid", DECIMAL_WIDEST)
    + line("cloneDataset", ZFS_NAME_WIDEST)
    + line("cloneGuid", DECIMAL_WIDEST)
    + line("cloneCreatetxg", DECIMAL_WIDEST)
    + line("pointer", POINTER_HEXADECIMAL_WIDEST);

/// The widest one inventory entry line may be:
/// `entry=`,
/// the kind,
/// three separators,
/// a GUID,
/// a `createtxg`,
/// a ZFS name and its newline.
pub const INVENTORY_ENTRY_LINE_MAXIMUM_BYTES: u64 =
    6 + 8 + 3 + DECIMAL_WIDEST + DECIMAL_WIDEST + ZFS_NAME_WIDEST + 1;

/// The inventory's header line and its newline.
const fn inventory_header_bytes() -> u64 {
    return INVENTORY_HEADER.len() as u64 + 1;
}

/// The widest one inventory's fixed lines may be,
/// which is the header and the eight fields of its grammar.
pub const INVENTORY_FIXED_BYTES: u64 = inventory_header_bytes()
    + line("kind", 6)
    + line("poolGuid", DECIMAL_WIDEST)
    + line("dataset", ZFS_NAME_WIDEST)
    + line("datasetGuid", DECIMAL_WIDEST)
    + line("bounds", ZFS_NAME_WIDEST)
    + line("boundsGuid", DECIMAL_WIDEST)
    + line("boundsCreatetxg", DECIMAL_WIDEST)
    + line("entries", DECIMAL_WIDEST);

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_bound_tests.rs"]
mod tests;
