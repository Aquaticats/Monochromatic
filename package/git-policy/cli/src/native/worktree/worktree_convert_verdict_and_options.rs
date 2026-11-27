//! What:
//!  What one conversion answers,
//! and what the command asks of it.
//! Why:
//!   The command prints one line per worktree and maps the verdicts to its exit status,
//! and the dry run and the abandon are modes of the same pass over the same facts,
//! so both are read by every part of a conversion.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Verdict = 'converted' | 'resumed' | 'would-convert' | 'already-subvolume' | 'abandoned' | { skipped: string[] };
//! type ConvertOptions = { dryRun: boolean; abandon: boolean; commandLineAttributes: boolean };
//! ```

/// What:
///  What a run did with one worktree.
/// Why:
///   The command prints one line per worktree and maps the verdicts to its exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = 'converted' | 'resumed' | 'would-convert' | 'already-subvolume' | 'abandoned' | { skipped: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The conversion ran from the start to the end.
    Converted,
    /// An unfinished conversion was completed.
    Resumed,
    /// A dry run found nothing in the way.
    WouldConvert,
    /// The path is a subvolume already.
    AlreadySubvolume,
    /// An unfinished conversion was undone before its swap.
    Abandoned,
    /// Something stands in the way;
    /// each reason is a sentence.
    Skipped(Vec<String>),
}

/// What:
///  What the command asks of one worktree.
/// Why:
///   The dry run and the abandon are modes of the same pass over the same facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertOptions = { dryRun: boolean; abandon: boolean; commandLineAttributes: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConvertOptions {
    /// List what would happen and change nothing.
    pub dry_run: bool,
    /// Undo an unfinished conversion that has not reached its swap.
    pub abandon: bool,
    /// The invocation chose an attribute source on the command line (`--attr-source`,
    /// `-c attr.tree`),
    /// which the worktree
    ///  files read here do not model.
    pub command_line_attributes: bool,
}
