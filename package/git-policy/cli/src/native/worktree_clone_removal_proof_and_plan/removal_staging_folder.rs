//! What:
//!  The staging folder one removal's unmount needs,
//!  made by the wrapper before it asks the handler to bind it.
//! Why:
//!  The handler's unmount turn opens that folder first and answers `cancelled`
//!  at `staging-folder` when it is absent,
//!  and the accepted staging design gives the folder's creation to the wrapper:
//!  "The wrapper creates one folder per removal",
//!  at the pin-checked cache and after that check
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision";
//!  `SPEC.md`,
//!  "the wrapper creates the folder with mode `0700` only after the `user-cache`
//!  pin check").
//!  `worktree_clone_unmount::staging_cleanup::create_the_staging_folder` is the
//!  function the staging module holds for that duty,
//!  and no release code called it,
//!  so every removal of a real clone asked the handler to bind a folder nothing
//!  had made and stopped there (issue 6669, cause B).
//!  The seam exists because the folder's place and the pin are this account's:
//!  a control on a person's machine answers it with a fixture cache and a pin it
//!  builds from this machine's own mount facts,
//!  where the release reads the account's home and the root-written pin.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! theStagingFolder.makeTheFolder(cloneGuid); // before the unmount is requested
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_unmount::staging_cleanup::create_the_staging_folder;
use crate::worktree_clone_unmount::user_cache_pin::{
    UserCachePin,
    passwd_home_of,
    read_pin,
};
use crate::worktree_clone_unmount::user_cache_pin_record::USER_CACHE_PIN_PATH;
use std::path::{
    Path,
    PathBuf,
};

/// The step word a refusal of this folder names,
/// which is the handler's own word for the same folder.
pub const STAGING_FOLDER_STEP: &str = "staging-folder";

/// The step word a refusal of this account's home names,
/// which is the handler's own for the same read.
pub const ACCOUNT_HOME_STEP: &str = "account-home";

/// The step word a refusal of the pin names,
/// which is the handler's own for the same record.
pub const USER_CACHE_PIN_STEP: &str = "user-cache-pin";

/// What:
///  The folder one removal's unmount binds,
///  as the step a removal takes before it asks the handler for anything.
/// Why:
///  A control answers this without an account's cache or a root-written pin,
///  and the release answers it with the staging module's own creation,
///  so the order that asks for an unmount is the same order in both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemovalStagingFolder { makeTheFolder(cloneGuid: DatasetGuid): void }
/// ```
pub trait RemovalStagingFolder {
    /// Make the folder this clone's unmount binds,
    /// or answer that one already stands.
    ///
    /// # Throws
    ///
    /// The account's home that could not be read,
    /// the pin that could not be read,
    /// the cache whose mount is not the pinned dataset,
    /// or an entry in the folder's name that is not a folder this route may use.
    fn make_the_folder(
        &self,
        clone_guid: DatasetGuid,
    ) -> Result<(), RemovalStop>;
}

/// What:
///  The release answer:
///  this account's passwd home,
///  the `user-cache` pin the one-time setup wrote,
///  and the staging module's own creation.
/// Why:
///  The pin is checked before anything is made,
///  so a cache whose mount is not the pinned dataset writes no entry on the home
///  underlay,
///  which is the ordering the accepted staging design rests on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = createTheStagingFolder(passwdHomeOf(geteuid()), readPin(PIN_PATH), cloneGuid);
/// ```
pub struct LiveRemovalStagingFolder;

impl RemovalStagingFolder for LiveRemovalStagingFolder {
    fn make_the_folder(
        &self,
        clone_guid: DatasetGuid,
    ) -> Result<(), RemovalStop> {
        // SAFETY: `geteuid` has no preconditions and reads this process's own identity.
        let uid: u32 = unsafe { libc::geteuid() };
        let home: PathBuf = passwd_home_of(uid).map_err(|refusal| {
            return a_staging_refusal(ACCOUNT_HOME_STEP, refusal.message());
        })?;
        let pin: UserCachePin = read_pin(Path::new(USER_CACHE_PIN_PATH)).map_err(|refusal| {
            return a_staging_refusal(USER_CACHE_PIN_STEP, refusal.message());
        })?;
        // What: the staging module's own creation, whose answer is whether it made
        //  the folder; a folder that already stands is the same state this step owes.
        // Why: the design makes no second folder for one clone, so the answer is
        //  read and dropped rather than refused.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // createTheStagingFolder(home, pin, cloneGuid);
        // ```
        return create_the_staging_folder(home.as_path(), &pin, clone_guid)
            .map(|_made| return ())
            .map_err(|said| return a_staging_refusal(STAGING_FOLDER_STEP, said));
    }
}

/// The stop one step of this folder leaves,
/// which names the step and the refusing code's own words.
fn a_staging_refusal(
    step: &str,
    said: String,
) -> RemovalStop {
    return RemovalStop::TheStagingFolderCouldNotBeMade {
        step: String::from(step),
        said,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_staging_folder_tests.rs"]
mod tests;
