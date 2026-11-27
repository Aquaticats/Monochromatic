//! What:
//!  The message and the identities of a commit the service lands,
//! made from the command line the way native
//!  `git commit` makes them when no editor opens.
//! Why:
//!  Everything here is text arithmetic that Git itself defines (clean-up,
//! sign-off,
//! trailers,
//! the subject of a fixup,
//!  the message and authorship of a reused commit),
//! so the wrapper embeds it instead of starting `git commit`;
//! each
//!  rule is compared field by field with the native commit in `binary_service_options_tests.rs`.
//! Whatever cannot be
//!  made without guessing returns nothing and native `git commit` handles the line.
//! Each step here has two faces:
//! the synchronous one a caller on a thread asks,
//! and the awaited one a task of the runtime asks
//!  (`build_message_awaiting`,
//! `identity_awaiting`).
//! The awaited face awaits every Git child of the step on the runtime and reads a `-F` file as a job of
//!  the blocking pool,
//! awaited no longer than `PREPARATION_READ_CEILING`,
//! so neither a child that never ends nor a mount that
//!  stopped answering holds a worker.
//! Both faces ask one request each and read one answer each,
//! so neither can answer where the other fails.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { message, author } = await buildMessage(invocation, parsed, committer);
//! ```

pub use super::service_message_the_identity::{
    Built,
    author_overrides,
    identity,
    identity_awaiting,
};
use super::service_message_the_text_steps::{
    strip_comment_lines,
    with_final_line_break,
};
pub use super::service_message_what_a_commit_says::{
    reused_commit,
    reused_commit_awaiting,
    subject_of,
    subject_of_awaiting,
};
use crate::blocking_jobs_within_the_budget::{
    TheAnswerOfAJobAwaited,
    run_a_job_awaited_no_longer_than,
};
use crate::commit_transaction::{
    CommitInvocation,
    TransactionStop,
    failed,
};
use crate::service_commit_args::{
    FixupKind,
    ServiceCommitArgs,
    cleanup_message,
};
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::PREPARATION_READ_CEILING;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What:
///  The message the command line stands for:
/// `-m` texts,
/// the `-F` file,
/// the message of the `-C` commit or the subject
///  line of a fixup (or,
/// for an amend with no message option,
/// the message of the commit it replaces),
/// cleaned as Git cleans it when no editor opens (or kept as given for `--cleanup=verbatim`),
/// then
///  given the sign-off and the `--trailer` lines by Git's own trailer interpreter.
/// Why:
///   Nothing here guesses:
/// an empty message without `--allow-empty-message`,
/// an unreadable file,
/// a commit Git cannot
///  resolve or a trailer Git refuses returns nothing,
/// and native `git commit` then gives the person its own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await buildMessage(invocation, parsed, committer)
/// ```
pub fn build_message(
    invocation: &CommitInvocation<'_>,
    parsed: &ServiceCommitArgs,
    committer: &str,
    previous_message: Option<&str>,
) -> Result<Option<Built>, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let mut reused_author: Option<String> = None;
    let sources: Vec<String> = if let Some(file) = parsed.file.as_ref() {
        // A file that cannot be read, or is not text, gives no message.
        let Some(bytes) = read_the_message_file(MessageFileQuestion {
            path: cwd.join(file),
        }) else {
            return Ok(None);
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return Ok(None);
        };
        vec![text]
    } else if let Some(revision) = parsed.reuse.as_deref() {
        let Some((author, message)) = reused_commit(context, cwd, revision)? else {
            return Ok(None);
        };
        reused_author = Some(author);
        vec![message]
    } else if let Some((kind, revision)) = parsed.fixup.as_ref() {
        let Some(subject) = subject_of(context, cwd, revision.as_str())? else {
            return Ok(None);
        };
        let prefix: &str = match kind {
            FixupKind::Fixup => "fixup!",
        };
        vec![format!("{prefix} {subject}")]
    } else if parsed.messages.is_empty()
        && parsed.amend
        && let Some(previous) = previous_message
    {
        // An amend with no message option keeps the message of the commit it replaces.
        vec![previous.to_string()]
    } else {
        parsed.messages.clone()
    };
    let Some(message): Option<String> = message_of_sources(sources, parsed) else {
        return Ok(None);
    };
    let trailers: Vec<String> = trailer_lines(parsed, committer);
    if trailers.is_empty() {
        return Ok(Some(Built {
            message,
            reused_author,
        }));
    }
    return trailed_answered_by(
        run_git(
            context,
            &trailers_request(cwd, trailers.as_slice(), message),
        ),
        reused_author,
    );
}

