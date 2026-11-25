//! What:
//!  One clone's removal claim:
//!  the folder and the name the claim lives under,
//!  the take that refuses a live removal and replaces a dead one's generation,
//!  and the release that ends it.
//! Why:
//!  The design gives the wrapper one claim per clone for the whole removal,
//!  of the repository's own generation claim kind,
//!  under `<common>/monochromatic-git-policy-cli-worktree-copy/removal/`
//!  and named `<guid>.removal`
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The removal claim").
//!  A live owner refuses the command with the owner's process identity,
//!  because a person must wait for or end that removal;
//!  a dead owner's claim is replaced by the next generation,
//!  which is the mechanism's ordinary takeover and needs nothing cleaned by hand.
//!  The claim is not the root handler's per-slot claim,
//!  which only root takes and which the command cannot take.
//!  Every writer that acts on one clone's removal takes this claim first:
//!  the removal's own entry,
//!  the wiring round that switches it on,
//!  and this command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = claim.take(guid); // a held claim, or a refusal naming the live owner
//! held.release();
//! ```

use crate::lock::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The folder the removal claims live in,
/// beside repository's other wrapper records inside one Git common directory.
pub const REMOVAL_CLAIM_FOLDER: &str = "monochromatic-git-policy-cli-worktree-copy/removal";

/// The word between the clone's GUID and the generation number.
pub const REMOVAL_CLAIM_WORD: &str = "removal";

/// The folder one repository's removal claims live in.
pub fn removal_claim_folder(common_dir: &Path) -> PathBuf {
    return common_dir.join(REMOVAL_CLAIM_FOLDER);
}

/// The prefix `take_claim` completes with a generation number,
/// which is the clone's canonical GUID and the claim's word.
pub fn removal_claim_prefix(clone_guid: DatasetGuid) -> OsString {
    return OsString::from(format!("{}.{}", clone_guid.file_name(), REMOVAL_CLAIM_WORD));
}

/// What:
///  A claim this run holds.
/// Why:
///  Every way out of the command releases it,
///  and a control needs a held claim that creates no file,
///  so the live claim and the control's answer live in one type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimHeld = { claim?: OwnerClaim; owner: string };
/// ```
#[derive(Debug)]
pub struct ClaimHeld {
    /// The claim file this process created,
    /// and nothing in a control.
    claim: Option<OwnerClaim>,
    /// The owner the claim names.
    owner: String,
}

impl ClaimHeld {
    /// What:
    ///  A claim a control holds,
    ///  which creates no file.
    /// Why:
    ///  The command's decisions must be provable without a repository,
    ///  and only a live take may create a claim file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ClaimHeld.forAControl('4242');
    /// ```
    pub fn for_a_control(owner: &str) -> ClaimHeld {
        return ClaimHeld { claim: None, owner: String::from(owner) };
    }

    /// The owner the claim names,
    /// which is this process for a live take.
    pub fn owner(&self) -> &str {
        return self.owner.as_str();
    }

    /// What:
    ///  Give the claim back.
    /// Why:
    ///  The removal claim marks a running operation,
    ///  and there is none once the command has printed,
    ///  so every path out releases it and a control releases nothing.
    ///
    /// # Errors
    /// The claim file could not be rewritten as a released claim.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// held.release();
    /// ```
    pub fn release(mut self) -> Result<(), CancelRefusal> {
        let Some(mut claim) = self.claim.take() else {
            return Ok(());
        };
        return claim.release().map_err(|error| return CancelRefusal::TheClaimCouldNotBeTaken { said: error.to_string() });
    }
}

/// What:
///  The removal claim,
///  as the command and the removal take it.
/// Why:
///  A control must be able to refuse with a live owner and to hold a claim without a file,
///  so the take is one seam and the release is the value it returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheRemovalClaim { take(guid: DatasetGuid): ClaimHeld }
/// ```
pub trait TheRemovalClaim {
    /// Take this clone's removal claim.
    ///
    /// # Errors
    /// A live removal,
    /// or a claim whose folder or file could not be used.
    fn take(&self, clone_guid: DatasetGuid) -> Result<ClaimHeld, CancelRefusal>;
}

/// What:
///  The release answer:
///  the repository's own generation claim,
///  in this route's folder and under this route's name.
/// Why:
///  The mechanism is already the repository's,
///  so this module adds the two names and no second lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = new LiveTheRemovalClaim(commonDir);
/// ```
pub struct LiveTheRemovalClaim<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
}

impl TheRemovalClaim for LiveTheRemovalClaim<'_> {
    fn take(&self, clone_guid: DatasetGuid) -> Result<ClaimHeld, CancelRefusal> {
        let folder: PathBuf = removal_claim_folder(self.common_dir);
        crate::private_storage::create_private_directory_all(folder.as_path()).map_err(|error| {
            return CancelRefusal::TheClaimCouldNotBeTaken { said: format!("the claim folder {} could not be made ({error})", folder.display()) };
        })?;
        let prefix: OsString = removal_claim_prefix(clone_guid);
        let outcome: ClaimOutcome = take_claim(folder.as_path(), prefix.as_os_str()).map_err(|error| {
            return CancelRefusal::TheClaimCouldNotBeTaken { said: error.to_string() };
        })?;
        return match outcome {
            ClaimOutcome::Held(claim) => Ok(ClaimHeld { claim: Some(claim), owner: format!("this command (pid {})", std::process::id()) }),
            ClaimOutcome::Busy(owner) => Err(CancelRefusal::ALiveRemoval {
                owner: format!("pid {} (its claim is {})", owner.pid, owner.path.display()),
            }),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_removal_claim_tests.rs"]
mod tests;
