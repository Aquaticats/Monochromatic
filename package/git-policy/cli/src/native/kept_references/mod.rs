//! What:
//!  The references the wrapper writes to keep a commit reachable,
//!  and the pruning that keeps them from accumulating for ever:
//!  the names that count as one,
//!  the record of when and for which worktree each was written,
//!  the limits,
//!  the one reading of the repository they are judged from,
//!  the pass that removes the ones no longer worth their objects,
//!  the line it leaves for what went,
//!  and the listing a person runs.
//! Why:
//!  A commit the wrapper built and could not put on a branch is kept on
//!  `refs/cli-git/kept/<batch>` (and `refs/cli-git/kept/<batch>-rescue` beside it),
//!  and nothing removed them afterwards,
//!  so the objects they hold grew without end
//!  (the owner's answer of 2026-11-12:
//!  "Keep the reference,
//! auto prune by size and age").
//!  The pass runs where no command waits for it
//!  (`kept_reference_pass.rs`),
//!  and every removal is the same one the runner's own words describe,
//!  which `doc/decision/` never had to name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // kept_references/index.ts: one `export * from './file'` per module below.
//! ```

/// Which reference names count as one the wrapper keeps a commit on,
/// and the batch it carries.
pub mod kept_reference_name;

/// The age,
/// the size and the floor a kept reference is judged by,
/// with the measurement each value rests on,
/// and the variables that change them.
pub mod kept_reference_limits;

/// What was written beside a kept reference when it was written:
/// when,
/// for which worktree,
/// for which batch.
pub mod kept_reference_record;

/// One reading of a repository's kept references:
/// which they are,
/// what each names,
/// which of their commits a branch has since reached,
/// and what the objects only they reach weigh.
pub mod kept_references_reading;

/// The pass:
/// what the limits make of one reading,
/// the conditional removal of each reference,
/// and the record and the log line that go with it,
/// with the answers it gives in a file of their own.
pub mod kept_reference_pass;

/// The answers one pass gives about a repository's kept references:
/// the reasons a reference goes or stays,
/// and the entry a person reads for each.
pub mod kept_reference_pass_answers;

/// The line one pruned kept reference leaves in the repository's log.
pub mod kept_reference_log;

/// The words a span of time is printed in,
/// which the listing and the log line both use.
pub mod kept_reference_words;

/// The text `git monochromatic-git-policy-cli kept-references` prints.
pub mod kept_reference_listing;

/// The command that prints the listing,
/// and the one that runs a pruning when a person asks.
pub mod kept_reference_command;
