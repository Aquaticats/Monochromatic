//! What:
//!  The commit an amend makes, as faces a task of the runtime may await:
//!  the message write as a job of the blocking pool,
//!  and every Git child awaited on the runtime.
//! Why:
//!   A task may neither wait on a filesystem nor enter the runtime to wait for a child,
//! and an amend is the one commit that is made again from the bytes of the commit it replaces.
//! The words of every refusal,
//! the object a commit made again writes and the headers it keeps are the synchronous faces',
//! which `service_amend_replayed.rs` still holds,
//! so a header line and a message reach the object the same way whichever face asked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await rebuiltOntoAwaiting(amended, site, tree, parent); await amendedFieldsAwaiting(invocation, sent, embedded);
//! ```

use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::commit_transaction::{CommitInvocation, TransactionStop, failed};
use crate::private_storage::write_private_file;
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_protocol::CommitRequest;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use std::path::Path;

use super::{
    AmendedCommit, BuildSite, MESSAGE_FILE_NAME, amended_from_embedded, embedded_with_a_source, reply_name, say_which_headers_are_dropped,
    signed_commit_request, stored_commit_fields, whole_commit_bytes, whole_commit_request,
};

/// The message one commit made again is written to,
/// owned so that a job of the blocking pool can take it.
struct MessageWrite {
    /// The file inside the commit's own scratch directory.
    path: std::path::PathBuf,
    /// The message,
    /// as bytes,
    /// because a commit's message is stored as the bytes Git was given.
    message: Vec<u8>,
}

/// The write one job of the pool does:
/// the message of the commit made again,
/// in its own scratch directory.
fn write_message(work: MessageWrite) -> std::io::Result<()> {
    return write_private_file(work.path.as_path(), work.message.as_slice());
}

/// What:
///  `rebuilt_onto` with its message write a job of the blocking pool and its Git child awaited on the
///  runtime.
/// Why:
///   A task of the runtime may neither wait on a filesystem nor enter the runtime to wait for a child,
/// and the
///  message is a file the commit's own scratch directory receives.
/// The write keeps no ceiling:
/// a file that was abandoned half-written is
///  one nothing reads,
/// and the job of the pool holds its place until the write ends.
/// The words of every refusal,
/// the order of the steps and the signature a commit made again carries are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rebuiltOntoAwaiting(amended, site, tree, parent) // the commit's object name
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the message cannot be written or Git cannot write the commit.
pub(crate) async fn rebuilt_onto_awaiting(amended: &AmendedCommit, site: &BuildSite<'_>, tree: &str, parent: &str) -> Result<String, TransactionStop> {
    let message_file = site.scratch.join(MESSAGE_FILE_NAME);
    let work: MessageWrite = MessageWrite { path: message_file.clone(), message: amended.message.clone() };
    if let Err(error) = run_blocking_job_within_the_budget(move || { return write_message(work); }).await {
        return Err(failed(format!("writing the message of the amended commit failed: {error}")));
    }
    if amended.signed {
        return signed_commit_awaiting(amended, site, tree, parent, message_file.as_path()).await;
    }
    return whole_commit_awaiting(amended, site, tree, parent).await;
}

/// `whole_commit` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await wholeCommitAwaiting(amended, site, tree, parent)
/// ```
///
/// # Errors
///
/// As `whole_commit`.
async fn whole_commit_awaiting(amended: &AmendedCommit, site: &BuildSite<'_>, tree: &str, parent: &str) -> Result<String, TransactionStop> {
    let written = run_git_checked_awaiting(site.context, &whole_commit_request(site, &whole_commit_bytes(amended, tree, parent))).await?;
    return Ok(reply_name(written.stdout.as_slice()));
}

/// `signed_commit` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await signedCommitAwaiting(amended, site, tree, parent, messageFile)
/// ```
///
/// # Errors
///
/// As `signed_commit`.
async fn signed_commit_awaiting(amended: &AmendedCommit, site: &BuildSite<'_>, tree: &str, parent: &str, message_file: &Path) -> Result<String, TransactionStop> {
    say_which_headers_are_dropped(amended);
    let written = run_git_checked_awaiting(site.context, &signed_commit_request(amended, site, tree, parent, message_file)).await?;
    return Ok(reply_name(written.stdout.as_slice()));
}

/// What:
///  `amended_fields` with the read of the commit Git built awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for that read,
/// and the fields it answers are what a moved branch puts
///  into the commit it makes.
/// The commit a service-built amend names is answered without Git,
/// as the synchronous face answers it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await amendedFieldsAwaiting(invocation, sent, embedded)
/// ```
///
/// # Errors
///
/// `TransactionStop` when the prepared commit cannot be read back.
#[cfg_attr(not(test), expect(dead_code, reason = "no production caller yet: the commit path calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"))]
pub(crate) async fn amended_fields_awaiting(invocation: &CommitInvocation<'_>, sent: &CommitRequest, embedded: Option<&Embedded>) -> Result<AmendedCommit, TransactionStop> {
    if let Some(made) = embedded_with_a_source(embedded) {
        return Ok(amended_from_embedded(made));
    }
    let Some(commit) = sent.commit.as_deref() else {
        return Err(failed(String::from("the amend prepared no commit whose message and authorship a later tip could keep")));
    };
    let read = run_git_checked_awaiting(invocation.context, &GitRequest::new(invocation.cwd, &["cat-file", "commit", commit])).await?;
    return match stored_commit_fields(read.stdout.as_slice()) {
        Some(fields) => Ok(fields),
        None => Err(failed(String::from("the commit this amend prepared could not be read back, so it cannot be made on the branch as it is now"))),
    };
}

