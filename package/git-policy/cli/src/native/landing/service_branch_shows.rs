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
/// Git,
/// started once and answered without failing the command when Git says no.
use crate::transaction_git::run_git;
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
///  Read the branch once and say whether it holds the commit this command prepared.
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
    // Whether the branch holds the commit, which the tip itself does and so does every commit under it:
    // `merge-base --is-ancestor` answers 0 for both, 1 for a commit the branch does not hold, and anything else is a read
    // that failed (finding 5 of the third review).
    let asked = run_git(context, &indexed_request(context, cwd, &["merge-base", "--is-ancestor", prepared, reference], None, None));
    let Ok(output) = asked else {
        return WhatTheBranchShows::Unread;
    };
    return match output.code {
        Some(0) => WhatTheBranchShows::Made,
        Some(1) => WhatTheBranchShows::NotMade,
        _ => WhatTheBranchShows::Unread,
    };
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
pub fn the_silent_service_after_a_prepare(context: &GitContext, cwd: &Path, reference: &str, prepared: Option<&str>) -> Option<ClientFailure> {
    let shows: WhatTheBranchShows = match prepared {
        Some(made) => what_the_branch_shows(context, cwd, reference, made),
        None => WhatTheBranchShows::Unread,
    };
    return the_silence_means(shows);
}

/// What:
///  What a silence means once the branch has been read.
/// Why:
///   A commit the branch shows as made is a landed commit and not a failure:
/// the owner's rule is that commits never fail,
/// so nothing is reported and the caller takes the landed path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theSilenceMeans = (shows) => shows === 'made' ? undefined : { shows };
/// ```
fn the_silence_means(shows: WhatTheBranchShows) -> Option<ClientFailure> {
    return match shows {
        // The branch holds the commit: the command did what it was asked, so it reports nothing and succeeds.
        WhatTheBranchShows::Made => None,
        what_the_branch_shows => Some(ClientFailure::ServiceSilentAfterPreparing { shows: what_the_branch_shows }),
    };
}

/// Controls for the branch read stay out of the release executable.
#[cfg(test)]
#[path = "service_branch_shows_tests.rs"]
mod tests;
