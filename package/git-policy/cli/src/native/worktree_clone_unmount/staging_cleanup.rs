//! What:
//!  The wrapper's cleanup of one staging folder:
//!  the claim read,
//!  the dead-owner test,
//!  the folder's identity compare,
//!  the mount and entry refusals,
//!  one `rmdir`,
//!  the record's unlink and the claim's release.
//! Why:
//!  The accepted design makes the cleanup `rmdir`-only and never recursive,
//!  stops at any mount,
//!  and removes a name only where its own claim and that claim's recorded
//!  identity agree with it
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "What the wrapper removes and when";
//!  `doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision").
//!  A folder that holds anything or carries a mount is kept and reported,
//!  because during an interrupted removal a clone is mounted inside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = cleanUpStagingFolder(request); // { kind: 'removed' } | { kind: 'kept'; step; said }
//! ```

use super::staged_folder_identity::{StagedFolderField, StagedFolderIdentity};
use super::staging_claim::{read_claim, ClaimRefusal, StagingClaim};
use super::staging_folder_name::{staging_claim_name, staging_folder_name, staging_record_name};
use crate::fs_beneath::{open_directory_at, remove_at};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  One folder's cleanup and every fact it is judged by.
/// Why:
///  The claim's folder,
/// the account's cache and the live mount table travel
///  together,
///  so a removal cannot check one clone's claim against another folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanupRequest = { cache; cachePath; claimDirectory; guid; ownerToken; mounts };
/// ```
pub struct CleanupRequest<'a> {
    /// The account's cache descriptor.
    pub cache: &'a std::fs::File,
    /// The account's cache folder.
    pub cache_path: &'a Path,
    /// The claim folder,
    /// held by a descriptor.
    pub claim_directory: &'a std::fs::File,
    /// The clone's GUID.
    pub guid: DatasetGuid,
    /// The claim's recorded owner token.
    pub owner_token: Option<&'a str>,
    /// This process's mount table.
    pub mounts: &'a dyn Fn() -> Vec<MountFacts>,
    /// The claim's own path,
    /// which the dead-owner test names.
    pub claim_path: &'a Path,
}

/// What:
///  What the cleanup did.
/// Why:
///  A person needs to know which of the three names left,
///  and a kept folder names the step that kept it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanupOutcome = { kind: 'removed'; record: boolean; claim: boolean } | { kind: 'released-no-folder' } | { kind: 'kept'; step; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CleanupOutcome {
    /// The folder left by `rmdir`,
    /// and the record and claim with it.
    Removed {
        /// Whether the record was there and was unlinked.
        record: bool,
        /// Whether the claim was there and was released.
        claim: bool,
    },
    /// The claim named a folder that was never made,
    /// which the sweep releases with a notice.
    ReleasedNoFolder,
    /// Nothing was removed,
    /// and the step says why.
    Kept {
        /// The step that kept it.
        step: String,
        /// What refused.
        said: String,
    },
}

impl CleanupOutcome {
    /// The word a report carries.
    pub fn word(&self) -> &'static str {
        return match self {
            CleanupOutcome::Removed { .. } => "removed",
            CleanupOutcome::ReleasedNoFolder => "released-no-folder",
            CleanupOutcome::Kept { .. } => "kept",
        };
    }
}

/// One refusal as a kept outcome.
fn kept(step: &str, said: String) -> CleanupOutcome {
    return CleanupOutcome::Kept { step: String::from(step), said };
}

/// One claim refusal as a kept outcome.
fn claim_kept(refusal: ClaimRefusal) -> CleanupOutcome {
    return kept("claim", refusal.message());
}

/// Whether a mount row names the folder's path.
fn a_mount_covers(mounts: &[MountFacts], path: &Path) -> bool {
    for facts in mounts {
        if facts.mount_point == path {
            return true;
        }
    }
    return false;
}

/// One identity mismatch in words.
fn moved(fields: &[StagedFolderField], recorded: &StagedFolderIdentity, found: &StagedFolderIdentity) -> String {
    return format!(
        "the folder differs from the claim's identity in the {}; the claim says {} and the folder reads {}",
        StagedFolderIdentity::field_names(fields),
        recorded.describe(),
        found.describe()
    );
}

