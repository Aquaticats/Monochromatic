//! What:
//!  The wrapper's own step before it asks the handler to unmount a clone:
//!  the account's home,
//!  the `user-cache` pin checked against the mount at its `.cache`,
//!  and the per-clone staging folder the handler binds.
//! Why:
//!  The handler moves the clone's mount to a folder under the account's
//!  `.cache`,
//!  and it refuses to make that folder itself:
//!  the wrapper creates it before the request,
//!  after the pin proves the cache dataset is what stands at `.cache`
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it";
//!  `worktree_clone_unmount/unmount_turn.rs` names the wrapper's creation in its
//!  `staging-folder` refusal).
//!  The pin is checked before the folder exists,
//!  because a bind made before the cache dataset mounts lands on the home
//!  underlay and is hidden when the dataset mounts later.
//!  This is a seam the order takes as its unmount request,
//!  so a control of the order still answers an unmount without a home and a
//!  cache.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unmount = withTheStagingFolder(new LiveRemovalUnmount(clone, guid));
//! await unmount.requestTheUnmount(intent); // makes ~/.cache/…-staging-<guid> first
//! ```

use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    LiveRemovalUnmount,
    RemovalUnmount,
    UnmountAnswer,
};
use crate::worktree_clone_unmount::staging_cleanup::create_the_staging_folder;
use crate::worktree_clone_unmount::user_cache_pin::{
    USER_CACHE_PIN_PATH,
    UserCachePin,
    passwd_home_of,
    read_pin,
};
use std::path::PathBuf;

/// What:
///  The release's unmount request,
///  with the staging folder the handler binds made before the request is
///  written.
/// Why:
///  The folder is the wrapper's and the request is step 9's,
///  so the one place both are known is the seam the order takes.
///  A folder that already stands is kept:
///  only the claim proves the wrapper made it,
///  and the handler reports it rather than replacing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemovalUnmount { requestTheUnmount(intent): Answer }
/// ```
pub(crate) struct LiveUnmountWithTheStagingFolder {
    /// Step 9's own request and wait.
    unmount: LiveRemovalUnmount,
    /// The clone whose staging folder the handler binds.
    clone_guid: DatasetGuid,
}

impl LiveUnmountWithTheStagingFolder {
    /// What:
    ///  The request for one clone,
    ///  over the live unmount the removal already opened.
    /// Why:
    ///  The identifier travels beside the request,
    ///  because the folder's name is derived from it and no second read may
    ///  decide another clone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const unmount = withTheStagingFolder(new LiveRemovalUnmount(clone, guid), guid);
    /// ```
    pub(crate) fn over(
        unmount: LiveRemovalUnmount,
        clone_guid: DatasetGuid,
    ) -> LiveUnmountWithTheStagingFolder {
        return LiveUnmountWithTheStagingFolder {
            unmount,
            clone_guid,
        };
    }
}

impl RemovalUnmount for LiveUnmountWithTheStagingFolder {
    fn request_the_unmount(
        &self,
        intent: &MountIntent,
    ) -> Result<UnmountAnswer, RemovalStop> {
        make_the_staging_folder(self.clone_guid)?;
        return self.unmount.request_the_unmount(intent);
    }
}

/// What:
///  The account's cache checked against the pin,
///  and one clone's staging folder made under it.
/// Why:
///  Each refusal names the step it stopped at,
///  because a removal that stops before its intent says which of the three
///  facts was missing:
///  the account's home,
///  the pin,
///  or the folder itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const home = passwdHomeOf(geteuid());
/// const pin = readPin(USER_CACHE_PIN_PATH);
/// makeTheStagingFolder(home, pin, guid);
/// ```
fn make_the_staging_folder(clone_guid: DatasetGuid) -> Result<(), RemovalStop> {
    let home: PathBuf = passwd_home_of(unsafe { libc::geteuid() }).map_err(|refusal| {
        return RemovalStop::UnmountFailed {
            step: String::from("staging-home"),
            said: refusal.message(),
        };
    })?;
    let pin: UserCachePin =
        read_pin(std::path::Path::new(USER_CACHE_PIN_PATH)).map_err(|refusal| {
            return RemovalStop::UnmountFailed {
                step: String::from("staging-pin"),
                said: refusal.message(),
            };
        })?;
    create_the_staging_folder(home.as_path(), &pin, clone_guid).map_err(|said| {
        return RemovalStop::UnmountFailed {
            step: String::from("staging-folder"),
            said,
        };
    })?;
    return Ok(());
}
