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
//! each using only files listed before it:
//!
//! - `staged_folder_error.rs`:
//!   the refusal the record and the identity share.
//! - `staged_folder_identity.rs`:
//!   the five recorded fields and their compare.
//! - `staging_folder_name.rs`:
//!   the folder,
//!   record and claim names one GUID derives.
//! - `staged_folder_record_text.rs`:
//!   the record's fields and the one JSON object it is written as,
//!   with the parse of that text.
//! - `staged_folder_record.rs`:
//!   the names the record is written under,
//!   its atomic write and its bounded read.
//! - `staging_call_words.rs`:
//!   the call names a receipt carries and one refusal's words.
//! - `staging_calls_contract.rs`:
//!   the trait every staging call and its control keeps.
//! - `staging_calls.rs`:
//!   the kernel calls:
//!   the bind,
//!   propagation change,
//!   move,
//!   unmount and handle reads,
//!   with the two kernel files they read.
//! - `slot_walk_calls.rs`:
//!   the checkout's root,
//!   the two final hops and the fresh lookup.
//! - `staged_record_reading.rs`:
//!   the boot-id test before the handle,
//!   the `ESTALE` pair and the identity compare.
//! - `staging_claim.rs`:
//!   the staging claim one repository keeps.
//! - `unmount_checks.rs`:
//!   the identity comparisons before any move.
//! - `reattach.rs`:
//!   the checked move back to the slot.
//! - `user_cache_pin_record.rs`:
//!   the pin's two fields,
//!   its file and its bounded read,
//!   and the words a person checks a refusal by.
//! - `user_cache_pin_home_and_cache.rs`:
//!   the account's passwd home and the cache descriptor.
//! - `user_cache_pin.rs`:
//!   the check of the cache's mount against the pin,
//!   the live check both the wrapper and the handler run.
//! - `unmount_turn_answers.rs`:
//!   what one turn answered and the refusals it builds.
//! - `unmount_turn_request.rs`:
//!   the facts and seams one turn reads and acts on.
//! - `unmount_record.rs`:
//!   the pre-bind shape checks and the record's write.
//! - `staged_unmount_held.rs`:
//!   the two unmounts a held descriptor takes,
//!   with the topmost read before each and the transition check after it.
//! - `staged_unmount_recovery.rs`:
//!   the finish of an interrupted unmount from the record's handle.
//! - `staged_unmount.rs`:
//!   the later sweep's reading of one record and the word it answers with.
//! - `staging_cleanup.rs`:
//!   the wrapper's rmdir-only cleanup and the folder's creation.
//! - `unmount_pair.rs`:
//!   the topmost read,
//!   the pair's two unmounts and the move back.
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

/// Why a staging record or folder identity could not be read.
pub mod staged_folder_error;

/// The five fields one staging folder's identity holds.
pub mod staged_folder_identity;

/// The record beside a staging folder and its write and read.
pub mod staged_folder_record;

/// The record's fields and the text it is written as.
pub mod staged_folder_record_text;

/// The `user-cache` pin's file and the words a refusal carries.
pub mod user_cache_pin_record;

/// The account's passwd home and the cache descriptor.
pub mod user_cache_pin_home_and_cache;

/// The `user-cache` pin's check against the cache's mount.
pub mod user_cache_pin;

/// The kernel calls the unmount route makes,
/// behind one trait.
pub mod staging_calls;

/// The call names a refusal carries,
/// and its errno.
pub mod staging_call_words;

/// The trait every staging call and its control keeps.
pub mod staging_calls_contract;

/// The order the staged-folder record is judged in.
pub mod staged_record_reading;

/// The walks to the slot and the dataset GUID read.
pub mod slot_walk_calls;

/// The identity comparisons the unmount makes before any move.
pub mod unmount_checks;

/// The checked move of a staged clone back to its slot.
pub mod reattach;

/// The record a removal writes before its first bind.
pub mod unmount_record;

/// One removal's unmount turn.
pub mod unmount_turn;

/// What one turn answered and the refusals it builds.
pub mod unmount_turn_answers;

/// The facts and seams one unmount turn reads and acts on.
pub mod unmount_turn_request;

/// The topmost read,
/// the pair's two unmounts and the move back.
pub mod unmount_pair;

/// The held-descriptor unmount and the later sweep from the record.
pub mod staged_unmount;

/// The two unmounts a held descriptor takes,
/// with their checks.
pub mod staged_unmount_held;

/// The finish of an interrupted unmount from the record's handle.
pub mod staged_unmount_recovery;

/// The staging claim one repository keeps.
pub mod staging_claim;

/// The wrapper's rmdir-only cleanup and the folder's creation.
pub mod staging_cleanup;

/// A recording implementation of the calls,
/// never in the release build.
#[cfg(test)]
pub(crate) mod staging_calls_control;
