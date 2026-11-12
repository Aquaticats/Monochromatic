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
//!  A marked path whose worktree file is gone is no edit of theirs:
//! a `skip-worktree` entry with no file is a sparse
//!  checkout's own state,
//! and Git reads a missing worktree file as one its update writes,
//! so both stay Git's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hidden = pathsTheStatusHides(context, root, touched); // marked entries whose bytes or mode differ from the index
//! ```

use crate::command::rule_worktree_dirty::{IGNORED_PATHS_QUERY_BYTES, TOP_LITERAL_PATHSPEC};
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_carry::{bytes_at, object_name_awaiting};
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

/// The tag `git ls-files -v` prints for a `skip-worktree` entry.
const SKIP_WORKTREE_TAG: &str = "S";

/// Whether one byte is the NUL that ends each entry of a `-z` listing.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte is the tab between an entry's header and its path.
fn is_a_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// One index entry one of the two marks hides from status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Marked = { mode: string; object: string; path: Uint8Array };
/// ```
struct Marked {
    mode: String,
    object: String,
    path: Vec<u8>,
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
        // Nothing at the path is no edit of the person's: a marked entry with no file is a sparse checkout's own state, and a
        //  missing worktree file is one Git's update writes back.
        let Some((bytes, mode)) = bytes_at(location.as_path())? else {
            continue;
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
///  The `git ls-files -v --stage -z` entries that carry a mark which hides them from status.
/// Why:
///   The mark is the first field `-v` prints,
/// and only an entry at stage `0` is a file's own entry,
/// so an
///  unmerged entry is left to the merge's own answer.
///  A lower-case tag is the assume-unchanged bit and an upper-case `S` is
/// skip-worktree,
///  as Git prints them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markedEntries(output) { return fields(output).flatMap(parse).filter(isHiddenMark); }
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
        if *stage != "0" || !is_a_hidden_mark(tag) {
            continue;
        }
        found.push(Marked { mode: String::from(*mode), object: String::from(*object), path: field[tab + 1..].to_vec() });
    }
    return found;
}

/// What:
///  Whether one `git ls-files -v` tag marks the entry as hidden from status.
/// Why:
///   `-v` lower-cases the tag of an assume-unchanged entry and prints `S` for a `skip-worktree` one,
/// so the
///  two marks are the only tags this read may treat as hidden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAHiddenMark = (tag) => tag === 'S' || (tag.length === 1 && tag === tag.toLowerCase());
/// ```
fn is_a_hidden_mark(tag: &str) -> bool {
    let mut letters = tag.bytes();
    return match (letters.next(), letters.next()) {
        (Some(letter), None) => letter == SKIP_WORKTREE_TAG.as_bytes()[0] || letter.is_ascii_lowercase(),
        _ => false,
    };
}

/// Reading the marks and comparing what the worktree holds with what the index records.
#[cfg(test)]
#[path = "merge_isolation_marks_the_status_hides_tests.rs"]
mod tests;
