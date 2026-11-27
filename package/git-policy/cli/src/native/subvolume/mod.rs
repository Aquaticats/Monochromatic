//! What:
//!  Btrfs subvolume worktrees and the filesystem primitives under them.
//! Why:
//!  Platform-specific code is easier to audit in one folder.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `fs_beneath_names.rs`:
//!   the plain names and descriptor paths a held-handle call is written with.
//! - `fs_beneath_identity.rs`:
//!   the device,
//!   inode and birth time that prove which object a name or a descriptor holds.
//! - `fs_beneath_open.rs`:
//!   `openat2` below a held directory,
//!   with the resolution bits that refuse every link and mount.
//! - `fs_beneath_entries.rs`:
//!   creating,
//!   reading,
//!   timing,
//!   renaming and removing one entry of a held directory.
//! - `fs_beneath_owned_removal.rs`:
//!   the lock file's removal,
//!   which compares the identity recorded when it was made,
//!   and the publishing of an open file under a new name.
//! - `fs_beneath_permissions.rs`:
//!   giving the owner write and search permission through the descriptor that was judged.
//! - `fs_beneath.rs`:
//!   directory-relative file operations that never follow a link:
//!   the held-handle primitives of the clone rung and of the snapshot's stub removal,
//!   re-exported from the six files above.
//! - `fs_beneath_mount_facts.rs`:
//!   what a held descriptor says about the mount it is on and the path the kernel finds at it now.
//! - `fs_beneath_walk_chain.rs`:
//!   the walk down a chain of one-name hops,
//!   and the last hop that may end on a mount.
//! - `fs_beneath_directory_chain.rs`:
//!   the directories a path walk has already opened,
//!   kept as one chain from the root,
//!   so a walk over paths in order opens each directory once.
//! - `fs_mount_options.rs`:
//!   the mount of a path read from `/proc/self/mountinfo`,
//!   with its options (`user_subvol_rm_allowed`).
//! - `fs_primitive_error.rs`:
//!   the typed failure of one Linux filesystem primitive:
//!   the call,
//!   the path and the error number.
//! - `fs_btrfs_ioctl_request_numbers.rs`:
//!   the request numbers the btrfs ioctls are called by,
//!   and the kernel encodings they are computed from.
//! - `fs_btrfs_ioctl_argument_structures.rs`:
//!   the kernel structures the btrfs ioctls read and write,
//!   each at the size its request number states.
//! - `fs_btrfs_ioctl.rs`:
//!   the calls that create,
//!   snapshot,
//!   destroy and read a subvolume,
//!   and the whole paths their callers hold.
//! - `fs_clone_exchange.rs`:
//!   `FICLONE` of one file into another and the atomic `RENAME_EXCHANGE` of two paths.
//! - `fs_kind.rs`:
//!   which filesystem holds a path (btrfs,
//!   overlayfs,
//!   other) and whether a directory is a btrfs subvolume root.
//! - `fs_uuid_of_source.rs`:
//!   the UUID of the filesystem on a mount's source device,
//!   from udev's `/dev/disk/by-uuid` links.
//! - `subvolume_anchor.rs`:
//!   an open directory that every operation inside it is addressed through,
//!   so a replaced path cannot redirect them.
//! - `subvolume_arguments.rs`:
//!   the grammar of `git monochromatic-git-policy-cli migrate-to-subvolume`.
//! - `subvolume_entry_count.rs`:
//!   the count of the entries under a checkout,
//!   from which the free space a migration needs follows.
//! - `subvolume_error.rs`:
//!   the failure type of the subvolume migration and its simulated-kill hook.
//! - `subvolume_file_write_steps.rs`:
//!   write-and-flush and directory-flush steps that the migration's writers share.
//! - `subvolume_journal_leftover_names.rs`:
//!   the names a killed journal write may leave beside the journal.
//! - `subvolume_journal_writes.rs`:
//!   the journal's durable writes:
//!   one file written exclusively and renamed over its target,
//!   and the error for a journal that cannot be understood.
//! - `subvolume_journal.rs`:
//!   the migration journal beside the checkout.
//! - `subvolume_lock.rs`:
//!   the index locks the migration holds,
//!   marked with a token that names the migration.
//! - `subvolume_platform.rs`:
//!   the operating-system operations of the migration behind one trait,
//!   and the Linux implementation.
//! - `subvolume_preflight_paths.rs`:
//!   the paths of a migration and the free space its metadata needs.
//! - `subvolume_preflight_refusal.rs`:
//!   one refusal and the two texts it is built from.
//! - `subvolume_preflight_git_state_refusals.rs`:
//!   the refusals about Git's own state in a checkout and its linked worktrees.
//! - `subvolume_preflight_migration_refusals.rs`:
//!   the refusals that stop a fresh or resumed migration before anything is created.
//! - `subvolume_preflight_rollback_refusals.rs`:
//!   the refusals of a rollback and of deleting the old tree.
//! - `subvolume_preflight.rs`:
//!   the paths of a migration and the refusals that stop it before anything is created.
//! - `subvolume_registered_worktrees.rs`:
//!   the linked worktrees of a main checkout and the registrations that are not one of them.
//! - `subvolume_remove_walk_boundary_and_refusal.rs`:
//!   the boundary a removal may not cross,
//!   and the words of the refusal it gives.
//! - `subvolume_remove_walk.rs`:
//!   empty a directory that is held open,
//!   one descriptor-relative step at a time,
//!   refusing every nested subvolume and mount point before anything is deleted.
//! - `subvolume_remove.rs`:
//!   removal of a directory tree whose directories may lack owner write permission.
//! - `subvolume_remove_around_look.rs`:
//!   the look that finds every volume below a tree without changing it.
//! - `subvolume_remove_around.rs`:
//!   the look that finds every volume below a tree without changing it,
//!   and the pass that empties a tree around each one it did not make.
//! - `subvolume_sha.rs`:
//!   SHA-1 and SHA-256,
//!   for the checksum of a rewritten Git index.
//! - `subvolume_index_rewrite_contract.rs`:
//!   what a caller of the index stat rewrite supplies and reads back.
//! - `subvolume_index_bytes.rs`:
//!   reading a Git index file's header,
//!   entries,
//!   extensions and checksum length.
//! - `subvolume_index.rs`:
//!   the stat rewrite of a Git index file's bytes.
//! - `subvolume_index_install_into_new_worktree.rs`:
//!   the first index of a new worktree:
//!   written into `index.lock` and linked under its name,
//!   nothing replaced.
//! - `subvolume_index_file_stat_fields.rs`:
//!   the stat fields an index entry stores,
//!   read from a file and compared with what an entry recorded.
//! - `subvolume_index_file_zeroed_stat.rs`:
//!   an index with the stat data of named paths zeroed,
//!   so Git hashes those files instead of trusting them.
//! - `subvolume_index_file_options.rs`:
//!   what a refresh of one index is told,
//!   how it ended,
//!   and the repository's hash length.
//! - `subvolume_index_file_rewrite_and_install.rs`:
//!   the rewrite of one index's stat fields and the install that replaces the index with it.
//! - `subvolume_index_files.rs`:
//!   the index stat rewrite applied to the checkout's index and to those of its linked worktrees.
//! - `subvolume_index_refresh_of_every_worktree.rs`:
//!   the index rewrite of a checkout and of every linked worktree inside it,
//!   side by side within the host's budget.
//! - `subvolume_sync_attrs_losses.rs`:
//!   what a copy of one entry could not carry,
//!   counted by kind.
//! - `subvolume_sync_attrs_extended.rs`:
//!   the extended attributes copied onto a copy,
//!   removed from it and compared with the original's.
//! - `subvolume_sync_attrs_owner_and_times.rs`:
//!   the owner,
//!   permission bits and times of a copy,
//!   set in the order that keeps them.
//! - `subvolume_sync_attrs_fifo.rs`:
//!   opening and creating the named pipes of a copied tree.
//! - `subvolume_sync_attrs.rs`:
//!   copying owner,
//!   permissions,
//!   times and extended attributes onto a copy.
//! - `subvolume_sync_pass_state_and_checks.rs`:
//!   the pieces every step of a tree sync shares:
//!   options,
//!   statistics,
//!   work entries,
//!   the pass state and the entry checks.
//! - `subvolume_sync_directory_visit.rs`:
//!   the walk of one directory of a tree pass,
//!   and the test-only callback a control installs at it.
//! - `subvolume_sync_source_child.rs`:
//!   the dispatch of one source child by what it is.
//! - `subvolume_sync.rs`:
//!   one idempotent pass that makes a directory tree match another,
//!   in process.
//! - `subvolume_pass_report.rs`:
//!   the lines a tree pass says about what it could not carry into the destination.
//! - `subvolume_run.rs`:
//!   the state a migration command carries between its steps.
//! - `subvolume_migrate_options.rs`:
//!   what a person asked a migration to do,
//!   and how one ended.
//! - `subvolume_migrate_steps.rs`:
//!   the steps that change the checkout:
//!   create,
//!   the index locks,
//!   the exchange,
//!   and the finish.
//! - `subvolume_migrate_words.rs`:
//!   the lines a migration says:
//!   the facts the run begins with,
//!   and the closing text with everything left to do.
//! - `subvolume_migrate.rs`:
//!   converting a main checkout into a btrfs subvolume,
//!   resumably.
//! - `subvolume_rollback_identity.rs`:
//!   which directory a name holds,
//!   by identity,
//!   for the carry and the rollback's exchange.
//! - `subvolume_rollback_stage.rs`:
//!   which directory the rollback deletes as the migration's stage,
//!   and the refusal when it is not the subvolume the migration made.
//! - `subvolume_rollback_subvolume_removal.rs`:
//!   removing a subvolume and every subvolume nested in it,
//!   deepest first.
//! - `subvolume_rollback_refusals.rs`:
//!   the refusals a rollback and a deletion of the old tree answer with.
//! - `subvolume_rollback_carry_back.rs`:
//!   carrying the person's work back out of the new subvolume.
//! - `subvolume_rollback_removal.rs`:
//!   deleting the old tree once the person asks for it.
//! - `subvolume_rollback_swap.rs`:
//!   the exchange that turns the checkout back into the original tree.
//! - `subvolume_rollback.rs`:
//!   undoing a migration and deleting the old tree.
//! - `subvolume_command.rs`:
//!   `git monochromatic-git-policy-cli migrate-to-subvolume`:
//!   root discovery,
//!   modes and exit statuses.
//! - `subvolume_sync_copy_link_record.rs`:
//!   the bookkeeping of a file with several names in a tree pass:
//!   whether a destination file is one of the recorded copies,
//!   recording a copy,
//!   and linking the next name to an earlier copy.
//! - `subvolume_sync_copy_one_entry.rs`:
//!   the copying of one source entry (file,
//!   symbolic link,
//!   named pipe) into the destination,
//!   as methods of the pass.
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

