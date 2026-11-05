//! What:
//!  Copying the ignored state of a linked worktree into a new one,
//!  with staging,
//!  journal and recovery.
//! Why:
//!  The copy is a transaction of its own with many small steps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // worktree_copying/index.ts: one `export * from './file'` per module below.
//! ```

/// The one failure type of the linked-worktree ignored-state copy and its diagnostics.
pub mod worktree_copy_error;

/// Which linked worktrees a worktree-creating Git command registered, before and after it ran.
pub mod worktree_copy_registry;

/// The ignored source paths a linked-worktree copy selects, and the path validation every selection passes.
pub mod worktree_copy_selection;

/// What a copy from a main checkout leaves out: caches, optionally run records, and other checkouts.
pub mod worktree_copy_prune;

/// The deterministic parent-first manifest of a copy, and the mode application to a staged copy.
pub mod worktree_copy_manifest;

/// Exact comparison of manifest entries against the filesystem, and the source stability check after staging.
pub mod worktree_copy_compare;

/// The private, validated stage of one source's ignored state beside a destination worktree.
pub mod worktree_copy_stage;

/// The copying of a staged snapshot's files and links into the stage, one worker per slice of the manifest.
#[cfg(unix)]
pub mod worktree_copy_stage_leaf_copying;

/// The exclusive creation of one destination entry from the stage, and the identity captured after it.
#[cfg(unix)]
pub mod worktree_copy_install_entry;

/// Installation of a validated stage into a new worktree, with preflight, batches and rollback on failure.
#[cfg(unix)]
pub mod worktree_copy_install;

/// Removal of the paths a failed installation created, only where their identity and content still prove them cli-git's own.
#[cfg(unix)]
pub mod worktree_copy_rollback;

/// The durable journal of one copy: private storage, atomic writes and reading.
#[cfg(unix)]
pub mod worktree_copy_journal;

/// The exact JSON encoding of a copy journal and the strict validation of every field recovery will act on.
#[cfg(unix)]
pub mod worktree_copy_journal_encoding_and_validation;

/// The removal of a finished or abandoned copy transaction's stage and journal.
#[cfg(unix)]
pub mod worktree_copy_journal_removal;

/// The durable claim on a stage container's name, and the removal of stages an interrupted copy left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim;

/// The judging of stage claims and the removal of the stages that interrupted copies and adds left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_reaping;

/// The append-only install log of one stage: per-batch intents and creations, with resumption.
#[cfg(unix)]
pub mod worktree_copy_install_log;

/// One journaled installation to an end, resumption from the log, and synchronization of a created worktree.
#[cfg(unix)]
pub mod worktree_copy_transaction;

/// The process that owns one item of a copy transaction, written into the item's name, and the proof that it still runs.
#[cfg(unix)]
pub mod worktree_copy_owner;

/// Recovery of interrupted copies: registration checks, discards with notices, resumption, and the pending check.
#[cfg(unix)]
pub mod worktree_copy_recovery;

/// The linked-worktree ignored-state copy around one forwarded Git invocation: the integrator's entry point.
#[cfg(unix)]
pub mod worktree_copy;

/// Which forwarded invocations the copy applies to, and the child run that performs it.
pub mod worktree_copy_forward;
