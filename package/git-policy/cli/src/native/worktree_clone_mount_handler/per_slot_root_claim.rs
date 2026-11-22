//! What:
//!  The per-slot claim every root handler takes before it reads a clone's
//!  desired state and acts on it:
//!  a root-owned directory under `/run`,
//!  one numbered claim file per slot,
//!  and a lock-free claim that never waits.
//! Why:
//!  The design lets parallel handlers sweep at once but forbids conflicting
//!  actions on the same mount,
//!  so the desired state and identities are rechecked under a per-slot root
//!  claim
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  A claim that waited would let one sweep's slow step hold every later
//!  sweep of the same slot behind it with no deadline,
//!  so a held slot is refused rather than queued,
//!  and the sweep that refused is free to try the slot on its next trigger.
//!  The claim is the repository's own lock-free claim
//!  (`lock/single_owner_claim.rs`,
//!  `take_claim`):
//!  a record made whole in a private file and published under the next
//!  generation number with one exclusive `link`,
//!  so the kernel lets exactly one claimant create it,
//!  the owner is named by process birth rather than by a reused process id,
//!  and a killed claimant leaves nothing to clean by hand.
//!  The directory is proved through its own descriptor before the claim is
//!  taken,
//!  and the claim's own calls then resolve that path again:
//!  a descriptor-relative `take_claim` would close that window,
//!  and the step that first calls this claim owns that extension.
//!  Rule `NLK` asks for a lock only where one cannot be avoided,
//!  and this claim does not need one:
//!  the takeover is the single exclusive creation that a lock would otherwise
//!  stand in for,
//!  and two claimants cannot both win it.
//!
//!  The claims live under `/run` beside the results and the readiness record,
//!  because a claim means nothing across a boot and every take writes and
//!  syncs a record:
//!  a durable folder would pay that sync per sweep and would keep one file for
//!  every slot ever seen.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claims = SlotClaimDirectory.live(); const claim = claims.claim('one');
//! ```

use crate::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::worktree_clone_mount::slot_name_refusal;
use std::ffi::{CString, OsStr};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// The directory the per-slot claims live in.
pub const SLOT_CLAIM_DIRECTORY: &str = "/run/git-worktree-clone-mount-slot-claims";

/// The directory's mode:
/// readable by everyone so a person can list it,
/// writable by root only.
pub const SLOT_CLAIM_DIRECTORY_MODE: u32 = 0o755;

/// The mode every claim generation file is made at,
/// which only root reads or writes.
pub const SLOT_CLAIM_FILE_MODE: u32 = 0o600;

/// The log tag of every decision this module takes.
const TAG: &str = "perSlotRootClaim";

/// What:
///  Why a claim could not be taken.
/// Why:
///   A handler that cannot prove it holds the slot must not act,
///  and a person needs to tell a held slot from a wrong directory,
///  from a name that is no slot and from a claim record that cannot be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimRefusal = { kind: 'directory' | 'owner' | 'mode' | 'slot-name' | 'held' | 'records'; ... };
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
    /// A claim record could not be listed,
    /// read or written,
    /// or one of them is not a claim this route wrote.
    Records {
        /// The slot.
        slot: String,
        /// What the record's own reading answered.
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
            ClaimRefusal::Held { slot } => format!("another handler holds the claim for slot `{slot}` right now"),
            ClaimRefusal::Records { slot, reason } => format!("the claim records for slot `{slot}` could not be read or written ({reason})"),
        };
        return format!(
            "{reason}. Nothing was opened on the clone's behalf, no mount was attached or taken down and no durable intent was changed. Ways forward: a held slot is retried by the slot's next trigger (a ZED event, the checkout provider's post-key-load hook, or a caller's sweep nudge); a wrong owner, a wrong mode or an unreadable claim record needs an administrator, and the `doctor` command reports a claim once plan step 10 adds it."
        );
    }
}