/// The request numbers the btrfs ioctls are called by,
/// and the kernel encodings they are computed from.
#[cfg(target_os = "linux")]
pub mod fs_btrfs_ioctl_request_numbers;

/// The kernel structures the btrfs ioctls read and write,
/// each at the size its request number states.
#[cfg(target_os = "linux")]
pub mod fs_btrfs_ioctl_argument_structures;

/// The calls that create,
/// snapshot,
/// destroy and read a subvolume,
/// and the whole paths their callers hold.
#[cfg(target_os = "linux")]
pub mod fs_btrfs_ioctl;

/// `FICLONE` of one file into another and the atomic `RENAME_EXCHANGE` of two paths.
#[cfg(target_os = "linux")]
pub mod fs_clone_exchange;

/// The plain names and the descriptor paths a held-handle call is written with.
#[cfg(target_os = "linux")]
pub mod fs_beneath_names;

/// The device,
/// inode and birth time that prove which object a name or a descriptor holds.
#[cfg(target_os = "linux")]
pub mod fs_beneath_identity;

/// `openat2` below a held directory,
/// with the resolution bits that refuse every link and mount.
#[cfg(target_os = "linux")]
pub mod fs_beneath_open;

/// Creating,
/// reading,
/// timing,
/// renaming and removing one entry of a held directory.
#[cfg(target_os = "linux")]
pub mod fs_beneath_entries;