/// What:
///  `build_message` with every Git child awaited on the runtime and the `-F` file read as a job of the
///  blocking pool.
/// Why:
///   A task of the runtime cannot call the synchronous face,
/// which enters the runtime to wait for its child
///  (`transaction_git_awaiting.rs`),
/// and the message is built from the landing once that landing is a task.
/// What the
///  sources are,
/// how they are cleaned and what the trailers are read from are the synchronous face's,
/// so the two
///  cannot differ in the words the person gets.
/// A file the ceiling or a refused thread leaves without an answer is the
///  same answer an unreadable file gives,
/// so no message waits on the file without end and native `git commit`
///  gives the person its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await buildMessageAwaiting(invocation, parsed, committer, previousMessage)
/// ```
pub async fn build_message_awaiting(
    invocation: &CommitInvocation<'_>,
    parsed: &ServiceCommitArgs,
    committer: &str,
    previous_message: Option<&str>,
) -> Result<Option<Built>, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let mut reused_author: Option<String> = None;
    let sources: Vec<String> = if let Some(file) = parsed.file.as_ref() {
        // A file that cannot be read, or is not text, gives no message.
        let Some(bytes) = message_file_awaiting(cwd.join(file).as_path()).await else {
            return Ok(None);
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return Ok(None);
        };
        vec![text]
    } else if let Some(revision) = parsed.reuse.as_deref() {
        let Some((author, message)) = reused_commit_awaiting(context, cwd, revision).await? else {
            return Ok(None);
        };
        reused_author = Some(author);
        vec![message]
    } else if let Some((kind, revision)) = parsed.fixup.as_ref() {
        let Some(subject) = subject_of_awaiting(context, cwd, revision.as_str()).await? else {
            return Ok(None);
        };
        let prefix: &str = match kind {
            FixupKind::Fixup => "fixup!",
        };
        vec![format!("{prefix} {subject}")]
    } else if parsed.messages.is_empty()
        && parsed.amend
        && let Some(previous) = previous_message
    {
        // An amend with no message option keeps the message of the commit it replaces.
        vec![previous.to_string()]
    } else {
        parsed.messages.clone()
    };
    let Some(message): Option<String> = message_of_sources(sources, parsed) else {
        return Ok(None);
    };
    let trailers: Vec<String> = trailer_lines(parsed, committer);
    if trailers.is_empty() {
        return Ok(Some(Built {
            message,
            reused_author,
        }));
    }
    return trailed_answered_by(
        run_git_awaiting(
            context,
            &trailers_request(cwd, trailers.as_slice(), message),
        )
        .await,
        reused_author,
    );
}

/// What:
///  The message the sources come to,
/// cleaned as Git cleans it when no editor opens (or kept as given for
///  `--cleanup=verbatim`),
/// and nothing when they leave no message and none was allowed.
/// Why:
///  Both faces clean the same sources the same way,
/// so a `--cleanup` mode or an empty message is one answer
///  for both of them,
/// and native `git commit` gives the person its own words wherever this gives nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function messageOfSources(sources: string[], parsed: ServiceCommitArgs): string | undefined;
/// ```
fn message_of_sources(
    sources: Vec<String>,
    parsed: &ServiceCommitArgs,
) -> Option<String> {
    let texts: Vec<String> = if parsed.strip {
        sources
            .iter()
            .map(String::as_str)
            .map(strip_comment_lines)
            .collect()
    } else {
        sources
    };
    let cleaned: Option<String> = if parsed.verbatim {
        Some(
            texts
                .iter()
                .map(with_final_line_break)
                .collect::<Vec<String>>()
                .join("\n"),
        )
    } else {
        cleanup_message(texts.as_slice())
    };
    return match cleaned {
        Some(text) if !text.trim().is_empty() || parsed.verbatim => Some(text),
        _ if parsed.allow_empty_message => Some(String::new()),
        _ => None,
    };
}

/// What:
///  The sign-off and `--trailer` lines Git's trailer interpreter is given,
/// in the order the message ends with them.
/// Why:
///  Both faces read the trailers from the same two inputs,
/// so a `-s` of the committer or a `--trailer`
///  written by the person reaches Git the same way from either of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trailerLines(parsed: ServiceCommitArgs, committer: string): string[];
/// ```
fn trailer_lines(
    parsed: &ServiceCommitArgs,
    committer: &str,
) -> Vec<String> {
    let mut trailers: Vec<String> = Vec::new();
    if parsed.signoff {
        // The name and email up to and including the `>`; the whole text when there is none.
        let who: &str = match committer.split_once('>') {
            Some((name_and_email, _time)) => &committer[..name_and_email.len() + 1],
            None => committer,
        };
        trailers.push(format!("Signed-off-by: {who}"));
    }
    trailers.extend(parsed.trailers.iter().cloned());
    return trailers;
}

