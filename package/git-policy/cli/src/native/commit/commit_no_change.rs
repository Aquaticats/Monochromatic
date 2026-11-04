//! What:
//!  The normalization-only commit: a correction whose complete intended tree equals the recorded base ends without an empty
//!  commit, and says so.
//! Why:
//!  The preparation (`service_prepare.rs`) decides it before anything is sent to the service; this module holds the finding's
//!  identity and its message, so the words of the finding stay in one place
//!  (`src/policy-engine/commit-transaction-no-change.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const message = noChangeMessage(conflicted);
//! ```

use crate::path_bytes::path_message_text;

/// The core of the no-change finding.
pub const NO_CHANGE_CORE_ID: &str = "commit-normalization";

/// The code of the no-change finding.
pub const NO_CHANGE_CODE: &str = "no-change";

/// What:
///  The message of the no-change finding.
/// Why:
///   Worktree copies that changed since are kept and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hasConflicts ? `No commit created: ... ${JSON.stringify(conflicted)}. Compare them with HEAD before retrying.` : '...'
/// ```
pub fn no_change_message(conflicted: &[Vec<u8>]) -> String {
    if conflicted.is_empty() {
        return String::from(
            "No commit created: normalization removed every selected change; matching worktree and index copies were reconciled. Hint: if you were committing only whitespace or final-newline edits, they were normalized away, so there is nothing to commit; run `git status` to confirm.",
        );
    }
    // The message shows each name as text; a name that is not UTF-8 shows its replacement characters.
    let mut names: Vec<String> = Vec::new();
    for path in conflicted {
        names.push(path_message_text(path.as_slice()));
    }
    let listed: String = serde_json::to_string(&names).unwrap_or_default();
    return format!(
        "No commit created: normalization removed every selected change; index copies were reconciled, but worktree copies changed and were kept: {listed}. Compare them with HEAD before retrying. Hint: if you were committing only whitespace or final-newline edits, they were normalized away; `git diff HEAD -- <path>` shows what still differs."
    );
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_no_change_tests.rs"]
mod tests;
