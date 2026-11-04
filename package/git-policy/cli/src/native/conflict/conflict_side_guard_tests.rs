//! What:
//!  The sentences a rejected conflict-side command gives when its copy could not be made.
//! Why:
//!  Each refusal must name the affected path,
//! say in plain words why,
//! and say what the person can do instead;
//! the advice
//!  differs for a refused system call,
//! a link or file in the way,
//! and a file that changed while it was copied.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(copyFailedText({ subject: 'dir', reason: 'is a symbolic link', kind: 'ancestor' })).toContain('Move `dir`');
//! ```

use super::{FailureKind, SaveFailure, copy_failed_text};

/// A failure of `kind` on `subject`.
fn failure(kind: FailureKind, subject: &str, reason: &str) -> SaveFailure {
    return SaveFailure { subject: String::from(subject), reason: String::from(reason), kind };
}

/// Every kind names its subject and its reason,
/// says nothing was changed,
/// and offers the deliberate override.
#[test]
fn every_failure_names_the_path_the_reason_and_what_to_do() {
    for (kind, subject, reason, advice) in [
        (FailureKind::Io, "a.rs", "No space left on device", "Free the space"),
        (FailureKind::Ancestor, "dir", "is a symbolic link", "Move `dir` out of the way"),
        (FailureKind::Changed, "a.rs", "changed while it was copied", "Run it again once nothing is writing to the file"),
    ] {
        let text: String = copy_failed_text(&failure(kind, subject, reason));
        assert!(text.contains(subject) && text.contains(reason), "{kind:?}: {text}");
        assert!(text.contains(advice), "{kind:?}: {text}");
        assert!(text.contains("nothing was changed") && text.contains("--no-enforce-worktree"), "{kind:?}: {text}");
    }
}
