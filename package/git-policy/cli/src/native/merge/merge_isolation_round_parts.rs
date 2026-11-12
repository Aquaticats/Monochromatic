//! What:
//!  The two decisions an isolated merge's round makes about what it leaves behind:
//!  the capture of the person's own work,
//!  and the words for a merge whose editor wrote no message.
//! Why:
//!  Both are made once per round and used nowhere else,
//! so they sit in a file of their own and keep `merge_isolation.rs`,
//! which decides the round's order,
//!  inside the package's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const captured = captureFor({ context, root, overlap }); ... editorStopped(reference, refusal)
//! ```

/// The person's own work in the paths the merge changes,
/// and the ref that keeps it.
use crate::merge_isolation_carry::{capture_work, keep_capture};
/// The person's work as the record keeps it.
use crate::merge_isolation_carry_parts::Capture;
/// Why a merge's editor left no message.
use crate::merge_isolation_editor::EditorRefusal;
/// Real Git.
use crate::transaction_git::GitContext;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The person's own work in `paths`,
/// captured as objects and kept on a ref of its own.
/// Why:
///   A ref written before the record and before the checkout changes is what makes the bytes survive a kill at any
///  instant (`SPEC.md`,
/// "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureFor(context, root, paths): Promise<Capture | null>;
/// ```
pub(crate) fn capture_for(context: &GitContext, root: &Path, paths: &[Vec<u8>]) -> Result<Option<Capture>, String> {
    if paths.is_empty() {
        return Ok(None);
    }
    let capture: Capture = capture_work(context, root, paths)?;
    return Ok(Some(keep_capture(context, root, &capture)?));
}

/// What:
///  The words for a merge whose editor left no message.
/// Why:
///   Everything the command would have changed is still as it was,
/// so the outcome is the person's message to rewrite,
///  and the sentence says both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editorStopped(reference: string, refusal: EditorRefusal): string;
/// ```
pub(crate) fn editor_stopped(reference: &str, refusal: &EditorRefusal) -> String {
    let reason: &str = match refusal {
        EditorRefusal::Stopped(detail) => detail.as_str(),
        EditorRefusal::Empty => "the editor left an empty message",
        EditorRefusal::NoEditor => "Git resolved no editor for this environment",
    };
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: {reason}, so the merge was not made and nothing in the repository changed. \
         Run the merge again to write the message.\n"
    );
}
