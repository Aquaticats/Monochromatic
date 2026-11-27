//! What:
//!  The names a snapshot made ahead of an add carries,
//!  the folder of the pool it stands in,
//!  and which of those entries a source owns,
//!  how old each is,
//!  and which ones an add keeps no longer.
//! Why:
//!  A snapshot ahead copies one source root,
//!  and one worktrees folder can hold the snapshots of an add run from the main checkout beside
//!  those of an add run from a linked worktree,
//!  so an add must never claim a copy of another tree.
//!  The name is the whole test for that,
//!  so no record has to be read:
//!  the first sixteen hexadecimal digits of the digest of the source's path,
//!  the millisecond it was made in,
//!  and a random tail.
//!  The time is in the name so the newest entry is found by sorting the names,
//!  and the age decides whether a copy nothing claimed stays.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_snapshot_ahead_asks.rs` holds the records that ask the service for a copy,
//!  and `worktree_snapshot_ahead.rs` holds the answer one add's look came to and the hand-over of
//!  the copies it proved stale.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const name = (root, at) => `${AHEAD_PREFIX}${key(root)}-${at.toString(16)}-${randomUuid()}`;
//! const mine = candidates(pool, key(root)); // newest first
//! ```

use crate::fs_beneath::{
    handle_path,
    open_root,
};
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::path_bytes::path_bytes_of;
use crate::random_id::random_uuid;
use crate::subvolume_sha::sha256;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    SystemTime,
};

/// What:
///  The start of the name of a snapshot the wrapper made ahead,
/// in the folder an add makes its stage in.
/// Why:
///   The name says what the directory is to a person reading the worktrees folder,
/// and it is the same prefix on every filesystem,
///  so one listing finds a source's entries wherever they are.
/// It has neither the stage prefix nor the shape a removal renames a
///  subvolume to,
/// so nothing that reads those names takes an entry for its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AHEAD_PREFIX = '.cli-git-worktree-ahead-';
/// ```
pub const AHEAD_PREFIX: &str = ".cli-git-worktree-ahead-";

/// What:
///  How many snapshots ahead one source keeps.
/// Why:
///  Three is what the measured cost and the measured arrivals need at the median:
/// one snapshot took 7,443.8 ms at
///  the median while adds started 2.8 s apart at the tenth percentile of the gap between two
/// (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
/// "Step 2"),
/// so 2.7 snapshots' worth of adds can arrive
///  inside one snapshot's cost.
/// The fourteen the worst measured call would need are not kept,
/// because
///  every entry pins the blocks of each file the source rewrites after it,
/// on a volume that is 95 percent full;
/// the
///  ceiling and the rung after the snapshot are what covers that case instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AHEAD_KEPT = 3;
/// ```
pub const AHEAD_KEPT: usize = 3;

/// What:
///  How old a snapshot ahead may be and still be kept for an add that has not arrived.
/// Why:
///   A snapshot pins the blocks of every file the source rewrites after it,
/// and nothing else would ever remove one,
/// so a
///  repository whose adds stopped must not keep them for good.
/// Ten minutes is longer than the measured gap between two
///  adds except an idle night ("Step 2"),
/// and every add removes what it finds older here,
/// so an active
///  repository never reaches it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AHEAD_OLDEST = 10 * 60 * 1000;
/// ```
pub const AHEAD_OLDEST: Duration = Duration::from_secs(10 * 60);

/// What:
///  How often the wait looks in the folder again.
/// Why:
///   Short enough that an add takes a snapshot the moment the service publishes one,
/// and long enough that the look
///  costs one listing rather than the processor.
/// It is also the delay the wait adds to an entry that arrived between two looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POLL = 25;
/// ```
pub(crate) const POLL: Duration = Duration::from_millis(25);

/// What:
///  The first sixteen hexadecimal digits of the digest of a source root's path,
/// which every name of that source's snapshots ahead carries.
/// Why:
///   A snapshot ahead copies one source root,
/// and one worktrees folder can hold the snapshots of an add run from the main
///  checkout beside those of an add run from a linked worktree.
/// An add must never claim a copy of another tree,
/// and the
///  name is the whole test for that,
/// so no record has to be read.
/// Sixteen digits are 64 bits of the digest,
/// which two
///  paths of one machine do not reach,
/// and they keep the name short enough to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const key = (root: string) => createHash('sha256').update(root).digest('hex').slice(0, 16);
/// ```
pub(crate) fn ahead_key(root: &Path) -> String {
    let digest: String = hexadecimal_of_bytes(&sha256(path_bytes_of(root)));
    return digest.chars().take(16).collect();
}

/// What:
///  The name a snapshot ahead of `source_root` takes when the service makes it,
/// `at` milliseconds after the epoch.
/// Why:
///   The time is in the name so the newest entry is found by sorting the names,
/// and the random part keeps two
///  services of one repository from choosing one name.
/// The key is what makes the name one source's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (root: string, at: number) => `${AHEAD_PREFIX}${key(root)}-${at.toString(16)}-${randomUUID().replaceAll('-', '')}`;
/// ```
pub fn ahead_name(
    source_root: &Path,
    at: SystemTime,
) -> Result<OsString, String> {
    let unique: String = match random_uuid() {
        Ok(made) => made.replace('-', ""),
        Err(error) => {
            return Err(format!(
                "no name can be made for a snapshot ahead ({})",
                error.0
            ));
        }
    };
    let milliseconds: u128 = at
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    return Ok(OsString::from(format!(
        "{AHEAD_PREFIX}{}-{milliseconds:016x}-{unique}",
        ahead_key(source_root)
    )));
}

