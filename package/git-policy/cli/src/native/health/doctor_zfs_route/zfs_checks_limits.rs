//! What:
//!  The numbers and words the kernel and the design fix:
//!  the error number that proves a dataset is gone,
//!  the object-set type a filesystem has,
//!  the `mountpoint` and `canmount` values that keep a dataset unmounted,
//!  and the `keystatus` and `encryption` values a loaded key is read by.
//! Why:
//!  A dataset whose name is gone is a finding a person can act on,
//!  while a read that failed for any other reason proves nothing and stays
//!  unknown,
//!  so the one number that separates them is named here beside the values the
//!  property checks compare against,
//!  and each value comes from the sweep's own reading of the same field rather
//!  than from a second copy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const NOT_FOUND = 2; // ENOENT
//! const LEGACY_MOUNTPOINT = 'legacy';
//! const CANMOUNT_OFF = 0;
//! ```
//region Limits the kernel and the design fix

/// The error number the kernel answers for an object that is not there.
///
/// Why:
///  A dataset whose name is gone is a finding a person can act on,
///  while a read that failed for any other reason proves nothing and stays
///  unknown,
///  so the one number that separates them is named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NOT_FOUND = 2; // ENOENT
/// ```
pub const NOT_FOUND_ERRNO: i32 = 2;

/// The object-set type a filesystem has,
/// from the sweep's own reading of the same field.
pub const FILESYSTEM_OBJECT_SET: u32 =
    crate::worktree_clone_mount_handler::sweep_dataset_facts::DMU_OBJECT_SET_FILESYSTEM;

/// The `mountpoint` of a dataset nothing mounts automatically.
pub const LEGACY_MOUNTPOINT: &str = "legacy";

/// The `canmount` value that keeps a dataset from an automatic mount.
pub const CANMOUNT_OFF: u64 = 0;

/// The `setuid` and `devices` value that removes the capability from a mount.
pub const FLAG_OFF: u64 = 0;

/// The `keystatus` value that means the key is not loaded.
pub const KEY_UNAVAILABLE: u64 =
    crate::worktree_clone_mount_handler::sweep_dataset_facts::KEY_STATUS_UNAVAILABLE;

/// The `encryption` value of a dataset that is not encrypted.
pub const ENCRYPTION_OFF: &str = "off";

/// The property the clone container's automatic mounting is read from.
pub const CANMOUNT_PROPERTY: &str = "canmount";

//endregion
