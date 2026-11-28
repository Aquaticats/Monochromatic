//! What:
//!  The fill's index:
//!  the captured index written under a private name in the
//!  new worktree's administrative directory,
//!  and published under the worktree's
//!  own name only once the tree is proven.
//! Why:
//!  A published index whose entry's clean form equals a blob makes Git's first
//!  status trust the file instead of reading it,
//!  so a path still holding the
//!  source's bytes must never be vouched for
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore",
//!  steps 5 and 8;
//!  issue 6323).
//!  Step 10's clean source has no path the fence writes,
//!  so the private index is
//!  the captured index with the extensions Git derives dropped and its checksum
//!  remade,
//!  and nothing in it is written through Git.
//!  Step 11 writes the fence's
//!  paths and the dirty half's reset through this same file.
//!  The name is `index.pending`,
//!  which no Git run reaches unless it is pointed
//!  there,
//!  so a kill leaves no index at all and the resume knows which write it
//!  reached.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pending = writeThePrivateIndex(admin, indexBytes, hashLength);
//! publishThePrivateIndex(admin); // one rename, replacing nothing
//! ```

use crate::fs_beneath::{
    create_file_at,
    rename_at,
};
use crate::subvolume_index::{
    IndexOutcome,
    RewriteContext,
    StatFields,
    StatLookup,
    StatTrust,
    StoredStat,
    rewrite_index_stat_with,
};
use std::ffi::OsStr;
use std::io::Write as _;
use std::path::Path;

/// The private name the fill's index waits under.
pub const PENDING_INDEX_NAME: &str = "index.pending";

/// The worktree's own index name.
pub const INDEX_NAME: &str = "index";

/// The lookup that finds no file,
/// so a rewrite keeps every entry's stored stat fields.
struct NoStats;

impl StatLookup for NoStats {
    fn stat_now(
        &mut self,
        _path: &[u8],
    ) -> Option<StatFields> {
        return None;
    }
}

/// The trust that trusts nothing,
/// so no entry's fields are replaced on a claim about a file.
struct NoTrust;

impl StatTrust for NoTrust {
    fn is_trusted(
        &mut self,
        _path: &[u8],
        _stored: &StoredStat,
    ) -> bool {
        return false;
    }
}

/// What:
///  The captured index as the fill's own bytes:
///  the same entries and stat fields,
///  the derived extensions dropped and the
///  checksum remade.
/// Why:
///  The seed is the index D1 compared,
///  never a second read of the copied file,
///  and the design drops the untracked-cache and fsmonitor extensions before the
///  private write.
///
/// # Errors
/// The index is not one this reader knows,
/// or it declares an extension that changes what its entries mean.
pub fn index_for_the_fill(
    seed: &[u8],
    hash_length: usize,
) -> Result<Vec<u8>, String> {
    let context: RewriteContext = RewriteContext {
        hash_length: Some(hash_length),
        index_mtime: None,
    };
    return match rewrite_index_stat_with(seed, &context, &mut NoStats, &mut NoTrust) {
        IndexOutcome::Rewritten {
            bytes,
            ..
        } => Ok(bytes),
        IndexOutcome::Skipped(reason) => Err(format!(
            "the captured index cannot be written for the fill ({reason}); Ways forward: run the add again with Git's own checkout"
        )),
    };
}

/// What:
///  Write the fill's index under its private name in the registration.
/// Why:
///  The file is made exclusively and flushed,
///  so a kill leaves either no file or
///  the whole of it,
///  and nothing at the worktree's own name.
///
/// # Errors
/// The private name already stands,
/// or the file cannot be written or flushed.
pub fn write_the_private_index(
    admin: &std::fs::File,
    bytes: &[u8],
) -> Result<(), String> {
    let mut file: std::fs::File = create_file_at(admin, OsStr::new(PENDING_INDEX_NAME), 0o600)
        .map_err(|error| {
            return format!("the fill's index {PENDING_INDEX_NAME} cannot be made ({error})");
        })?;
    file.write_all(bytes).map_err(|error| {
        return format!("the fill's index {PENDING_INDEX_NAME} cannot be written ({error})");
    })?;
    file.sync_all().map_err(|error| {
        return format!("the fill's index {PENDING_INDEX_NAME} cannot be flushed ({error})");
    })?;
    return Ok(());
}

/// What:
///  Give the private index the worktree's own name,
///  replacing nothing.
/// Why:
///  One rename makes the worktree's index whole in one instant,
///  and a name that
///  already stands is another command's file and never replaced.
///
/// # Errors
/// The private name does not stand,
/// the worktree's name already stands,
/// or the rename was refused.
pub fn publish_the_private_index(
    admin: &std::fs::File,
    admin_path: &Path,
) -> Result<(), String> {
    if !admin_path.join(PENDING_INDEX_NAME).is_file() {
        return Err(String::from(
            "the fill's index is not under its private name, so there is nothing to publish",
        ));
    }
    rename_at(admin, OsStr::new(PENDING_INDEX_NAME), admin, OsStr::new(INDEX_NAME), false).map_err(|error| {
        return format!("the fill's index cannot take the worktree's own name ({error}); Ways forward: remove the file that stands at {} and run the add again", admin_path.join(INDEX_NAME).display());
    })?;
    admin.sync_all().map_err(|error| {
        return format!(
            "the registration {} cannot be flushed after the index took its name ({error})",
            admin_path.display()
        );
    })?;
    return Ok(());
}

/// Whether the fill's index waits under its private name.
pub fn pending_stands(admin_path: &Path) -> bool {
    return admin_path.join(PENDING_INDEX_NAME).is_file();
}

/// Whether the worktree's own index stands.
pub fn published_stands(admin_path: &Path) -> bool {
    return admin_path.join(INDEX_NAME).is_file();
}

/// What:
///  Whether a resume reads the index's state as "the rename happened".
/// Why:
///  Nothing else removes the private name,
///  so its absence with the real name
///  present is the one state in which the publication ran.
pub fn the_publication_ran(admin_path: &Path) -> bool {
    return !pending_stands(admin_path) && published_stands(admin_path);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_index_tests.rs"]
mod tests;
