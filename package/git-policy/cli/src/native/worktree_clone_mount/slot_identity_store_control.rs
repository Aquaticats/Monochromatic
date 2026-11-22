//! What:
//!  Whose the store folder and its records must be,
//!  which permission bits say another account could write them,
//!  and the refusal that names the account which could.
//! Why:
//!  A record decides which folder a clone may be attached to,
//!  so a store the attach's account could rewrite would make every recorded identity advisory.
//!  The folder and every record answer the same two terms,
//!  and the reader that opens each record and the store that opens the folder
//!  both ask this one function,
//!  so the terms and their sentence are written once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = storeControlRefusal(owner, mode & 0o7777); // string | undefined
//! ```

use super::clone_folder_identity::PERMISSION_BITS;
use super::slot_identity_store_refusals::StoreRefusal;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The mode the store folder is made with:
/// root writes it,
/// and an account may read but not add,
/// rename or remove an entry in it.
pub const STORE_FOLDER_MODE: u32 = 0o755;

/// The mode each record file is made with:
/// readable by every account,
/// writable by root alone.
pub const RECORD_MODE: u32 = 0o644;

/// The owner of the store folder and of every record in it:
/// root,
/// because only the root-run setup,
/// sweep and adoption modes write them.
pub const STORE_OWNER: u32 = 0;

/// The permission bits that let an account other than the owner write:
/// group write and other write.
pub const WRITABLE_BY_OTHERS: u32 = 0o022;

/// What:
///  Why a folder or record file this route reads is not the one under root control,
///  or nothing when it is.
/// Why:
///   A record decides which folder a clone may be attached to,
///  so a store the attach's account could rewrite would make every recorded identity advisory.
///  A folder owned by root and holding no group or other write bit is one only root writes;
///  the account the attach runs as may still read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = storeControlRefusal(owner, mode & 0o7777); // string | undefined
/// ```
pub fn store_control_refusal(owner: u32, mode: u32) -> Option<String> {
    if owner != STORE_OWNER {
        return Some(format!(
            "its owner is user {owner} and not root, so the account that owns it could rewrite the identities this route compares against"
        ));
    }
    if mode & WRITABLE_BY_OTHERS != 0 {
        return Some(format!(
            "its permission bits are {mode:04o}, and {WRITABLE_BY_OTHERS:04o} would let another account write it"
        ));
    }
    return None;
}


/// Whether the entry at `path` is under root control,
/// and the refusal naming what lets this account write it when it is not.
///
/// Why:
///  The check reads the descriptor that was opened,
///  so a swap between a name and the read cannot change which object was judged.
pub(super) fn check_under_root_control(path: &Path, what: &str, file: &std::fs::File) -> Result<(), StoreRefusal> {
    let metadata: std::fs::Metadata = file.metadata().map_err(|error| return StoreRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() })?;
    return match store_control_refusal(metadata.uid(), metadata.mode() & PERMISSION_BITS) {
        Some(reason) => Err(StoreRefusal::NotUnderRootControl { path: path.to_path_buf(), what: what.to_string(), reason }),
        None => Ok(()),
    };
}
