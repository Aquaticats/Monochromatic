//! What:
//!  The paths a merge changes whose edit Git's ordinary status does not show,
//! because their index entry carries the
//!  `assume-unchanged` or `skip-worktree` mark,
//! and the comparison that tells an edit there from a path that still holds
//!  the branch's bytes.
//! Why:
//!   `git status`,
//! `git diff-files`,
//! `git diff-index` and `git ls-files --modified` all read a marked entry as "do not compare the
//!  worktree copy",
//! so a person's edit at such a path enters neither the paths a merge would write over nor the layers
//!  its carry keeps.
//!  Git's own update of the checkout compares the marked entry anyway and refuses the whole
//!  merge,
//! which the wrapper then reports as `merge-isolation/not-applied` (a cross-vendor audit of 0.6.1-alpha.10,
//!  "`assume-unchanged` and `skip-worktree` edits make the isolated merge fail").
//!  The marks are read here so the edit is work to carry
//!  like any other,
//! and `SPEC.md`,
//! "Merges and commits never fail",
//! "Carry edits forward",
//! applies to it unchanged.
//!  A marked path whose worktree file is gone is told apart by which mark it is:
//! a `skip-worktree` entry with no file is a sparse checkout's own state,
//! which Git's update writes back,
//! while an `assume-unchanged` one has no such reading and the absence is a deletion of the person's the merge would undo.
//!  A name that holds no bytes at all,
//! a directory,
//! a named pipe,
//! a socket or a device,
//! is not read here:
//!  the gates that stop a merge for such an entry own it, and reading
//! a pipe could wait for a writer nothing promises.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hidden = pathsTheStatusHides(context, root, touched); // marked entries whose bytes or mode differ from the index
//! ```

use crate::command::rule_worktree_dirty::{IGNORED_PATHS_QUERY_BYTES, TOP_LITERAL_PATHSPEC};
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_carry::{WorktreeEntry, object_name_awaiting, worktree_entry_at};
use crate::merge_isolation_carry_reads::run_bytes_awaiting;
use crate::merge_isolation_changes::worktree_path;
/// One path's exact bytes,
/// as the system reads them.
use crate::policy::path_bytes::os_from_path_bytes;
use crate::transaction_git::{GitContext, GitOutput};
/// `OsString` spells a path Git reads as bytes that need not be text.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The tag `git ls-files -v` prints for a `skip-worktree` entry,
/// upper case when that bit stands alone and lower case when the assume-unchanged bit is set with it.
const SKIP_WORKTREE_TAG: u8 = b'S';

/// Whether one byte is the NUL that ends each entry of a `-z` listing.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte is the tab between an entry's header and its path.
fn is_a_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// One index entry one of the two marks hides from status,
/// and which mark it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Marked = { mark: HiddenMark; mode: string; object: string; path: Uint8Array };
/// ```
struct Marked {
    mark: HiddenMark,
    mode: String,
    object: String,
    path: Vec<u8>,
}

/// One of the two index bits that hide a path from status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HiddenMark = 'assume-unchanged' | 'skip-worktree';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
enum HiddenMark {
    /// The `assume-unchanged` bit:
    /// Git does not compare the worktree copy with the entry.
    AssumeUnchanged,
    /// The `skip-worktree` bit,
    /// which a sparse checkout sets for a path it leaves out of the worktree.
    SkipWorktree,
}

impl HiddenMark {
    /// What:
    ///  Whether this mark on its own explains a worktree copy that is not there.
    /// Why:
    ///   A `skip-worktree` entry with no file is a sparse checkout's own state,
    /// and Git's update of the checkout
    ///  writes such a file back,
    /// so there is nothing of the person's to keep.
    ///  An `assume-unchanged` entry has no such reading:
    /// nothing but the person removes that file,
    /// and the
    ///  merge's update would put it back,
    /// undoing a deletion they made with no word about it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const absenceIsTheMarksOwn = (mark) => mark === 'skip-worktree';
    /// ```
    fn absence_is_the_marks_own(&self) -> bool {
        return *self == HiddenMark::SkipWorktree;
    }
}

