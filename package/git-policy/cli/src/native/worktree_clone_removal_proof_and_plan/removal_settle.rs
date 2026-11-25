//! What:
//!  The Git settle of one removal:
//!  the managed lock released with the destruction proof,
//!  the matching registration and the slot's pointer removed by the recipe's own eight steps,
//!  and the leftover underlay folder removed only when it is the recorded one and empty.
//! Why:
//!  The project decided the registration is settled once the clone is destroyed,
//!  and that a separate durable record carries the origin cleanup
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Project decisions",
//!  "Git registration when origin cleanup is blocked"),
//!  so the settle never waits for the origin snapshot and never removes its record.
//!  The registration goes only when its identity is the one the plan recorded,
//!  the pointer only when its bytes name that registration,
//!  and the underlay only when it has the recorded identity and holds nothing,
//!  because the slot's path is reused by the next add and a mistaken removal is not recoverable
//!  (the proposal's "Removal and recovery",
//!  step 5).
//!  The lock is released by the one function that may release it,
//!  which takes the clone's destruction proof,
//!  and a lock the removal finds gone is written again first,
//!  because the decision of 2026-11-22 has the settle release a lock that was there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = settleTheRegistration({ commonDir, slot, slotPath, plan, proof });
//! ```

use crate::entries_of_one_open_folder::entry_names;
use crate::fs_beneath::{ObjectId, identity_of, open_directory_at, remove_at};
use crate::worktree_clone_binding_record::registration_recipe::{NoSeams, ToClear, clear_registration};
use crate::worktree_clone_managed_lock::clone_destruction_proof::CloneDestructionProof;
use crate::worktree_clone_managed_lock::lock_installation::{InstallAnswer, write_the_lost_lock};
use crate::worktree_clone_managed_lock::lock_release::{ReleaseOutcome, release_the_managed_lock};
use crate::worktree_clone_managed_lock::recorded_lock_facts::{RecordedLockFacts, read_the_recorded_facts};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  What one settle did,
///  and the lines it owes the person reading the removal.
/// Why:
///  A settle that wrote a lost lock again says so,
///  and the answer names every step so a control asserts the act and not only a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettleAnswer = { wroteLock; releasedLock; removedRegistration; removedUnderlay; lines };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettleAnswer {
    /// Whether the settle wrote a managed lock again before releasing it.
    pub wrote_the_lost_lock: bool,
    /// Whether the managed lock was released.
    pub released_the_lock: bool,
    /// Whether the matching registration and the slot's pointer went.
    pub removed_the_registration: bool,
    /// Whether the leftover underlay folder went.
    pub removed_the_underlay: bool,
    /// The lines the removal prints about this settle.
    pub lines: Vec<String>,
}

/// What:
///  The identities one settle settles.
/// Why:
///  A registration of another identity is another removal's,
///  and the underlay's recorded identity is what tells this slot's empty folder from a person's,
///  so both travel with the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettleRequest = { commonDir; slot; slotPath; registrationId; admin; slotIdentity; proof };
/// ```
pub struct SettleRequest<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The slot's name,
    /// which is the binding's key.
    pub slot: &'a str,
    /// The slot's whole path,
    /// as the plan's `targetHex` holds it.
    pub slot_path: &'a Path,
    /// The registration's directory name under Git's `worktrees` folder.
    pub registration_id: &'a [u8],
    /// The registration's recorded identity,
    /// as the plan's `admin` field spells it.
    pub admin: &'a str,
    /// The underlay folder's recorded identity,
    /// when the binding recorded one.
    pub slot_identity: Option<crate::worktree_clone_binding_record::binding_encoding::FolderIdentity>,
    /// The proof that this slot's clone is gone.
    pub proof: &'a CloneDestructionProof,
}

/// What:
///  The seam a removal settles through.
/// Why:
///  The live answer writes real files and removes a real registration,
///  and a control answers the same request without a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RegistrationSettle { settle(request): SettleAnswer }
/// ```
pub trait RegistrationSettle {
    /// Settle one removal's registration,
    /// pointer and underlay,
    /// and release its managed lock.
    ///
    /// # Throws
    ///
    /// A registration of another identity,
    /// a pointer that names another registration,
    /// or a call the system refused.
    fn settle_the_registration(&self, request: &SettleRequest<'_>) -> Result<SettleAnswer, RemovalStop>;
}

/// What:
///  The release answer:
///  the recipe's own steps and the one lock release,
///  through the descriptors this route already pins.
/// Why:
///  The settle is the only removal-side step that writes outside the wrapper's own folder,
///  so it runs through the same descriptor-relative calls the add's rollback runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = TheLiveSettle;
/// ```
pub struct TheLiveSettle;

impl RegistrationSettle for TheLiveSettle {
    fn settle_the_registration(&self, request: &SettleRequest<'_>) -> Result<SettleAnswer, RemovalStop> {
        let facts: RecordedLockFacts = read_the_recorded_facts(request.common_dir, request.slot).map_err(|refusal| {
            return RemovalStop::TheSettleRefused { step: String::from("the binding's lock facts"), said: format!("{refusal:?}") };
        })?;
        if facts.clone.dataset != request.proof.clone_dataset() {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the recorded clone"),
                said: format!("the binding names `{}` and the proof names `{}`", facts.clone.dataset, request.proof.clone_dataset()),
            });
        }
        let worktrees: PathBuf = request.common_dir.join("worktrees");
        let registration_name: String = String::from_utf8_lossy(request.registration_id).into_owned();
        let registration_path: PathBuf = worktrees.join(registration_name.as_str());
        let registration: std::fs::File = std::fs::File::open(registration_path.as_path()).map_err(|error| {
            return RemovalStop::TheSettleRefused { step: String::from("the registration"), said: error.to_string() };
        })?;
        let live: ObjectId = identity_of(&registration).map_err(|error| return RemovalStop::TheSettleRefused { step: String::from("the registration's identity"), said: error.to_string() })?;
        if live.encode() != request.admin {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the registration's identity"),
                said: format!("the plan recorded `{}` and the directory answers `{}`", request.admin, live.encode()),
            });
        }
        let mut answer: SettleAnswer = SettleAnswer { wrote_the_lost_lock: false, released_the_lock: false, removed_the_registration: false, removed_the_underlay: false, lines: Vec::new() };
        match write_the_lost_lock(registration_path.as_path(), request.slot, &facts.nonce) {
            InstallAnswer::Wrote { line } => {
                answer.wrote_the_lost_lock = true;
                answer.lines.push(line);
            }
            InstallAnswer::ALockStands => {}
            InstallAnswer::Refused { reason } => return Err(RemovalStop::TheSettleRefused { step: String::from("writing the lost lock"), said: reason }),
        }
        match release_the_managed_lock(&registration, registration_path.as_path(), request.slot, &facts, request.proof) {
            Ok(ReleaseOutcome { .. }) => answer.released_the_lock = true,
            Err(refusal) => return Err(RemovalStop::TheSettleRefused { step: String::from("the managed lock"), said: refusal.words() as String }),
        }
        answer.removed_the_registration = clear_the_registration(request, &worktrees, registration_name.as_str())?;
        answer.removed_the_underlay = remove_the_underlay(request)?;
        return Ok(answer);
    }
}

