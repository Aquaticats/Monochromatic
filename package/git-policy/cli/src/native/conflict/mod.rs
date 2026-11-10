//! What:
//!  Conflict handling around commits and checkouts:
//!  side flags,
//!  saved copies of conflicted files and leftover markers.
//! Why:
//!  These modules answer one question,
//!  what happens to a file that has conflict state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // conflict/index.ts: one `export * from './file'` per module below.
//! ```

/// The `checkout` and `restore` command lines that take one side of a conflicted file.
pub mod command_conflict_side;

/// Finding and copying the file a conflict-side command replaces.
pub mod conflict_side_source;

/// The operation in progress and the conflicted paths.
pub mod conflict_side_state;

/// Whether every token of a command line names a conflicted file.
pub mod conflict_side_decision;

/// The pathspec modes and magic of a conflict-side line.
pub mod conflict_side_modes;

/// The notices of a conflict-side command.
pub mod conflict_side_notice;

/// The saved copy of each file a conflict-side command replaces.
pub mod conflict_sides;

/// The linked-worktree policy's carve-out for conflicted files during a merge,
/// rebase,
/// cherry-pick or revert.
pub mod conflict_side_guard;

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
