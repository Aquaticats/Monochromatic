//! What:
//!  The removals themselves,
//!  run in the order `settling_steps` states and proved gone one by one.
//! Why:
//!  The command removes evidence,
//!  so every step's answer is checked:
//!  a name removed is read again and a name that still answers stops the run
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command",
//!  "What it removes").
//!  The registration goes through step 10's own recipe rather than a second
//!  removal of the same eight names,
//!  and the lock is written again with this add's sentence before it,
//!  because the command holds the slot and has proved the registration
//!  (the decision of 2026-11-22, and the lock design's S2).
//!  A person's own lock refuses:
//!  `git worktree unlock` is the way forward and no command removes words it did
//!  not write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gone = makeTheRemovals(to, steps, claim);
//! ```

use crate::lock::single_owner_claim::OwnerClaim;
use crate::worktree_clone_binding_record::lock_words::{LockStanding, is_unattributable_prefix, lock_file_bytes, lock_standing_of};
use crate::worktree_clone_binding_record::registration_home::{binding_name, claim_prefix, clone_inventory_name, home_folder, origin_inventory_name, plans_folder};
use crate::worktree_clone_binding_record::registration_recipe::{NoSeams, RecipeRefusal, ToClear, clear_registration};
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_steps::SettlingStep;
use std::ffi::{OsStr, OsString};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// The lock file's name inside a registration.
const LOCK_NAME: &str = "locked";

/// What:
///  The names one run removes,
///  each a literal path below a folder the caller holds.
/// Why:
///  The runner takes no path from a record that a slot name did not build,
///  and the tests read the same names the run uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalsToMake = { commonDir; slot; slotPath; registration; registrationName; cloneGuid; nonce };
/// ```
pub struct RemovalsToMake<'a> {
    /// The Git common directory.
    pub common_dir: &'a Path,
    /// The slot's own name,
    /// which the three record names are built from.
    pub slot: &'a str,
    /// The slot's path.
    pub slot_path: &'a Path,
    /// The registration the record names,
    /// whose own name is what the recipe removes.
    pub registration: &'a Path,
    /// The nonce the lock's sentence is recomputed from.
    pub nonce: &'a RequestNonce,
    /// The clone's GUID,
    /// whose name the plan is filed under.
    pub clone_guid: crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid,
}

/// What:
///  What one run removed,
///  in the order it removed it.
/// Why:
///  A person reads which records went,
///  and a control asserts the order without listing three folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalsGone = { steps; files; lockWritten; registrationRemoved };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovalsGone {
    /// The steps this run made,
    /// in order,
    /// with the steps a run had nothing to do left out.
    pub steps: Vec<SettlingStep>,
    /// The files this run removed,
    /// in order.
    pub files: Vec<String>,
    /// Whether the lost lock was written again.
    pub lock_written: bool,
    /// Whether the registration's own names went.
    pub registration_removed: bool,
    /// Whether a plan went.
    pub plan_removed: bool,
    /// Whether the plan folder went with it.
    pub plan_folder_removed: bool,
}

/// What:
///  The registration's name below Git's `worktrees` folder.
/// Why:
///  The recipe works from names below held descriptors,
///  so the registration's last component is what it removes.
///
/// # Throws
/// A registration path with no last component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = registrationNameOf(path);
/// ```
pub fn the_registration_name(registration: &Path) -> Result<OsString, SettlingStop> {
    return registration.file_name().map(OsStr::to_os_string).ok_or_else(|| {
        return SettlingStop::TheRecipeStopped { said: format!("the registration {} names no folder below Git's `worktrees`", registration.display()) };
    });
}

/// What:
///  The registration's lock written again,
///  which one truncating write of the sentence makes whole.
/// Why:
///  A name that is gone is the installer's case and a name that is a proper
///  prefix is the killed write the next run completes,
///  and both end in the same one write.
///  Anything else is a person's own lock and refuses.
///
/// # Throws
/// A lock that is not this add's,
/// or the file that could not be written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = writeTheLostLock(registration, nonce);
/// ```
fn write_the_lost_lock(registration: &Path, nonce: &RequestNonce) -> Result<bool, SettlingStop> {
    let path: PathBuf = registration.join(LOCK_NAME);
    let bytes: Vec<u8> = match std::fs::read(path.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(SettlingStop::TheRecipeStopped { said: format!("the registration's lock {} could not be read ({error})", path.display()) }),
    };
    match lock_standing_of(bytes.as_slice(), nonce) {
        LockStanding::Verified => return Ok(false),
        // A proper prefix of this add's sentence is a killed write, and bytes too short to attribute are the empty
        // file Git's own `worktree lock` leaves. Both end in the one truncating write below.
        LockStanding::CutShort { .. } => {}
        LockStanding::Foreign => {
            if !is_unattributable_prefix(bytes.as_slice()) {
                return Err(SettlingStop::TheLockIsNotThisAdds { path });
            }
        }
    }
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.as_path())
        .map_err(|error| return SettlingStop::TheRecipeStopped { said: format!("the lock {} could not be written ({error})", path.display()) })?;
    file.write_all(lock_file_bytes(nonce).as_slice())
        .map_err(|error| return SettlingStop::TheRecipeStopped { said: format!("the lock {} could not be written ({error})", path.display()) })?;
    file.sync_all().map_err(|error| return SettlingStop::TheRecipeStopped { said: format!("the lock {} could not be synced ({error})", path.display()) })?;
    return Ok(true);
}

