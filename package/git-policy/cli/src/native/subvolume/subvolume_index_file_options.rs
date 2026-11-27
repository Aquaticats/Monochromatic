//! What:
//!  What a rewrite of one index file is told beyond its paths,
//!  how the rewrite ended,
//!  and the repository's hash length that goes into those options.
//! Why:
//!  The lock decides how the index is protected from a Git command that writes it,
//!  the reference trees decide which entries Git recorded for the files the copy was made from,
//!  and the hash length decides how an index with an all-zero checksum is read.
//!  A caller that must not go on after a failed refresh reads the status rather than the sentence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const options: RefreshOptions = { lock: 'acquire', references: [], hashLength: repositoryHashLength(gitDir) };
//! ```

use crate::plain_file::{
    SMALL_FILE_LIMIT,
    read_plain_file,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  How the rewrite of one index file is protected from a Git command that writes the same index.
///  `Arc<str>` shares the token of the migration whose lock is already held,
///  because a rewrite that runs as a job of the blocking pool must own what it reads.
/// Why:
///   Git replaces an index through `index.lock`.
/// A rewrite that runs by itself takes that lock,
/// as Git does,
/// before it
///  reads the index and holds it until the new index is in place.
/// The migration already holds the lock of every index
///  while it completes the new checkout,
/// so its rewrite uses the held lock instead of creating one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLock = { kind: 'acquire' } | { kind: 'held'; token: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexLock {
    /// Create `index.lock` for the duration of the rewrite and remove it after.
    Acquire,
    /// `index.lock` holds this token already;
    /// leave it in place for its owner to release.
    Held(std::sync::Arc<str>),
}

/// What:
///  What a rewrite of one index file is told beyond the paths.
///  `&'a [PathBuf]` borrows the list of trees.
/// Why:
///   The lock decides how the index is protected,
/// the references decide which entries Git recorded for the files the
///  copy was made from,
/// and the hash length (read from the repository's configuration) decides how an index with an
///  all-zero checksum is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefreshOptions = { lock: IndexLock; references: string[]; hashLength?: number };
/// ```
#[derive(Clone, Debug)]
pub struct RefreshOptions<'a> {
    /// How the index is locked while it is rewritten.
    pub lock: IndexLock,
    /// Trees whose files the index may describe (the tree the copy was made from).
    /// An entry is refreshed only if the
    ///  file in one of them still has the stat fields the entry records;
    /// empty means every entry may be refreshed.
    pub references: &'a [PathBuf],
    /// The repository's hash length in bytes,
    /// when known.
    pub hash_length: Option<usize>,
}

/// How a refresh ended,
/// apart from the words that describe it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshStatus {
    /// The new index was installed.
    Refreshed,
    /// There is no index file,
    /// which is nothing to refresh and no failure.
    NoIndex,
    /// The index was left as it was,
    /// for the reason in the line.
    Skipped,
}

/// A refresh's line for the person and its status for the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshReport {
    /// The one-line report.
    pub line: String,
    /// How it ended.
    pub status: RefreshStatus,
}

/// What:
///  The repository's hash length in bytes from its configuration file:
/// 32 when `extensions.objectformat` is `sha256`,
///  20 when the file has no such setting,
/// nothing when the file cannot be read.
/// Why:
///   An index does not say which hash it uses;
/// the configuration does.
/// The reading is the plain `[section]` and
///  `key = value` grammar,
/// case-insensitive for section and key names,
/// without includes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryHashLength(gitDirectory: string): 20 | 32 | undefined;
/// ```
pub fn repository_hash_length(git_directory: &Path) -> Option<usize> {
    let text: String = String::from_utf8_lossy(
        read_plain_file(&git_directory.join("config"), SMALL_FILE_LIMIT)
            .ok()?
            .as_slice(),
    )
    .into_owned();
    let mut in_extensions: bool = false;
    for raw in text.lines() {
        let line: &str = raw.trim();
        if let Some(header) = line.strip_prefix('[') {
            in_extensions = match header.split(']').next() {
                Some(name) => name.trim().eq_ignore_ascii_case("extensions"),
                None => false,
            };
            continue;
        }
        if !in_extensions {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("objectformat")
            && value.trim().eq_ignore_ascii_case("sha256")
        {
            return Some(32);
        }
    }
    return Some(20);
}