/// What:
///  The millisecond a name says its snapshot was made in,
/// or nothing when the name is no snapshot ahead.
/// Why:
///   The age of an entry decides whether it is kept,
/// and the newest is the one an add proves first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const madeAt = (name: string) => BigInt('0x' + name.slice(AHEAD_PREFIX.length).split('-')[1]);
/// ```
pub(crate) fn made_at(name: &OsStr) -> Option<u128> {
    let text: &str = name.to_str()?;
    let after_prefix: &str = text.strip_prefix(AHEAD_PREFIX)?;
    let (_key, after_key) = after_prefix.split_once('-')?;
    let (milliseconds, unique) = after_key.split_once('-')?;
    if unique.is_empty() {
        return None;
    }
    return u128::from_str_radix(milliseconds, 16).ok();
}

/// What:
///  Whether a name is one of `key`'s snapshots ahead.
/// Why:
///   One folder can hold the snapshots of several source roots,
/// and only this source's own are candidates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOfThisSource = (name: string, key: string) => name.startsWith(`${AHEAD_PREFIX}${key}-`) && madeAt(name) !== undefined;
/// ```
pub(crate) fn is_of_this_source(
    name: &OsStr,
    key: &str,
) -> bool {
    let Some(text) = name.to_str() else {
        return false;
    };
    let Some(rest) = text.strip_prefix(AHEAD_PREFIX) else {
        return false;
    };
    let Some((found, _rest)) = rest.split_once('-') else {
        return false;
    };
    return found == key && made_at(name).is_some();
}

/// What:
///  The order of two entries by the millisecond each was made in,
/// newest first.
/// Why:
///   The newest candidate is the one most likely to still equal the source's index,
/// so it is proven first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const newestFirst = (left, right) => right.made - left.made;
/// ```
fn newest_first(
    left: &(u128, OsString),
    right: &(u128, OsString),
) -> std::cmp::Ordering {
    return right.0.cmp(&left.0);
}

/// What:
///  The names of `key`'s snapshots ahead in the held folder,
/// newest first.
/// Why:
///   The newest is the one most likely to still equal the source's index,
/// so it is proven first;
/// an older one is
///  proven only when the newer ones were claimed or failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const candidates = (dir, key) => readdirSync(handlePath(dir)).filter((name) => isOfThisSource(name, key)).sort((left, right) => madeAt(right) - madeAt(left));
/// ```
pub(crate) fn candidates(
    folder: &File,
    key: &str,
) -> Vec<OsString> {
    let Ok(listing) = std::fs::read_dir(handle_path(folder)) else {
        return Vec::new();
    };
    let mut found: Vec<(u128, OsString)> = Vec::new();
    for entry in listing.flatten() {
        let name: OsString = entry.file_name();
        if !is_of_this_source(name.as_os_str(), key) {
            continue;
        }
        let Some(at) = made_at(name.as_os_str()) else {
            continue;
        };
        found.push((at, name));
    }
    found.sort_by(newest_first);
    let mut names: Vec<OsString> = Vec::with_capacity(found.len());
    for (_at, name) in found {
        names.push(name);
    }
    return names;
}

/// What:
///  The names of the snapshots ahead in `folder` this add keeps no longer:
/// everything after the newest `AHEAD_KEPT`,
/// and everything older than
///  `AHEAD_OLDEST`.
/// Why:
///   Every add leaves the pool no larger,
/// and one that claimed nothing must still not keep a growing pile,
/// while an
///  entry no add claimed for ten minutes is one nothing needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overKept = (names, now) => names.filter((name, index) => index >= AHEAD_KEPT || now - madeAt(name) > AHEAD_OLDEST);
/// ```
pub(crate) fn over_kept(
    names: &[OsString],
    now: u128,
) -> Vec<OsString> {
    let mut over: Vec<OsString> = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let too_old: bool = match made_at(name.as_os_str()) {
            Some(made) => older_than_kept(made, now),
            None => true,
        };
        if index >= AHEAD_KEPT || too_old {
            over.push(name.clone());
        }
    }
    return over;
}

/// What:
///  Whether a copy made `made` milliseconds after the epoch is older than an add keeps.
/// Why:
///   The age is the one number that decides whether a copy a repository is not adding to stays,
/// and one named
///  relation keeps the rule in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const olderThanKept = (made, now) => now - made > AHEAD_OLDEST;
/// ```
fn older_than_kept(
    made: u128,
    now: u128,
) -> bool {
    return now.saturating_sub(made) > AHEAD_OLDEST.as_millis();
}

/// What:
///  The names of the snapshots ahead in `folder` that belong to `source_root`,
/// newest first.
/// Why:
///   The age sweep and the diagnostics ask the same question the claim asks,
/// so one function answers it for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = (folder, root) => candidates(folder, key(root));
/// ```
pub fn kept_in(
    folder: &Path,
    source_root: &Path,
) -> Vec<OsString> {
    let Ok(held) = open_root(folder) else {
        return Vec::new();
    };
    return candidates(&held, ahead_key(source_root).as_str());
}
