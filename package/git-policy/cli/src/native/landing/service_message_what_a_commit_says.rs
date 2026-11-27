//! What:
//!  What an existing commit says:
//!  its message and its subject,
//!  read out of Git's own record of it,
//!  for an amend that reuses them and for a fixup that names one.
//! Why:
//!   An amend or a `--fixup` commit repeats the words of the commit it replaces,
//!  and a subject is the first line after the encoding header Git wrote;
//!  both are read from the raw commit rather than from `git log`,
//!  so no formatting rule of a newer Git can change them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = await reusedCommitAwaiting(invocation, revision);
//! const subject = await subjectOfAwaiting(invocation, revision);
//! ```

use super::service_message_the_text_steps::{
    author_of_header_line,
    is_encoding_header,
    without_final_line_breaks,
};
use crate::commit_transaction::{
    TransactionStop,
    failed,
};
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::Path;

/// The text of a Git query that must succeed for the line to be covered.
fn query(
    context: &GitContext,
    cwd: &Path,
    arguments: &[String],
) -> Result<Option<Vec<u8>>, TransactionStop> {
    return text_answered_by(run_git(context, &GitRequest::new(cwd, arguments)));
}

/// `query` with its Git child awaited on the runtime.
async fn query_awaiting(
    context: &GitContext,
    cwd: &Path,
    arguments: &[String],
) -> Result<Option<Vec<u8>>, TransactionStop> {
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
fn text_answered_by(
    answered: std::io::Result<GitOutput>
) -> Result<Option<Vec<u8>>, TransactionStop> {
    let output = match answered {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("reading the repository failed: {error:?}"))),
    };
    return Ok(if output.code == Some(0) {
        Some(output.stdout)
    } else {
        None
    });
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
pub fn reused_commit(
    context: &GitContext,
    cwd: &Path,
    revision: &str,
) -> Result<Option<(String, String)>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(oid) = query(context, cwd, the_commit_named_by(revision).as_slice())? else {
        return Ok(None);
    };
    let named: String = String::from_utf8_lossy(oid.as_slice()).trim().to_string();
    let Some(raw) = query(context, cwd, the_raw_commit(named.as_str()).as_slice())? else {
        return Ok(None);
    };
    return Ok(reused_of(raw));
}

/// `reused_commit` with its Git children awaited on the runtime.
pub async fn reused_commit_awaiting(
    context: &GitContext,
    cwd: &Path,
    revision: &str,
) -> Result<Option<(String, String)>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(oid) = query_awaiting(context, cwd, the_commit_named_by(revision).as_slice()).await?
    else {
        return Ok(None);
    };
    let named: String = String::from_utf8_lossy(oid.as_slice()).trim().to_string();
    let Some(raw) = query_awaiting(context, cwd, the_raw_commit(named.as_str()).as_slice()).await?
    else {
        return Ok(None);
    };
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
    return vec![
        String::from("rev-parse"),
        String::from("--verify"),
        String::from("--quiet"),
        format!("{revision}^{{commit}}"),
    ];
}

/// The `cat-file` question that reads the raw object `named`.
fn the_raw_commit(named: &str) -> Vec<String> {
    return vec![
        String::from("cat-file"),
        String::from("commit"),
        named.to_string(),
    ];
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
    let Ok(text) = String::from_utf8(raw) else {
        return None;
    };
    let (header, message): (&str, &str) = text.split_once("\n\n")?;
    // A commit in another encoding, or with extra headers that change the message's meaning, is Git's to reuse.
    if header.lines().any(is_encoding_header) {
        return None;
    }
    let author: &str = header.lines().find_map(author_of_header_line)?;
    return Some((author.to_string(), message.to_string()));
}

/// What:
///  The subject Git puts after `fixup! ` for the commit `revision` names.
/// Why:
///  A `--fixup` commit's message is that prefix and another commit's subject,
/// so a commit Git cannot resolve gives
///  nothing and native `git commit` builds the line.
/// Both faces ask one question (`the_subject_of`) and read one answer
///  (`subject_answered_by`),
/// so neither can name another commit's subject than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await subjectOf(git, cwd, '-C' value)
/// ```
pub fn subject_of(
    context: &GitContext,
    cwd: &Path,
    revision: &str,
) -> Result<Option<String>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(bytes) = query(context, cwd, the_subject_of(revision).as_slice())? else {
        return Ok(None);
    };
    return Ok(subject_answered_by(bytes));
}

/// `subject_of` with its Git child awaited on the runtime.
pub async fn subject_of_awaiting(
    context: &GitContext,
    cwd: &Path,
    revision: &str,
) -> Result<Option<String>, TransactionStop> {
    if revision.starts_with('-') {
        return Ok(None);
    }
    let Some(bytes) = query_awaiting(context, cwd, the_subject_of(revision).as_slice()).await?
    else {
        return Ok(None);
    };
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
    return vec![
        String::from("log"),
        String::from("-1"),
        String::from("--format=%s"),
        revision.to_string(),
        String::from("--"),
    ];
}

/// The subject of a `log` answer,
/// or nothing when the bytes are not text.
fn subject_answered_by(bytes: Vec<u8>) -> Option<String> {
    return String::from_utf8(bytes).ok().map(without_final_line_breaks);
}
