//! What:
//!  The question a removal's caller asks again as the removal runs,
//!  immediately before anything is destroyed,
//!  and the printing of the request that carries it.
//! Why:
//!   A removal deletes what its caller proved it may delete,
//!  and that proof takes time:
//!  it starts Git and walks the tree.
//!  What the caller read before it may have changed by now,
//!  and a worktree added inside the one being removed,
//!  or a record written into its Git directory,
//!  would go with it.
//!  This is the last moment the caller can look,
//!  so nothing but the destroy follows its answer.
//!
//! The rule, and it holds on every route:
//!  **the last look is the last thing the wrapper does before the destroy.**
//!  A step that is not the destroy itself stands before the look, however late it seems:
//!  a listing taken after the look is the stale side of the race the look exists for,
//!  and work the wrapper does between the look and the delete is work nobody asked the guard about.
//!  The destroy step itself is the destroy and its own journal lines,
//!  which must be written immediately before the kernel call or the rename:
//!  the line that says the destroy is under way so a killed command's next run can finish it,
//!  and, on the route that destroys here, the line that says the files left the path.
//!  Written any earlier, a refusal would leave a record claiming a deletion that never happened.
//!
//! What stands between each route's last look and its destroy, before and after this rule:
//!
//! - The typed subvolume removal (`worktree_remove.rs::remove_subvolume_worktree`).
//!   Before: the last look, then the deletion listing (`RemovalDeletions::of`, two Git queries), the
//!   `begun` line of the deletion record, the destroy's journal line, the hand-over to the commit service (a rename that
//!   takes the files off the path), and, when the hand-over fails, the mark that the files leave and the kernel call.
//!   After: the listing and its `begun` line stand before the look, and the look is followed by the destroy's journal
//!   line and then by the rename or the kernel call, and by nothing else.
//! - The commit service's removal (`worktree_remove_hand_over.rs::remove_the_worktree_of`).
//!   Before: the look, then the listing, the `begun` line, and real Git.
//!   After: the listing and its `begun` line stand before the look, and real Git is started immediately after it.
//! - The person's plain forwarded removal (`worktree_copy_forward.rs::run_served`).
//!   Before: the look, then the lifecycle (`run_settled`), which recovers unfinished copies, settles whatever it finds and
//!   only then starts the person's Git (`ForwardedGit::run`).
//!   After: the look is asked inside that step, after the recovery and immediately before the child is started, and a
//!   refusal starts nothing.
//! - The removal that goes around a volume it did not make (`worktree_remove_around.rs::go_around`), reached from the
//!   typed route after its look: what it deletes is decided by its own walk of the tree, and the look is the last thing
//!   before that walk.
//! - The landed sweep (`worktree_landed_removal.rs`): it destroys through the typed subvolume route or hands the removal to
//!   the service, so each of those routes' order is the order it gets, and its own look (`NothingNewWouldGoWithIt`) is the
//!   one the subvolume route asks.
//!
//! What no route can close is real Git's own run:
//!  between the wrapper's answer and Git's first unlink stand one process start and Git's own reading of the worktree,
//!  which for `git worktree remove` is `git status --porcelain --ignore-submodules=none` inside it
//!  (Git v2.56.0,
//!  `builtin/worktree.c`):
//!  a modified or untracked file that status sees is refused by Git itself with its own words,
//!  and a nested worktree under a name the outer worktree ignores is not in that status, which is why the look, not Git,
//!  is what keeps it.
//!  The question is a trait rather than a closure,
//!  because a closure a caller captures its own state in is what this codebase keeps out of its calls.
//!  It is its own file because `worktree_remove.rs` is at the crate's line ceiling,
//!  and because the trait and the printing of a request that carries one are read together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface LastLook { ask(): void } // throws the reason nothing is removed
//! beforeTheDestroy(request, target); // the reason, or nothing
//! ```

use super::{RemoveRequest, TAG};
use crate::diagnostic_log::debug;
use std::path::Path;

/// What:
///  A question a removal's caller asks again as the removal runs,
///  immediately before anything is destroyed.
/// Why:
///   See this module's description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface LastLook { ask(): void }
/// ```
pub trait LastLook {
    /// The question,
    /// which answers the reason nothing is removed.
    fn ask(&self) -> Result<(), String>;
}

impl std::fmt::Debug for RemoveRequest<'_> {
    /// A request prints the directory it works on and whether its caller asks a question again;
    /// a question itself is not text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter
            .debug_struct("RemoveRequest")
            .field("plan", &self.plan)
            .field("common_dir", &self.common_dir)
            .field("last_look", &self.last_look.is_some())
            .finish();
    }
}

/// What:
///  The answer of the caller's own last look,
///  asked after the proof of cleanliness,
///  or nothing when the caller asks no question.
/// Why:
///   The proof starts Git and walks the tree,
///  so a caller that read what a removal deletes before it read too early:
///  this is the last moment before the destroy at which it can be asked,
///  and a refusal changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeTheDestroy(request, target) // throws the caller's reason
/// ```
pub(super) fn before_the_destroy(request: &RemoveRequest<'_>, target: &Path) -> Result<(), String> {
    let Some(look) = request.last_look else { return Ok(()) };
    let answer: Result<(), String> = look.ask();
    if let Err(reason) = answer.as_ref() {
        debug(TAG, format!("{} is no longer one this removal may destroy ({reason}); nothing is removed", target.display()).as_str());
    }
    return answer;
}