/// The lock file's removal,
/// which compares the identity recorded when it was made,
/// and the publishing of an open file under a new name.
#[cfg(target_os = "linux")]
pub mod fs_beneath_owned_removal;

/// Giving the owner write and search permission through the descriptor that was judged.
#[cfg(target_os = "linux")]
pub mod fs_beneath_permissions;

/// Directory-relative file operations that never follow a link:
/// the held-handle primitives of the clone rung and of the
/// snapshot's stub removal.
#[cfg(target_os = "linux")]
pub mod fs_beneath;

/// What a held descriptor says about the mount it is on and the path the kernel finds at it now.
#[cfg(target_os = "linux")]
pub mod fs_beneath_mount_facts;

/// The walk down a chain of one-name hops,
/// and the last hop that may end on a mount.
#[cfg(target_os = "linux")]
pub mod fs_beneath_walk_chain;

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

/// What a caller of the index stat rewrite supplies and reads back.
#[cfg(target_os = "linux")]
pub mod subvolume_index_rewrite_contract;

/// Reading a Git index file's header,
/// entries,
/// extensions and checksum length.
#[cfg(target_os = "linux")]
pub mod subvolume_index_bytes;

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

/// The stat fields an index entry stores,
/// read from a file and compared with what an entry recorded.
#[cfg(target_os = "linux")]
pub mod subvolume_index_file_stat_fields;

/// An index with the stat data of named paths zeroed,
/// so Git hashes those files instead of trusting them.
#[cfg(target_os = "linux")]
pub mod subvolume_index_file_zeroed_stat;

