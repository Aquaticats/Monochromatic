//! What:
//!  The spellings of the wrapper-only escape hatches several modules share.
//! Why:
//!  A command module puts its own hatch first in the flag list it hands the option
//!      tokenizer,
//!  and the control table lists the same spellings;
//!  one constant per
//!      spelling keeps both in step.
//!  Removal is not done here:
//!  `wrapper_invocation.rs`
//!      deletes wrapper tokens by the positions the tokenizer reports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const WORKTREE_ENFORCEMENT_ESCAPE_HATCH = '--no-enforce-worktree';
//! ```

/// What:
///  Wrapper-only flag suppressing linked-worktree enforcement for one invocation.
///       `&str` is borrowed text;
///  here it is baked into the program.
/// Why:
///   `git stash`,
///  `git clean` and `git reset` name it as their own hatch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WORKTREE_ENFORCEMENT_ESCAPE_HATCH = '--no-enforce-worktree';
/// ```
pub const WORKTREE_ENFORCEMENT_ESCAPE_HATCH: &str = "--no-enforce-worktree";

/// Wrapper-only flag skipping ignored-state synchronization into new worktrees.
pub const WORKTREE_COPY_ESCAPE_HATCH: &str = "--no-worktree-copy";

/// Wrapper-only flag suppressing worktree-first branch creation enforcement for one
/// `branch`,
///  `checkout` or `switch` invocation.
pub const BRANCH_WORKTREE_ESCAPE_HATCH: &str = "--no-enforce-worktree-branch";
