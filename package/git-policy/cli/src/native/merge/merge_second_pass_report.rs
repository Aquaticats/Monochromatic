//! What:
//!  The words that list a merged part,
//! the journal entry for a merged file,
//! and the sentence that says how to read the blobs a part names.
//! Why:
//!  A person checks a resolution by what a part line says,
//! and reads an input after the pass only through the object name that line gives;
//! the rendering is nothing the merge itself needs,
//! so it lives in a file of its own
//!  that `merge_second_pass.rs` imports and that imports nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // reportLines(prefix, merged) / journalEntry(operation, merged) / journalMerged(identifier, operation, merged)
//! ```

use crate::client_journal::record_entry_as;
use crate::diagnostic_log::debug;
use crate::journald_native::Priority;
use crate::merge_second_pass::Merged;
use crate::merge_second_pass_conflicts::{Conflict, Stage};
use crate::path_bytes::path_message_text;
use crate::word_merge::{LineSpan, MergedPart};

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeReport";

/// The words every line about a merged part carries,
/// so a reader finds them in a long output.
pub const REPORT_MARK: &str = "word-level merge";

/// The journal field that names the file of an entry.
pub const FILE_FIELD: &str = "CLI_GIT_WORD_MERGE_FILE";

/// The journal field that names the operation that produced the conflict.
pub const OPERATION_FIELD: &str = "CLI_GIT_WORD_MERGE_OPERATION";

/// The journal field that counts the parts of the file that were merged by word.
pub const PARTS_FIELD: &str = "CLI_GIT_WORD_MERGE_PARTS";

/// The journal field that names the three inputs of a merged file,
/// so `git cat-file blob <oid>` reaches each of them after the stages are gone.
pub const BLOBS_FIELD: &str = "CLI_GIT_WORD_MERGE_BLOBS";

/// The sentence that says how to read an input a part line names.
pub const INPUTS_HINT: &str = "Each part line ends with the three blobs it came from; `git cat-file blob <oid>` shows one.";

/// The most characters of a merged part a report line shows.
pub const EXCERPT_CHARACTERS: usize = 100;

/// The words for a run of lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// span.count === 0 ? `no line (before line ${span.first})` : span.count === 1 ? `line ${span.first}` : `lines ${span.first} to ${span.first + span.count - 1}`
/// ```
fn span_text(span: &LineSpan) -> String {
    return match span.count {
        0 => format!("no line (before line {})", span.first),
        1 => format!("line {}", span.first),
        count => format!("lines {} to {}", span.first, span.first + count - 1),
    };
}

/// What:
///  What a merged part became,
/// as one escaped line of at most `EXCERPT_CHARACTERS` characters.
/// Why:
///   The reader sees the new text without opening the file;
/// escaping keeps a terminal control sequence in a file from
///  reaching the terminal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(text).slice(0, EXCERPT_CHARACTERS)
/// ```
fn excerpt(merged: &[u8]) -> String {
    let escaped: String = String::from_utf8_lossy(merged).escape_debug().to_string();
    return match escaped.char_indices().nth(EXCERPT_CHARACTERS) {
        Some((end, _)) => format!("\"{}...\"", &escaped[..end]),
        None => format!("\"{escaped}\""),
    };
}

/// What:
///  The words that name one input of a part as a blob that outlives the pass.
/// Why:
///   The stages the part was merged from are gone from the index once the file is staged as resolved,
/// so
///  `git show :2:<path>` answers nothing;
/// the object name is the one handle a person can still open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blobText = (stage?: Stage) => (stage ? `blob ${stage.oid}` : 'no blob');
/// ```
fn blob_text(stage: Option<&Stage>) -> String {
    let Some(found) = stage else {
        return String::from("no blob");
    };
    return format!("blob {}", found.oid);
}

/// One part as a sentence fragment:
/// where it was in the three texts,
/// what it became,
/// and the three blobs those texts are.
pub fn part_text(part: &MergedPart, stages: &Conflict) -> String {
    return format!(
        "base {}, ours {}, theirs {} became {} from base {}, ours {}, theirs {}",
        span_text(&part.base),
        span_text(&part.ours),
        span_text(&part.theirs),
        excerpt(part.merged.as_slice()),
        blob_text(stages.base.as_ref()),
        blob_text(stages.ours.as_ref()),
        blob_text(stages.theirs.as_ref())
    );
}

/// What:
///  The lines that list a merged file:
/// one per part,
/// or one for the file when it needed no word pass.
/// Why:
///   The owner's decision is that every merged part is visible by file and by part.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportLines(prefix: string, merged: Merged): string[];
/// ```
pub fn report_lines(prefix: &str, merged: &Merged) -> Vec<String> {
    let path: String = path_message_text(merged.path.as_slice());
    if merged.parts.is_empty() {
        return vec![format!("{prefix}{REPORT_MARK}: {path}: resolved without a word pass (every region took one side)")];
    }
    let mut lines: Vec<String> = Vec::new();
    for (at, part) in merged.parts.iter().enumerate() {
        lines.push(format!("{prefix}{REPORT_MARK}: {path}: part {} of {}: {}", at + 1, merged.parts.len(), part_text(part, &merged.stages)));
    }
    return lines;
}

/// The object name of one input,
/// or `none` when the merge had no such stage.
fn blob_oid(stage: Option<&Stage>) -> String {
    let Some(found) = stage else {
        return String::from("none");
    };
    return found.oid.clone();
}

/// What:
///  The message and the fields of the journal entry for a merged file.
/// Why:
///   One entry names the file,
/// the operation,
/// every part and the three blobs,
/// so `journalctl` finds a merge by file and
///  `git cat-file blob` reaches each input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalEntry('merge', merged) // { message, fields }
/// ```
pub fn journal_entry(operation: &str, merged: &Merged) -> (String, Vec<(&'static str, String)>) {
    let path: String = path_message_text(merged.path.as_slice());
    let message: String = report_lines("", merged).join("; ");
    let blobs: String = format!(
        "base={},ours={},theirs={}",
        blob_oid(merged.stages.base.as_ref()),
        blob_oid(merged.stages.ours.as_ref()),
        blob_oid(merged.stages.theirs.as_ref())
    );
    return (
        message,
        vec![(FILE_FIELD, path), (OPERATION_FIELD, operation.to_owned()), (PARTS_FIELD, merged.parts.len().to_string()), (BLOBS_FIELD, blobs)],
    );
}

/// What:
///  Send one journal entry for a merged file under the identifier of the program that merged it.
/// Why:
///   The wrapper's standard error is a terminal or a capture that is gone when the command ends;
/// the journal keeps the
///  file and each part for later.
/// Code that runs inside the service sends under the service's identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordEntryAs(identifier, 'warning', `word-level merge: ${path}: ${parts.join('; ')}`, { CLI_GIT_WORD_MERGE_FILE: path, ... });
/// ```
pub fn journal_merged(identifier: &str, operation: &str, merged: &Merged) {
    let (message, fields) = journal_entry(operation, merged);
    let sent: bool = record_entry_as(identifier, Priority::Warning, message.as_str(), fields.as_slice());
    debug(TAG, format!("journal entry for a merged file: sent={sent}").as_str());
}
