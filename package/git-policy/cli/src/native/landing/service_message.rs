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

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::commit_transaction::{CommitInvocation, TransactionStop, failed};
use crate::service_commit_args::{FixupKind, ServiceCommitArgs, cleanup_message};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::PREPARATION_READ_CEILING;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What a command line's message came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Built {
    /// The whole commit message.
    pub message: String,
    /// The author line a reused commit supplies,
    /// `Name <email> seconds zone`.
    pub reused_author: Option<String>,
}

/// One identity line (`Name <email> seconds zone`) from Git's own resolution of the variable,
/// with the given overrides;
///  nothing when Git cannot make one (a date it does not read),
/// so native `git commit` gives its own message.
pub fn identity(context: &GitContext, cwd: &Path, variable: &str, overrides: &[(&str, String)]) -> Result<Option<String>, TransactionStop> {
    return identity_answered(run_git(context, &identity_request(cwd, variable, overrides)));
}

/// What:
///  `identity` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot call the synchronous face,
/// which enters the runtime to wait for its child
///  (`transaction_git_awaiting.rs`),
/// and the identities are read from the landing once that landing is a task.
/// The question and the
///  reading of the answer are the synchronous face's,
/// so the two cannot differ in what they make of Git's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await identityAwaiting(git, cwd, 'GIT_AUTHOR_IDENT', overrides)
/// ```
pub async fn identity_awaiting(context: &GitContext, cwd: &Path, variable: &str, overrides: &[(&str, String)]) -> Result<Option<String>, TransactionStop> {
    return identity_answered(run_git_awaiting(context, &identity_request(cwd, variable, overrides)).await);
}

/// What:
///  The question `identity` asks Git.
/// Why:
///  Both faces ask the one question,
/// so neither can read another variable or miss an override the other
///  carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd, args: ['var', variable], environment: overrides };
/// ```
fn identity_request(cwd: &Path, variable: &str, overrides: &[(&str, String)]) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(cwd, &["var", variable]);
    for (name, value) in overrides {
        request.environment.push((OsString::from(name), OsString::from(value)));
    }
    return request;
}

/// What:
///  The identity line Git's answer names,
/// or nothing for a variable Git does not resolve.
/// Why:
///  Both faces read Git's answer through this one function,
/// so an unusable date and an identity Git
///  cannot make are the same answer for both of them,
/// and a failure names the same step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function identityAnswered(answered: GitOutput | Error): string | undefined; // throws on a failed start
/// ```
///
/// # Errors
///
/// `TransactionStop` naming the identity when Git cannot be started.
fn identity_answered(answered: std::io::Result<GitOutput>) -> Result<Option<String>, TransactionStop> {
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("reading the commit identity failed: {error:?}"))),
    };
    if output.code != Some(0) {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string()));
}

/// The author variables `--author` and `--date` stand for.
pub fn author_overrides(parsed: &ServiceCommitArgs) -> Vec<(&'static str, String)> {
    let mut overrides: Vec<(&'static str, String)> = Vec::new();
    if let Some(author) = parsed.author.as_deref() {
        let (name, rest): (&str, &str) = author.split_once('<').unwrap_or((author, ""));
        overrides.push(("GIT_AUTHOR_NAME", name.trim().to_string()));
        overrides.push(("GIT_AUTHOR_EMAIL", rest.trim_end().trim_end_matches('>').to_string()));
    }
    if let Some(date) = parsed.date.as_deref() {
        overrides.push(("GIT_AUTHOR_DATE", date.to_string()));
    }
    return overrides;
}

/// The text of a Git query that must succeed for the line to be covered.
fn query(context: &GitContext, cwd: &Path, arguments: &[String]) -> Result<Option<Vec<u8>>, TransactionStop> {
    return text_answered_by(run_git(context, &GitRequest::new(cwd, arguments)));
}

/// `query` with its Git child awaited on the runtime.
async fn query_awaiting(context: &GitContext, cwd: &Path, arguments: &[String]) -> Result<Option<Vec<u8>>, TransactionStop> {
    return text_answered_by(run_git_awaiting(context, &GitRequest::new(cwd, arguments)).await);
}

/// What:
///  The text Git's answer to a `query` carries,
/// or nothing for a question Git could not answer.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a name Git does not resolve is no text for
///  both of them and a failed start names the same step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textAnsweredBy(answered: GitOutput | Error): Buffer | undefined; // throws on a failed start
/// ```
///
/// # Errors
///
/// `TransactionStop` naming the repository when Git cannot be started.
fn text_answered_by(answered: std::io::Result<GitOutput>) -> Result<Option<Vec<u8>>, TransactionStop> {
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("reading the repository failed: {error:?}"))),
    };
    return Ok(if output.code == Some(0) { Some(output.stdout) } else { None });
}

