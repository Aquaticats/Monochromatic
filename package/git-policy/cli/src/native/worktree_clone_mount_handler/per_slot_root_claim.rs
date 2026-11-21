//! What:
//!  The per-slot claim every root handler takes before it reads a clone's
//!  desired state and acts on it:
//!  a root-owned directory of lock files under `/var/lib`,
//!  one file per slot,
//!  and an exclusive,
//!  non-waiting lock that ends when the claim is dropped.
//! Why:
//!  The design lets parallel handlers sweep at once but forbids conflicting
//!  actions on the same mount,
//!  so the desired state and identities are rechecked under a per-slot root
//!  claim
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering";
//!  "Removal and recovery" says the claim serializes cooperating wrapper
//!  operations only).
//!  A claim that waited would let one sweep's slow step hold every later
//!  sweep of the same slot behind it with no deadline,
//!  so a held slot is refused rather than queued,
//!  and the sweep that refused is free to try the slot on its next trigger.
//!  The design names no path for the claim:
//!  it sits beside step 4's slot identity store under root control,
//!  which is the narrowest place the records already keep such state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claims = SlotClaimDirectory.live(); const claim = claims.claim('one');
//! ```

use crate::worktree_clone_mount::slot_name_refusal;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// The directory the per-slot claims live in.
pub const SLOT_CLAIM_DIRECTORY: &str = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-claims";

/// The directory's mode:
/// readable by everyone so `doctor` can list it,
/// writable by root only.
pub const SLOT_CLAIM_DIRECTORY_MODE: u32 = 0o755;

/// One claim file's mode,
/// which only root reads or writes.
pub const SLOT_CLAIM_FILE_MODE: u32 = 0o600;

/// The suffix of one slot's claim file.
pub const SLOT_CLAIM_SUFFIX: &str = ".claim";

/// What:
///  Why a claim could not be taken.
/// Why:
///   A handler that cannot prove it holds the slot must not act,
///  and a person needs to tell a held slot from a wrong directory and from a
///  name that is no slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimRefusal = { kind: 'directory' | 'owner' | 'mode' | 'slot-name' | 'held' | 'file'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimRefusal {
    /// The directory could not be made,
    /// opened or examined.
    Directory {
        /// The path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// The directory is not owned by the expected account.
    DirectoryOwner {
        /// The path.
        path: PathBuf,
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The directory does not carry its designed mode.
    DirectoryMode {
        /// The path.
        path: PathBuf,
        /// The mode found.
        found: u32,
    },
    /// The name is no plain slot name.
    SlotName {
        /// The name as given.
        slot: String,
        /// Why it is not a slot name.
        reason: String,
    },
    /// Another handler holds this slot.
    Held {
        /// The slot.
        slot: String,
    },
    /// The claim file could not be opened,
    /// examined or locked.
    File {
        /// The slot.
        slot: String,
        /// What the call answered.
        reason: String,
    },
}

impl ClaimRefusal {
    /// The refusal in words,
    /// naming the slot and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            ClaimRefusal::Directory { path, reason } => format!("the per-slot claim directory {} could not be made or opened ({reason})", path.display()),
            ClaimRefusal::DirectoryOwner { path, found, wanted } => format!(
                "the per-slot claim directory {} is owned by user {found}, and this route requires user {wanted}",
                path.display()
            ),
            ClaimRefusal::DirectoryMode { path, found } => format!(
                "the per-slot claim directory {} has mode {found:04o}, and this route requires {SLOT_CLAIM_DIRECTORY_MODE:04o}",
                path.display()
            ),
            ClaimRefusal::SlotName { slot, reason } => format!("`{slot}` is no slot name: {reason}"),
            ClaimRefusal::Held { slot } => format!(
                "another handler holds the claim for slot `{slot}` right now"
            ),
            ClaimRefusal::File { slot, reason } => format!("the claim file for slot `{slot}` could not be opened or locked ({reason})"),
        };
        return format!(
            "{reason}. Nothing was opened on the clone's behalf, no mount was attached or taken down and no durable intent was changed. Ways forward: a held slot is retried by the slot's next trigger (a ZED event, the checkout provider's post-key-load hook, or a caller's sweep nudge); a wrong owner or mode needs an administrator, and `doctor` reports both."
        );
    }
}

/// What:
///  Where per-slot claims live and which account must own them.
/// Why:
///   The live place requires root,
///  and a control hands in a fixture directory it owns,
///  so the lock is exercised without `/var/lib`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SlotClaimDirectory { constructor(readonly directory: string, readonly owner: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotClaimDirectory {
    /// The directory the claim files live in.
    directory: PathBuf,
    /// The account every directory and claim file must belong to.
    owner: u32,
}

impl SlotClaimDirectory {
    /// The place the release handler claims in.
    pub fn live() -> Self {
        return SlotClaimDirectory { directory: PathBuf::from(SLOT_CLAIM_DIRECTORY), owner: 0 };
    }

    /// A place at a fixture directory,
    /// owned by the account the control expects.
    pub fn at(directory: PathBuf, owner: u32) -> Self {
        return SlotClaimDirectory { directory, owner };
    }

