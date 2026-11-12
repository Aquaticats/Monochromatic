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