/// What:
///  Where per-slot claims live and which account must own them.
/// Why:
///   The live place requires root,
///  and a control hands in a fixture directory it owns,
///  so the claim is exercised without `/run`.
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

    /// One slot's claim file at one generation,
    /// which is the name the shared claim publishes.
    pub fn claim_generation_file(&self, slot: &str, generation: u64) -> PathBuf {
        return self.directory.join(format!("{slot}.{generation}"));
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
        // The descriptor is opened with no link followed and its owner is checked before its mode is changed,
        // so a link or another account's folder is refused without being written.
        let directory: std::fs::File = self.open_owned_directory()?;
        directory.set_permissions(std::fs::Permissions::from_mode(SLOT_CLAIM_DIRECTORY_MODE)).map_err(|error| {
            return ClaimRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() };
        })?;
        let facts: std::fs::Metadata = directory
            .metadata()
            .map_err(|error| return ClaimRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() })?;
        let permission_bits: u32 = facts.mode() & 0o7777;
        if permission_bits != SLOT_CLAIM_DIRECTORY_MODE {
            return Err(ClaimRefusal::DirectoryMode { path: path.to_path_buf(), found: permission_bits });
        }
        drop(directory);
        return Ok(());
    }

    /// Take one slot's claim without waiting,
    /// or refuse because a live handler holds it.
    ///
    /// # Errors
    /// The name is no slot name,
    /// the directory is not the designed root-owned folder,
    /// another handler holds the slot,
    /// or a claim record could not be read or written.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const claim = claims.claim('one'); // { held } or { busy, owner }
    /// ```
    pub fn claim(&self, slot: &str) -> Result<HeldSlotClaim, ClaimRefusal> {
        if let Some(reason) = slot_name_refusal(slot) {
            return Err(ClaimRefusal::SlotName { slot: String::from(slot), reason });
        }
        // The shared claim makes path-based calls and checks no owner or mode of its directory, so the designed folder is
        // proved through its own descriptor first, and the claim is then taken inside a folder only root can replace.
        // Its own calls still resolve that path again, so the window between this proof and the claim is open; closing it
        // needs a descriptor-relative take_claim (the shared module's openat, linkat and unlinkat) and belongs to the step
        // that first calls this claim.
        self.open_directory()?;
        let outcome: ClaimOutcome = take_claim(self.directory.as_path(), OsStr::new(slot)).map_err(|error| {
            return ClaimRefusal::Records { slot: String::from(slot), reason: error.to_string() };
        })?;
        return match outcome {
            ClaimOutcome::Held(claim) => Ok(HeldSlotClaim { claim, slot: String::from(slot) }),
            ClaimOutcome::Busy(owner) => {
                crate::diagnostic_log::debug(TAG, format!("refused slot `{slot}`: process {} holds it in {}", owner.pid, owner.path.display()).as_str());
                Err(ClaimRefusal::Held { slot: String::from(slot) })
            }
        };
    }

    /// Open the directory and prove its owner and mode.
    fn open_directory(&self) -> Result<std::fs::File, ClaimRefusal> {
        let directory: std::fs::File = self.open_owned_directory()?;
        let path: &Path = self.directory.as_path();
        let facts: std::fs::Metadata =
            directory.metadata().map_err(|error| return ClaimRefusal::Directory { path: path.to_path_buf(), reason: error.to_string() })?;
        let permission_bits: u32 = facts.mode() & 0o7777;
        if permission_bits != SLOT_CLAIM_DIRECTORY_MODE {
            return Err(ClaimRefusal::DirectoryMode { path: path.to_path_buf(), found: permission_bits });
        }
        return Ok(directory);
    }

    /// Open the directory with no link followed and prove its owner,
    /// before any mode is read or changed.
    fn open_owned_directory(&self) -> Result<std::fs::File, ClaimRefusal> {
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
        return Ok(directory);
    }
}

/// What:
///  One handler's proved hold on one slot.
/// Why:
///   The claim lives exactly as long as this value,
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
    /// The shared claim's own hold,
    /// which names this process and publishes the next generation.
    claim: OwnerClaim,
    /// The slot the claim names.
    slot: String,
}

impl HeldSlotClaim {
    /// The slot this claim holds.
    pub fn slot(&self) -> &str {
        return self.slot.as_str();
    }

    /// Give the claim up now rather than when the value is dropped.
    pub fn release(mut self) {
        if let Err(error) = self.claim.release() {
            crate::diagnostic_log::warn(TAG, format!("slot `{}` keeps a claim that names this process: {error}", self.slot).as_str());
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "per_slot_root_claim_tests.rs"]
mod tests;
