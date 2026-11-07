//! What:
//!  The number of paths one Git command of the candidate layer names at a time.
//! Why:
//!  The prediction,
//! the attribute query and the commit listing all split a long path list into commands of this
//!  size,
//! so the number lives in a file of its own that none of them has to import from another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const PATHSPEC_CHUNK_SIZE = 2_048;
//! ```

/// What:
///  The most pathspecs one `git diff-index` receives.
///  `usize` is the type of counts.
/// Why:
///   The installed wrapper passes candidate paths 2,048 at a time
///       (`commit-transaction-candidate-batch.ts`),
///  far below any platform's argument
///       limit;
///  one process per 2,048 changed paths keeps the count bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PATHSPEC_CHUNK_SIZE = 2_048;
/// ```
///  Exempt from a citation under `LQM` (class:
///  chunk size):
///  it splits one command into several starts of the same
///  command,
///  so the result is the same and only the number of starts changes.
///  Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
///  "Exempt limits";
///  the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub const PATHSPEC_CHUNK_SIZE: usize = 2_048;
