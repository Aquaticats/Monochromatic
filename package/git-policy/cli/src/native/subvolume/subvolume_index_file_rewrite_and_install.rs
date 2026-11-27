//! What:
//!  The rewrite of one index file's stat fields and the install that puts the result in place:
//!  the lookup of what each entry's file reports now,
//!  the check that the entry still describes the file the copy was made from,
//!  and the write-and-rename that replaces the index,
//!  or a staging file beside it,
//!  only while the name still holds the file this call wrote.
//! Why:
//!  Replacing the change time and inode of an entry Git could not trust,
//!  or of a file that changed after Git looked,
//!  would make Git believe the stored blob is current,
//!  so the check comes before the write.
//!  The install writes through the descriptor it made and renames only once the name is proved to hold that file,
//!  so a link or another process's file put at the staging name is never written through or published.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const installed = rewriteAndInstall(bytes, indexMtime, [indexPath, tree, lockPath], options, lockFile);
//! ```

use crate::subvolume_file_write_steps::write_all_then_sync;
use crate::subvolume_index::{
    IndexOutcome,
    RewriteContext,
    StatFields,
    StatLookup,
    StatTrust,
    StoredStat,
    rewrite_index_stat_with,
};
use crate::subvolume_index_file_options::{
    RefreshOptions,
    RefreshStatus,
};
use crate::subvolume_index_file_stat_fields::{
    matches_stored_exactly,
    stat_fields,
};
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// Remove the file at `path` if it is the very file `held` is open on (same device and inode).
pub(crate) fn remove_if_same_file(
    path: &Path,
    held: &std::fs::File,
) {
    let same: bool = match (std::fs::symlink_metadata(path), held.metadata()) {
        (Ok(named), Ok(own)) => {
            named.dev() == own.dev() && named.ino() == own.ino() && named.file_type().is_file()
        }
        _ => false,
    };
    if same {
        let _removed = std::fs::remove_file(path);
    }
}

/// What:
///  The stat fields of a path as the tree being rewritten reports them now.
/// Why:
///   The rewrite asks for each entry's path;
/// the tree it describes is the only state the answer needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StatOfTree implements StatLookup { constructor(readonly tree: string) {} statNow(path) { return statFields(this.tree, path); } }
/// ```
struct StatOfTree<'a> {
    /// The tree the index describes.
    tree: &'a Path,
}

impl StatLookup for StatOfTree<'_> {
    fn stat_now(
        &mut self,
        path: &[u8],
    ) -> Option<StatFields> {
        return stat_fields(self.tree, path);
    }
}

/// What:
///  Whether an entry still agrees with the trees the checkout was copied from.
/// Why:
///   The rewrite trusts an entry's stored fields only for a file that is still the one the copy was made from,
/// which an
///  entry that recorded no nanoseconds cannot prove;
/// such an entry keeps its fields and Git hashes its file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AgreeWithReferences implements StatTrust { isTrusted(path, stored) { return agreesWithAReference(this.references, path, stored); } }
/// ```
struct AgreeWithReferences<'a> {
    /// The trees the checkout was copied from,
    /// one per reference.
    references: &'a [PathBuf],
}

impl StatTrust for AgreeWithReferences<'_> {
    fn is_trusted(
        &mut self,
        path: &[u8],
        stored: &StoredStat,
    ) -> bool {
        return agrees_with_a_reference(self.references, path, stored);
    }
}

