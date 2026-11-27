//! What:
//!  The prefix every private stage container's name begins with.
//! Why:
//!  Four readers ask about one stage name and must answer the same way:
//!  the selection refuses a path that names a stage,
//!  the prune rule leaves another command's stage container out,
//!  the sweeps and the records recognize the container a claim names,
//!  and the journal's validator refuses a stage path written any other way.
//!  The prefix sits below all of them so none of them has to use another reader to spell it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const STAGE_PREFIX = '.cli-git-worktree-copy-';
//! ```

/// What:
///  The prefix of the name of every stage container a copy makes beside its destination.
/// Why:
///   The copy reserves these names:
/// a selected path that holds one would copy the copy into itself,
/// and every sweep tells this copy's stage from an unrelated folder by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAGE_PREFIX = '.cli-git-worktree-copy-';
/// ```
pub const STAGE_PREFIX: &str = ".cli-git-worktree-copy-";