/// What a refresh of one index is told,
/// how it ended,
/// and the repository's hash length.
#[cfg(target_os = "linux")]
pub mod subvolume_index_file_options;

/// The rewrite of one index's stat fields and the install that replaces the index with it.
#[cfg(target_os = "linux")]
pub mod subvolume_index_file_rewrite_and_install;

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

/// What a copy of one entry could not carry,
/// counted by kind.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs_losses;

/// The extended attributes copied onto a copy,
/// removed from it and compared with the original's.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs_extended;

/// The owner,
/// permission bits and times of a copy,
/// set in the order that keeps them.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs_owner_and_times;

/// Opening and creating the named pipes of a copied tree.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs_fifo;

/// Copying owner,
/// permissions,
/// times and extended attributes onto a copy.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_attrs;

/// The walk of one directory of a tree pass,
/// and the test-only callback a control installs at it.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_directory_visit;

/// The dispatch of one source child by what it is.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_source_child;

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

/// The bookkeeping of a file with several names in a tree pass:
/// whether a destination file is one of the recorded copies,
/// recording a copy,
/// and linking the next name to an earlier copy.
#[cfg(target_os = "linux")]
pub mod subvolume_sync_copy_link_record;

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

/// The look that finds every volume below a tree without changing it.
pub mod subvolume_remove_around_look;

/// The look that finds every volume below a tree without changing it, and the pass that empties a tree around
/// each one it did not make.
pub mod subvolume_remove_around;

/// The boundary a removal may not cross,
/// and the words of the refusal it gives.
pub mod subvolume_remove_walk_boundary_and_refusal;

pub mod subvolume_remove_walk;

/// The moment a removal has read a directory's entries and not yet removed them, for a control that acts there.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_remove_listing_test_support;

/// The names a killed journal write may leave beside the journal.
#[cfg(target_os = "linux")]
pub mod subvolume_journal_leftover_names;

/// The journal's durable writes:
/// one file written exclusively and renamed over its target,
/// and the error for a journal that cannot be understood.
#[cfg(target_os = "linux")]
pub mod subvolume_journal_writes;

/// The migration journal beside the checkout.
#[cfg(target_os = "linux")]
pub mod subvolume_journal;

/// The count of the entries under a checkout,
/// from which the free space a migration needs follows.
#[cfg(target_os = "linux")]
pub mod subvolume_entry_count;

/// The paths of a migration and the free space its metadata needs.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight_paths;

/// One refusal and the two texts it is built from.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight_refusal;

/// The refusals about Git's own state in a checkout and its linked worktrees.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight_git_state_refusals;

/// The refusals that stop a fresh or resumed migration before anything is created.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight_migration_refusals;

/// The refusals of a rollback and of deleting the old tree.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight_rollback_refusals;

/// The paths of a migration and the refusals that stop it before anything is created.
#[cfg(target_os = "linux")]
pub mod subvolume_preflight;

/// The state a migration command carries between its steps.
#[cfg(target_os = "linux")]
pub mod subvolume_run;

/// What a person asked a migration to do,
/// and how one ended.
#[cfg(target_os = "linux")]
pub mod subvolume_migrate_options;

/// The steps that change the checkout:
/// create,
/// the index locks,
/// the exchange,
/// and the finish.
#[cfg(target_os = "linux")]
pub mod subvolume_migrate_steps;

/// The lines a migration says:
/// the facts the run begins with,
/// and the closing text with everything left to do.
#[cfg(target_os = "linux")]
pub mod subvolume_migrate_words;

/// Converting a main checkout into a btrfs subvolume,
/// resumably.
#[cfg(target_os = "linux")]
pub mod subvolume_migrate;

/// The refusals a rollback and a deletion of the old tree answer with.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback_refusals;

/// Carrying the person's work back out of the new subvolume.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback_carry_back;

/// Deleting the old tree once the person asks for it.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback_removal;

/// The exchange that turns the checkout back into the original tree.
#[cfg(target_os = "linux")]
pub mod subvolume_rollback_swap;

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

/// A stand-in `Platform` on a plain directory tree.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_test_support_plain_platform;

/// A stand-in platform that can fail a clone and every hard link.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_test_support_hooked_platform;

/// A stand-in platform that answers that it flushes its writes.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_test_support_flushing_platform;

/// The fixture checkouts and run helpers of the subvolume migration controls.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_test_support_fixtures;

/// The stand-in platform and fixture checkouts of the subvolume migration controls.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod subvolume_test_support;

/// The migration controls that need the real btrfs ioctls,
/// `FICLONE` and exchange.
#[cfg(all(test, target_os = "linux"))]
mod subvolume_btrfs_tests;
