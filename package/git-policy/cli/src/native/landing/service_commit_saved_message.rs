//! What:
//!  Keeping the message a person typed when the commit it belonged to cannot be made.
//! Why:
//!  The text is the person's,
//! and it exists only in this process:
//!  a conflict with the commit that landed first,
//!  and a branch that moved after the editor ran,
//!  both end the commit,
//!  and both leave the text in a file beside the repository's Git data,
//!  whose name the command prints.
//!  Nothing else removes such a file,
//!  so a control that waits for a repository to be clean must not count it
//!  (`binary/binary_support_scratch_left_behind.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = keepTheTypedMessage(gitDir, id, text); // the path, or undefined when it could not be written
//! ```

/// The name a saved message of a request has,
/// which is the request's identity cut to its first eight characters.
const SAVED_MESSAGE_PREFIX: &str = "monochromatic-git-policy-cli-saved-message-";

/// Warning lines for a message that could not be written.
use crate::diagnostic_log::warn;
/// A path.
use std::path::{Path, PathBuf};

/// What:
///  The path the typed message of request `id` is kept at,
///  written with `text` when there is one and it can be.
///  `None` when there is no text or the write failed.
/// Why:
///   The caller needs the path to name it in what the person reads,
///  and the file is what lets them commit again with the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepTheTypedMessage(conflict.gitDir, id, lastMessage)
/// ```
pub(crate) fn keep_the_typed_message(git_dir: &Path, id: &str, text: Option<&str>) -> Option<PathBuf> {
    let typed: &str = text?;
    let saved: PathBuf = git_dir.join(format!("{SAVED_MESSAGE_PREFIX}{}", id.get(..8).unwrap_or(id)));
    return match std::fs::write(&saved, typed) {
        Ok(()) => Some(saved),
        Err(error) => {
            warn("serviceCommit", format!("saving the message failed: {error}").as_str());
            None
        }
    };
}

/// The saved message and the file it goes in are controlled in the test module.
#[cfg(test)]
#[path = "service_commit_saved_message_tests.rs"]
mod tests;
