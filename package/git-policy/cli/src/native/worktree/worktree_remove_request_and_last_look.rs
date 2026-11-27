//! What:
//!  What one removal is asked to do,
//!  and one question its caller answers while it runs.
//! Why:
//!  Two subjects read together and named by every file of the removal:
//!  the request the command hands in,
//!  which carries the repository the person invoked,
//!  the plan of the directory they named,
//!  and the last look the caller asks for again before anything is destroyed.
//!  They are one file apart from `worktree_remove.rs` because that file is at
//!  the crate's length limit,
//!  and because the around pass, the hand-over and the last look itself all name
//!  the request rather than the steps it starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RemoveRequest = { context: GitContext; plan: RemovePlan; commonDir: string; lastLook?: LastLook };
//! interface LastLook { ask(): void } // throws the reason nothing is removed
//! ```

// What:     `use crate::transaction_git::GitContext;` imports the type a request
//           carries for starting real Git, from the file that owns it.
// Why:      The cleanliness proof starts Git, so a request names the repository
//           that was invoked and nothing of this file can be written without it.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { GitContext } from '../transaction/git_context';
// ```
use crate::transaction_git::GitContext;

// What:     `use crate::worktree_command_facts::RemovePlan;` imports the record of
//           the one `git worktree remove` this plan serves: the target, the operand
//           and whether the person forced it.
// Why:      A request carries the plan beside the repository, and both are read by
//           every step of the removal.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemovePlan } from './worktree_command_facts';
// ```
use crate::worktree_command_facts::RemovePlan;

// What:     `use std::path::Path;` imports the borrowed path type.
// Why:      The common Git directory a request carries is lent, never owned:
//           the caller keeps the path and only the removal reads it.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — a path is a string, and nothing is lent or owned.
// ```
use std::path::Path;

/// What:
///  `pub(crate) const TAG: &str = "worktreeRemove"` is the name every log line of
///  the removal subject carries.
/// Why:
///  A diagnostic is read beside the others of its subject,
///  and the subject was split over several files,
///  so the tag stays one word wherever a line of it is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TAG = "worktreeRemove";
/// ```
pub(crate) const TAG: &str = "worktreeRemove";

/// What:
///  `pub trait LastLook` is the question a removal's caller asks again as the
///  removal runs, immediately before anything is destroyed.
///  `ask` answers the reason nothing is removed, or nothing.
/// Why:
///  A removal deletes what its caller proved it may delete, and that proof takes
///  time: it starts Git and walks the tree.
///  What the caller read before may have changed by then,
///  and a worktree added inside the one being removed,
///  or a record written into its Git directory,
///  would go with it.
///  The question is a trait rather than a closure,
///  because a closure a caller captures its own state in is what this codebase
///  keeps out of its calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface LastLook { ask(): void } // throws the reason nothing is removed
/// ```
pub trait LastLook {
    /// The question,
    /// which answers the reason nothing is removed.
    fn ask(&self) -> Result<(), String>;
}

/// What:
///  `pub struct RemoveRequest<'a>` is what one removal is asked to do and in which
///  repository: the `GitContext` to start Git with, the `RemovePlan` of the
///  directory named, the canonical common Git directory of the invoked repository,
///  and the caller's `LastLook` when it asks one.
///  The `'a` is the lifetime of every part of it, because a request only lends them.
/// Why:
///   The repository that was invoked decides which registrations count;
///  the worktree's own pointers never do.
///  `#[derive(Clone, Copy)]` lets a step hand the same request on without moving it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveRequest = { context: GitContext; plan: RemovePlan; commonDir: string; lastLook?: LastLook };
/// ```
#[derive(Clone, Copy)]
pub struct RemoveRequest<'a> {
    /// How to start real Git for the cleanliness proof.
    pub context: &'a GitContext,
    /// The directory the person named.
    pub plan: &'a RemovePlan,
    /// The canonical common Git directory of the repository the command was run in.
    pub common_dir: &'a Path,
    /// The question the caller asks again as the removal runs,
    /// immediately before anything is destroyed,
    /// whose refusal is why nothing is removed.
    pub last_look: Option<&'a dyn LastLook>,
}

// What:     `impl std::fmt::Debug for RemoveRequest<'_>` writes how a request reads
//           in a log line: the plan of the directory, the common directory, and
//           whether the caller asks a question again.
//           `&self` borrows it, and `formatter` is the sink it writes into.
// Why:      A request travels through every step of a removal, and the step that
//           refuses prints it; a question itself is not text, so only whether one
//           stands is written.
// Gotcha:   `&'a dyn LastLook` could not be printed even if it were wanted: the
//           trait answers a refusal and holds nothing a line could show.
//
// In TS you'd write (pseudocode):
// ```ts
// class RemoveRequest { toString() { return `RemoveRequest(${this.plan}, ${this.commonDir}, ${this.lastLook !== undefined})`; } }
// ```
impl std::fmt::Debug for RemoveRequest<'_> {
    /// A request prints the directory it works on and whether its caller asks a question again;
    /// a question itself is not text.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        // What:     `formatter.debug_struct(...)...finish()` builds the printed form
        //           field by field and answers the formatter's own result.
        // Why:      One line per part of the request is what makes a log line read
        //           as the request it printed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return `RemoveRequest { plan: ${plan}, commonDir: ${commonDir}, lastLook: ${lastLook !== undefined} }`;
        // ```
        return formatter
            .debug_struct("RemoveRequest")
            .field("plan", &self.plan)
            .field("common_dir", &self.common_dir)
            .field("last_look", &self.last_look.is_some())
            .finish();
    }
}
