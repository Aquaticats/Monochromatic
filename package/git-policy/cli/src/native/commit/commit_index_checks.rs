//! What:
//!  The questions about the private index that this process answers without a Git start:
//!  whether it holds exactly the base revision's tree,
//!  and which of the selected paths the real index stages as additions.
//! Why:
//!  An index that already holds the base has nothing for `read-tree --reset` to remove,
//!  so the in-process reader saves a Git start and the rewrite of the whole index;
//!  a question the reader declines is answered the slow way,
//!  never by guessing.
//!  Native `git commit -- <path>` takes an addition's worktree content without asking an ignore rule,
//!  and the private index starts from the base,
//!  so those additions have to be carried into it (issue #1807).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! readerAnswer(['diff-index', '--cached', '--name-only', '-z', '--no-renames', base], privateIndex)?.stdout.length === 0
//! ```

/// The selection a commit names,
///  whose pathspec file this step reads.
use crate::commit_index_selection::IndexSelection;
/// The staged additions carried into the explicit-path index,
/// and the pathspec-file reader.
use crate::commit_staged_additions::{
    StagedAddition,
    carry_additions,
    pathspecs_of_file,
    staged_additions_selected,
};
/// The private index and object store a commit's preparation selects,
///  and the request builder of its Git starts.
use crate::private_index_request::{
    CaptureIndexes,
    indexed_request,
};
/// The transaction Git wrapper:
///  the context a question is asked in,
///  and the failure a carried addition reports.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// What:
///  Whether the private index holds exactly the tree of the base revision,
/// answered without starting Git.
/// Why:
///   The in-process reader of `diff-index --cached` lists the paths whose entry differs from the tree;
/// it declines an index
///  it does not fully understand (unmerged,
/// intent-to-add,
/// sparse,
/// split) and a tree it cannot read,
/// and a declined question
///  is "not known to be equal",
/// which takes the slow way.
///  The private index is the one asked and not the captured one it was copied from:
///  a copy the intent-to-add entries were left out of is missing files the base holds,
///  and the reset below is what puts them back into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readerAnswer(['diff-index', '--cached', '--name-only', '-z', '--no-renames', base], privateIndex)?.stdout.length === 0
/// ```
pub(crate) fn index_equals_base(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    base_revision: &str,
) -> bool {
    // The question is answered in process from the whole captured index and the base tree, which no Git child covers, so the
    // record names it: an explicit-path preparation whose milliseconds went here would otherwise look unexplained.
    let began: std::time::Instant = std::time::Instant::now();
    let equals: bool = reads_itself_as_the_base(context, cwd, indexes, base_revision);
    crate::prepare_work_spans::note_span(
        crate::prepare_work_spans::INDEX_EQUALS_BASE,
        began.elapsed(),
    );
    return equals;
}

/// What:
///  `index_equals_base` without the note of how long the question took.
/// Why:
///   The note covers the question a reader declined as well as the one it answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readerAnswer(...)?.stdout.length === 0
/// ```
fn reads_itself_as_the_base(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    base_revision: &str,
) -> bool {
    let request = indexed_request(
        context,
        cwd,
        &[
            "diff-index",
            "--cached",
            "--name-only",
            "-z",
            "--no-renames",
            base_revision,
        ],
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    return match crate::repo_read_hook::try_request(context, &request) {
        Some(answer) => answer.code == Some(0) && answer.stdout.is_empty(),
        None => false,
    };
}

/// What:
///  Makes the selected paths that the real index stages as additions tracked in the private index.
/// Why:
///   Native `git commit -- <path>` takes such a path's worktree content without asking an ignore rule,
///  and the private index,
/// which starts from the base,
/// would otherwise apply the rule (issue #1807).
///  A pathspec file that this wrapper cannot read leaves the selection to Git alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await carrySelectedStagedAdditions({ context, cwd, indexes, selection });
/// ```
pub(crate) fn carry_selected_staged_additions(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    let mut pathspecs: Vec<OsString> = selection.pathspecs.to_vec();
    if let Some(file) = selection.pathspec_file {
        // A file that cannot be read now fails in `git add` with Git's own message.
        // The read is the plain one, so a named pipe at the name answers at once instead of holding the commit, and the
        //  handle's own type decides what is read; the bound is the one the selection readers take.
        let Ok(bytes) = crate::plain_file::read_plain_file(
            file,
            crate::commit_selection_staging::PATHSPEC_FILE_LIMIT,
        ) else {
            return Ok(());
        };
        let Some(listed) = pathspecs_of_file(bytes.as_slice(), selection.pathspec_file_nul) else {
            return Ok(());
        };
        pathspecs.extend(listed);
    }
    let additions: Vec<StagedAddition> = staged_additions_selected(
        context,
        cwd,
        indexes.captured_index.as_path(),
        selection.base_revision,
        pathspecs.as_slice(),
    )?;
    return carry_additions(
        context,
        cwd,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        additions.as_slice(),
    );
}
