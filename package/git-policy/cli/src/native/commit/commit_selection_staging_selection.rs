//! What:
//!  What one selection gives the staging step:
//!  its elements when they can be read,
//!  the pathspec file itself when it is too large to name one element per argument,
//!  and nothing when this reader cannot take that file.
//! Why:
//!  A question about which paths a selection takes must be asked about the same elements the staging takes,
//!  and a pathspec file is one form of the same selection.
//!  A file is read through `plain_file::read_plain_file`,
//!  so a named pipe at its name answers at once instead of holding the commit for a writer that may never come,
//!  and the handle's own type decides what is read rather than a look at the path that a swap could change.
//!  Every reason to decline answers as the staging did before this reading:
//!  the file is handed to Git itself and Git reports what it finds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Selected = { kind: 'items', items: string[] } | { kind: 'file' } | { kind: 'unreadable' };
//! ```

/// The selection a commit names,
///  whose pathspec file and elements this reader takes.
use crate::commit_index_selection::IndexSelection;
/// The staged additions' pathspec-file reader,
///  which answers no element when the file cannot be read.
use crate::commit_staged_additions::pathspecs_of_file;
/// `OsString` owns a whole element.
use std::ffi::OsString;

/// What:
///  What one selection gives this staging step:
/// its elements when they can be read,
/// the pathspec file itself when it is too large to name,
/// which is what an unreadable one leaves to Git,
/// and nothing for an empty selection.
/// Why:
///   Every shape stages differently,
/// and a reader that does not take a file must not change what Git does with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selected = { kind: 'items', items: string[] } | { kind: 'file' } | { kind: 'unreadable' };
/// ```
pub(crate) enum Selected {
    /// The selection's elements,
    /// from the command line or from a pathspec file.
    Items(Vec<OsString>),
    /// A pathspec file with no element.
    Empty,
    /// A pathspec file too large to name on one command line.
    File,
    /// A pathspec file this reader cannot take.
    Unreadable,
}

/// The most bytes of pathspec elements this wrapper puts on one command line instead of naming the pathspec file.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// above it the file is passed as a file with `--update`,
/// so the commit still takes the tracked paths and only a file nothing staged stays out;
/// one argument may not exceed 128 KiB (`MAX_ARG_STRLEN`) and a whole Linux command line stays under 2 MiB (`ARG_MAX`),
/// so this bound leaves room for the rest of the command.
pub(crate) const ITEMS_BYTES_LIMIT: u64 = 64 * 1024;

/// The most bytes of a pathspec file this reader takes when it asks Git about the same paths.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file larger than this is not a selection a commit can carry,
/// and its elements would hold twice its bytes in memory,
/// so this reader leaves the file to Git,
/// which reports what it finds when the private staging opens it.
/// The 1 GiB is the bound the in-process index reads already take (`INDEX_LIMIT_BYTES` in
/// `commit_named_ignored_files.rs`,
/// `conflict_side_guard.rs` and
/// `merge_second_pass_worktree.rs`).
pub(crate) const PATHSPEC_FILE_LIMIT: u64 = 1 << 30;

/// What:
///  Every element the selection names,
/// read from the pathspec file when it names one,
/// and nothing when this reader cannot take that file.
/// Why:
///   A question about which paths a selection takes must be asked about the same elements the staging takes,
/// and a pathspec file is one form of the same selection:
///  the command line carries its elements as arguments,
/// so a file's paths would otherwise go unasked.
///  Every reason to decline answers with nothing,
/// because the staging hands the file to Git itself and Git then reports what it finds,
/// which is the answer that was right before this reading.
///  The file is read through `plain_file::read_plain_file`,
/// so a named pipe at its name answers at once instead of holding the commit for a writer that may never come,
/// and the handle's own type decides what is read rather than a look at the path that a swap could change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const elements = selection.pathspecFile ? pathspecsOfFile(readPlainFile(selection.pathspecFile, PATHSPEC_FILE_LIMIT), selection.nul) : selection.pathspecs;
/// ```
pub(crate) fn selection_elements(selection: &IndexSelection) -> Option<Vec<OsString>> {
    let Some(file) = selection.pathspec_file else {
        return Some(selection.pathspecs.to_vec());
    };
    let bytes: Vec<u8> = crate::plain_file::read_plain_file(file, PATHSPEC_FILE_LIMIT).ok()?;
    return pathspecs_of_file(bytes.as_slice(), selection.pathspec_file_nul);
}

/// What:
///  The elements one selection names,
/// and which form the staging step takes for them.
/// Why:
///   Telling an exact file name from a directory or a pattern needs the elements,
/// and a file's elements are the same selection a command line carries.
///  The file is read on the handle's own type (`plain_file::read_plain_file`),
/// so a named pipe at its name is one this reader cannot take rather than a wait for a writer,
/// and a file past the bound is still passed to Git as a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selected = pathspecFile ? readPlainFile(pathspecFile, ITEMS_BYTES_LIMIT).length <= LIMIT ? items(pathspecsOfFile(...)) : { kind: 'file' } : items(pathspecs);
/// ```
pub(crate) fn selection_items(selection: &IndexSelection) -> Selected {
    let Some(file) = selection.pathspec_file else {
        if selection.pathspecs.is_empty() {
            return Selected::Empty;
        }
        return Selected::Items(selection.pathspecs.to_vec());
    };
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file(file, ITEMS_BYTES_LIMIT) {
        Ok(read) => read,
        // A file past the bound is handed to Git as a file; any other refusal is one this reader cannot take.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => return Selected::File,
        Err(_unreadable) => return Selected::Unreadable,
    };
    let Some(items) = pathspecs_of_file(bytes.as_slice(), selection.pathspec_file_nul) else {
        return Selected::Unreadable;
    };
    if items.is_empty() {
        return Selected::Empty;
    }
    return Selected::Items(items);
}