/// What:
///  The paths of `touched` whose index entry carries one of the two marks
///  and whose worktree entry differs from
///  what the index holds.
/// Why:
///   Exactly these are the person's work in the paths a merge changes that Git's status hides,
/// and they are
///  the ones a capture has to hold before the checkout changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hidden = await pathsTheStatusHides({ context, root, touched });
/// ```
pub fn paths_the_status_hides(context: &GitContext, root: &Path, touched: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    return match run_on_runtime(paths_the_status_hides_awaiting(context, root, touched)) {
        Ok(found) => found,
        Err(refusal) => Err(format!("{refusal}")),
    };
}

/// What:
///  The same read of the marks,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every entry whose marks are read is read again through Git,
///  and a synchronous start registers its pipes by entering the runtime,
/// which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hidden = await pathsTheStatusHides({ context, root, touched });
/// ```
async fn paths_the_status_hides_awaiting(context: &GitContext, root: &Path, touched: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    if touched.is_empty() {
        return Ok(Vec::new());
    }
    let mut hidden: Vec<Vec<u8>> = Vec::new();
    for query in pathspec_queries(touched) {
        let mut arguments: Vec<OsString> = vec![
            OsString::from("--no-literal-pathspecs"),
            OsString::from("ls-files"),
            OsString::from("-v"),
            OsString::from("--stage"),
            OsString::from("-z"),
            OsString::from("--"),
        ];
        arguments.extend(query);
        let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
        if !output.succeeded() {
            return Err(format!("git ls-files failed: {}", output.error_text()));
        }
        let found: Vec<Vec<u8>> = hidden_work_awaiting(context, root, touched, output.stdout.as_slice()).await?;
        hidden.extend(found);
    }
    return Ok(hidden);
}

/// What:
///  The paths of one listing whose worktree entry differs from what its marked index entry records.
/// Why:
///   Every entry is compared by the clean-filtered object its bytes make and the mode it has,
/// and the listing is
///  only a candidate set:
/// a path outside the merge's own or one with no worktree file is no work to carry.
///  An entry that holds no bytes at all,
/// a directory,
/// a named pipe,
/// a socket or a device,
///  is left to the gates that
/// stop the merge for such an entry before the first byte is written:
/// reading one could wait for a
///  writer that never comes,
/// and no layer of the carry can hold it.
///  A marked path whose worktree file is gone is no edit of the person's only where the mark is
///  the reason it is gone (`HiddenMark::absence_is_the_marks_own`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of markedEntries(stdout)) if (touched.has(entry.path) && differs(entry)) hidden.push(entry.path);
/// ```
async fn hidden_work_awaiting(context: &GitContext, root: &Path, touched: &[Vec<u8>], listing: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut hidden: Vec<Vec<u8>> = Vec::new();
    for entry in marked_entries(listing) {
        // A listing answer that names a path outside the merge's own is not one of its paths.
        if !touched.contains(&entry.path) {
            continue;
        }
        let location: PathBuf = worktree_path(root, entry.path.as_slice());
        let found: WorktreeEntry = match worktree_entry_at(location.as_path()) {
            Ok(found) => found,
            // A name that cannot be read is not an answer about the person's work, so the merge stays native Git's.
            Err(reason) => return Err(reason),
        };
        let (bytes, mode): (Vec<u8>, &'static str) = match found {
            WorktreeEntry::Holds { bytes, mode } => (bytes, mode),
            WorktreeEntry::NoBytes(_) => continue,
            // Nothing at the path is an edit only when the mark is not the reason it is gone: a sparse checkout leaves
            //  the file out, while an `assume-unchanged` entry with nothing at its path is a deletion of the person's.
            WorktreeEntry::Absent => {
                if !entry.mark.absence_is_the_marks_own() {
                    hidden.push(entry.path);
                }
                continue;
            }
        };
        if mode == entry.mode.as_str()
            && object_name_awaiting(context, root, entry.path.as_slice(), bytes.as_slice()).await? == entry.object
        {
            continue;
        }
        hidden.push(entry.path);
    }
    return Ok(hidden);
}

