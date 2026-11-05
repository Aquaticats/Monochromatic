//! What:
//!  Which committed files still hold conflict marker lines, read from the committed content, and the one line that tells a
//!  person a commit of named paths left their merge open.
//! Why:
//!  A commit of a path that was in conflict used to claim markers for every such path, so a file the person had resolved by
//!  editing (and not staged) was reported as full of markers although its committed blob held none, and the remedy it gave,
//!  another commit of the path, never finished the merge. The claim is now made only for a blob that holds a marker line as
//!  Git writes it, and the open merge is said on its own with the commands that finish it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claimed = paths.filter((path) => hasMarkerLines(blobAt(commit, path), markerSizeOf(path)));
//! ```

/// Running real Git for the committed blobs and the attribute.
use crate::transaction_git::{GitContext, GitRequest, run_git};
/// Owned operating-system text.
use std::ffi::OsString;
/// Raw bytes to and from operating-system text.
use std::os::unix::ffi::OsStringExt;
/// Borrowed filesystem paths.
use std::path::Path;

/// Git's default marker size (`DEFAULT_CONFLICT_MARKER_SIZE`, `ll-merge.h`), used when the attribute gives none.
pub const DEFAULT_MARKER_SIZE: usize = 7;

/// What:
///  The marker size a `git check-attr conflict-marker-size` value stands for.
/// Why:
///   Git reads the attribute as a number and uses its default for anything else, zero included (`ll_merge`, `ll-merge.c`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markerSize = (value: string) => { const n = Number.parseInt(value, 10); return n > 0 ? n : 7; };
/// ```
pub fn marker_size_of(value: &str) -> usize {
    // `parse::<usize>()` fails on text that is not a plain non-negative number; `.ok()` turns the failure into nothing.
    return match value.trim().parse::<usize>().ok() {
        Some(size) if size > 0 => size,
        _ => DEFAULT_MARKER_SIZE,
    };
}

/// What:
///  Whether `content` has a marker line of `size` characters that opens or closes a conflict (`<` or `>`).
/// Why:
///   Git's own test of a marker line (`is_conflict_marker`, `xdiff-interface.c`) is `size` equal characters from `<`, `=`, `>` or
///  `|`, then white space; a line of `=` alone also underlines a heading, so it is not by itself a sign of a conflict, while a
///  conflict always has an opening and a closing line, and a leftover keeps at least one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hasMarkerLines = (content: Uint8Array, size: number) => lines(content).some((line) => isMarker(line, size, ['<', '>']));
/// ```
pub fn has_marker_lines(content: &[u8], size: usize) -> bool {
    // `split_inclusive` keeps each line's line feed, which is the white space that must follow the marker.
    for line in content.split_inclusive(is_line_feed_byte) {
        // A line shorter than the marker plus one character cannot be a marker.
        if line.len() <= size {
            continue;
        }
        let first: u8 = line[0];
        if first != b'<' && first != b'>' {
            continue;
        }
        let mut all_same: bool = true;
        for byte in &line[..size] {
            if *byte != first {
                all_same = false;
                break;
            }
        }
        if all_same && line[size].is_ascii_whitespace() {
            return true;
        }
    }
    return false;
}

/// Whether one byte is the line feed that ends a line. `&u8` borrows the byte, as `split_inclusive` hands it over.
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether one byte is the NUL byte that ends each field of `git check-attr -z`. `&u8` borrows the byte.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The marker size of every path, one `git check-attr` start for all of them.
/// Why:
///   The attribute is read from the working tree as Git reads it when it writes the markers; a failed start gives the default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function markerSizes(context, root, paths): Promise<number[]>;
/// ```
fn marker_sizes(context: &GitContext, root: &Path, paths: &[Vec<u8>]) -> Vec<usize> {
    let mut arguments: Vec<OsString> = vec![OsString::from("check-attr"), OsString::from("-z"), OsString::from("conflict-marker-size"), OsString::from("--")];
    for path in paths {
        arguments.push(OsString::from_vec(path.clone()));
    }
    let mut request: GitRequest = GitRequest::new(root, &[] as &[&str]);
    request.arguments = arguments;
    // Every path answers with three NUL-ended fields: the path, the attribute, the value.
    let answered: Vec<Vec<u8>> = match run_git(context, &request) {
        Ok(output) if output.succeeded() => output.stdout.split(is_nul_byte).map(<[u8]>::to_vec).collect(),
        _ => Vec::new(),
    };
    let mut sizes: Vec<usize> = Vec::new();
    for index in 0..paths.len() {
        let value: String = match answered.get(index * 3 + 2) {
            Some(bytes) => String::from_utf8_lossy(bytes).into_owned(),
            None => String::new(),
        };
        sizes.push(marker_size_of(value.as_str()));
    }
    return sizes;
}

/// What:
///  The paths among `paths` whose blob in `commit` still holds marker lines.
/// Why:
///   A path whose committed content has none is resolved, and no marker is claimed for it; a path the commit deleted has no blob
///  and is resolved too. With no path to look at, no Git starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pathsWithMarkers(context, root, commit, paths): Promise<Uint8Array[]>;
/// ```
pub fn paths_with_markers(context: &GitContext, root: &Path, commit: &str, paths: &[Vec<u8>]) -> Vec<Vec<u8>> {
    // An ordinary commit committed no path that was unmerged: nothing is read and no Git starts, which the start budgets pin.
    if paths.is_empty() {
        return Vec::new();
    }
    let sizes: Vec<usize> = marker_sizes(context, root, paths);
    let mut marked: Vec<Vec<u8>> = Vec::new();
    for (path, size) in paths.iter().zip(sizes) {
        // `<commit>:<path>` names the blob the commit holds at the path, from the top of the repository.
        let mut name: Vec<u8> = format!("{commit}:").into_bytes();
        name.extend_from_slice(path.as_slice());
        let mut request: GitRequest = GitRequest::new(root, &[] as &[&str]);
        request.arguments = vec![OsString::from("cat-file"), OsString::from("blob"), OsString::from_vec(name)];
        if let Ok(output) = run_git(context, &request)
            && output.succeeded()
            && has_marker_lines(output.stdout.as_slice(), size)
        {
            marked.push(path.clone());
        }
    }
    return marked;
}

/// What:
///  The line a commit of named paths prints when it leaves a merge or a cherry-pick open.
/// Why:
///   The commit takes only the named paths, so the operation is still in progress; a person who resolved the last file and
///  committed it by name would otherwise never learn that the operation needs its own conclusion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const openLine = (merge: boolean) => `cli-git: note: the ${merge ? 'merge' : 'cherry-pick'} is still in progress ...`;
/// ```
pub fn open_operation_line(merge: bool) -> String {
    let (operation, continuing): (&str, &str) = if merge { ("merge", "git merge --continue") } else { ("cherry-pick", "git cherry-pick --continue") };
    return format!(
        "cli-git: note: the {operation} is still in progress, because this commit took only the paths you named. To finish it, \
         stage any file that is still unmerged (git add <path>), then run git commit with no paths, or {continuing}.\n"
    );
}

/// Controls for the marker test, the size and the lines.
#[cfg(test)]
#[path = "conflict_markers_tests.rs"]
mod tests;