/// What:
///  Clean one staging folder up.
/// Why:
///  Every proof runs before the one `rmdir`,
///  and the record and the claim are unlinked only after it,
///  so a refused folder keeps all three names for `doctor`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = cleanUpStagingFolder(request);
/// ```
pub fn clean_up_staging_folder(request: &CleanupRequest<'_>) -> CleanupOutcome {
    let claim: StagingClaim = match read_claim(request.claim_directory, request.guid) {
        Ok(Some(claim)) => claim,
        Ok(None) => return kept("claim", String::from("no staging claim stands for this clone, so nothing proves the wrapper made the folder and it is kept and reported")),
        Err(refusal) => return claim_kept(refusal),
    };
    let token: Option<&str> = request.owner_token.or(Some(claim.owner.as_str()));
    match crate::worktree_copy_owner::token_owner_is_dead(token, request.claim_path) {
        Ok(true) => {}
        Ok(false) => return kept("owner", String::from("the claim's owner is still alive, so this pass keeps the folder rather than taking another command's work")),
        Err(error) => return kept("owner", format!("the claim's owner could not be proved dead: {}", error.0)),
    }
    let name: String = staging_folder_name(request.guid);
    let folder: std::fs::File = match open_directory_at(request.cache, OsString::from(name.as_str()).as_os_str()) {
        Ok(folder) => folder,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => return release_the_claim(request, &claim, name.as_str()),
        Err(error) if error.raw_os_error() == Some(libc::EXDEV) => return kept("mount", format!("a mount covers the folder {name} in the host namespace, so it is kept and reported")),
        Err(error) => return kept("folder", format!("the folder {name} could not be held without crossing a mount: {error}")),
    };
    let found: StagedFolderIdentity = match StagedFolderIdentity::of(&folder) {
        Ok(found) => found,
        Err(refusal) => return kept("identity", refusal.message()),
    };
    let fields: Vec<StagedFolderField> = found.differences(&claim.identity);
    if !fields.is_empty() {
        return kept("identity", moved(fields.as_slice(), &claim.identity, &found));
    }
    let path: PathBuf = request.cache_path.join(name.as_str());
    if a_mount_covers((request.mounts)().as_slice(), path.as_path()) {
        return kept("mount", format!("a row of this process's mount table names {}, so the folder carries a mount and is kept and reported", path.display()));
    }
    match crate::entries_of_one_open_folder::entry_names(&folder) {
        Ok(names) if names.is_empty() => {}
        Ok(names) => return kept("holds-entries", format!("the folder {name} holds {} entries, so it is kept and reported rather than deleted with them", names.len())),
        Err(error) => return kept("holds-entries", format!("the folder {name} could not be listed: {error}")),
    }
    drop(folder);
    if let Err(error) = remove_at(request.cache, OsString::from(name.as_str()).as_os_str(), true) {
        return kept("rmdir", format!("`rmdir` of {name} refused: {error}. The folder and its record are kept and reported."));
    }
    let record_name: String = staging_record_name(request.guid);
    let record: bool = remove_at(request.cache, OsString::from(record_name.as_str()).as_os_str(), false).is_ok();
    let claim_name: OsString = OsString::from(staging_claim_name(request.guid));
    let removed_claim: bool = remove_at(request.claim_directory, claim_name.as_os_str(), false).is_ok();
    return CleanupOutcome::Removed { record, claim: removed_claim };
}

/// Release a claim whose folder was never made.
fn release_the_claim(request: &CleanupRequest<'_>, _claim: &StagingClaim, _name: &str) -> CleanupOutcome {
    let claim_name: OsString = OsString::from(staging_claim_name(request.guid));
    return match remove_at(request.claim_directory, claim_name.as_os_str(), false) {
        Ok(()) => CleanupOutcome::ReleasedNoFolder,
        Err(error) => kept("claim", format!("the claim's folder is absent and its claim could not be released: {error}")),
    };
}

/// What:
///  Make the per-removal folder after the pin check,
///  which is the wrapper's creation step.
/// Why:
///  A bind made before the `user-cache` dataset mounts lands on the home
///  underlay and is hidden when the dataset mounts later,
///  so the pin is checked before the folder exists
///  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
///  "Decision").
///  A folder that is already there is reported rather than replaced,
///  because only the claim proves the wrapper made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = createTheStagingFolder(home, pin, guid);
/// ```
pub fn create_the_staging_folder(home: &Path, pin: &crate::user_cache_pin::UserCachePin, guid: DatasetGuid) -> Result<bool, String> {
    let cache: std::fs::File = crate::user_cache_pin::open_cache(home).map_err(|refusal| return refusal.message())?;
    crate::user_cache_pin::check_live_pin(&cache, pin).map_err(|refusal| return refusal.message())?;
    return make_staging_folder(&cache, guid);
}

/// What:
///  The account's cache folder reached from its home,
///  which the wrapper's creation step opens before it makes a folder.
/// Why:
///  The accessor keeps the deliberate one-mount hop in this module beside the
///  cleanup that reads the same descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cache = theAccountCache(home);
/// ```
pub fn the_account_cache(home: &Path) -> Result<std::fs::File, String> {
    return crate::user_cache_pin::open_cache(home).map_err(|refusal| return refusal.message());
}

/// What:
///  Make the per-removal folder after the pin check.
/// Why:
///  The wrapper creates it,
///  and it refuses an entry it did not make rather than removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = makeStagingFolder(cache, guid);
/// ```
pub fn make_staging_folder(cache: &std::fs::File, guid: DatasetGuid) -> Result<bool, String> {
    let name: OsString = OsString::from(staging_folder_name(guid));
    return crate::fs_beneath::make_directory_at(cache, name.as_os_str(), 0o700).map_err(|error| return format!("the staging folder {name:?} could not be made: {error}"));
}

/// The mount a cache descriptor is on,
/// read by the creation step before any folder is made.
pub fn mount_of(cache: &std::fs::File) -> Result<MountFacts, String> {
    return crate::user_cache_pin::cache_mount_facts(cache).map_err(|refusal| return refusal.message());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staging_cleanup_tests.rs"]
mod tests;
