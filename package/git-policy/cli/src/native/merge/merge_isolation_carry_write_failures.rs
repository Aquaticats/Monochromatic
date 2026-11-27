//! What:
//!  Why a carry or a clear stopped,
//!  and the words a write that stopped leaves about the copy it kept.
//! Why:
//!   The clear before a merge,
//!  the carry of each layer after it and a putting back all write through one step,
//!  so one failure names what stopped any of them,
//!  and a line about a kept copy must spell each of its two paths as one shell word.
//!  The step that writes is `merge_isolation_carry_writes.rs`,
//!  which cannot hold these words itself without passing its own line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = commandsThatEndTheCopy(aside, path);
//! ```

/// The one owner of a path spelled for a shell,
/// so a command the line prints takes the path as one word.
use crate::shell_quote::shell_quote;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Why a carry or a clear stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarryFailure = { kind: 'index-held'; detail: string } | { kind: 'failed'; detail: string } | { kind: 'foreign'; path: Uint8Array } | { kind: 'changed'; path: Uint8Array };
/// ```
#[derive(Debug)]
pub enum CarryFailure {
    /// Another program held `index.lock` until the wait's bound,
    /// so nothing was written.
    IndexHeld(String),
    /// A read or a write failed;
    /// the capture still holds the bytes.
    Failed(String),
    /// One path holds bytes neither the capture nor the change explains.
    Foreign(Vec<u8>),
    /// One path was written after this step judged it,
    /// so nothing was written to it and its bytes are the person's.
    Changed(Vec<u8>),
}

/// The words of a failure that stopped one step's write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failureDetail = (failure) => failure.kind === 'failed' ? failure.detail : 'it could not be written';
/// ```
pub(crate) fn failure_detail(failure: &CarryFailure) -> String {
    return match failure {
        CarryFailure::Failed(detail) | CarryFailure::IndexHeld(detail) => {
            String::from(detail.as_str())
        }
        CarryFailure::Foreign(path) | CarryFailure::Changed(path) => {
            crate::path_bytes::path_message_text(path.as_slice())
        }
    };
}

/// What:
///  The words of the two commands that put a copy this merge kept back over its path,
/// and drop it.
/// Why:
///   Both paths are copied out of the line and run,
/// and a name with a space reaches `mv` as several arguments otherwise,
///  so each is quoted for the shell
/// (`crate::shell_quote`,
/// the one owner of that spelling).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commands = (aside, path) => `mv -- ${shellQuote(aside)} ${shellQuote(path)} puts the copy back over what is there, rm -- ${shellQuote(aside)} drops the copy`;
/// ```
pub(crate) fn commands_that_end_the_copy(
    aside: &Path,
    path: &[u8],
) -> String {
    let shown_aside: String = shell_quote(aside.as_os_str().as_encoded_bytes());
    let shown_path: String = shell_quote(path);
    return format!(
        "`mv -- {shown_aside} {shown_path}` puts the copy back over what is there, `rm -- {shown_aside}` drops the copy"
    );
}
