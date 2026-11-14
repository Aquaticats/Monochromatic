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
//! The read has two faces:
//! the synchronous one a caller on a thread asks (`what_the_branch_shows`,
//! `the_silent_service_after_a_prepare`),
//! and the awaited one a task of the runtime asks
//!  (`what_the_branch_shows_awaiting`,
//! `the_silent_service_after_a_prepare_awaiting`),
//! which awaits every Git child on the runtime.
//! A branch is named by the bytes Git printed for it,
//! so the question is built from those bytes (`ref_name.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shows = await whatTheBranchShowsAwaiting(context, cwd, reference, prepared);
//! ```

/// The context a Git read runs in.
use crate::transaction_git::GitContext;
/// Git,
/// started once and answered without failing the command when Git says no.
use crate::transaction_git::run_git;
/// The same run awaited on the runtime,
/// for a caller that is a task.
use crate::transaction_git_awaiting::run_git_awaiting;
/// The output one bounded read of the branch gives.
use crate::transaction_git::{GitOutput, GitRequest};
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
/// The name of a reference as a message states it.
use crate::service_commit_input_notes_and_embedded_commit::switched_branch_name;

/// What:
///  The question one read of the branch asks Git.
/// Why:
///  Both faces ask the one question,
/// so neither can read another commit's standing on the branch than the other.
///  The name is asked as the bytes Git printed for it:
/// a spelling with replacement characters would ask about another reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = indexedRequest(context, cwd, ['merge-base', '--is-ancestor', prepared, reference]);
/// ```
fn branch_shows_request(context: &GitContext, cwd: &Path, reference: &crate::ref_name::RefName, prepared: &str) -> GitRequest {
    // Whether the branch holds the commit, which the tip itself does and so does every commit under it:
    // `merge-base --is-ancestor` answers 0 for both, 1 for a commit the branch does not hold, and anything else is a read
    // that failed (finding 5 of the third review).
    let words: [std::ffi::OsString; 4] = [
        std::ffi::OsString::from("merge-base"),
        std::ffi::OsString::from("--is-ancestor"),
        std::ffi::OsString::from(prepared),
        reference.as_os_str().to_os_string(),
    ];
    return indexed_request(context, cwd, words.as_slice(), None, None);
}

/// What:
///  What Git's answer to `branch_shows_request` says the branch shows.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a read that could not be started,
/// a commit the branch
///  does not hold and a question Git could not answer stay three different answers for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showsAnsweredBy(asked) { return asked.error ? 'unread' : asked.code === 0 ? 'made' : asked.code === 1 ? 'notMade' : 'unread'; }
/// ```
fn shows_answered_by(asked: std::io::Result<GitOutput>) -> WhatTheBranchShows {
    let Ok(output) = asked else {
        return WhatTheBranchShows::Unread;
    };
    return match output.code {
        Some(0) => WhatTheBranchShows::Made,
        Some(1) => WhatTheBranchShows::NotMade,
        _ => WhatTheBranchShows::Unread,
    };
}

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
    return shows_answered_by(run_git(context, &branch_shows_request(context, cwd, reference, prepared)));
}

/// What:
///  `what_the_branch_shows` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and the branch is read for the
///  silence message that a command which became a task prints.
/// The question and the reading of Git's answer are the
///  synchronous face's,
/// so the two cannot differ in what they claim about the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shows = await whatTheBranchShowsAwaiting(context, cwd, reference, prepared);
/// ```
pub async fn what_the_branch_shows_awaiting(context: &GitContext, cwd: &Path, reference: &crate::ref_name::RefName, prepared: &str) -> WhatTheBranchShows {
    return shows_answered_by(run_git_awaiting(context, &branch_shows_request(context, cwd, reference, prepared)).await);
}

/// The plain failure of a branch that moved under a commit that cannot be made again on another tip.
pub(crate) fn branch_moved(ready: &Ready) -> TransactionStop {
    let branch: String = ready.reference.short().spoken();
    return crate::commit_transaction::client_stop(ClientFailure::BranchMoved { branch });
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
    return Some(crate::commit_transaction::client_stop(failure));
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
///  `the_silent_service_after_a_prepare` with its read of the branch awaited on the runtime.
/// Why:
///   A task of the runtime may not wait on a child it starts itself,
/// so the failure a silent service is reported with is built
///  through this face.
/// The branch is read only when the request left a prepared commit,
/// and what a silence means once it was
///  read is the synchronous face's,
/// so the two cannot report different words for one silence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = await theSilentServiceAfterAPrepareAwaiting(context, cwd, reference, prepared);
/// ```
pub async fn the_silent_service_after_a_prepare_awaiting(context: &GitContext, cwd: &Path, reference: &crate::ref_name::RefName, prepared: Option<&str>) -> Option<ClientFailure> {
    let shows: WhatTheBranchShows = match prepared {
        Some(made) => what_the_branch_shows_awaiting(context, cwd, reference, made).await,
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

/// Controls for the awaited read of the branch stay out of the release executable.
#[cfg(test)]
#[path = "service_branch_shows_awaiting_tests.rs"]
mod awaiting_tests;
