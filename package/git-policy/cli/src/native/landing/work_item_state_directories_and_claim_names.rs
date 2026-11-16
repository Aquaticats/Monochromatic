//! What:
//!  The folders an item moves through (`pending`,
//! `claimed`,
//! `recovering`,
//! `done`,
//! `failed`),
//! the names of the claim files
//!  inside them (id,
//! generation,
//! reading),
//! and the listing of a folder oldest first.
//! Why:
//!  The queue of work items (`work_item.rs`),
//! the recovery of claims (`work_item_recovery.rs`) and the renewal of claims all read
//!  and write these names;
//! they sit in a file that uses none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stateDirectory(workDir, 'claimed'); claimFileName(id, generation, now); parseClaimName(name); entriesOldestFirst(workDir, state)
//! ```

use crate::claim_time::ClaimTime;
use crate::random_id::random_uuid;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
/// The state directories,
/// in the order an item moves through them.
pub const PENDING: &str = "pending";

/// The state of a work item whose producer has not approved it yet:
/// nothing drains it,
/// and only an explicit promotion makes it runnable.
/// It is where a landed commit's push waits for the post-commit gate.
pub const HELD: &str = "held";
/// Held by one consumer,
/// with a lease.
pub const CLAIMED: &str = "claimed";
/// Held by one instance that is returning an expired claim to `pending`;
/// see `requeue_with`.
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
///  The order of two directory entries:
/// older first,
/// and by name when the times are equal.
/// Why:
///  `entries_oldest_first` sorts with it;
/// the sort is stable,
/// so equal entries keep their order.
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

/// Separates the parts of a claim's file name;
/// an id is a kind and a hexadecimal object name and holds none.
pub(crate) const NAME_SEPARATOR: char = '~';

/// What:
///  The id an entry of `claimed` or `recovering` is the claim of:
/// the part of its name before the generation.
/// Why:
///   A claim by an earlier release is named by the id alone,
/// so a name with no separator is its own id.
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
///   The generation is unique to one acquisition and the reading says when the holder last proved it was alive,
/// so the name
///  alone tells every other instance whose claim this is and whether it ran out,
/// and a rename of exactly that name is the only
///  way to take it:
/// a holder that renews changes the name,
/// so a take made from an older look finds nothing to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${id}~${token}~${stamp.encode()}`
/// ```
pub fn claim_file_name(id: &str, token: &str, stamp: &ClaimTime) -> String {
    return format!("{id}{NAME_SEPARATOR}{token}{NAME_SEPARATOR}{}", stamp.encode());
}

/// The generation and the reading in a claim's file name,
/// or nothing for a name by an earlier release.
pub(crate) fn parse_claim_name(name: &str) -> Option<(&str, ClaimTime)> {
    let (_, rest) = name.split_once(NAME_SEPARATOR)?;
    let (token, stamp) = rest.split_once(NAME_SEPARATOR)?;
    if token.is_empty() || !token.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    return Some((token, ClaimTime::decode(stamp)?));
}

/// What:
///  Whether `name` is the file of the claim of `id` of the generation `token`.
/// Why:
///   A claim is found by its id and the generation of its acquisition,
/// so another instance's claim of the same item is not taken for this one's,
/// and a settlement or a renewal reaches only the claim it was made for.
/// A claim by an earlier release has no generation,
/// so any name of that id is its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesClaim(name, id, token) // idOfEntry(name) === id && (token === '' || parseClaimName(name)?.[0] === token)
/// ```
pub(crate) fn names_claim(name: &str, id: &str, token: &str) -> bool {
    if id_of_entry(name) != id {
        return false;
    }
    if token.is_empty() {
        return true;
    }
    return match parse_claim_name(name) {
        Some((found, _stamp)) => found == token,
        None => false,
    };
}

/// A fresh generation:
/// 32 random hexadecimal digits.
pub(crate) fn new_generation() -> Result<String, String> {
    return match random_uuid() {
        Ok(uuid) => Ok(uuid.replace('-', "")),
        Err(error) => Err(format!("making the generation of a claim failed: {}", error.0)),
    };
}

/// What:
///  The names one walk over a state directory reached, oldest first,
///  and the error that stopped the walk before the directory's end.
/// Why:
///   A queue that is empty and one that could not be read to its end are two answers,
/// and a reader that says where a record is must tell them apart:
/// "could not tell" is never "none".
/// The names reached are kept whatever ended the walk,
/// so a walk that stopped part-way gives the names it has rather than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheWalkReached = { names: [string, Date][]; stoppedBy: Error | undefined }
/// ```
#[derive(Debug)]
pub(crate) struct WhatTheWalkReached {
    /// The names reached, oldest first.
    pub(crate) names: Vec<(String, SystemTime)>,
    /// The error that stopped the walk, or nothing when it reached the directory's end.
    pub(crate) stopped_by: Option<std::io::Error>,
}