    /// The directory.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// One slot's claim file.
    pub fn claim_file(&self, slot: &str) -> PathBuf {
        return self.directory.join(format!("{slot}{SLOT_CLAIM_SUFFIX}"));
    }

    /// Make the directory when it is absent,
    /// at its designed mode,
    /// and prove it is the designed one.
    ///
    /// # Errors
    /// The directory could not be made or given its mode,
    /// or it is owned by another account.
    pub fn prepare(&self) -> Result<(), ClaimRefusal> {
        let path: &Path = self.directory.as_path();
        if let Err(error) = std::fs::create_dir_all(path) {
            return Err(ClaimRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() });
        }
        if let Err(error) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(SLOT_CLAIM_DIRECTORY_MODE)) {
            return Err(ClaimRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() });
        }
        let directory: std::fs::File = self.open_directory()?;
        drop(directory);
        return Ok(());
    }

    /// Take one slot's claim without waiting.
    ///
    /// # Errors
    /// The name is no slot name,
    /// the directory is not the designed root-owned folder,
    /// another handler holds the slot,
    /// or the claim file could not be opened or locked.
    pub fn claim(&self, slot: &str) -> Result<HeldSlotClaim, ClaimRefusal> {
        if let Some(reason) = slot_name_refusal(slot) {
            return Err(ClaimRefusal::SlotName { slot: String::from(slot), reason });
        }
        let directory: std::fs::File = self.open_directory()?;
        let name: String = format!("{slot}{SLOT_CLAIM_SUFFIX}");
        let file: std::fs::File = open_claim_at(&directory, name.as_str()).map_err(|error| return ClaimRefusal::File { slot: String::from(slot), reason: error.to_string() })?;
        let facts: std::fs::Metadata = file
            .metadata()
            .map_err(|error| return ClaimRefusal::File { slot: String::from(slot), reason: error.to_string() })?;
        if facts.uid() != self.owner {
            return Err(ClaimRefusal::File {
                slot: String::from(slot),
                reason: format!("the claim file is owned by user {} and this route requires user {}", facts.uid(), self.owner),
            });
        }
        let locked: Result<(), rustix::io::Errno> = rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive);
        return match locked {
            Ok(()) => Ok(HeldSlotClaim { file, slot: String::from(slot) }),
            Err(error) if error == rustix::io::Errno::WOULDBLOCK => Err(ClaimRefusal::Held { slot: String::from(slot) }),
            Err(error) => Err(ClaimRefusal::File { slot: String::from(slot), reason: error.to_string() }),
        };
    }

    /// Open the directory and prove its owner and mode.
    fn open_directory(&self) -> Result<std::fs::File, ClaimRefusal> {
        let path: &Path = self.directory.as_path();
        let text: CString =
            CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_nul| return ClaimRefusal::Directory { path: path.to_path_buf(), reason: String::from("the path holds a NUL byte") })?;
        // SAFETY: `text` is a NUL-terminated string that outlives the call.
        let descriptor: libc::c_int = unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
        if descriptor < 0 {
            return Err(ClaimRefusal::Directory { path: path.to_path_buf(), reason: std::io::Error::last_os_error().to_string() });
        }
        // SAFETY: `open` returned a new descriptor that nothing else owns.
        let directory: std::fs::File = std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
        let facts: std::fs::Metadata =
            directory.metadata().map_err(|error| return ClaimRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() })?;
        if facts.uid() != self.owner {
            return Err(ClaimRefusal::DirectoryOwner { path: path.to_path_buf(), found: facts.uid(), wanted: self.owner });
        }
        let permission_bits: u32 = facts.mode() & 0o777;
        if permission_bits != SLOT_CLAIM_DIRECTORY_MODE {
            return Err(ClaimRefusal::DirectoryMode { path: path.to_path_buf(), found: permission_bits });
        }
        return Ok(directory);
    }
}

/// What:
///  One handler's proved hold on one slot.
/// Why:
///   The lock lives exactly as long as this value,
///  so a caller cannot take the claim and then act after dropping it by
///  accident,
///  and the slot's name travels with the hold for a report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HeldSlotClaim { readonly slot: string; release(): void }
/// ```
#[derive(Debug)]
pub struct HeldSlotClaim {
    /// The claim file,
    /// whose descriptor holds the lock.
    file: std::fs::File,
    /// The slot the claim names.
    slot: String,
}

impl HeldSlotClaim {
    /// The slot this claim holds.
    pub fn slot(&self) -> &str {
        return self.slot.as_str();
    }

    /// Give the claim up now rather than when the value is dropped.
    pub fn release(self) {
        drop(self.file);
    }
}

/// Open one claim file in the directory,
/// creating it at the designed mode and following no link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openat(directory, name, O_CREAT | O_RDWR | O_NOFOLLOW, 0600);
/// ```
fn open_claim_at(directory: &std::fs::File, name: &str) -> std::io::Result<std::fs::File> {
    let text: CString = CString::new(name).map_err(|_nul| return std::io::Error::from_raw_os_error(libc::EINVAL))?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            text.as_ptr(),
            libc::O_CREAT | libc::O_RDWR | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            SLOT_CLAIM_FILE_MODE as libc::mode_t,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "per_slot_root_claim_tests.rs"]
mod tests;