/// What:
///  The question that gives the message its trailers.
/// Why:
///  Both faces ask the one question,
/// so neither can hand Git another trailer list or another message than the
///  other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd, args: ['interpret-trailers', ...trailers.flatMap((t) => ['--trailer', t])], input: message };
/// ```
fn trailers_request(
    cwd: &Path,
    trailers: &[String],
    message: String,
) -> GitRequest {
    let mut arguments: Vec<String> = vec![String::from("interpret-trailers")];
    for trailer in trailers {
        arguments.push(String::from("--trailer"));
        arguments.push(trailer.clone());
    }
    let mut request: GitRequest = GitRequest::new(cwd, arguments.as_slice());
    request.input = Some(message.into_bytes());
    return request;
}

/// What:
///  What the trailer interpreter made of the message,
/// or nothing for a message Git refused or one that is not
///  text,
/// with the author line of a reused commit when one supplied it.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a refusal and a non-text answer are the same answer
///  for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trailedAnsweredBy(answered: GitOutput | Error, reusedAuthor?: string): Built | undefined; // throws on a failed start
/// ```
///
/// # Errors
///
/// `TransactionStop` naming the message when Git cannot be started.
fn trailed_answered_by(
    answered: std::io::Result<GitOutput>,
    reused_author: Option<String>,
) -> Result<Option<Built>, TransactionStop> {
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => {
            return Err(failed(format!(
                "preparing the commit message failed: {error:?}"
            )));
        }
    };
    if output.code != Some(0) {
        return Ok(None);
    }
    return match String::from_utf8(output.stdout) {
        Ok(message) => Ok(Some(Built {
            message,
            reused_author,
        })),
        Err(_not_text) => Ok(None),
    };
}

/// The one path one message-file question names,
/// owned so that a job of the blocking pool can take it.
struct MessageFileQuestion {
    /// The file the command line's `-F` names,
    /// read relative to the working directory.
    path: PathBuf,
}

/// What:
///  The bytes of the message file,
/// or nothing for a file that cannot be read.
/// Why:
///  Both faces read the file through this one function,
/// so a file that is not there,
/// one this user may not read and
///  one that is a directory give the same answer from either of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheMessageFile(question: MessageFileQuestion): Buffer | undefined;
/// ```
fn read_the_message_file(question: MessageFileQuestion) -> Option<Vec<u8>> {
    let Ok(bytes) = std::fs::read(question.path.as_path()) else {
        return None;
    };
    return Some(bytes);
}

/// What:
///  `read_the_message_file` as a job of the blocking pool,
/// awaited no longer than `ceiling`.
/// Why:
///   A task of the runtime may not wait on a filesystem:
/// the read waits in the kernel on a mount that stopped
///  answering,
/// which would hold a worker the runtime needs,
/// and no timer on that worker can end it.
/// A read
///  the ceiling or a refused thread leaves without an answer is answered as an unreadable file is,
/// so
///  the caller gives native `git commit` the line and nothing waits on the file for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function messageFileWithin(ceilingMs: number, path: string): Promise<Buffer | undefined>;
/// ```
async fn message_file_within(
    ceiling: Duration,
    path: &Path,
) -> Option<Vec<u8>> {
    let question: MessageFileQuestion = MessageFileQuestion {
        path: path.to_path_buf(),
    };
    return match run_a_job_awaited_no_longer_than(ceiling, question, read_the_message_file).await {
        TheAnswerOfAJobAwaited::Ran(read) => read,
        // No thread was given, or the job outlasted its ceiling: the file said nothing this task may wait for.
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

/// `message_file_within` with the wait a preparation's own file read is granted
/// (`wait_ceilings.rs`,
/// `PREPARATION_READ_CEILING`).
async fn message_file_awaiting(path: &Path) -> Option<Vec<u8>> {
    return message_file_within(PREPARATION_READ_CEILING, path).await;
}

/// Controls for the awaited faces of the message and the identities stay out of the release executable.
#[cfg(test)]
#[path = "service_message_awaiting_tests.rs"]
mod awaiting_tests;
