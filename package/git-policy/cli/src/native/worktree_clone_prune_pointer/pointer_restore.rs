//! What:
//!  The one write that puts the recorded pointer back at a slot,
//!  whole and by one rename,
//!  and the answers that say why it did not write.
//! Why:
//!  The design keeps the underlay pointer for `git worktree prune` and records
//!  the exact bytes Git wrote,
//!  so a pointer an account deleted or changed can be restored from the
//!  record rather than guessed
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The write never follows a link and never replaces a directory:
//!  a directory at `.git` is the copied Git administration,
//!  which is a different object,
//!  and a link there is not a pointer this route wrote.
//!  A staging name holds the bytes until the rename,
//!  so the slot keeps a `.git` at every instant,
//!  exactly as the add's exchange leaves it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = restoreTheSlotPointer(slot, site, record.pointer); // { kind: 'written', bytes }
//! ```

use crate::fs_beneath::{
    entry_metadata_at,
    open_root,
};
use crate::publishing_entries_in_one_open_folder::{
    create_new_entry,
    fill_entry_and_sync,
    remove_entry,
    rename_entry_replacing,
    sync_folder,
};
use crate::worktree_clone_prune_pointer::pointer_state::PointerSite;
use std::ffi::OsStr;
use std::path::Path;

/// Git's name for the file that names a worktree's registration.
pub const SLOT_POINTER_NAME: &str = ".git";

/// The mode Git `2.56.1` writes a worktree pointer with,
/// measured on this host with `git worktree add --no-checkout`
/// (`-rw-r--r--`).
pub const POINTER_FILE_MODE: u32 = 0o644;

/// The bytes of the staging name's random part.
const STAGING_RANDOM_BYTES: usize = 16;

/// What:
///  What one restore did,
///  or why it wrote nothing.
/// Why:
///  A caller prints one line and acts on the answer,
///  and a refusal is not a failure of the command:
///  nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RestoreOutcome = { kind: 'written'; bytes } | { kind: 'already-intact' } | { kind: 'nothing-to-write'; why } | { kind: 'refused'; reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestoreOutcome {
    /// The recorded bytes were written,
    /// and the slot holds a pointer again.
    Written {
        /// How many bytes were written.
        bytes: usize,
    },
    /// The slot already holds the recorded bytes,
    /// so nothing was written.
    AlreadyIntact,
    /// The record holds no bytes to write,
    /// so nothing was written.
    NothingToWrite {
        /// Why there was nothing to write.
        why: String,
    },
    /// The write refused and changed nothing.
    Refused {
        /// What refused.
        reason: String,
    },
}

impl RestoreOutcome {
    /// The sentence a person reads.
    pub fn line(
        &self,
        slot_path: &Path,
        site: &PointerSite,
    ) -> String {
        let shown: String = slot_path.display().to_string();
        return match self {
            RestoreOutcome::Written {
                bytes,
            } => format!(
                "restored the pointer at `{shown}` from the bytes the add recorded ({bytes} bytes), so Git reads the registration again"
            ),
            RestoreOutcome::AlreadyIntact => format!(
                "the pointer at `{shown}` is the copy the add recorded, so nothing was written"
            ),
            RestoreOutcome::NothingToWrite {
                why,
            } => format!("the pointer at `{shown}` was not restored: {why}"),
            RestoreOutcome::Refused {
                reason,
            } => format!(
                "the pointer at `{shown}` was not restored: {reason} {}",
                site.sentence()
            ),
        };
    }
}

/// The staging name this write fills before it takes `.git`'s name.
fn staging_name() -> std::io::Result<String> {
    let mut random: [u8; STAGING_RANDOM_BYTES] = [0; STAGING_RANDOM_BYTES];
    getrandom::fill(&mut random).map_err(|error| {
        return std::io::Error::other(format!("the random source failed ({error})"));
    })?;
    let digits: String = random
        .iter()
        .map(|byte| return format!("{byte:02x}"))
        .collect();
    return Ok(format!("{SLOT_POINTER_NAME}.pointer-restore.{digits}"));
}

