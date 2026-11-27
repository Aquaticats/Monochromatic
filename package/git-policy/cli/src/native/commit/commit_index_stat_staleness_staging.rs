//! What:
//!  What a commit stages from the worktree,
//!  as the stat-data question reads it.
//! Why:
//!  `git add` writes the stat data of every path it stages into the private index afresh,
//!  so the stat data the real index records for those paths cannot cost the scratch commit a read,
//!  and the question has to know which paths they are.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type WorktreeStaging = { named: string[] } | { none: true };
//! ```

/// What a commit stages from the worktree,
/// so its recorded stat data cannot cost the scratch commit a read.
pub enum WorktreeStaging<'a> {
    /// The positional pathspecs of an explicit-path commit,
    /// exactly as the command line gave them.
    Named(&'a [std::ffi::OsString]),
    /// Nothing:
    /// the commit takes its content from the index or from the merged tree.
    None,
}
