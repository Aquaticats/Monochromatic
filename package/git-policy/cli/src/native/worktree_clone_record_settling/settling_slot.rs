//! What:
//!  What stands at one slot and at the registration its record names:
//!  the `.git` pointer's words,
//!  the registration's identity against `registrationIdentity=`,
//!  the registration's own entries,
//!  the slot's entries beside the pointer,
//!  and the mount table's answer for the slot.
//! Why:
//!  The settling command's fourth read decides three things the removals depend on
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command",
//!  read 4):
//!  whether the abandoned registration is this add's,
//!  whether a checkout has begun,
//!  and whether anything is mounted at the slot.
//!  A `.git` that names another registration and a registration of another
//!  identity are stops,
//!  because the recipe would remove a folder this record never made.
//!  A registration the command's own recipe already removed counts as that step
//!  done,
//!  so a rerun after a kill goes on to the plan and the inventories.
//!  The checkout test is the slot's own entry count beside `.git`,
//!  read while nothing is mounted there,
//!  which is the design's one test.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = readTheSlot(record, commonDir, mounts);
//! ```

use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_slot_state::slot_state_mounts::{
    SlotMount,
    slot_mount_of,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasets;
use std::path::{
    Path,
    PathBuf,
};

/// The pointer's own opening words,
/// which Git writes and the add's finder reads.
const POINTER_PREFIX: &str = "gitdir: ";

/// The pointer file's name inside the slot.
pub const POINTER_NAME: &str = ".git";

/// What:
///  The slot's `.git` and what its bytes name.
/// Why:
///  An absent pointer is the state a killed recipe leaves,
///  and the recipe treats its own missing name as that step done,
///  while a pointer naming anything else is another worktree's entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pointer = { kind: 'absent' } | { kind: 'names-this' } | { kind: 'names-another'; found: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThePointer {
    /// No `.git` stands at the slot.
    Absent,
    /// The pointer names the registration this record names.
    NamesThis,
    /// The pointer names another registration,
    /// or holds words that name none.
    NamesAnother {
        /// The pointer's words.
        found: String,
    },
}

/// What:
///  The registration the record names and what the filesystem answers.
/// Why:
///  A registration gone is a kill inside this command,
///  which the recipe's own `ENOENT` rule reads as done,
///  and one of another identity is another folder's and never this run's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Registration = { kind: 'gone' } | { kind: 'stands'; identity } | { kind: 'differs'; live };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheRegistration {
    /// The registration the record names is not there.
    Gone,
    /// It stands with the identity the record holds.
    Stands {
        /// The folder's live identity.
        identity: FolderIdentity,
    },
    /// It stands with another identity.
    Differs {
        /// The folder's live identity.
        live: FolderIdentity,
    },
}

/// What:
///  Everything the removals decide from,
/// read once.
/// Why:
///  The lines a person confirms print each entry,
///  and the two orders read the checkout answer and the registration's state,
///  so one read supplies both and they cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slot = { path; pointer; registration; entries; registrationEntries; mount };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheSlot {
    /// The slot's path.
    pub path: PathBuf,
    /// The slot's `.git`.
    pub pointer: ThePointer,
    /// The registration the record names.
    pub registration: TheRegistration,
    /// The slot's entries beside `.git`,
    /// sorted.
    pub entries_beside_the_pointer: Vec<String>,
    /// The registration's own entries,
    /// sorted,
    /// and empty when it is gone.
    pub registration_entries: Vec<String>,
    /// What the mount table says about the slot.
    pub mount: SlotMount,
}

impl TheSlot {
    /// What:
    ///  Whether a checkout has begun there,
    ///  which is the design's one test:
    ///  an entry besides `.git` in an unmounted slot.
    /// Why:
    ///  With a checkout begun the person's files stand in the slot,
    ///  so only the wrapper's records go and the registration and the worktree
    ///  stay for `git worktree remove`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const begun = slot.entriesBesideThePointer.length > 0;
    /// ```
    pub fn a_checkout_has_begun(&self) -> bool {
        return !self.entries_beside_the_pointer.is_empty();
    }
}

/// What:
///  One folder's entry names,
///  sorted,
///  with a missing folder read as no entries.
/// Why:
///  Both the slot and the registration are read for their entries,
///  and a folder that is gone proves nothing stands in it.
///
/// # Throws
/// The listing that refused,
/// because nothing proves what a folder it cannot list holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = entryNames(folder); // [] when the folder is gone
/// ```
fn entry_names(folder: &Path) -> Result<Vec<String>, String> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(folder) {
        Ok(listed) => listed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return error.to_string())?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    return Ok(names);
}

/// The pointer's target,
/// with Git's own line ending trimmed.
fn the_pointer_names(bytes: &[u8]) -> Option<PathBuf> {
    let text: String = String::from_utf8_lossy(bytes)
        .trim_end_matches('\n')
        .to_string();
    let target: &str = text.strip_prefix(POINTER_PREFIX)?;
    return Some(PathBuf::from(target));
}

