//! What:
//!  The check of the account's `.cache` mount against the pin:
//!  the mount's source,
//!  the mount root's inode,
//!  and the one live check the wrapper and the handler both run.
//! Why:
//!  The source says whether the cache dataset is mounted at all,
//!  and the inode says whether the mount at `.cache` is that dataset's root
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision").
//!  The check sits apart from the pin's file and the account's lookup,
//!  because both sides of the route compare the same two fields and one body
//!  keeps them from comparing different ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkUserCacheMount(cache, pin, facts);
//! checkTheLivePin(cache, pin);
//! ```

use crate::fs_beneath_mount_facts::mount_id_of;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::mount_facts_of;
use std::os::unix::fs::MetadataExt;

/// The account's passwd home and the cache descriptor it opens.
pub use super::user_cache_pin_home_and_cache::{
    open_cache,
    passwd_home_of,
};

/// The pin's fields,
/// its file and its words.
pub use super::user_cache_pin_record::{
    MAXIMUM_PIN_BYTES,
    PIN_KIND,
    PIN_VERSION,
    PinRefusal,
    USER_CACHE_PIN_PATH,
    UserCachePin,
    pin_from_text,
    read_pin,
    write_pin,
};

/// What:
///  Check the cache's own mount against the pin,
///  then check that the mount's root is the folder the descriptor holds.
/// Why:
///  The first check is the ordering check,
///  and the second rejects a mount at `~/.cache` whose root is not the
///  dataset's root,
///  such as a bind of a subdirectory of the same dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkUserCacheMount(cache, pin, facts);
/// ```
pub fn check_pin(
    cache: &std::fs::File,
    pin: &UserCachePin,
    facts: &MountFacts,
) -> Result<(), PinRefusal> {
    if facts.source != pin.source {
        return Err(PinRefusal::Source {
            pinned: pin.source.clone(),
            found: facts.source.clone(),
        });
    }
    let inode: u64 = cache
        .metadata()
        .map_err(|error| {
            return PinRefusal::Unreadable {
                path: String::from("the account's `.cache`"),
                reason: error.to_string(),
            };
        })?
        .ino();
    if inode != pin.root_inode {
        return Err(PinRefusal::Inode {
            pinned: pin.root_inode,
            found: inode,
        });
    }
    return Ok(());
}

/// What:
///  The mount facts of the mount a held cache descriptor is on.
/// Why:
///  The check compares the mount the descriptor is on,
///  never a path resolved again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = cacheMountFacts(cache);
/// ```
pub fn cache_mount_facts(cache: &std::fs::File) -> Result<MountFacts, PinRefusal> {
    let mount_id: u64 = mount_id_of(cache).map_err(|error| {
        return PinRefusal::Unreadable {
            path: String::from("the account's `.cache`"),
            reason: error.to_string(),
        };
    })?;
    return mount_facts_of(mount_id).map_err(|refusal| {
        return PinRefusal::Unreadable {
            path: String::from(crate::fs_mount_options::MOUNTINFO_PATH),
            reason: refusal.message(),
        };
    });
}

/// What:
///  The live check both the wrapper and the handler run:
///  the cache's mount against the pin and its root inode.
/// Why:
///  One body keeps the two sides from comparing different fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkTheLivePin(cache, pin);
/// ```
pub fn check_live_pin(
    cache: &std::fs::File,
    pin: &UserCachePin,
) -> Result<(), PinRefusal> {
    let facts: MountFacts = cache_mount_facts(cache)?;
    return check_pin(cache, pin, &facts);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "user_cache_pin_tests.rs"]
mod tests;
