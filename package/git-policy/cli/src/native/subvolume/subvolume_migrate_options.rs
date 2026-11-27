//! What:
//!  What a person asked a migration to do and how one ended:
//! the two switches and the dry run,
//! and the four verdicts a finished command gives.
//! Why:
//!  The command maps each verdict to an exit status and a closing text,
//! so the two types read together,
//! apart from the steps that read the options (`subvolume_migrate_steps.rs`)
//! and the words said about them (`subvolume_migrate_words.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MigrateOptions = { dryRun: boolean; nestedWorktrees: boolean; refreshIndexes: boolean };
//! type Verdict = { kind: 'migrated' } | { kind: 'already' } | { kind: 'dry-run' } | { kind: 'refused'; refusals: Refusal[] };
//! ```

use crate::subvolume_preflight::Refusal;

/// What:
///  What the person asked for the migration to do.
/// Why:
///   The two switches are the script's `--no-nested-worktrees` and `--no-refresh-indexes`;
/// the dry run previews.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MigrateOptions = { dryRun: boolean; nestedWorktrees: boolean; refreshIndexes: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrateOptions {
    /// Preview only:
    /// nothing is created,
    /// written or journaled.
    pub dry_run: bool,
    /// Create the linked worktrees inside the checkout as nested subvolumes.
    pub nested_worktrees: bool,
    /// Rewrite the Git index stat fields after the exchange.
    pub refresh_indexes: bool,
}

/// What:
///  How a migration command ended,
/// when it did not fail.
/// Why:
///   The command maps each to an exit status and the closing text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'migrated' } | { kind: 'already' } | { kind: 'dry-run' } | { kind: 'refused'; refusals: Refusal[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The checkout is now a subvolume and verified.
    Migrated,
    /// The checkout already was one (with a finished journal or none).
    AlreadyDone,
    /// The preview was printed.
    DryRun,
    /// Nothing was changed:
    /// these conditions stand in the way.
    Refused(Vec<Refusal>),
}
