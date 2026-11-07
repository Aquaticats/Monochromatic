//! What:
//!  Move every entry of a folder an earlier release wrote (`cli-git`) into the folder under the current name,
//! then remove the
//!  emptied earlier folder.
//! Where a folder exists in both,
//! its entries are moved the same way,
//! one level at a time.
//! Where a
//!  file or a link has the same name in both,
//! the earlier one is kept beside the current one with a fixed suffix.
//! Why:
//!  The first release under the new name found the earlier folder alone and renamed it in place;
//! when both exist the earlier
//!  folder still holds saved copies,
//! work records and notices that nothing reads,
//! and the owner said "never discard user data".
//!  Every step is one rename that never replaces,
//! so a kill at any instruction leaves each entry under exactly one of the two
//!  names,
//! and the next run moves what is left.
//! Nothing is copied and nothing is deleted but folders that are empty.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await moveEntries('.git/cli-git', '.git/monochromatic-git-policy-cli'); // then rmdir('.git/cli-git')
//! ```

use crate::earlier_log_lines_added_to_current_log::FoldSteps;
use crate::private_storage::sync_directory;
use crate::rename_that_never_replaces::rename_noreplace;
use crate::wrapper_names_old_and_new::{KEPT_BESIDE_SUFFIX, RenamedName};
use std::path::{Path, PathBuf};

/// How the merge of one folder ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FolderFold {
    /// The earlier folder did not exist.
    Nothing,
    /// Every entry moved and the earlier folder is gone.
    Folded {
        /// How many entries moved by a rename (a folder that exists in both is not counted,
        /// its entries are).
        moved: usize,
    },
    /// The earlier folder is still there because entries stayed in it;
    /// each is listed.
    Left {
        /// The entries that stayed,
        /// for a message to the person.
        kept: Vec<PathBuf>,
    },
    /// Every entry moved,
    /// but the emptied earlier folder could not be removed (a read-only Git directory);
    /// the reason is the
    ///  operating system's.
    NotRemoved {
        /// The operating system's error,
        /// as text.
        reason: String,
    },
    /// A control stopped the merge at the named step.
    Stopped(&'static str),
}

/// Whether a path is a real folder,
/// not a link to one.
fn is_real_folder(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).map(|found| return found.file_type().is_dir()).unwrap_or(false);
}

/// The name an earlier entry takes beside a current entry of the same name.
fn kept_beside_name(name: &std::ffi::OsStr) -> std::ffi::OsString {
    let mut kept: std::ffi::OsString = name.to_os_string();
    kept.push(KEPT_BESIDE_SUFFIX);
    return kept;
}

/// What:
///  Merge the earlier folder of `name` in `parent` into the current one.
/// Why:
///   A work stack of pairs of folders walks the earlier folder and the folders that exist in both,
/// so the depth is bounded by
///  the stack,
/// not the call stack.
/// Folders that were emptied are removed last,
/// deepest first;
/// a folder that is not empty (an
///  earlier program created an entry in it meanwhile,
/// or an entry kept its place) stays,
/// and the next run moves the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function foldFolder(parent: string, name: RenamedName, steps: FoldSteps): FolderFold;
/// ```
///
/// # Errors
/// A failure of the file system (a read-only Git directory) is returned at the first entry that cannot move;
/// the entries already moved stay moved and the rest stay in the earlier folder.
pub fn fold_folder(parent: &Path, name: RenamedName, steps: &mut dyn FoldSteps) -> std::io::Result<FolderFold> {
    let earlier: PathBuf = parent.join(name.earlier);
    let current: PathBuf = parent.join(name.current);
    if !is_real_folder(earlier.as_path()) {
        // A file or a link under the earlier name is not a folder this fold knows how to merge.
        return match std::fs::symlink_metadata(earlier.as_path()) {
            Ok(_) => Ok(FolderFold::Left { kept: vec![earlier] }),
            Err(_) => Ok(FolderFold::Nothing),
        };
    }
    if !is_real_folder(current.as_path()) {
        return Ok(FolderFold::Left { kept: vec![current] });
    }
    let mut moved: usize = 0;
    let mut kept: Vec<PathBuf> = Vec::new();
    let mut visited: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut pending: Vec<(PathBuf, PathBuf)> = vec![(earlier.clone(), current.clone())];
    while let Some((from_folder, into_folder)) = pending.pop() {
        visited.push((from_folder.clone(), into_folder.clone()));
        let listing = match std::fs::read_dir(from_folder.as_path()) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        // The names are read before any is moved, so a move never changes what the listing returns.
        let names: Vec<std::ffi::OsString> = listing.map(|entry| return entry.map(|found| return found.file_name())).collect::<Result<_, _>>()?;
        for entry_name in names {
            let source: PathBuf = from_folder.join(entry_name.as_os_str());
            let target: PathBuf = into_folder.join(entry_name.as_os_str());
            if !steps.may_continue("before-move-entry") {
                return Ok(FolderFold::Stopped("before-move-entry"));
            }
            match rename_noreplace(source.as_path(), target.as_path()) {
                Ok(()) => moved += 1,
                // Another process moved it first.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if is_real_folder(source.as_path()) && is_real_folder(target.as_path()) {
                        pending.push((source, target));
                        continue;
                    }
                    let beside: PathBuf = into_folder.join(kept_beside_name(entry_name.as_os_str()));
                    match rename_noreplace(source.as_path(), beside.as_path()) {
                        Ok(()) => moved += 1,
                        Err(second) if second.kind() == std::io::ErrorKind::NotFound => {}
                        Err(second) if second.kind() == std::io::ErrorKind::AlreadyExists => kept.push(source),
                        Err(second) => return Err(second),
                    }
                }
                Err(error) => return Err(error),
            }
        }
    }
    let mut top_removal_error: Option<std::io::Error> = None;
    for (from_folder, into_folder) in visited.iter().rev() {
        if !steps.may_continue("before-remove-emptied-folder") {
            return Ok(FolderFold::Stopped("before-remove-emptied-folder"));
        }
        // Only an empty folder is removed; one that still holds an entry stays, and so does every folder above it.
        let removal: std::io::Result<()> = std::fs::remove_dir(from_folder.as_path());
        if let Err(error) = removal
            && *from_folder == earlier
        {
            top_removal_error = Some(error);
        }
        let _durable = sync_directory(into_folder.as_path());
    }
    let _durable = sync_directory(parent);
    if is_real_folder(earlier.as_path()) {
        if !kept.is_empty() {
            return Ok(FolderFold::Left { kept });
        }
        return match top_removal_error {
            // Not empty: entries that an earlier program created during the merge stay for the next run.
            Some(error) if error.kind() != std::io::ErrorKind::DirectoryNotEmpty => Ok(FolderFold::NotRemoved { reason: error.to_string() }),
            _ => Ok(FolderFold::Left { kept: vec![earlier] }),
        };
    }
    return Ok(FolderFold::Folded { moved });
}

#[cfg(test)]
#[path = "earlier_folder_entries_moved_into_current_folder_tests.rs"]
mod tests;
