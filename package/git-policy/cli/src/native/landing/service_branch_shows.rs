//! What:
//!  What the branch shows once the commit service went silent after a commit was prepared.
//! Why:
//!   The silence message told the person it was not known whether the commit was made,
//! while the service still held the commit and landed it later
//!  (2026-11-09T07:44:20Z:
//! it landed 73 s after the message,
//! under the first of two requests).
//! One bounded read of the branch turns that into a fact and waits for nothing,
//! and a branch that cannot be read keeps the honest "not known".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shows = whatTheBranchShows(context, cwd, reference, prepared);
//! ```

/// The context a Git read runs in.
use crate::transaction_git::GitContext;
/// The one required Git start and its output.
use crate::transaction_git::run_git_checked;
/// A request with the private index and object directory of a capture.
use crate::private_index_request::indexed_request;
/// What the branch shows,
/// and the failure that prints it.
use crate::service_messages::{ClientFailure, WhatTheBranchShows};
/// A path on the filesystem.
use std::path::Path;
/// The prepared commit the service lands.
use crate::service_prepare::Ready;
/// A command that stops without a result.
use crate::commit_transaction::TransactionStop;
/// The words of a failure.
use crate::service_messages::render;

/// What:
///  Read the branch once and say whether it is at the commit this command prepared.
/// Why:
///   The silence message told the person it was not known whether the commit was made,
/// while the service still held the commit and landed it later
/// (2026-11-09T07:44:20Z:
/// it landed 73 s after the message).
/// One bounded read turns that into a fact and waits for nothing,
/// and a branch that cannot be read keeps the honest "not known".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shows = whatTheBranchShows(context, cwd, reference, prepared);
/// ```
pub fn what_the_branch_shows(context: &GitContext, cwd: &Path, reference: &str, prepared: &str) -> WhatTheBranchShows {
    let read = run_git_checked(context, &indexed_request(cwd, &["rev-parse", "--verify", reference], None, None));
    let Ok(output) = read else {
        return WhatTheBranchShows::Unread;
    };
    let head: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    if head.is_empty() {
        return WhatTheBranchShows::Unread;
    }
    return if head == prepared { WhatTheBranchShows::Made } else { WhatTheBranchShows::NotMade };
}

/// The plain failure of a branch that moved under a commit that cannot be made again on another tip.
pub(crate) fn branch_moved(ready: &Ready) -> TransactionStop {
    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
    return TransactionStop::Message(render(&ClientFailure::BranchMoved { branch }));
}

/// What:
///  A path as text for a message.
/// Why:
///  A failure names the file a typed message was saved in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const displayText = (file: string) => file;
/// ```
pub(crate) fn display_text(file: &Path) -> String {
    return file.display().to_string();
}

/// What:
///  The failure to print when the service went silent after a commit was prepared.
/// Why:
///   The message must state what the branch shows,
/// and a commit the service builds itself from the request leaves the wrapper no commit to compare it with,
/// which is the honest "not known" answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = theSilentServiceAfterAPrepare(context, cwd, reference, prepared);
/// ```
pub fn the_silent_service_after_a_prepare(context: &GitContext, cwd: &Path, reference: &str, prepared: Option<&str>) -> ClientFailure {
    let shows: WhatTheBranchShows = match prepared {
        Some(made) => what_the_branch_shows(context, cwd, reference, made),
        None => WhatTheBranchShows::Unread,
    };
    return ClientFailure::ServiceSilentAfterPreparing { shows };
}

/// Controls for the branch read stay out of the release executable.
#[cfg(test)]
#[path = "service_branch_shows_tests.rs"]
mod tests;
