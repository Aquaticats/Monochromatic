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

/// Which linked worktrees a worktree-creating Git command registered,
/// before and after it ran.
pub mod worktree_copy_registry;

/// The bytes of one repository path,
/// how its components are joined and compared,
/// and the validation every one passes.
pub mod worktree_copy_repository_path;

/// Whether an entry a walk of a live checkout listed is still there when the walk reaches it,
/// and that its absence is no failure.
pub mod worktree_copy_vanished_entry;

/// The ignored source paths a linked-worktree copy selects,
/// and the path validation every selection passes.
pub mod worktree_copy_selection;

/// What a copy from a main checkout leaves out:
/// caches,
/// optionally run records,
/// and other checkouts.
pub mod worktree_copy_prune;

/// The deterministic parent-first manifest of a copy,
/// and the mode application to a staged copy.
pub mod worktree_copy_manifest;

/// Exact comparison of manifest entries against the filesystem,
/// and the source stability check after staging.
pub mod worktree_copy_compare;

/// The private,
/// validated stage of one source's ignored state beside a destination worktree.
pub mod worktree_copy_stage;

/// The copying of a staged snapshot's files and links into the stage,
/// one worker per slice of the manifest.
#[cfg(unix)]
pub mod worktree_copy_stage_leaf_copying;

/// Settling a staged copy against its source:
/// the passes that copy a changed entry again and leave a vanished one out.
#[cfg(unix)]
pub mod worktree_copy_settling;

/// The exclusive creation of one destination entry from the stage,
/// and the identity captured after it.
#[cfg(unix)]
pub mod worktree_copy_install_entry;

/// The publication of one staged regular file at its destination name in one step:
/// unnamed while it is filled,
/// with the entry's mode before it can be read,
/// and linked into place only when whole.
#[cfg(unix)]
pub mod worktree_copy_publish_file;

/// Installation of a validated stage into a new worktree,
/// with preflight,
/// batches and rollback on failure.
#[cfg(unix)]
pub mod worktree_copy_install;

/// Removal of the paths a failed installation created,
/// only where their identity and content still prove them monochromatic-git-policy-cli's own.
#[cfg(unix)]
pub mod worktree_copy_rollback;

/// The durable journal of one copy:
/// private storage,
/// atomic writes and reading.
#[cfg(unix)]
pub mod worktree_copy_journal;

/// The exact JSON encoding of a copy journal and the strict validation of every field recovery will act on.
#[cfg(unix)]
pub mod worktree_copy_journal_encoding_and_validation;

/// The removal of a finished or abandoned copy transaction's stage and journal.
#[cfg(unix)]
pub mod worktree_copy_journal_removal;

/// The durable claim on a stage container's name,
/// and the removal of stages an interrupted copy left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim;

/// The judging of stage claims and the removal of the stages that interrupted copies left behind.
#[cfg(unix)]
pub mod worktree_copy_stage_claim_reaping;

/// The durable record of the object one stage's creation made:
/// its identity,
/// and the instant the wrapper last wrote into it.
#[cfg(unix)]
pub mod worktree_copy_stage_made_object;

/// The first entry of a stage container that the record of its creation's object cannot account for.
#[cfg(target_os = "linux")]
pub mod worktree_copy_stage_entries_the_record_accounts_for;

/// Whether that record proves the folder standing at a container's name,
/// and the words a sweep says about a stage it keeps or removes.
#[cfg(unix)]
pub mod worktree_copy_stage_reaping_proof_and_words;

/// What an interrupted `git worktree add` settles to,
/// with the proof its checkout is whole or gone.
#[cfg(unix)]
pub mod worktree_copy_interrupted_add_settlement;

/// The registration one staged copy is bound to,
/// recorded beside its payload and checked before every write into a worktree.
#[cfg(unix)]
pub mod worktree_copy_stage_binding;

/// The durable record of a worktree left without part of its source's ignored state,
/// reported by every later command until the worktree is gone or replaced.
#[cfg(unix)]
pub mod worktree_copy_unfilled_record;

/// The append-only install log of one stage:
/// per-batch intents and creations,
/// with resumption.
#[cfg(unix)]
pub mod worktree_copy_install_log;

/// One journaled installation to an end,
/// and resumption from the log.
#[cfg(unix)]
pub mod worktree_copy_transaction;

/// Whose destination a copy may write into,
/// how the reservation answers it,
/// and the line that says so when it may not.
#[cfg(unix)]
pub mod worktree_copy_notices;

/// The staging,
/// journaling and installation of one created worktree's ignored state,
/// bound to the registration real Git made.
#[cfg(unix)]
pub mod worktree_copy_synchronize;

/// What becomes of the destination a refused `git worktree add` left:
/// the empty folder this command made is removed,
/// and one it left with something in it is named on the command's output.
#[cfg(unix)]
pub mod worktree_copy_after_refusal;

/// The process that owns one item of a copy transaction,
/// written into the item's name,
/// and the proof that it still runs.
#[cfg(unix)]
pub mod worktree_copy_owner;

/// Which of a repository's registrations names one destination,
/// read from the administrative directories rather than from the destination's pointer.
#[cfg(unix)]
pub mod worktree_copy_registration_naming;

/// The bound one command holds when it finishes a copy it did not start,
/// and the words a copy stopped at that bound is reported with.
#[cfg(unix)]
pub mod worktree_copy_ceiling;

/// The checkout an interrupted `git worktree add` never finished,
/// finished by a command in the worktree Git registered for it.
#[cfg(unix)]
pub mod worktree_copy_interrupted_checkout;

/// The ignored-state copy an interrupted `git worktree add` still owes,
/// made by a command in the worktree it fills.
#[cfg(unix)]
pub mod worktree_copy_owed_copy;
/// Recovery of interrupted copies:
/// registration checks,
/// discards with notices,
/// resumption,
/// and the pending check.
#[cfg(unix)]
pub mod worktree_copy_recovery;

/// Whether a source worktree is itself the destination of a copy that still runs:
/// a copy stages nothing from it and tells the person why.
#[cfg(unix)]
pub mod worktree_copy_source_still_being_filled;

/// The linked-worktree ignored-state copy around one forwarded Git invocation:
/// the integrator's entry point.
#[cfg(unix)]
pub mod worktree_copy;

/// What is left to do once a created worktree holds its ignored state:
/// its package-manager repair,
/// and the trash a snapshot
///  left there,
/// handed to the commit service.
#[cfg(unix)]
pub mod worktree_copy_after_lock;

/// Which forwarded invocations the copy applies to,
/// and the child run that performs it.
pub mod worktree_copy_forward;
