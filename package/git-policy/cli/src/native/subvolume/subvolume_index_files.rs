//! What:
//!  The linked worktrees of a main checkout,
//! found from `.git/worktrees/*/gitdir`,
//! and the stat rewrite applied to the
//!  index files of the checkout and of those worktrees on disk.
//! Why:
//!  The subvolume migration copies the main checkout and the linked worktrees inside it,
//! so each of their indexes needs
//!  the rewrite of `subvolume_index.rs`.
//! A file in `.git/worktrees/<name>/gitdir` holds the path of the worktree's
//!  `.git` file,
//! so the worktree is its parent directory (the same reading the migration script used).
//! An index is
//!  replaced through `index.lock` as Git replaces it,
//! so a running Git command is never raced:
//! the lock is taken
//!  before the index is read,
//! and when it exists the index is left as copied and the report says so.
//! The migration
//!  holds the locks itself while it completes the new checkout,
//! and its rewrite then runs under them.
//!
//! Why the lock stays:
//!  `index.lock` is Git's protocol for replacing an index;
//! the rewrite reads the index,
//! computes new stat fields and renames
//!  a file over it,
//! which is lost if a Git writer replaces the index in between,
//! and only Git's own lock excludes Git.
//! The
//!  rewrite takes it with `O_EXCL`,
//! never waits (a lock that exists skips the index and says so),
//! holds it for one index and
//!  only from the read to the rename,
//! and gives it back as Git does.
//! Concurrency among this wrapper's own commands never
//!  reaches this file:
//! `single_owner_claim.rs` gives one command a checkout or a worktree without a lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const { adminDirectory, worktree } of linkedWorktrees(root)) refreshIndexFile(join(adminDirectory, 'index'), worktree);
//! ```
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a caller writes `subvolume_index_files::refresh_index_file` as it did before:
//! `subvolume_index_file_stat_fields` reads and compares an entry's stat fields,
//! `subvolume_index_file_zeroed_stat` installs an index that vouches for no path,
//! `subvolume_index_file_options` holds what a refresh is told and how it ended,
//! and `subvolume_index_file_rewrite_and_install` rewrites one index and installs the result.

use crate::plain_file::read_plain_file;
pub use crate::subvolume_index_file_options::{
    IndexLock,
    RefreshOptions,
    RefreshReport,
    RefreshStatus,
    repository_hash_length,
};
use crate::subvolume_index_file_rewrite_and_install::{
    remove_if_same_file,
    rewrite_and_install,
};
pub(crate) use crate::subvolume_index_file_stat_fields::{
    fields_of,
    matches_stored,
    matches_stored_exactly,
};
pub use crate::subvolume_index_file_zeroed_stat::with_zeroed_stat;
pub use crate::subvolume_index_install_into_new_worktree::{
    InstallSeams,
    discard_index_under_a_name,
    install_index_in,
    install_index_in_with,
    publish_index_under_a_name,
    write_index_under_a_name,
};
use crate::subvolume_lock::is_ours;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The linked worktrees of a main checkout and the registrations that are not one of them.
/// Why:
///   They are read together from `.git/worktrees`;
/// see `subvolume_registered_worktrees.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './subvolume_registered_worktrees';
/// ```
pub use crate::subvolume_registered_worktrees::{
    LeftOutWorktree,
    LinkedWorktree,
    LinkedWorktreeScan,
    linked_worktrees_inside,
    scan_linked_worktrees,
};

/// The call the refresh makes at its seam when no control wants one:
/// none.
fn do_nothing() {}

/// The largest index file this reads.
///  A plain file is read to its end,
/// so the bound is what keeps a file that is not an index from being read whole into memory;
///  the largest index this repository holds is 3,097,973 bytes (measured 2026-11-09,
/// one of its linked worktrees' `.git/worktrees/<name>/index`),
/// and a gigabyte is that with room for a repository several hundred times larger.
///  The merge pass reads an index up to the same bound (`INDEX_LIMIT_BYTES`,
/// `merge_second_pass_worktree.rs`).
const INDEX_FILE_LIMIT: u64 = 1 << 30;

/// What:
///  Rewrite one index file in place and report what happened in one line.
/// Why:
///   A missing index is not an error (a worktree never checked out has none);
/// a held `index.lock` and every declined
///  shape are reported,
/// not hidden,
/// because the cost is a slow first `git status` there.
/// The lock comes first,
/// then
///  the read,
/// so a Git command that installs a newer index in between is never overwritten with an older one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFileWith(indexPath: string, tree: string, options: RefreshOptions): string;
/// ```
pub fn refresh_index_file_with(
    index_path: &Path,
    tree: &Path,
    options: &RefreshOptions<'_>,
) -> String {
    return refresh_index_file_seamed(index_path, tree, options, &mut do_nothing);
}

