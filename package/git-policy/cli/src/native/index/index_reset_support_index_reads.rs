//! What:
//!  What a control reads of a scene after a reset:
//!  the index file's bytes,
//!  one path's stat data,
//!  the entry the index holds for a path,
//!  the listing `git ls-files --stage --debug` prints,
//!  and the byte lists the follow-up passes.
//! Why:
//!  The differential controls compare bytes and entries rather than a
//!  summary,
//!  so each of these is read whole and exactly as the follow-up reads it,
//!  and a failure carries the one thing the two arms disagree on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = indexBytes(repo);
//! const stat = statOfFile(path);
//! const entry = entryOf(repo, 'a.txt');
//! const listing = listingOf(repo);
//! ```

/// Real Git run in the repository,
/// for the listing a failure prints.
use super::git_starts::git_with;
/// The repository and the paths it is read for.
use std::path::Path;
/// The bytes of the repository's index file.
pub(crate) fn index_bytes(repo: &Path) -> Vec<u8> {
    return std::fs::read(repo.join(".git").join("index")).expect("index");
}

/// What:
///  The stat data an index entry records for the file at `path`,
/// each field cut to the 32 bits an entry stores.
/// Why:
///   A control that says the landing recorded the worktree's stat data compares the entry with this,
/// and a comparison
///  of the full 64-bit fields would never be equal.
pub(crate) fn stat_of_file(path: &Path) -> crate::index_stat::StatData {
    use std::os::unix::fs::MetadataExt as _;
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).expect("the file");
    return crate::index_stat::StatData {
        ctime_sec: metadata.ctime() as u32,
        ctime_nsec: metadata.ctime_nsec() as u32,
        mtime_sec: metadata.mtime() as u32,
        mtime_nsec: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
}

/// What:
///  The entry the index file of `repo` holds for `path`.
/// Why:
///   A control that asserts what the landing recorded for one path reads the one entry,
/// with everything the entry carries.
pub(crate) fn entry_of(
    repo: &Path,
    path: &str,
) -> crate::index_full::FullEntry {
    let bytes: Vec<u8> = index_bytes(repo);
    let index: crate::index_full::FullIndex =
        crate::index_full::parse_full_index(bytes.as_slice(), 20).expect("parsed index");
    return index
        .entries
        .into_iter()
        .find(|entry| {
            return entry.path == path.as_bytes();
        })
        .expect("the entry the path names");
}

/// What `git ls-files --stage --debug` says,
/// for a failure message.
pub(crate) fn listing(repo: &Path) -> String {
    return String::from_utf8_lossy(
        &git_with(repo, &[], &["ls-files", "--stage", "--debug"], None).stdout,
    )
    .into_owned();
}

/// The paths as the byte lists the follow-up passes.
pub(crate) fn named(paths: &[&str]) -> Vec<Vec<u8>> {
    return paths
        .iter()
        .map(|path| {
            return path.as_bytes().to_vec();
        })
        .collect();
}
