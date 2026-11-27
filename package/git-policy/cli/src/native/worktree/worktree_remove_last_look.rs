//! What:
//!  The last look a removal's caller is asked for again as the removal runs,
//!  immediately before anything is destroyed.
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
//!  The question itself,
//!  and the request that carries it,
//!  are in `worktree_remove_request_and_last_look.rs`;
//!  this file is the one step of the removal that asks it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! beforeTheDestroy(request, target); // the reason, or nothing
//! ```

// What:     `use crate::worktree_remove_request_and_last_look::{RemoveRequest, TAG};`
//           imports the request this step reads and the name its log line carries.
// Why:      The step asks the request's own question, so it needs the request and
//           nothing else of the removal's other steps.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemoveRequest } from './worktree_remove_request_and_last_look';
// ```
use crate::diagnostic_log::debug;
use crate::worktree_remove_request_and_last_look::{
    RemoveRequest,
    TAG,
};
use std::path::Path;

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
pub(super) fn before_the_destroy(
    request: &RemoveRequest<'_>,
    target: &Path,
) -> Result<(), String> {
    let Some(look) = request.last_look else {
        return Ok(());
    };
    let answer: Result<(), String> = look.ask();
    if let Err(reason) = answer.as_ref() {
        debug(
            TAG,
            format!(
                "{} is no longer one this removal may destroy ({reason}); nothing is removed",
                target.display()
            )
            .as_str(),
        );
    }
    return answer;
}