/// The registration and the slot's pointer,
/// removed by the recipe's eight steps.
///
/// # Throws
/// A refusal naming the entry that stands where the recipe expected nothing.
fn clear_the_registration(request: &SettleRequest<'_>, worktrees: &Path, registration_name: &str) -> Result<bool, RemovalStop> {
    let Some(slot_name) = request.slot_path.file_name() else {
        return Err(RemovalStop::TheSettleRefused { step: String::from("the slot's path"), said: String::from("it names no last component") });
    };
    let Some(slot_parent) = request.slot_path.parent() else {
        return Err(RemovalStop::TheSettleRefused { step: String::from("the slot's path"), said: String::from("it names no parent") });
    };
    let slot_parent_file: std::fs::File = std::fs::File::open(slot_parent).map_err(|error| {
        return RemovalStop::TheSettleRefused { step: String::from("the slot's parent"), said: error.to_string() };
    })?;
    let worktrees_parent: std::fs::File = std::fs::File::open(worktrees).map_err(|error| return RemovalStop::TheSettleRefused { step: String::from("Git's worktrees folder"), said: error.to_string() })?;
    let to: ToClear<'_> = ToClear {
        slot_parent: &slot_parent_file,
        slot_pointer_name: slot_name,
        slot: request.slot_path,
        worktrees_parent: &worktrees_parent,
        registration_name: OsStr::new(registration_name),
        registration: &worktrees.join(registration_name),
    };
    return match clear_registration(&to, &mut NoSeams) {
        Ok(()) => Ok(true),
        Err(refusal) => Err(RemovalStop::TheSettleRefused { step: String::from("the registration"), said: refusal.message() }),
    };
}

/// The leftover underlay folder,
/// removed only when it has the recorded identity and holds nothing.
///
/// # Throws
/// A folder of another identity,
/// one that still holds an entry,
/// or a call the system refused.
fn remove_the_underlay(request: &SettleRequest<'_>) -> Result<bool, RemovalStop> {
    let Some(slot_name) = request.slot_path.file_name() else {
        return Err(RemovalStop::TheSettleRefused { step: String::from("the slot's path"), said: String::from("it names no last component") });
    };
    let Some(slot_parent) = request.slot_path.parent() else {
        return Err(RemovalStop::TheSettleRefused { step: String::from("the slot's path"), said: String::from("it names no parent") });
    };
    let parent: std::fs::File = std::fs::File::open(slot_parent).map_err(|error| return RemovalStop::TheSettleRefused { step: String::from("the slot's parent"), said: error.to_string() })?;
    let folder: std::fs::File = match open_directory_at(&parent, slot_name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(RemovalStop::TheSettleRefused { step: String::from("the underlay folder"), said: error.to_string() }),
    };
    let live: ObjectId = identity_of(&folder).map_err(|error| return RemovalStop::TheSettleRefused { step: String::from("the underlay's identity"), said: error.to_string() })?;
    if let Some(recorded) = request.slot_identity
        && (live.inode != recorded.inode() || live.birth != recorded.birth())
    {
        return Err(RemovalStop::TheSettleRefused {
            step: String::from("the underlay's identity"),
            said: format!("the binding recorded `{}` and the folder answers `{}`", recorded.encode(), live.encode()),
        });
    }
    let entries: Vec<std::ffi::OsString> = entry_names(&folder).map_err(|error| {
        return RemovalStop::TheSettleRefused { step: String::from("the underlay's entries"), said: error.to_string() };
    })?;
    if !entries.is_empty() {
        return Err(RemovalStop::TheSettleRefused {
            step: String::from("the underlay's entries"),
            said: String::from("the folder still holds an entry, and a removal never removes a folder a person filled"),
        });
    }
    return match remove_at(&parent, slot_name, true) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(RemovalStop::TheSettleRefused { step: String::from("the underlay folder"), said: error.to_string() }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_settle_tests.rs"]
mod tests;