/// `refresh_index_file_with` with a call at the moment right after the rewrite,
/// before the lock is looked at again,
/// where a
///  control may let another command take the lock name.
pub fn refresh_index_file_seamed(
    index_path: &Path,
    tree: &Path,
    options: &RefreshOptions<'_>,
    after_rewrite: &mut dyn FnMut(),
) -> String {
    return refresh_index_file_report_seamed(index_path, tree, options, after_rewrite).line;
}

/// `refresh_index_file_with` that says how it ended,
/// so a caller that must not go on after a failed refresh can stop.
pub fn refresh_index_file_report(
    index_path: &Path,
    tree: &Path,
    options: &RefreshOptions<'_>,
) -> RefreshReport {
    return refresh_index_file_report_seamed(index_path, tree, options, &mut do_nothing);
}

/// `refresh_index_file_report` with a call at the moment right after the rewrite.
pub fn refresh_index_file_report_seamed(
    index_path: &Path,
    tree: &Path,
    options: &RefreshOptions<'_>,
    after_rewrite: &mut dyn FnMut(),
) -> RefreshReport {
    let mut lock_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
    lock_name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(lock_name);
    let mut created: Option<std::fs::File> = None;
    match &options.lock {
        IndexLock::Acquire => match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(opened) => created = Some(opened),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return RefreshReport {
                    line: String::from(
                        "skipped (index.lock exists: a Git command is running there; its first status will be slow)",
                    ),
                    status: RefreshStatus::Skipped,
                };
            }
            Err(error) => {
                return RefreshReport {
                    line: format!("skipped (cannot create the lock: {error})"),
                    status: RefreshStatus::Skipped,
                };
            }
        },
        IndexLock::Held(token) => {
            if !is_ours(lock_path.as_path(), token.as_ref()) {
                return RefreshReport {
                    line: String::from(
                        "skipped (index.lock is held by another Git command; its first status will be slow)",
                    ),
                    status: RefreshStatus::Skipped,
                };
            }
        }
    }
    let (outcome, status): (String, RefreshStatus) =
        rewrite_under_lock(index_path, tree, options, created.as_mut(), &lock_path);
    after_rewrite();
    // Once the lock was renamed over the index the name is any Git command's to take; it is never touched again. Before that,
    // the lock is removed only if the name still holds the file this call made.
    if let (Some(file), true) = (created.as_ref(), status != RefreshStatus::Refreshed) {
        remove_if_same_file(lock_path.as_path(), file);
    }
    return RefreshReport {
        line: outcome,
        status,
    };
}

/// The read,
/// rewrite and install of one index while its lock is held;
/// `lock_file` is the lock when this call made it.
/// The flag
///  says whether the new index was published,
/// after which the lock name is not this call's any more.
fn rewrite_under_lock(
    index_path: &Path,
    tree: &Path,
    options: &RefreshOptions<'_>,
    lock_file: Option<&mut std::fs::File>,
    lock_path: &Path,
) -> (String, RefreshStatus) {
    let index_mtime: Option<(u32, u32)> = match std::fs::metadata(index_path) {
        Ok(metadata) => Some((metadata.mtime() as u32, metadata.mtime_nsec() as u32)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (String::from("no index file"), RefreshStatus::NoIndex);
        }
        Err(error) => {
            return (
                format!("skipped (cannot examine it: {error})"),
                RefreshStatus::Skipped,
            );
        }
    };
    let bytes: Vec<u8> = match read_plain_file(index_path, INDEX_FILE_LIMIT) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (String::from("no index file"), RefreshStatus::NoIndex);
        }
        Err(error) => {
            return (
                format!("skipped (cannot read it: {error})"),
                RefreshStatus::Skipped,
            );
        }
    };
    return rewrite_and_install(
        bytes.as_slice(),
        index_mtime,
        [index_path, tree, lock_path],
        options,
        lock_file,
    );
}

/// What:
///  Rewrite one index file in place under a lock,
/// trusting every entry;
/// see `refresh_index_file_with`.
/// Why:
///   The shape the controls use when nothing else is known about the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFileLocked(indexPath: string, tree: string, lock: IndexLock): string;
/// ```
pub fn refresh_index_file_locked(
    index_path: &Path,
    tree: &Path,
    lock: IndexLock,
) -> String {
    return refresh_index_file_with(
        index_path,
        tree,
        &RefreshOptions {
            lock,
            references: &[],
            hash_length: None,
        },
    );
}

/// What:
///  Rewrite one index file in place under a lock the rewrite takes itself;
/// see `refresh_index_file_locked`.
/// Why:
///   The common case:
/// nothing holds the lock yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshIndexFile(indexPath: string, tree: string): string;
/// ```
pub fn refresh_index_file(
    index_path: &Path,
    tree: &Path,
) -> String {
    return refresh_index_file_locked(index_path, tree, IndexLock::Acquire);
}

/// Index file controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_files_tests.rs"]
mod tests;

/// The controls for indexes rewritten side by side stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_parallel_tests.rs"]
mod parallel_tests;
