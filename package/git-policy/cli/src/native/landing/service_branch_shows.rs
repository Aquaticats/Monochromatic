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
/// The name of a reference as a message states it.
use crate::service_commit_input_notes_and_embedded_commit::switched_branch_name;

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
pub fn what_the_branch_shows(context: &GitContext, cwd: &Path, reference: &crate::ref_name::RefName, prepared: &str) -> WhatTheBranchShows {
    // Whether the branch holds the commit, which the tip itself does and so does every commit under it:
    // `merge-base --is-ancestor` answers 0 for both, 1 for a commit the branch does not hold, and anything else is a read
    // that failed (finding 5 of the third review).
    let words: [std::ffi::OsString; 4] = [
        std::ffi::OsString::from("merge-base"),
        std::ffi::OsString::from("--is-ancestor"),
        std::ffi::OsString::from(prepared),
        reference.as_os_str().to_os_string(),
    ];
    let asked = run_git(context, &indexed_request(context, cwd, words.as_slice(), None, None));
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
    let branch: String = ready.reference.short().spoken();
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
///  The stop a commit gets when the worktree is no longer on the branch the commit was prepared on.
/// Why:
///   A preparation that found another branch cannot land what it prepared,
/// so the command ends like native Git's,
///  with the typed message kept in its file;
/// landing it on the branch as it is now would commit the person's change onto work they did not prepare it against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stop = refusedWhenTheBranchSwitched(firstReference, reference, messageFile);
/// ```
pub(crate) fn refused_when_the_branch_switched(
    first_reference: Option<&crate::ref_name::RefName>,
    reference: &crate::ref_name::RefName,
    fixed_message: Option<&Path>,
) -> Option<TransactionStop> {
    // A commit whose first preparation found this same branch has not switched anywhere, and names are compared as bytes.
    let first_branch: &crate::ref_name::RefName = match first_reference {
        Some(known) => known,
        None => reference,
    };
    if first_branch == reference {
        return None;
    }
    let failure: ClientFailure = ClientFailure::BranchSwitched {
        branch: switched_branch_name(first_branch),
        now: switched_branch_name(reference),
        file: fixed_message.map(display_text),
    };
    return Some(TransactionStop::Message(render(&failure)));
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
pub fn the_silent_service_after_a_prepare(context: &GitContext, cwd: &Path, reference: &crate::ref_name::RefName, prepared: Option<&str>) -> Option<ClientFailure> {
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