/// What:
///  The raw object of a commit named by the person:
/// its author line and its message.
/// Why:
///   A revision that looks like an option is refused before Git sees it,
/// and a commit Git cannot resolve,
/// a commit
///  in another encoding or one with no author header gives nothing,
/// so native `git commit` handles the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reused = await reusedCommit(git, cwd, '-C' value)
/// ```
fn reused_commit(context: &GitContext, cwd: &Path, revision: &str) -> Result<Option<(String, String)>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(oid) = query(context, cwd, the_commit_named_by(revision).as_slice())? else { return Ok(None) };
    let named: String = String::from_utf8_lossy(oid.as_slice()).trim().to_string();
    let Some(raw) = query(context, cwd, the_raw_commit(named.as_str()).as_slice())? else { return Ok(None) };
    return Ok(reused_of(raw));
}

/// `reused_commit` with its Git children awaited on the runtime.
async fn reused_commit_awaiting(context: &GitContext, cwd: &Path, revision: &str) -> Result<Option<(String, String)>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(oid) = query_awaiting(context, cwd, the_commit_named_by(revision).as_slice()).await? else { return Ok(None) };
    let named: String = String::from_utf8_lossy(oid.as_slice()).trim().to_string();
    let Some(raw) = query_awaiting(context, cwd, the_raw_commit(named.as_str()).as_slice()).await? else { return Ok(None) };
    return Ok(reused_of(raw));
}

/// What:
///  The `rev-parse` question that resolves `revision` to the commit it names.
/// Why:
///  Both faces ask this one question,
/// so neither can resolve a different name than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['rev-parse', '--verify', '--quiet', `${revision}^{commit}`]
/// ```
fn the_commit_named_by(revision: &str) -> Vec<String> {
    return vec![String::from("rev-parse"), String::from("--verify"), String::from("--quiet"), format!("{revision}^{{commit}}")];
}

/// The `cat-file` question that reads the raw object `named`.
fn the_raw_commit(named: &str) -> Vec<String> {
    return vec![String::from("cat-file"), String::from("commit"), named.to_string()];
}

/// What:
///  What a raw commit object says,
/// as its author line and its message.
/// Why:
///  Both faces read the same object through this one function,
/// so a commit in another encoding,
/// one with extra
///  headers and one with no author line give nothing for both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reusedOf(raw: Buffer): [author: string, message: string] | undefined;
/// ```
fn reused_of(raw: Vec<u8>) -> Option<(String, String)> {
    let Ok(text) = String::from_utf8(raw) else { return None };
    let (header, message): (&str, &str) = text.split_once("\n\n")?;
    // A commit in another encoding, or with extra headers that change the message's meaning, is Git's to reuse.
    if header.lines().any(is_encoding_header) {
        return None;
    }
    let author: &str = header.lines().find_map(author_of_header_line)?;
    return Some((author.to_string(), message.to_string()));
}

/// `reused_commit`'s subject question and the reading of its answer, shared by both faces.
fn subject_of(context: &GitContext, cwd: &Path, revision: &str) -> Result<Option<String>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(bytes) = query(context, cwd, the_subject_of(revision).as_slice())? else { return Ok(None) };
    return Ok(subject_answered_by(bytes));
}

/// `subject_of` with its Git child awaited on the runtime.
async fn subject_of_awaiting(context: &GitContext, cwd: &Path, revision: &str) -> Result<Option<String>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(bytes) = query_awaiting(context, cwd, the_subject_of(revision).as_slice()).await? else { return Ok(None) };
    return Ok(subject_answered_by(bytes));
}

/// What:
///  The `log` question that names the subject Git puts after `fixup! ` or `squash! `.
/// Why:
///  Both faces ask this one question,
/// so neither can read another commit's subject than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['log', '-1', '--format=%s', revision, '--']
/// ```
fn the_subject_of(revision: &str) -> Vec<String> {
    return vec![String::from("log"), String::from("-1"), String::from("--format=%s"), revision.to_string(), String::from("--")];
}

/// The subject of a `log` answer,
/// or nothing when the bytes are not text.
fn subject_answered_by(bytes: Vec<u8>) -> Option<String> {
    return String::from_utf8(bytes).ok().map(without_final_line_breaks);
}

/// What:
///  Whether a header line of a raw commit names an encoding.
/// Why:
///  A commit in another encoding is Git's to reuse,
/// so `reused_commit` gives it up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEncodingHeader = (line: string) => line.startsWith('encoding ');
/// ```
fn is_encoding_header(line: &str) -> bool {
    return line.starts_with("encoding ");
}

/// What:
///  The author line a raw commit header line carries,
/// nothing for any other line.
/// Why:
///  `reused_commit` takes the first such line as the author.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const authorOfHeaderLine = (line: string) => line.startsWith('author ') ? line.slice(7) : undefined;
/// ```
fn author_of_header_line(line: &str) -> Option<&str> {
    return line.strip_prefix("author ");
}

/// What:
///  A text without the line breaks at its end.
/// Why:
///  The subject Git prints after `fixup! ` ends in a newline that is not part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutFinalLineBreaks = (text: string) => text.replace(/\n+$/, '');
/// ```
fn without_final_line_breaks(text: String) -> String {
    return text.trim_end_matches('\n').to_string();
}

