//! What:
//!  The unmount half of the ZFS worktree clone route:
//!  one removal's staging folder under the account's `~/.cache`,
//!  the record beside it that lets a later process finish the unmount,
//!  the `user-cache` pin that orders the bind,
//!  the identity comparisons that guard every move,
//!  the stacked-mount rollback,
//!  the checked reattach,
//!  and the wrapper's `rmdir`-only cleanup.
//! Why:
//!  Plan step 9 builds the route the accepted design and its staging design
//!  settled:
//!  the handler reopens the recorded parent,
//!  verifies the slot and the mount,
//!  binds the staging folder and makes the bind private,
//!  moves the clone onto it,
//!  verifies the placement,
//!  releases the descriptors that would hold it busy,
//!  and unmounts from the held descriptor without force or lazy detach
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging";
//!  `doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision").
//!  The module lives beside the attach rather than inside the sweep,
//!  so the sweep's turn calls one dispatch function and two other rounds can
//!  change the sweep's own files without moving the unmount.
//!
//! Files in this folder,
//! each using only the files listed before it:
//!
//! - `staging_folder_name.rs`:
//!   the folder, record and claim names one GUID derives.
//! - `staged_folder_identity.rs`:
//!   the five recorded fields and their compare.
//! - `staged_folder_record.rs`:
//!   the record's shape,
//!   its atomic write and its bounded read.
//! - `user_cache_pin.rs`:
//!   the `user-cache` pin,
//!   the account's passwd home and the cache descriptor.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountOneClone(context, verified, request); // the sweep's unmount arm
//! ```

/// The names one GUID derives: the folder, its record and its claim.
pub mod staging_folder_name;

/// The five fields one staging folder's identity holds.
pub mod staged_folder_identity;

/// The record beside a staging folder and its write and read.
pub mod staged_folder_record;

/// The `user-cache` pin, the account's home and the cache descriptor.
pub mod user_cache_pin;
