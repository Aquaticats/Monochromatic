//! What:
//!  Which commit scratch trees a later command removes,
//!  and which it leaves alone because the command that made them may still be reading them.
//! Why:
//!  The wrapper never removes what it did not create,
//!  and it must not remove a tree a live command still uses:
//!  a commit can hold its tree for as long as a person leaves an editor open in the scratch repository,
//!  so a tree is kept fresh for as long as its command lives
//!  (`service_scratch_renewal.rs`)
//!  and no reader of the tree is asked whether it is done.
//!  Age is therefore the only evidence a collection has,
//!  and a tree is old enough only once nothing that named it can still read it:
//!  a request that a killed client left queued is landing on it,
//!  so a tree goes when it has not been written for `SCRATCH_TREE_ABANDONED_AFTER`,
//!  and the same decision removes the other entries a scratch place holds.
//!  Reading the process that made a tree instead asked `/proc` a question that cannot answer this:
//!  a killed client's tree was collected at once,
//! from under the landing that still needed it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mayGo = (entry, metadata, now, day) => age(metadata, now) > (isACommitTree(entry) ? TWO_HOURS : day);
//! ```

/// The prefix of a commit's scratch tree,
/// and what an identity in a name may hold.
use crate::wrapper_names_old_and_new::{SERVICE_SCRATCH_PREFIX, is_identity_byte};
/// A path,
/// a moment and a span of time.
use std::path::Path;
use std::time::{Duration, SystemTime};

/// How old a commit scratch tree must be before any command may remove it.
///  Measured:
/// the longest whole commit in the commit timing log of 2026-11-03 was 750.6 s,
///  and `ANSWER_CEILING`
///  (`wait_ceilings.rs`)
///  ends every wait of a commit at 20 minutes;
///  a live command renews its tree every `SCRATCH_REFRESH_EVERY`,
///  so only a tree whose command is gone reaches this age,
///  and a landing that a killed client left queued reads its tree long before it.
///  Two hours is six times that ceiling.
pub const SCRATCH_TREE_ABANDONED_AFTER: Duration = Duration::from_secs(2 * 60 * 60);

/// The most entries one sweep removes,
/// so collecting what earlier commands left never becomes the work this command is doing.
///  Measured:
/// a commit's scratch tree of 2026-11-09 held 9,081,951 bytes in three files on this host,
///  so eight of them are about 73 MiB,
///  and the next command removes the next eight.
pub const SCRATCH_TREES_REMOVED_PER_COMMAND: usize = 8;

/// What:
///  Whether the entry at `path`,
/// listed with `metadata`,
/// may be removed now.
/// Why:
///   Two rules that disagree undo each other,
///  so the sweep of a scratch place and every other collection ask this one question.
///  A commit tree goes once it has not been written for `SCRATCH_TREE_ABANDONED_AFTER`:
///  its command renews it while it lives,
///  and a landing that a killed client left queued reads it inside that age.
///  Every other entry of a scratch place is the wrapper's own and goes by the age the caller gives it,
///  because nothing else ever collects it.
///  An entry whose age cannot be told,
///  and one dated in the future,
///  is never old enough:
/// a wrong clock must not delete live files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mayGo(entry, metadata, now, day) // true when nothing may still read it
/// ```
pub fn may_go(path: &Path, metadata: &std::fs::Metadata, now: SystemTime, max_age: Duration) -> bool {
    let age: Option<Duration> = age_before(metadata, now);
    let allowed: Duration = if names_a_trees_file(path) && metadata.is_dir() { SCRATCH_TREE_ABANDONED_AFTER } else { max_age };
    return match age {
        Some(found) => found > allowed,
        None => false,
    };
}

/// Whether `path` is named as one of the wrapper's commit scratch trees:
/// the prefix and an identity of letters,
/// digits and dashes.
fn names_a_trees_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else { return false };
    let Some(identity) = name.strip_prefix(SERVICE_SCRATCH_PREFIX) else { return false };
    return !identity.is_empty() && identity.as_bytes().iter().all(is_identity_byte);
}

/// How long before `now` the entry was last modified,
/// or nothing when the time cannot be told or lies in the future.
fn age_before(metadata: &std::fs::Metadata, now: SystemTime) -> Option<Duration> {
    let modified: SystemTime = metadata.modified().ok()?;
    return now.duration_since(modified).ok();
}

/// The decision is controlled in the test module.
#[cfg(test)]
#[path = "service_scratch_abandonment_tests.rs"]
mod tests;