/// The rewrite of the stat fields of `bytes` and the install of the result over `index_path`,
/// with `paths` naming the index,
/// the tree it
///  describes and the lock file.
pub(crate) fn rewrite_and_install(
    bytes: &[u8],
    index_mtime: Option<(u32, u32)>,
    paths: [&Path; 3],
    options: &RefreshOptions<'_>,
    lock_file: Option<&mut std::fs::File>,
) -> (String, RefreshStatus) {
    let [index_path, tree, lock_path]: [&Path; 3] = paths;
    let context: RewriteContext = RewriteContext {
        hash_length: options.hash_length,
        index_mtime,
    };
    let outcome: IndexOutcome = rewrite_index_stat_with(
        bytes,
        &context,
        &mut StatOfTree {
            tree,
        },
        &mut AgreeWithReferences {
            references: options.references,
        },
    );
    let (rewritten, updated, total): (Vec<u8>, usize, usize) = match outcome {
        IndexOutcome::Rewritten {
            bytes: new_bytes,
            updated,
            total,
        } => (new_bytes, updated, total),
        IndexOutcome::Skipped(reason) => {
            return (format!("skipped ({reason})"), RefreshStatus::Skipped);
        }
    };
    // With its own lock the new index is written into the lock file and the lock is renamed over the index, as Git does.
    // Under a held lock the lock stays, and a sibling file is renamed over the index instead.
    let installed: std::io::Result<()> = match lock_file {
        Some(file) => write_sync_rename(file, rewritten.as_slice(), lock_path, index_path),
        None => stage_and_replace(index_path, rewritten.as_slice()),
    };
    if let Err(error) = installed {
        return (
            format!("skipped (cannot install the new index: {error})"),
            RefreshStatus::Skipped,
        );
    }
    return (
        format!("{updated} of {total} entries refreshed"),
        RefreshStatus::Refreshed,
    );
}

/// Whether some reference tree has the file `path` as the entry `stored` records;
/// with no reference tree,
/// every entry agrees.
///
/// A reference tree proves which bytes the copy was made from only where the entry recorded its times to the
///  nanosecond:
/// an entry that recorded none keeps its own fields,
/// which no longer match the copy,
/// so Git hashes the file
///  instead of trusting fields nothing proved.
fn agrees_with_a_reference(
    references: &[PathBuf],
    path: &[u8],
    stored: &StoredStat,
) -> bool {
    if references.is_empty() {
        return true;
    }
    for reference in references {
        if let Some(now) = stat_fields(reference.as_path(), path)
            && matches_stored_exactly(&now, stored)
        {
            return true;
        }
    }
    return false;
}

/// Write `bytes` into `file`,
/// flush it to disk,
/// then rename `from` over `to`;
/// each step runs only if the one before it worked.
fn write_sync_rename(
    file: &mut std::fs::File,
    bytes: &[u8],
    from: &Path,
    to: &Path,
) -> std::io::Result<()> {
    write_all_then_sync(file, bytes)?;
    return std::fs::rename(from, to);
}

/// What:
///  Write `bytes` to a staging file beside `index_path`,
/// created exclusively,
/// and rename it over the index.
/// Why:
///   Under a held lock no Git command writes the index,
/// but the staging name is still checked:
/// it is created with
///  `O_EXCL`,
/// which fails on an existing file or link instead of following or truncating it,
/// written and synced through its
///  own descriptor,
/// and renamed only while the name still holds that file.
/// A staging file that cannot be proven the call's
///  is left where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stageAndReplace(indexPath, bytes);
/// ```
fn stage_and_replace(
    index_path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mut sibling_name: std::ffi::OsString = index_path.as_os_str().to_os_string();
    sibling_name.push(".cli-git-new");
    let sibling: PathBuf = PathBuf::from(sibling_name);
    let mut staged: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&sibling)?;
    let written: std::io::Result<()> =
        write_sync_and_rename_if_same(&mut staged, bytes, sibling.as_path(), index_path);
    if written.is_err() {
        remove_if_same_file(sibling.as_path(), &staged);
    }
    return written;
}

/// Write `bytes` into `staged`,
/// flush it,
/// and rename `sibling` over `index_path` only while `sibling` still names `staged`.
fn write_sync_and_rename_if_same(
    staged: &mut std::fs::File,
    bytes: &[u8],
    sibling: &Path,
    index_path: &Path,
) -> std::io::Result<()> {
    write_all_then_sync(staged, bytes)?;
    let named = std::fs::symlink_metadata(sibling)?;
    let own = staged.metadata()?;
    if named.dev() != own.dev() || named.ino() != own.ino() {
        return Err(std::io::Error::other(
            "the staging file was replaced while the index was written",
        ));
    }
    return std::fs::rename(sibling, index_path);
}