/// What:
///  Whether a line is not a comment:
/// it does not start with `#`.
/// Why:
///  `strip_comment_lines` keeps these lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotCommentLine = (line: string) => !line.startsWith('#');
/// ```
fn is_not_comment_line(line: &&str) -> bool {
    return !line.starts_with('#');
}

/// What:
///  A text with a line break at its end,
/// added when it has none.
/// Why:
///  A verbatim message keeps every text as given,
/// each ending in a line break.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withFinalLineBreak = (text: string) => text.endsWith('\n') ? text : text + '\n';
/// ```
fn with_final_line_break(text: &String) -> String {
    return if text.ends_with('\n') { text.clone() } else { format!("{text}\n") };
}

/// The lines of a message without those that start with the comment character,
/// as `--cleanup=strip` drops them.
pub fn strip_comment_lines(text: &str) -> String {
    return text.split('\n').filter(is_not_comment_line).collect::<Vec<&str>>().join("\n");
}

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
        let Some(bytes) = read_the_message_file(MessageFileQuestion { path: cwd.join(file) }) else { return Ok(None) };
        let Ok(text) = String::from_utf8(bytes) else { return Ok(None) };
        vec![text]
    } else if let Some(revision) = parsed.reuse.as_deref() {
        let Some((author, message)) = reused_commit(context, cwd, revision)? else { return Ok(None) };
        reused_author = Some(author);
        vec![message]
    } else if let Some((kind, revision)) = parsed.fixup.as_ref() {
        let Some(subject) = subject_of(context, cwd, revision.as_str())? else { return Ok(None) };
        let prefix: &str = match kind {
            FixupKind::Fixup => "fixup!",
        };
        vec![format!("{prefix} {subject}")]
    } else if parsed.messages.is_empty() && parsed.amend && let Some(previous) = previous_message {
        // An amend with no message option keeps the message of the commit it replaces.
        vec![previous.to_string()]
    } else {
        parsed.messages.clone()
    };
    let Some(message): Option<String> = message_of_sources(sources, parsed) else { return Ok(None) };
    let trailers: Vec<String> = trailer_lines(parsed, committer);
    if trailers.is_empty() {
        return Ok(Some(Built { message, reused_author }));
    }
    return trailed_answered_by(run_git(context, &trailers_request(cwd, trailers.as_slice(), message)), reused_author);
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
        let Some(bytes) = message_file_awaiting(cwd.join(file).as_path()).await else { return Ok(None) };
        let Ok(text) = String::from_utf8(bytes) else { return Ok(None) };
        vec![text]
    } else if let Some(revision) = parsed.reuse.as_deref() {
        let Some((author, message)) = reused_commit_awaiting(context, cwd, revision).await? else { return Ok(None) };
        reused_author = Some(author);
        vec![message]
    } else if let Some((kind, revision)) = parsed.fixup.as_ref() {
        let Some(subject) = subject_of_awaiting(context, cwd, revision.as_str()).await? else { return Ok(None) };
        let prefix: &str = match kind {
            FixupKind::Fixup => "fixup!",
        };
        vec![format!("{prefix} {subject}")]
    } else if parsed.messages.is_empty() && parsed.amend && let Some(previous) = previous_message {
        // An amend with no message option keeps the message of the commit it replaces.
        vec![previous.to_string()]
    } else {
        parsed.messages.clone()
    };
    let Some(message): Option<String> = message_of_sources(sources, parsed) else { return Ok(None) };
    let trailers: Vec<String> = trailer_lines(parsed, committer);
    if trailers.is_empty() {
        return Ok(Some(Built { message, reused_author }));
    }
    return trailed_answered_by(run_git_awaiting(context, &trailers_request(cwd, trailers.as_slice(), message)).await, reused_author);
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
fn message_of_sources(sources: Vec<String>, parsed: &ServiceCommitArgs) -> Option<String> {
    let texts: Vec<String> = if parsed.strip { sources.iter().map(String::as_str).map(strip_comment_lines).collect() } else { sources };
    let cleaned: Option<String> = if parsed.verbatim {
        Some(texts.iter().map(with_final_line_break).collect::<Vec<String>>().join("\n"))
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
fn trailer_lines(parsed: &ServiceCommitArgs, committer: &str) -> Vec<String> {
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
fn trailers_request(cwd: &Path, trailers: &[String], message: String) -> GitRequest {
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
fn trailed_answered_by(answered: std::io::Result<GitOutput>, reused_author: Option<String>) -> Result<Option<Built>, TransactionStop> {
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("preparing the commit message failed: {error:?}"))),
    };
    if output.code != Some(0) {
        return Ok(None);
    }
    return match String::from_utf8(output.stdout) {
        Ok(message) => Ok(Some(Built { message, reused_author })),
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
    let Ok(bytes) = std::fs::read(question.path.as_path()) else { return None };
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
async fn message_file_within(ceiling: Duration, path: &Path) -> Option<Vec<u8>> {
    let question: MessageFileQuestion = MessageFileQuestion { path: path.to_path_buf() };
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
