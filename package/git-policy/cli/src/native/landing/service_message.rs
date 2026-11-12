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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { message, author } = await buildMessage(invocation, parsed, committer);
//! ```

use crate::commit_transaction::{CommitInvocation, TransactionStop, failed};
use crate::service_commit_args::{FixupKind, ServiceCommitArgs, cleanup_message};
use crate::service_commit_utf8_as_git_writes_it::repaired_text_bytes;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::OsString;
use std::path::Path;

/// What a command line's message came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Built {
    /// The whole commit message.
    pub message: String,
    /// The author line a reused commit supplies,
    /// `Name <email> seconds zone`.
    pub reused_author: Option<String>,
}

/// What:
///  One identity line (`Name <email> seconds zone`) from Git's own resolution of the variable,
///  with the given overrides,
///  and whether Git's byte repair changed it.
/// Why:
///   The identity is the person's bytes,
/// not text:
///  a name held in Latin-1 is not UTF-8,
///  and real `git commit` records the bytes of its own repair rather than a replacement character,
///  so the bytes are read as they are and repaired by Git's rule
///  (`service_commit_utf8_as_git_writes_it.rs`).
///  Nothing is answered when Git cannot make a line at all (a date it does not read),
///  so native `git commit` gives its own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = await resolvedIdentity(context, cwd, 'GIT_COMMITTER_IDENT', overrides); // { text, repaired } | undefined
/// ```
pub fn identity(context: &GitContext, cwd: &Path, variable: &str, overrides: &[(&str, String)]) -> Result<Option<(String, bool)>, TransactionStop> {
    let mut request: GitRequest = GitRequest::new(cwd, &["var", variable]);
    for (name, value) in overrides {
        request.environment.push((OsString::from(name), OsString::from(value)));
    }
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("reading the commit identity failed: {error:?}"))),
    };
    if output.code != Some(0) {
        return Ok(None);
    }
    // Git writes the line and one line break; a name cannot hold one, because Git removes those from it.
    let line: &[u8] = output.stdout.strip_suffix(b"\n").unwrap_or(output.stdout.as_slice());
    let (text, repaired): (Option<String>, bool) = repaired_text_bytes(line);
    return Ok(text.map(|found| { return (found, repaired); }));
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
fn query(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<Option<Vec<u8>>, TransactionStop> {
    let output = match run_git(context, &GitRequest::new(cwd, arguments)) {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("reading the repository failed: {error:?}"))),
    };
    return Ok(if output.code == Some(0) { Some(output.stdout) } else { None });
}

/// The raw object of a commit named by the person:
/// its author line and its message.
fn reused_commit(context: &GitContext, cwd: &Path, revision: &str) -> Result<Option<(String, String)>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(oid) = query(context, cwd, &["rev-parse", "--verify", "--quiet", format!("{revision}^{{commit}}").as_str()])? else { return Ok(None) };
    let Some(raw) = query(context, cwd, &["cat-file", "commit", String::from_utf8_lossy(oid.as_slice()).trim()])? else { return Ok(None) };
    let Ok(text) = String::from_utf8(raw) else { return Ok(None) };
    let Some((header, message)) = text.split_once("\n\n") else { return Ok(None) };
    // A commit in another encoding, or with extra headers that change the message's meaning, is Git's to reuse.
    if header.lines().any(is_encoding_header) {
        return Ok(None);
    }
    let Some(author) = header.lines().find_map(author_of_header_line) else { return Ok(None) };
    return Ok(Some((author.to_string(), message.to_string())));
}

/// The subject Git puts after `fixup! ` or `squash! `.
fn subject_of(context: &GitContext, cwd: &Path, revision: &str) -> Result<Option<String>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(bytes) = query(context, cwd, &["log", "-1", "--format=%s", revision, "--"])? else { return Ok(None) };
    return Ok(String::from_utf8(bytes).ok().map(without_final_line_breaks));
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
        let Ok(bytes) = std::fs::read(cwd.join(file)) else { return Ok(None) };
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
    let texts: Vec<String> = if parsed.strip { sources.iter().map(String::as_str).map(strip_comment_lines).collect() } else { sources };
    let cleaned: Option<String> = if parsed.verbatim {
        Some(texts.iter().map(with_final_line_break).collect::<Vec<String>>().join("\n"))
    } else {
        cleanup_message(texts.as_slice())
    };
    // A message is empty only when nothing in it but Git's own white space is left: a no-break space is text to Git.
    let message: String = match cleaned {
        Some(text) if !text.trim_matches(crate::service_commit_args::git_takes_off_the_end).is_empty() || parsed.verbatim => text,
        _ if parsed.allow_empty_message => String::new(),
        _ => return Ok(None),
    };
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
    if trailers.is_empty() {
        return Ok(Some(Built { message, reused_author }));
    }
    let mut arguments: Vec<String> = vec![String::from("interpret-trailers")];
    for trailer in &trailers {
        arguments.push(String::from("--trailer"));
        arguments.push(trailer.clone());
    }
    let mut request: GitRequest = GitRequest::new(cwd, arguments.as_slice());
    request.input = Some(message.into_bytes());
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("preparing the commit message failed: {error:?}"))),
    };
    if output.code != Some(0) {
        return Ok(None);
    }
    return Ok(match String::from_utf8(output.stdout) {
        Ok(trailed) => Some(Built { message: trailed, reused_author }),
        Err(_not_text) => None,
    });
}