impl WhatTheWalkReached {
    /// Whether the walk reached the directory's end.
    pub(crate) fn reached_the_end(&self) -> bool {
        return self.stopped_by.is_none();
    }
}

/// One entry of a directory as a walk gives it,
/// or the error the directory gave for it.
///  An entry whose time cannot be read counts as the oldest,
/// so a record an earlier release wrote is still the first the queue reaches.
fn one_entry(listed: std::io::Result<std::fs::DirEntry>) -> std::io::Result<(String, SystemTime)> {
    let entry: std::fs::DirEntry = listed?;
    let modified: SystemTime = match entry.metadata() {
        Ok(meta) => meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        Err(_unreadable) => SystemTime::UNIX_EPOCH,
    };
    return Ok((entry.file_name().to_string_lossy().into_owned(), modified));
}

/// What:
///  The names `entries` reaches, oldest first, and the error that stopped it.
/// Why:
///   The stopping is the walk's, so it is read where the walk runs and not by a caller that has
/// already lost the names the walk reached;
/// a control hands in a walk of its own through this body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesReached(entries) // { names, stoppedBy }
/// ```
fn names_reached(entries: &mut dyn Iterator<Item = std::io::Result<(String, SystemTime)>>) -> WhatTheWalkReached {
    let mut names: Vec<(String, SystemTime)> = Vec::new();
    let mut stopped_by: Option<std::io::Error> = None;
    for reached in entries {
        match reached {
            Ok(entry) => names.push(entry),
            Err(error) => {
                stopped_by = Some(error);
                break;
            }
        }
    }
    names.sort_by(oldest_first);
    return WhatTheWalkReached { names, stopped_by };
}

/// What:
///  The names in a state directory with their modification times, oldest first,
///  and the error that stopped the walk when it did not reach the end.
/// Why:
///  The count of listings a control reads is made here, where the reading is,
/// so the faces of a listing count once per pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theNamesIn(workDir, state) // { names, stoppedBy }
/// ```
pub(crate) fn the_names_in(work_dir: &Path, state: &str) -> WhatTheWalkReached {
    #[cfg(test)]
    count_listing();
    let listed = match std::fs::read_dir(state_directory(work_dir, state)) {
        Ok(listed) => listed,
        Err(error) => return WhatTheWalkReached { names: Vec::new(), stopped_by: Some(error) },
    };
    let mut entries = listed.map(one_entry);
    return names_reached(&mut entries);
}

/// What:
///  The names in a state directory with their modification times, oldest first.
/// Why:
///  The callers that read a queue to work on it read the names a walk reached and say nothing about the error that stopped it;
/// a caller that *says* where a record is takes `the_names_in` itself, which keeps that error.
pub(crate) fn entries_oldest_first(work_dir: &Path, state: &str) -> Vec<(String, SystemTime)> {
    return the_names_the_work_takes(the_names_in(work_dir, state));
}

/// What:
///  The names a caller that only works with the queue takes from a walk.
/// Why:
///   One body decides what such a caller sees,
/// so a control can hand in a walk that stopped part-way and read exactly what the callers would.
///  A walk that stopped part-way gives the names it reached:
/// a queue that failed on one name is not an empty queue,
/// and a caller that read none of them would take nothing that is owed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theNamesTheWorkTakes(reached) // string[]
/// ```
fn the_names_the_work_takes(reached: WhatTheWalkReached) -> Vec<(String, SystemTime)> {
    return reached.names;
}

#[cfg(test)]
thread_local! {
    /// How many listings of a state directory this thread made since the last reading.
    static LISTINGS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// In a control build,
/// count one listing of a state directory.
/// Why:
///   A control proves that a drain of many claims takes one listing per directory rather than one per claim (`RG2`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listings += 1
/// ```
#[cfg(test)]
fn count_listing() {
    LISTINGS.with(|listings| { listings.set(listings.get() + 1); });
}

/// What:
///  How many listings of a state directory this thread made since the last reading,
/// the reading resetting the count.
/// Why:
///   The count is the listing the code itself makes,
/// so a control states how many passes over the queue a drain costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = listings; listings = 0; return count
/// ```
#[cfg(test)]
pub(crate) fn take_listings() -> usize {
    return LISTINGS.with(std::cell::Cell::take);
}

/// Controls for the names one walk reaches stay out of the release executable.
#[cfg(test)]
#[path = "work_item_state_directories_and_claim_names_tests.rs"]
mod tests;
