//! What:
//!  Everything one unmount turn reads and acts on:
//!  the verified checkout,
//!  the clone's slot,
//!  the mount the removal expects,
//!  the recorded slot identity,
//!  the account's cache and the `user-cache` pin,
//!  and the calls and walks the turn makes.
//! Why:
//!  The facts and the seams travel together,
//!  so the caller cannot pass the slot of one clone with the GUID of another,
//!  and every check of the route takes one argument rather than a dozen.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type UnmountRequest = { checkout; slot; nonce; expected; slotIdentity; pin; cache; cachePath; boot; calls; walk; guids };
//! ```

use super::slot_walk_calls::{
    DatasetGuids,
    SlotWalk,
};
use super::staged_folder_identity::StagedFolderIdentity;
use super::staging_calls::StagingCalls;
use super::unmount_checks::ExpectedMount;
use super::user_cache_pin::UserCachePin;
use std::path::Path;

/// What:
///  Everything one unmount turn reads and acts on.
/// Why:
///  The facts and the seams travel together,
///  so the caller cannot pass the slot of one clone with the GUID of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnmountRequest = { checkout; slot; nonce; expected; pin; cache; cachePath; boot; calls; walk; guids };
/// ```
pub struct UnmountRequest<'a> {
    /// The verified checkout.
    pub checkout: &'a Path,
    /// The clone's slot name.
    pub slot: &'a str,
    /// The intent's nonce.
    pub nonce: &'a str,
    /// The mount the removal expects.
    pub expected: &'a ExpectedMount,
    /// The slot folder's identity the store recorded.
    pub slot_identity: &'a StagedFolderIdentity,
    /// The `user-cache` pin.
    pub pin: &'a UserCachePin,
    /// The account's cache descriptor.
    pub cache: &'a std::fs::File,
    /// The cache folder's path.
    pub cache_path: &'a Path,
    /// This boot's identifier.
    pub boot: &'a str,
    /// The mount calls.
    pub calls: &'a dyn StagingCalls,
    /// The walks.
    pub walk: &'a dyn SlotWalk,
    /// The dataset GUID read.
    pub guids: &'a dyn DatasetGuids,
}