/// What:
///  Put the recorded pointer bytes back at one slot,
///  when the slot shows a copy this route wrote.
/// Why:
///  The record's bytes are the pointer Git was given,
///  so writing them restores the link a prune and a repair read,
///  and the write refuses anything it cannot attribute:
///  a directory,
///  a link,
///  or a mount this route did not put there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = restoreTheSlotPointer(slotPath, site, recordedBytes);
/// ```
pub fn restore_the_slot_pointer(
    slot_path: &Path,
    site: &PointerSite,
    recorded: Option<&[u8]>,
) -> RestoreOutcome {
    match site {
        PointerSite::Underlay | PointerSite::AtTheCloneRoot => {}
        PointerSite::AnotherMount {
            ..
        }
        | PointerSite::Unprovable {
            ..
        } => {
            return RestoreOutcome::Refused {
                reason: String::from(
                    "the slot does not show a copy of the pointer this route wrote",
                ),
            };
        }
    }
    let Some(bytes) = recorded else {
        return RestoreOutcome::NothingToWrite {
            why: String::from("the record holds no pointer bytes to restore"),
        };
    };
    let folder: std::fs::File = match open_root(slot_path) {
        Ok(opened) => opened,
        Err(error) => {
            return RestoreOutcome::Refused {
                reason: format!(
                    "the slot folder cannot be opened without following a link ({error})"
                ),
            };
        }
    };
    let pointer_name: &OsStr = OsStr::new(SLOT_POINTER_NAME);
    match entry_metadata_at(&folder, pointer_name) {
        Ok(metadata) => {
            let kind: std::fs::FileType = metadata.file_type();
            if kind.is_symlink() {
                return RestoreOutcome::Refused {
                    reason: String::from(
                        "a symbolic link stands at `.git`, and this route never writes through one",
                    ),
                };
            }
            if kind.is_dir() {
                return RestoreOutcome::Refused {
                    reason: String::from(
                        "a directory stands at `.git`, which is the copied Git administration and not a pointer, so this route leaves it as it is",
                    ),
                };
            }
            if !kind.is_file() {
                return RestoreOutcome::Refused {
                    reason: String::from(
                        "`.git` holds neither a regular file nor a directory, so this route leaves it as it is",
                    ),
                };
            }
            let current: Vec<u8> = match std::fs::read(slot_path.join(SLOT_POINTER_NAME)) {
                Ok(found) => found,
                Err(error) => {
                    return RestoreOutcome::Refused {
                        reason: format!("the pointer cannot be read ({error})"),
                    };
                }
            };
            if current.as_slice() == bytes {
                return RestoreOutcome::AlreadyIntact;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return RestoreOutcome::Refused {
                reason: format!("what stands at `.git` cannot be examined ({error})"),
            };
        }
    }
    let staging: String = match staging_name() {
        Ok(filled) => filled,
        Err(error) => {
            return RestoreOutcome::Refused {
                reason: error.to_string(),
            };
        }
    };
    let mut file: std::fs::File =
        match create_new_entry(&folder, OsStr::new(staging.as_str()), POINTER_FILE_MODE) {
            Ok(opened) => opened,
            Err(error) => {
                return RestoreOutcome::Refused {
                    reason: format!("the staging file for the pointer cannot be made ({error})"),
                };
            }
        };
    if let Err(error) = fill_entry_and_sync(&mut file, bytes, POINTER_FILE_MODE) {
        drop(file);
        let _dropped: std::io::Result<()> = remove_entry(&folder, OsStr::new(staging.as_str()));
        return RestoreOutcome::Refused {
            reason: format!("the staging file for the pointer cannot be written ({error})"),
        };
    }
    drop(file);
    if let Err(error) = rename_entry_replacing(&folder, OsStr::new(staging.as_str()), pointer_name)
    {
        let _dropped: std::io::Result<()> = remove_entry(&folder, OsStr::new(staging.as_str()));
        return RestoreOutcome::Refused {
            reason: format!("the pointer cannot be given its name ({error})"),
        };
    }
    if let Err(error) = sync_folder(&folder) {
        return RestoreOutcome::Refused {
            reason: format!(
                "the pointer is written and the slot folder cannot be flushed ({error}), so a crash may drop it"
            ),
        };
    }
    return RestoreOutcome::Written {
        bytes: bytes.len(),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pointer_restore_tests.rs"]
mod tests;
