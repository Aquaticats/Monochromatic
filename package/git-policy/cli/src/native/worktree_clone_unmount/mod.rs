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
//!   the folder,
//!   record and claim names one GUID derives.
//! - `staged_folder_identity.rs`:
//!   the five recorded fields and their compare.
//! - `staged_folder_record.rs`:
//!   the record's shape,
//!   its atomic write and its bounded read.
//! - `user_cache_pin.rs`:
//!   the `user-cache` pin,
//!   the account's passwd home and the cache descriptor.
//! - `staging_calls.rs`:
//!   the bind,
//!   propagation change,
//!   move,
//!   unmount and handle calls,
//!   and the reads that decide whether they may run.
//! - `staged_record_reading.rs`:
//!   the boot-id test before the handle,
//!   the `ESTALE` pair and the identity compare.
//! - `slot_walk_calls.rs`:
//!   the checkout's root,
//!   the two final hops and the fresh lookup.
//! - `unmount_checks.rs`:
//!   the identity comparisons before any move.
//! - `reattach.rs`:
//!   the checked move back to the slot.
//! - `unmount_turn.rs`:
//!   the record,
//!   bind,
//!   move,
//!   placement,
//!   unmount and busy answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountOneClone(context, verified, request); // the sweep's unmount arm
//! ```

/// The names one GUID derives:
/// the folder,
/// its record and its claim.
pub mod staging_folder_name;

/// The five fields one staging folder's identity holds.
pub mod staged_folder_identity;

/// The record beside a staging folder and its write and read.
pub mod staged_folder_record;

/// The `user-cache` pin,
/// the account's home and the cache descriptor.
pub mod user_cache_pin;

/// The kernel calls the unmount route makes,
/// behind one trait.
pub mod staging_calls;

/// The order the staged-folder record is judged in.
pub mod staged_record_reading;

/// The walks to the slot and the dataset GUID read.
pub mod slot_walk_calls;

/// The identity comparisons the unmount makes before any move.
pub mod unmount_checks;

/// The checked move of a staged clone back to its slot.
pub mod reattach;

/// One removal's unmount turn.
pub mod unmount_turn;

/// The held-descriptor unmount and the later sweep from the record.
pub mod staged_unmount;

/// The staging claim one repository keeps.
pub mod staging_claim;

/// The wrapper's rmdir-only cleanup and the folder's creation.
pub mod staging_cleanup;

/// A recording implementation of the calls,
/// never in the release build.
#[cfg(test)]
pub(crate) mod staging_calls_control;
