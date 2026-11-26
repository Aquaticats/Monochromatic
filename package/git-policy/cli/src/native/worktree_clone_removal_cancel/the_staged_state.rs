//! What:
//!  The staging folder a cancellation meets:
//!  whether one stands,
//!  its `rmdir`-only settlement on the answer,
//!  and the state in which it is kept and reported.
//! Why:
//!  The design settles the staging folder,
//!  its record and its claim on the answer that puts the clone back,
//!  by the accepted staging design's own rule
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The act" and "Staging" sections).
//!  The removal's own cleanup is step 9's:
//!  a folder that holds anything or carries a mount is kept and reported,
//!  the claim must name this clone and a dead owner,
//!  and the folder's identity must match the claim's,
//!  so this module judges nothing itself and only decides whether there is anything to settle at
//!  all.
//!  The design has the handler's stopped-unmount answer carry the bind's state.
//!  Until plan step 9's build adds that field,
//!  the live answer reads the caller's own mount table the same way step 9's cleanup does,
//!  which is the read the design's refusal for a retained bind also rests on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settled = staged.settle(guid); // { kind: 'no-staged-folder' } | { kind: 'removed' } | { kind: 'kept'; step; said }
//! ```

use crate::fs_mount_options::{MOUNTINFO_PATH, MountFacts, parse_mount_table};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_unmount::staging_claim::{CLAIM_FOLDER, staging_claim_path};
use crate::worktree_clone_unmount::staging_cleanup::{CleanupOutcome, CleanupRequest, clean_up_staging_folder, the_account_cache};
use crate::worktree_clone_unmount::staging_folder_name::{staging_folder_name, staging_record_name};
use std::path::{Path, PathBuf};

/// What:
///  What became of one clone's staged state.
/// Why:
///  The report names which of the three names left,
///  and a kept folder names the step that kept it,
///  so a person learns from the report whether a folder still holds a mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Settlement = { kind: 'no-staged-folder' } | { kind: 'removed'; record; claim } | { kind: 'released-no-folder' } | { kind: 'kept'; step; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StagedSettlement {
    /// No staging folder,
    /// record or claim stood for this clone.
    NoStagedFolder,
    /// The folder left by `rmdir`,
    /// and its record and claim with it.
    Removed {
        /// Whether the record was there and was unlinked.
        record: bool,
        /// Whether the claim was there and was released.
        claim: bool,
    },
    /// The claim named a folder that was never made,
    /// and both left.
    ReleasedNoFolder,
    /// Everything was kept,
    /// and this is the step that kept it.
    Retained {
        /// The step that kept the folder.
        step: String,
        /// What refused.
        said: String,
    },
}

impl StagedSettlement {
    /// Whether every name was kept,
    /// which the command refuses on and the cancelling removal reports.
    pub fn is_retained(&self) -> bool {
        return matches!(self, StagedSettlement::Retained { .. });
    }
}

/// What:
///  The staged state,
///  as the command and the cancelling removal settle it.
/// Why:
///  A control answers this seam without a cache,
///  a claim folder or a mount table,
///  and the release answers it with step 9's own cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheStagedFolder { settle(guid: DatasetGuid): StagedSettlement }
/// ```
pub trait TheStagedFolder {
    /// Settle this clone's staged state on the answer that put the clone back.
    fn settle(&self, clone_guid: DatasetGuid) -> StagedSettlement;
}

/// What:
///  The release answer:
///  the account's cache,
///  the repository's claim folder
///  and this process's mount table,
///  fed to step 9's `rmdir`-only cleanup.
/// Why:
///  The folder is the account's and the claim is the repository's,
///  and both are read through descriptors so no step resolves a path twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = new LiveTheStagedFolder(home, commonDir);
/// ```
pub struct LiveTheStagedFolder<'a> {
    /// The account's home.
    pub home: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
}

impl TheStagedFolder for LiveTheStagedFolder<'_> {
    fn settle(&self, clone_guid: DatasetGuid) -> StagedSettlement {
        let folder_name: String = staging_folder_name(clone_guid);
        let record_name: String = staging_record_name(clone_guid);
        let claim_path: PathBuf = staging_claim_path(self.common_dir, clone_guid);
        let cache_path: PathBuf = self.home.join(".cache");
        let stands: bool = std::fs::symlink_metadata(cache_path.join(record_name.as_str())).is_ok()
            || std::fs::symlink_metadata(cache_path.join(folder_name.as_str())).is_ok()
            || std::fs::symlink_metadata(claim_path.as_path()).is_ok();
        if !stands {
            return StagedSettlement::NoStagedFolder;
        }
        let cache: std::fs::File = match the_account_cache(self.home) {
            Ok(cache) => cache,
            Err(said) => return kept("cache", said),
        };
        let claim_folder: PathBuf = self.common_dir.join(CLAIM_FOLDER);
        let claim_directory: std::fs::File = match std::fs::File::open(claim_folder.as_path()) {
            Ok(folder) => folder,
            Err(error) => return kept("claim-folder", format!("the claim folder {} could not be held ({error})", claim_folder.display())),
        };
        let mounts = || return this_process_mounts();
        let request: CleanupRequest<'_> = CleanupRequest {
            cache: &cache,
            cache_path: cache_path.as_path(),
            claim_directory: &claim_directory,
            guid: clone_guid,
            owner_token: None,
            mounts: &mounts,
            claim_path: claim_path.as_path(),
        };
        return of_the_cleanup(clean_up_staging_folder(&request));
    }
}

/// One cleanup's outcome as the settlement the report reads.
fn of_the_cleanup(outcome: CleanupOutcome) -> StagedSettlement {
    return match outcome {
        CleanupOutcome::Removed { record, claim } => StagedSettlement::Removed { record, claim },
        CleanupOutcome::ReleasedNoFolder => StagedSettlement::ReleasedNoFolder,
        CleanupOutcome::Kept { step, said } => StagedSettlement::Retained { step, said },
    };
}

/// One kept folder.
fn kept(step: &str, said: String) -> StagedSettlement {
    return StagedSettlement::Retained { step: String::from(step), said };
}

/// This process's mount table,
/// which is the one a staging bind in this namespace appears in.
fn this_process_mounts() -> Vec<MountFacts> {
    let text: Vec<u8> = match std::fs::read(MOUNTINFO_PATH) {
        Ok(text) => text,
        Err(error) => {
            crate::diagnostic_log::debug(TAG, format!("the mount table {MOUNTINFO_PATH} could not be read ({error})").as_str());
            return Vec::new();
        }
    };
    return parse_mount_table(text.as_slice());
}

/// The log tag of every decision this module takes.
const TAG: &str = "theStagedFolderSettlement";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_staged_state_tests.rs"]
mod tests;
