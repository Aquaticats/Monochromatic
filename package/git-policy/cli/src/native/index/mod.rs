//! What:
//!  In-process reading and writing of the Git index file and its extensions.
//! Why:
//!  The index format is one specification with many parts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index/index.ts: one `export * from './file'` per module below.
//! ```

/// Byte searches the index readers share:
/// first position of a byte,
/// a leading run,
/// any non-zero byte.
pub mod index_byte_search;

/// In-process index update:
/// stat fields and Git's varint.
pub mod index_stat;

/// A read of the assume-unchanged,
/// skip-worktree and intent-to-add marks of an index file,
/// and of the paths the intent-to-add entries name.
///  Declared after `index_stat`,
/// whose varint a version 4 entry's strip count is read with.
pub mod index_flags;

/// In-process index update:
/// Git's EWAH bitmap,
/// for the untracked-cache and fsmonitor extensions.
pub mod index_ewah;

/// In-process index update:
/// the cache-tree extension.
pub mod index_cache_tree;

/// In-process index update:
/// the resolve-undo extension.
pub mod index_resolve_undo;

/// In-process index update:
/// the untracked-cache extension as directories,
/// and its invalidation.
pub mod index_untracked;

/// In-process index update:
/// reading and writing the untracked-cache extension.
pub mod index_untracked_io;

/// In-process index update:
/// the fsmonitor extension.
pub mod index_fsmonitor;

/// In-process index update:
/// a whole index file read into entries and kept extensions.
pub mod index_full;

/// In-process index update:
/// a whole index file written as Git writes it.
pub mod index_full_write;

/// In-process index update:
/// the configuration that decides the extensions and the decline cases.
#[cfg(unix)]
pub mod index_settings;

/// In-process index update:
/// a path-limited mixed reset of an index held in memory.
#[cfg(unix)]
pub mod index_reset;

/// In-process index update:
/// the names Git refuses under its protection settings,
/// which the reset leaves to Git.
#[cfg(unix)]
pub mod index_reset_path_names;

/// In-process index update:
/// the choice of where a new entry's stat data comes from,
/// and the trait a proof implements.
#[cfg(unix)]
pub mod index_reset_worktree_stat_source;

/// In-process index update:
/// the worktree's stat data for a path the reset writes,
/// recorded only where the file provably holds the commit's bytes.
#[cfg(target_os = "linux")]
pub mod index_reset_worktree_stat;

/// In-process index update:
/// the trees a path-limited reset reads.
#[cfg(unix)]
pub mod index_reset_tree;

/// In-process index update:
/// the lock proof,
/// the temporary file and the file mode of the landing's private index.
#[cfg(target_os = "linux")]
pub mod index_reset_publish;

/// In-process index update:
/// the landing's follow-up on an index file.
#[cfg(target_os = "linux")]
pub mod index_reset_file;

/// Index files real Git wrote,
/// for the controls of the readers and writers;
/// never part of the release executable.
#[cfg(test)]
#[cfg(target_os = "linux")]
pub(crate) mod index_reset_support_probe;
