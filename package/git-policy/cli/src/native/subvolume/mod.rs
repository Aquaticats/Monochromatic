//! What:
//!  Btrfs subvolume worktrees and the filesystem primitives under them.
//! Why:
//!  Platform-specific code is easier to audit in one folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // subvolume/index.ts: one `export * from './file'` per module below.
//! ```

/// The typed failure of one Linux filesystem primitive:
///  the call,
/// the path and the error number.
#[cfg(target_os = "linux")]
pub mod fs_primitive_error;

/// Which filesystem holds a path (btrfs,
/// overlayfs,
/// other) and whether a directory is a btrfs subvolume root.
#[cfg(target_os = "linux")]
pub mod fs_kind;

/// The mount of a path read from `/proc/self/mountinfo`,
/// with its options (`user_subvol_rm_allowed`).
#[cfg(target_os = "linux")]
pub mod fs_mount_options;

/// The UUID of the filesystem on a mount's source device,
/// from udev's `/dev/disk/by-uuid` links.
#[cfg(target_os = "linux")]
pub mod fs_uuid_of_source;

/// The btrfs ioctls that create,
/// snapshot and destroy a subvolume,
/// called in process.
#[cfg(target_os = "linux")]
pub mod fs_btrfs_ioctl;

/// `FICLONE` of one file into another and the atomic `RENAME_EXCHANGE` of two paths.
#[cfg(target_os = "linux")]
pub mod fs_clone_exchange;

/// Directory-relative file operations that never follow a link:
/// the held-handle primitives of the clone rung and of the
/// snapshot's stub removal.
#[cfg(target_os = "linux")]
pub mod fs_beneath;

/// The directories a path walk has already opened,
/// kept as one chain from the root,
/// so a walk over paths in order opens each directory once.
#[cfg(target_os = "linux")]
pub mod fs_beneath_directory_chain;

/// The btrfs fixture root of the primitive and migration controls,
/// and the loud report where there is none.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod fs_btrfs_test_support;

/// SHA-1 and SHA-256,
/// for the checksum of a rewritten Git index.
#[cfg(target_os = "linux")]
pub mod subvolume_sha;

/// The stat rewrite of a Git index file's bytes.
#[cfg(target_os = "linux")]
pub mod subvolume_index;

/// The first index of a new worktree:
/// written into `index.lock` and linked under its name,
/// nothing replaced.
#[cfg(target_os = "linux")]
pub mod subvolume_index_install_into_new_worktree;

/// The linked worktrees of a main checkout and the registrations that are not one of them.
#[cfg(target_os = "linux")]
pub mod subvolume_registered_worktrees;

/// The index stat rewrite applied to the checkout's index and to those of its linked worktrees.
#[cfg(target_os = "linux")]
pub mod subvolume_index_files;

/// The index rewrite of a checkout and of every linked worktree inside it,
/// side by side within the host's budget.
#[cfg(target_os = "linux")]
pub mod subvolume_index_refresh_of_every_worktree;

/// Write-and-flush and directory-flush steps that the migration's writers share.
#[cfg(target_os = "linux")]
pub mod subvolume_file_write_steps;

/// The failure type of the subvolume migration and its simulated-kill hook.
#[cfg(target_os = "linux")]
pub mod subvolume_error;

/// The operating-system operations of the migration behind one trait,
/// and the Linux implementation.
#[cfg(target_os = "linux")]
pub mod subvolume_platform;

/// Copying owner,
/// permissions,
/// times and extended attributes onto a copy.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs;

/// One idempotent pass that makes a directory tree match another,
/// in process.
#[cfg(target_os = "linux")]
pub mod subvolume_sync;

/// The lines a tree pass says about what it could not carry into the destination.
#[cfg(target_os = "linux")]
pub mod subvolume_pass_report;

/// The pieces every step of a tree sync shares:
/// options,
/// statistics,
/// work entries,
/// the pass state and the entry checks.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_pass_state_and_checks;

/// The copying of one source entry (file,
/// symbolic link,
/// named pipe) into the destination,
/// as methods of the pass.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_copy_one_entry;

/// The index locks the migration holds,
/// marked with a token that names the migration.
#[cfg(target_os = "linux")]
pub mod subvolume_lock;

/// An open directory that every operation inside it is addressed through,
/// so a replaced path cannot redirect them.
#[cfg(target_os = "linux")]
pub mod subvolume_anchor;

/// Removal of a directory tree whose directories may lack owner write permission.
#[cfg(target_os = "linux")]
pub mod subvolume_remove;

pub mod subvolume_remove_walk;

/// The moment a removal has read a directory's entries and not yet removed them, for a control that acts there.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_remove_listing_test_support;

/// The migration journal beside the checkout.
#[cfg(target_os = "linux")]
pub mod subvolume_journal;

/// The count of the entries under a checkout,
/// from which the free space a migration needs follows.
#[cfg(target_os = "linux")]
pub mod subvolume_entry_count;

/// The paths of a migration and the refusals that stop it before anything is created.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight;

/// The state a migration command carries between its steps.
#[cfg(target_os = "linux")]
pub mod subvolume_run;

/// Converting a main checkout into a btrfs subvolume,
/// resumably.
#[cfg(target_os = "linux")]
pub mod subvolume_migrate;

/// Undoing a migration and deleting the old tree.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback;

/// Which directory the rollback deletes as the migration's stage,
/// and the refusal when it is not the subvolume the migration made.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback_stage;

/// Removing a subvolume and every subvolume nested in it,
/// deepest first.
pub mod subvolume_rollback_subvolume_removal;

/// Which directory a name holds,
/// by identity,
/// for the carry and the rollback's exchange.
pub mod subvolume_rollback_identity;

/// The grammar of `git monochromatic-git-policy-cli migrate-to-subvolume`.
pub mod subvolume_arguments;

/// `git monochromatic-git-policy-cli migrate-to-subvolume`:
/// root discovery,
/// modes and exit statuses.
#[cfg(target_os = "linux")]
pub mod subvolume_command;

/// The stand-in platform and fixture checkouts of the subvolume migration controls.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_test_support;

/// The migration controls that need the real btrfs ioctls,
/// `FICLONE` and exchange.
#[cfg(all(test, target_os = "linux"))]
mod subvolume_btrfs_tests;
