//! What:
//!  An index file with the stat data of the paths given set to zero,
//!  and its checksum made good again.
//! Why:
//!  Git skips a file only when its entry's stat data describes the file it finds,
//!  so an index that must not vouch for a path until Git has written it is installed with that path's stat data
//!  zeroed:
//!  no real file has a zero size,
//!  modification time,
//!  change time or inode,
//!  so the first `git status` hashes the file instead of trusting it.
//!  The entries' object names and modes are untouched,
//!  so Git writes each path from its own blob.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const seed = withZeroedStat(bytes, paths, hashLength); // undefined when the layout is not known
//! ```

use crate::subvolume_index::{
    HEADER,
    Located,
    STAT,
    checksum_kind,
    walk_entries,
};
use crate::subvolume_sha::{
    sha1,
    sha256,
};
use std::collections::HashSet;

/// The offset of an entry's mode inside its fixed stat fields.
/// Why:
///  it is the one field of the forty that is not stat data,
///  so zeroing an entry's stat data step keeps the mode.
const MODE_AT: usize = 24;

/// What:
///  `bytes` with the stat data of every entry whose path is in `paths` set to zero,
///  and the checksum made good,
///  or nothing when the file is not an index this reader knows.
/// Why:
///  Git skips a file only when its entry's stat data describes the file it finds
///  (`read-cache.c`,
///  `ie_match_stat`),
///  so an index that must not vouch for a path until Git has written it is installed with that path's
///  stat data zeroed:
///  no real file has a zero size,
///  modification time,
///  change time or inode,
///  so the first `git status` hashes the file instead of trusting it.
/// The entries' object names and modes are untouched,
///  so Git writes each path from its own blob.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seed = withZeroedStat(bytes, paths, hashLength) ?? throw new Error('unknown index layout');
/// ```
pub fn with_zeroed_stat(
    bytes: &[u8],
    paths: &HashSet<Vec<u8>>,
    hash_length: usize,
) -> Option<Vec<u8>> {
    if bytes.len() < HEADER + hash_length {
        return None;
    }
    let (length, zero_checksum): (usize, bool) = checksum_kind(bytes, Some(hash_length)).ok()?;
    let (entries, _end): (Vec<Located>, usize) = walk_entries(bytes, length)?;
    let limit: usize = bytes.len().checked_sub(length)?;
    let mut output: Vec<u8> = bytes[..limit].to_vec();
    for entry in &entries {
        if !paths.contains(entry.path.as_slice()) {
            continue;
        }
        let mode_end: usize = entry.start.checked_add(MODE_AT)?;
        let stat_end: usize = entry.start.checked_add(STAT)?;
        if stat_end > limit {
            return None;
        }
        output[entry.start..mode_end].fill(0);
        output[mode_end + 4..stat_end].fill(0);
    }
    if zero_checksum {
        output.extend(std::iter::repeat_n(0u8, length));
    } else if length == 20 {
        let digest: [u8; 20] = sha1(output.as_slice());
        output.extend_from_slice(&digest);
    } else {
        let digest: [u8; 32] = sha256(output.as_slice());
        output.extend_from_slice(&digest);
    }
    return Some(output);
}