/// What:
///  The path arguments of one or more listings,
///  each naming one repository path exactly and anchored at its top.
/// Why:
///   A tree path is data,
/// not a pattern and not a name relative to the directory the command was typed in,
/// and one
///  start carries what the system accepts,
/// so a merge that changes more paths than one start holds is asked in bounded
///  passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queries = chunk(touched.map((p) => concat(TOP_LITERAL_PATHSPEC, p)), IGNORED_PATHS_QUERY_BYTES);
/// ```
fn pathspec_queries(paths: &[Vec<u8>]) -> Vec<Vec<OsString>> {
    let mut queries: Vec<Vec<OsString>> = Vec::new();
    let mut current: Vec<OsString> = Vec::new();
    let mut carried: usize = 0;
    for path in paths {
        let mut spec: Vec<u8> = TOP_LITERAL_PATHSPEC.to_vec();
        spec.extend_from_slice(path);
        if carried > 0 && carried + spec.len() > IGNORED_PATHS_QUERY_BYTES {
            // The finished chunk goes and the same allocation starts the next one.
            queries.push(std::mem::take(&mut current));
            carried = 0;
        }
        carried += spec.len();
        current.push(os_from_path_bytes(spec.as_slice()));
    }
    if carried > 0 {
        queries.push(current);
    }
    return queries;
}

/// What:
///  The `git ls-files -v --stage -z` entries that carry a mark which hides them from status,
/// and which mark that is.
/// Why:
///   The mark is the first field `-v` prints,
/// and only an entry at stage `0` is a file's own entry,
/// so an
///  unmerged entry is left to the merge's own answer.
///  A lower-case tag is the assume-unchanged bit and an upper-case `S` is
/// skip-worktree,
///  as Git prints them;
/// a `skip-worktree` entry that also carries the assume-unchanged bit prints as a lower-case `s`,
///  and stays skip-worktree here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedEntries(output) { return fields(output).flatMap(parse).filter(hasAHiddenMark); }
/// ```
fn marked_entries(output: &[u8]) -> Vec<Marked> {
    let mut found: Vec<Marked> = Vec::new();
    for field in output.split(is_nul) {
        let Some(tab) = field.iter().position(is_a_tab) else {
            continue;
        };
        let header: String = String::from_utf8_lossy(&field[..tab]).into_owned();
        let parts: Vec<&str> = header.split(' ').collect();
        let [tag, mode, object, stage] = parts.as_slice() else {
            continue;
        };
        if *stage != "0" {
            continue;
        }
        let Some(mark) = hidden_mark_of(tag) else {
            continue;
        };
        found.push(Marked { mark, mode: String::from(*mode), object: String::from(*object), path: field[tab + 1..].to_vec() });
    }
    return found;
}

/// What:
///  Which hidden mark one `git ls-files -v` tag stands for,
/// when it stands for one.
/// Why:
///   `-v` lower-cases the tag of an assume-unchanged entry and prints `S` for a `skip-worktree` one,
/// which
///  is `s` when both bits are set,
/// so those are the only tags this read may treat as hidden and the two
///  marks are told apart by the letter alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hiddenMarkOf = (tag) => tag === 'S' || tag === 's' ? 'skip-worktree' : /^[a-z]$/.test(tag) ? 'assume-unchanged' : null;
/// ```
fn hidden_mark_of(tag: &str) -> Option<HiddenMark> {
    let mut letters = tag.bytes();
    let letter: u8 = match (letters.next(), letters.next()) {
        (Some(letter), None) => letter,
        _ => return None,
    };
    if letter.eq_ignore_ascii_case(&SKIP_WORKTREE_TAG) {
        return Some(HiddenMark::SkipWorktree);
    }
    if letter.is_ascii_lowercase() {
        return Some(HiddenMark::AssumeUnchanged);
    }
    return None;
}

/// Reading the marks and comparing what the worktree holds with what the index records.
#[cfg(test)]
#[path = "merge_isolation_marks_the_status_hides_tests.rs"]
mod tests;
