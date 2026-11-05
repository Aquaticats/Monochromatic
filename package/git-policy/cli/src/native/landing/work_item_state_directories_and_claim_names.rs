//! What:
//!  The folders an item moves through (`pending`, `claimed`, `recovering`, `done`, `failed`), the names of the claim files
//!  inside them (id, generation, reading), and the listing of a folder oldest first.
//! Why:
//!  The queue of work items (`work_item.rs`), the recovery of claims (`work_item_recovery.rs`) and the renewal of claims all read
//!  and write these names; they sit in a file that uses none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stateDirectory(workDir, 'claimed'); claimFileName(id, generation, now); parseClaimName(name); entriesOldestFirst(workDir, state)
//! ```

use crate::claim_time::ClaimTime;
use crate::random_id::random_uuid;
pub use crate::work_item_recovery::{Recovered, next_claim_expiry, requeue_expired, requeue_expired_reporting};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
/// The state directories, in the order an item moves through them.
pub const PENDING: &str = "pending";
/// Held by one consumer, with a lease.
pub const CLAIMED: &str = "claimed";
/// Held by one instance that is returning an expired claim to `pending`; see `requeue_with`.
pub const RECOVERING: &str = "recovering";
/// Finished.
pub const DONE: &str = "done";
/// Given up after too many tries.
pub const FAILED: &str = "failed";

/// The directory of one state.
pub fn state_directory(work_dir: &Path, state: &str) -> PathBuf {
    return work_dir.join(state);
}

/// What:
///  The order of two directory entries: older first, and by name when the times are equal.
/// Why:
///  `entries_oldest_first` sorts with it; the sort is stable, so equal entries keep their order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.sort((a, b) => a.time - b.time || (a.name < b.name ? -1 : a.name > b.name ? 1 : 0))
/// ```
pub(crate) fn oldest_first(left: &(String, SystemTime), right: &(String, SystemTime)) -> std::cmp::Ordering {
    return match left.1.cmp(&right.1) {
        std::cmp::Ordering::Equal => left.0.cmp(&right.0),
        other => other,
    };
}

/// Separates the parts of a claim's file name; an id is a kind and a hexadecimal object name and holds none.
pub(crate) const NAME_SEPARATOR: char = '~';

/// What:
///  The id an entry of `claimed` or `recovering` is the claim of: the part of its name before the generation.
/// Why:
///   A claim by an earlier release is named by the id alone, so a name with no separator is its own id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.split('~')[0]
/// ```
pub fn id_of_entry(name: &str) -> &str {
    return match name.split_once(NAME_SEPARATOR) {
        Some((id, _generation)) => id,
        None => name,
    };
}

/// What:
///  The name of the file that holds the claim of `id` of the generation `token` as of the reading `stamp`.
/// Why:
///   The generation is unique to one acquisition and the reading says when the holder last proved it was alive, so the name
///  alone tells every other instance whose claim this is and whether it ran out, and a rename of exactly that name is the only
///  way to take it: a holder that renews changes the name, so a take made from an older look finds nothing to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${id}~${token}~${stamp.encode()}`
/// ```
pub fn claim_file_name(id: &str, token: &str, stamp: &ClaimTime) -> String {
    return format!("{id}{NAME_SEPARATOR}{token}{NAME_SEPARATOR}{}", stamp.encode());
}

/// The generation and the reading in a claim's file name, or nothing for a name by an earlier release.
pub(crate) fn parse_claim_name(name: &str) -> Option<(&str, ClaimTime)> {
    let (_, rest) = name.split_once(NAME_SEPARATOR)?;
    let (token, stamp) = rest.split_once(NAME_SEPARATOR)?;
    if token.is_empty() || !token.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    return Some((token, ClaimTime::decode(stamp)?));
}

/// A fresh generation: 32 random hexadecimal digits.
pub(crate) fn new_generation() -> Result<String, String> {
    return match random_uuid() {
        Ok(uuid) => Ok(uuid.replace('-', "")),
        Err(error) => Err(format!("making the generation of a claim failed: {}", error.0)),
    };
}

/// The names in a state directory with their modification times, oldest first.
pub(crate) fn entries_oldest_first(work_dir: &Path, state: &str) -> Vec<(String, SystemTime)> {
    let mut found: Vec<(String, SystemTime)> = Vec::new();
    if let Ok(listing) = std::fs::read_dir(state_directory(work_dir, state)) {
        for entry in listing.flatten() {
            // An entry whose time cannot be read counts as the oldest.
            let modified: SystemTime = match entry.metadata() {
                Ok(meta) => meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                Err(_unreadable) => SystemTime::UNIX_EPOCH,
            };
            found.push((entry.file_name().to_string_lossy().into_owned(), modified));
        }
    }
    found.sort_by(oldest_first);
    return found;
}
