//! What:
//!  Conflict handling around commits and checkouts:
//!  side flags,
//!  saved copies of conflicted files and leftover markers.
//! Why:
//!  These modules answer one question,
//!  what happens to a file that has conflict state.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `abort_discard_guard.rs`:
//!   the linked-worktree policy's carve-out for the abort and skip forms during an operation.
//! - `autostash_carry.rs`:
//!   the stash and conflict stages Git's apply of an autostash left in the commit's scratch repository,
//!   carried back.
//! - `command_conflict_side.rs`:
//!   the `checkout` and `restore` command lines that take one side of a conflicted file.
//! - `conflict_markers.rs`:
//!   the marker lines a committed file still holds,
//!   and the line about an operation a commit left open.
//! - `conflict_side_modes.rs`:
//!   the pathspec modes and their spellings,
//!   which decide whether a token names one file.
//! - `conflict_side_source.rs`:
//!   finding and copying the file a command replaces.
//! - `conflict_side_state.rs`:
//!   the operation in progress and the conflicted paths.
//! - `conflict_side_guard.rs`:
//!   the linked-worktree policy's reading of a `checkout` or `restore` that would replace a conflicted file,
//!   and the refusal that names every file it would replace.
//! - `discard_audit_facts.rs`:
//!   the named facts one audit record carries.
//! - `discard_audit_pins.rs`:
//!   whether one audit generation names a saved copy still on disk,
//!   which keeps it past the bound every log shares.
//! - `discard_audit.rs`:
//!   the append-only record of every command form that can discard uncommitted state.
//! - `abort_recheck.rs`:
//!   the second measurement of an abort or a skip,
//!   owed immediately before Git starts.
//! - `discard_audit_record.rs`:
//!   everything one audit record states,
//!   and its line as JSON.
//! - `discarded_prune_limits.rs`:
//!   the two limits of the folder of saved copies,
//!   their defaults and the reading of a size such as `256m`.
//! - `discarded_store_records.rs`:
//!   the plain values of the descriptor-based reader of the folder of saved copies.
//! - `discarded_store.rs`:
//!   the folder of saved copies opened once without following a link,
//!   with every read and removal addressed through descriptors.
//! - `discarded_prune_stamps.rs`:
//!   the time stamps in the names of saved-copy directories.
//! - `discarded_prune_plan.rs`:
//!   what a pass removes and why.
//! - `discarded_prune_audit_line.rs`:
//!   the audit line one removed directory leaves.
//! - `discarded_prune_problems.rs`:
//!   what can go wrong in a pass over the folder,
//!   the words that say it and the listing.
//! - `conflict_sides.rs`:
//!   the saved copy of each file a command replaces.
//! - `discarded_nested_repository.rs`:
//!   the move of the nested Git repositories a clean would delete beside the audit log.
//! - `discarded_prune_listing.rs`:
//!   the folder of saved copies read as it stands:
//!   every directory named like the wrapper's own,
//!   oldest first,
//!   with its size and identity and what could not be read.
//! - `discarded_prune_removal.rs`:
//!   removing one directory a pass chose,
//!   and the audit line that says what went,
//!   with the outcome a pass reports.
//! - `discarded_prune.rs`:
//!   the bound on the folder of saved copies of conflicted files:
//!   age,
//!   size,
//!   audit and doctor report.
//! - `config_discarded.rs`:
//!   the `discardedCopies` section of `monochromatic-git-policy-cli.config.jsonc`.
//! - `discarded_report.rs`:
//!   what the folder of saved copies holds,
//!   and the text the doctor prints for it,
//!   read without moving anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // conflict/index.ts: one `export * from './file'` per module below.
//! ```

/// The `checkout` and `restore` command lines that take one side of a conflicted file.
pub mod command_conflict_side;

/// Finding and copying the file a command replaces.
pub mod conflict_side_source;

/// The operation in progress and the conflicted paths.
pub mod conflict_side_state;

/// The pathspec modes and their spellings, which decide whether a token names one file.
pub mod conflict_side_modes;

/// The saved copy of each file a command replaces.
pub mod conflict_sides;

/// The move of the nested Git repositories a clean would delete beside the audit log.
pub mod discarded_nested_repository;

/// The linked-worktree policy's reading of a `checkout` or `restore` that would replace a conflicted file,
///  and the refusal that names every file it would replace.
pub mod conflict_side_guard;

/// The linked-worktree policy's carve-out for the abort and skip forms during an operation.
pub mod abort_discard_guard;

/// The second measurement of an abort or a skip,
/// owed immediately before Git starts.
pub mod abort_recheck;

/// The append-only record of every command form that can discard uncommitted state.
pub mod discard_audit;

/// The named facts one audit record carries.
pub mod discard_audit_facts;

/// Everything one audit record states,
/// and its line as JSON.
pub mod discard_audit_record;

/// Whether one audit generation names a saved copy still on disk,
/// which keeps it past the bound every log shares.
pub mod discard_audit_pins;

/// The `discardedCopies` section of `monochromatic-git-policy-cli.config.jsonc`.
pub mod config_discarded;

/// The folder of saved copies read as it stands:
/// every directory named like the wrapper's own,
/// oldest first,
/// with its size and identity and what could not be read.
pub mod discarded_prune_listing;

/// Removing one directory a pass chose,
/// and the audit line that says what went.
pub mod discarded_prune_removal;

/// The bound on the folder of saved copies of conflicted files:
/// age,
/// size,
/// audit and doctor report.
pub mod discarded_prune;

/// What the folder of saved copies holds,
/// and the text the doctor prints for it,
/// read without moving anything.
pub mod discarded_report;

/// The audit line one removed directory leaves.
pub mod discarded_prune_audit_line;

/// The two limits of the folder of saved copies,
/// their defaults and the reading of a size such as `256m`.
pub mod discarded_prune_limits;

/// What a pass removes and why.
pub mod discarded_prune_plan;

/// What can go wrong in a pass over the folder,
/// the words that say it and the listing.
pub mod discarded_prune_problems;

/// The time stamps in the names of saved-copy directories.
pub mod discarded_prune_stamps;

/// The folder of saved copies opened once without following a link,
/// with every read and removal addressed through descriptors.
pub mod discarded_store;

/// The plain values of the descriptor-based reader of the folder of saved copies.
pub mod discarded_store_records;

/// The marker lines a committed file still holds,
/// and the line about an operation a commit left open.
pub mod conflict_markers;

/// The stash and conflict stages Git's apply of an autostash left in the commit's scratch repository,
/// carried back.
pub mod autostash_carry;
