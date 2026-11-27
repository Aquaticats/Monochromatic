//! What:
//!  A record read back:
//!  the kind its first line names,
//!  the branch,
//!  the commit,
//!  and the words the lines after them carry.
//! Why:
//!  A record may have been written by this release or by the one before the data folder's rename,
//!  and a file without a first line is the finished text an earlier release wrote;
//!  one reading serves the step that shows a record and the step that settles it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseRecord(text: string, reportMagic: string): ParsedRecord | undefined;
//! ```

/// Import the first lines,
/// the field names and the kinds a record can carry,
/// with the earlier spelling one of them may be written in.
use super::push_failure_notice_record_text::{
    BRANCH_FIELD,
    COMMIT_FIELD,
    LOST_MAGIC,
    RECORD_MAGIC,
    RecordKind,
    SKIPPED_MAGIC,
    UNFINISHED_MAGIC,
    UNFINISHED_NO_RETRY_MAGIC,
    earlier_spelling,
};
/// Import the data folder's names,
/// because a record written before the rename carries the earlier spelling of its first line.
use crate::wrapper_names_old_and_new::DATA_FOLDER;

/// What:
///  The current first line a record's first line stands for:
/// the line itself,
/// or the same line in the spelling an earlier
///  release wrote (`cli-git push notice 1`).
/// Why:
///   The folder of the records is moved to its current name in place,
/// so records written before the rename are still there with
///  their own first lines,
/// and they are records of the same fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const currentMagic = (line: string) => MAGICS.find((magic) => line === magic || line === magic.replace(CURRENT, EARLIER))
/// ```
/// The first line a record of a push that succeeded carries is named by the caller,
/// because that line belongs to the reports module,
/// and this file reads a record without reaching for another module's records.
fn current_magic_of(
    line: &str,
    report_magic: &'static str,
) -> Option<&'static str> {
    for current in [
        RECORD_MAGIC,
        UNFINISHED_MAGIC,
        UNFINISHED_NO_RETRY_MAGIC,
        SKIPPED_MAGIC,
        report_magic,
        LOST_MAGIC,
    ] {
        let earlier: Option<String> = current
            .strip_prefix(DATA_FOLDER.current)
            .map(earlier_spelling);
        if line == current || earlier.as_deref() == Some(line) {
            return Some(current);
        }
    }
    return None;
}

/// The branch,
/// the commit and the kind of a record written by this release or by the release before the rename.
pub(super) fn parse_record<'a>(
    text: &'a str,
    report_magic: &'static str,
) -> Option<(&'a str, &'a str, RecordKind<'a>)> {
    let mut lines = text.splitn(5, '\n');
    let magic: &str = current_magic_of(lines.next()?, report_magic)?;
    let branch: &str = lines.next()?.strip_prefix(BRANCH_FIELD)?;
    let commit: &str = lines.next()?.strip_prefix(COMMIT_FIELD)?;
    if !lines.next()?.is_empty() {
        return None;
    }
    let reason: &str = lines.next().unwrap_or("");
    return Some((
        branch,
        commit,
        match magic {
            LOST_MAGIC => RecordKind::Lost,
            SKIPPED_MAGIC => RecordKind::Skipped(reason),
            UNFINISHED_MAGIC => RecordKind::Unfinished(reason),
            UNFINISHED_NO_RETRY_MAGIC => RecordKind::UnfinishedWithNoRetry(reason),
            other if other == report_magic => RecordKind::Reported(reason),
            _ => RecordKind::Failed(reason),
        },
    ));
}