/// What:
///  The slot's `.git` read against the registration the record names.
/// Why:
///  The recipe's first step removes only a pointer whose bytes name this
///  registration,
///  so a pointer naming another folder is refused here rather than at the
///  removal.
///
/// # Throws
/// A pointer that cannot be read,
/// which proves nothing about the slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pointer = readThePointer(slot, registration);
/// ```
fn read_the_pointer(
    slot: &Path,
    registration: &Path,
) -> Result<ThePointer, SettlingStop> {
    let path: PathBuf = slot.join(POINTER_NAME);
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ThePointer::Absent);
        }
        Err(error) => {
            return Err(SettlingStop::TheSlotCouldNotBeRead {
                path,
                said: error.to_string(),
            });
        }
    };
    if !metadata.is_file() {
        return Ok(ThePointer::NamesAnother {
            found: format!(
                "a `.git` that is no file, whose mode is {:04o}",
                mode_of(&metadata)
            ),
        });
    }
    let bytes: Vec<u8> = std::fs::read(path.as_path()).map_err(|error| {
        return SettlingStop::TheSlotCouldNotBeRead {
            path: path.clone(),
            said: error.to_string(),
        };
    })?;
    return match the_pointer_names(bytes.as_slice()) {
        Some(target) if target == registration => Ok(ThePointer::NamesThis),
        _ => Ok(ThePointer::NamesAnother {
            found: String::from_utf8_lossy(bytes.as_slice())
                .trim_end_matches('\n')
                .to_string(),
        }),
    };
}

/// One entry's permission bits,
/// which a pointer that is no file is described by.
fn mode_of(metadata: &std::fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        return metadata.mode() & 0o777;
    }
    #[cfg(not(unix))]
    {
        drop(metadata);
        return 0;
    }
}

/// What:
///  The registration the record names,
///  identified against `registrationIdentity=`.
/// Why:
///  The folder's inode and birth time are what the record holds,
///  so a registration removed and made again is refused here.
///
/// # Throws
/// A registration that stands and cannot be identified.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registration = readTheRegistration(recordedPath, recorded);
/// ```
fn read_the_registration(
    registration: &Path,
    recorded: FolderIdentity,
) -> Result<TheRegistration, SettlingStop> {
    match std::fs::symlink_metadata(registration) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(TheRegistration::Gone);
        }
        Err(error) => {
            return Err(SettlingStop::TheRegistrationDoesNotAnswer {
                path: registration.to_path_buf(),
                said: error.to_string(),
            });
        }
        Ok(_) => {}
    }
    let live: FolderIdentity = FolderIdentity::of_folder(registration).map_err(|error| {
        return SettlingStop::TheRegistrationDoesNotAnswer {
            path: registration.to_path_buf(),
            said: error.to_string(),
        };
    })?;
    return match live == recorded {
        true => Ok(TheRegistration::Stands {
            identity: live,
        }),
        false => Ok(TheRegistration::Differs {
            live,
        }),
    };
}

/// What:
///  The slot and its registration read once,
///  or the stop the first refusal names.
/// Why:
///  The mount is read first,
///  because a mounted slot's entries are the mount's and not a checkout's,
///  and the design refuses a mount at the slot outright.
///
/// # Throws
/// A mount at the slot,
/// an unreadable mount table,
/// or a pointer or registration that proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = readTheSlot(path, registrationPath, recordedIdentity, mounts);
/// ```
pub fn read_the_slot(
    slot: &Path,
    registration: &Path,
    recorded_identity: FolderIdentity,
    mounts: &dyn MountedDatasets,
) -> Result<TheSlot, SettlingStop> {
    let mount: SlotMount = slot_mount_of(mounts, slot);
    match &mount {
        SlotMount::Something {
            described,
        } => {
            return Err(SettlingStop::AMountAtTheSlot {
                described: described.clone(),
            });
        }
        SlotMount::Unreadable {
            reason,
        } => {
            return Err(SettlingStop::TheMountTableCouldNotBeRead {
                said: reason.clone(),
            });
        }
        SlotMount::Nothing => {}
    }
    let pointer: ThePointer = read_the_pointer(slot, registration)?;
    let registration_state: TheRegistration =
        read_the_registration(registration, recorded_identity)?;
    let entries: Vec<String> = entry_names(slot)
        .map_err(|said| {
            return SettlingStop::TheSlotCouldNotBeRead {
                path: slot.to_path_buf(),
                said,
            };
        })?
        .into_iter()
        .filter(|name| return name != POINTER_NAME)
        .collect();
    let registration_entries: Vec<String> = match registration_state {
        TheRegistration::Stands {
            ..
        } => entry_names(registration).map_err(|said| {
            return SettlingStop::TheRegistrationDoesNotAnswer {
                path: registration.to_path_buf(),
                said,
            };
        })?,
        TheRegistration::Gone
        | TheRegistration::Differs {
            ..
        } => Vec::new(),
    };
    return Ok(TheSlot {
        path: slot.to_path_buf(),
        pointer,
        registration: registration_state,
        entries_beside_the_pointer: entries,
        registration_entries,
        mount,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_slot_tests.rs"]
mod tests;
