//! What:
//!  The answers of one `git rev-parse` report:
//!  one answer with its terminator removed,
//!  the answers of a whole report,
//!  a pathname from one answer's bytes,
//!  and the object format.
//! Why:
//!  A report is bytes,
//!  and a pathname may hold the line feed that separates two answers or bytes that are not UTF-8;
//!  reading an answer as text lost both,
//!  so the caller counts the answers before it trusts their places.
//!  The layout capture reads every report through these,
//!  and they are a file of their own so its own file stays within the package's size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answers = splitReport(await git(['rev-parse', ...])); answers.length === 7 ? reportOf(answers) : askedFor()
//! ```

/// A pathname from the bytes Git printed for it,
///  and one answer without its terminating line feed.
use crate::hooks_path_values::{path_answer, without_the_terminator};
/// Failure text of one Git start the layout reports as unusable.
use crate::transaction_git::GitFailure;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  One answer of the layout report without the single line feed Git ends it with,
///  refusing an empty answer.
/// Why:
///   An empty answer means Git gave an incomplete layout,
///  which must stop the transaction before any mutation,
///  and a pathname ending in a line feed of its own keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = withoutTheTerminator(answer); if (value.length === 0) throw new Error('incomplete layout')
/// ```
pub(crate) fn required_answer(answer: &[u8]) -> Result<&[u8], GitFailure> {
    let value: &[u8] = without_the_terminator(answer);
    if value.is_empty() {
        return Err(GitFailure(String::from("Git returned incomplete repository layout.")));
    }
    return Ok(value);
}

/// What:
///  One reported pathname as a path,
///  resolved against the invocation directory.
/// Why:
///   Git may report a relative path,
/// and the transaction stores absolute ones;
/// the bytes Git printed are the path (`path_bytes.rs`),
///  so a name that is not UTF-8 is the name it printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reported = pathFromBytes(answer); isAbsolute(reported) ? reported : resolve(cwd, reported)
/// ```
pub(crate) fn absolute_reported(cwd: &Path, answer: &[u8]) -> Result<PathBuf, GitFailure> {
    let reported: PathBuf = path_answer(required_answer(answer)?);
    return Ok(if reported.is_absolute() {
        reported
    } else {
        cwd.join(reported)
    });
}

/// What:
///  Whether one byte ends an answer of the report.
/// Why:
///  Git writes one line feed after every answer,
///  and the report is split at those bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnswerBreak = (byte: number) => byte === 0x0a;
/// ```
fn is_answer_break(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The answers of one report,
///  split at the line feeds Git wrote between them.
/// Why:
///  Git ends every answer with one line feed and this one ends the report;
///  a pathname that holds a line feed of its own keeps it inside its answer,
///  which is why the caller counts the answers before it trusts their places.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutTheTerminator(stdout).split('\n')
/// ```
pub(crate) fn answers_of(stdout: &[u8]) -> Vec<&[u8]> {
    let body: &[u8] = without_the_terminator(stdout);
    if body.is_empty() {
        return Vec::new();
    }
    return body.split(is_answer_break).collect();
}

/// What:
///  The object format Git printed,
///  which is the one answer of the report that is text.
/// Why:
///  A format the wrapper cannot read is a failure and not a guess;
/// the paths of the layout are bytes and never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder('utf-8', { fatal: true }).decode(answer)
/// ```
pub(crate) fn object_format_of(answer: &[u8]) -> Result<String, GitFailure> {
    return match std::str::from_utf8(required_answer(answer)?) {
        Ok(text) => Ok(String::from(text)),
        Err(_) => Err(GitFailure(String::from("Git returned non-UTF-8 output."))),
    };
}