/// What:
///  The recipe's eight removals over the registration this record names.
/// Why:
///  Step 10's own order is the only one whose every kill leaves a row the
///  finder still recognises,
///  so a second copy here would be a second grammar.
///
/// # Throws
/// The refusal the recipe printed,
/// or a folder that cannot be opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheRegistration(to);
/// ```
fn clear_the_registration(to: &RemovalsToMake<'_>) -> Result<(), SettlingStop> {
    let registration_name: OsString = the_registration_name(to.registration)?;
    let slot_parent_path: PathBuf = to.slot_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        return SettlingStop::TheRecipeStopped { said: format!("the slot {} names no parent folder", to.slot_path.display()) };
    })?;
    let worktrees_parent_path: PathBuf = to.common_dir.join("worktrees");
    let slot_parent: std::fs::File = crate::fs_beneath::open_root(slot_parent_path.as_path())
        .map_err(|error| return SettlingStop::TheRecipeStopped { said: format!("the folder {} cannot be opened ({error})", slot_parent_path.display()) })?;
    let worktrees_parent: std::fs::File = crate::fs_beneath::open_root(worktrees_parent_path.as_path())
        .map_err(|error| return SettlingStop::TheRecipeStopped { said: format!("the folder {} cannot be opened ({error})", worktrees_parent_path.display()) })?;
    let to_clear: ToClear<'_> = ToClear {
        slot_parent: &slot_parent,
        slot_pointer_name: to.slot_path.file_name().ok_or_else(|| return SettlingStop::TheRecipeStopped { said: String::from("the slot names no folder") })?,
        slot: to.slot_path,
        worktrees_parent: &worktrees_parent,
        registration_name: registration_name.as_os_str(),
        registration: to.registration,
    };
    return clear_registration(&to_clear, &mut NoSeams).map_err(|refusal: RecipeRefusal| return SettlingStop::TheRecipeStopped { said: refusal.message() });
}

/// What:
///  One file removed and proved gone.
/// Why:
///  A removal that answered and left the name behind would let the run release
///  the claim over a record a later reader still finds.
///
/// # Throws
/// The file that could not be removed or that still answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeAndProve(path);
/// ```
fn remove_and_prove(path: &Path) -> Result<(), SettlingStop> {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(SettlingStop::TheRecordCouldNotGo { path: path.to_path_buf(), said: error.to_string() }),
    }
    return match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SettlingStop::TheRecordCouldNotGo { path: path.to_path_buf(), said: error.to_string() }),
        Ok(_) => Err(SettlingStop::TheRecordCouldNotGo { path: path.to_path_buf(), said: String::from("the name still answers after its removal") }),
    };
}

/// What:
///  The whole order run,
///  the claim released last.
/// Why:
///  The claim is what keeps a second command out of the slot while the records
///  go,
///  so it is released only after the last record is proved gone.
///
/// # Throws
/// The first step that refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = makeTheRemovals(to, steps, claim);
/// ```
pub fn make_the_removals(to: &RemovalsToMake<'_>, steps: &[SettlingStep], claim: &mut OwnerClaim) -> Result<RemovalsGone, SettlingStop> {
    let home: PathBuf = home_folder(to.common_dir);
    let mut gone: RemovalsGone = RemovalsGone {
        steps: Vec::new(),
        files: Vec::new(),
        lock_written: false,
        registration_removed: false,
        plan_removed: false,
        plan_folder_removed: false,
    };
    for step in steps {
        match step {
            SettlingStep::TheLostLock => {
                gone.lock_written = write_the_lost_lock(to.registration, to.nonce)?;
            }
            SettlingStep::TheRegistrationRecipe => {
                clear_the_registration(to)?;
                gone.registration_removed = true;
            }
            SettlingStep::ThePlan => {
                let path: PathBuf = plans_folder(to.common_dir).join(to.clone_guid.file_name());
                remove_and_prove(path.as_path())?;
                gone.plan_removed = true;
                gone.files.push(to.clone_guid.file_name());
                if std::fs::remove_dir(plans_folder(to.common_dir).as_path()).is_ok() {
                    gone.plan_folder_removed = true;
                }
            }
            SettlingStep::TheOriginInventory | SettlingStep::TheCloneInventory | SettlingStep::TheBinding => {
                let name: String = match step {
                    SettlingStep::TheOriginInventory => origin_inventory_name(to.slot),
                    SettlingStep::TheCloneInventory => clone_inventory_name(to.slot),
                    _ => binding_name(to.slot),
                };
                remove_and_prove(home.join(name.as_str()).as_path())?;
                gone.files.push(name);
            }
            SettlingStep::TheClaim => {
                claim
                    .release()
                    .map_err(|error| return SettlingStop::TheClaimCannotBeReleased { slot: String::from(to.slot), said: format!("{error:?}") })?;
            }
        }
        gone.steps.push(*step);
    }
    return Ok(gone);
}

/// What:
///  The claim's own prefix,
///  which the caller takes the slot's claim with.
/// Why:
///  The command and the add must name one claim,
///  or two commands could work on one slot at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takeClaim(home, claimPrefix(slot));
/// ```
pub fn the_claim_prefix(slot: &str) -> OsString {
    return claim_prefix(slot);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_removals_tests.rs"]
mod tests;
