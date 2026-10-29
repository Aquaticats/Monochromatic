//! What:
//!  The staged additions an explicit-path commit must carry into its private index, and the pathspec-file reader
//!  that tells which selection they belong to.
//! Why:
//!  A path staged as an addition in the real index is tracked there, so native `git commit -- <path>` takes its
//!  worktree content without asking any ignore rule.
//!  The private index of an explicit-path commit starts from the recorded base, where such a path is untracked,
//!  so `git add` applies the ignore rules: a directory pathspec silently matches nothing, and a file pathspec fails
//!  with a hint to force-add that `commit` does not accept.
//!  Carrying the real index's entry first makes the path tracked in the private index, and `git add` then
//!  refreshes it from the worktree exactly as native Git does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! git diff-index --cached --raw --no-abbrev -z --diff-filter=A <base> -- <pathspecs>
//! ```

use super::capture_order::indexed_request;
use super::path_bytes::os_from_path_bytes;
use super::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
use std::ffi::OsString;
use std::path::Path;

/// How many carried entries one `git update-index` start takes,
///  so a long selection never nears the operating system's argument limit.
const ENTRIES_PER_START: usize = 200;

/// What:
///  One entry staged as an addition in the real index.
/// Why:
///   The private index needs the mode, the object and the path to carry it over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedAddition = { mode: string; oid: string; path: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedAddition {
    /// Octal file mode, such as `100644`.
    pub mode: String,
    /// Full object name of the staged content.
    pub oid: String,
    /// Repository path as bytes, so a name that is not UTF-8 survives.
    pub path: Vec<u8>,
}

/// What:
///  The additions staged in the real index that the pathspecs select.
/// Why:
///   Only these paths make the private index disagree with native Git about what is tracked; the read touches no
///  index, object store or worktree file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git diff-index --cached --raw --no-abbrev -z --diff-filter=A <base> -- <pathspecs>
/// ```
pub fn staged_additions_selected(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    base_revision: &str,
    pathspecs: &[OsString],
) -> Result<Vec<StagedAddition>, GitFailure> {
    if pathspecs.is_empty() {
        return Ok(Vec::new());
    }
    let mut arguments: Vec<OsString> = [
        "diff-index",
        "--cached",
        "--raw",
        "--no-abbrev",
        "--no-renames",
        "-z",
        "--diff-filter=A",
        base_revision,
        "--",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    arguments.extend(pathspecs.iter().cloned());
    let output: GitOutput = run_git_checked(context, &indexed_request(cwd, &arguments, Some(real_index), None))?;
    return parse_raw_additions(output.stdout.as_slice());
}

/// What:
///  Reads the NUL-separated raw records of `git diff-index --raw -z`: a `:old new oldoid newoid status` line, then
///  the path.
/// Why:
///   A malformed record means the read cannot be trusted, and carrying a wrong entry would commit wrong content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [meta, path] of pairs(output.split('\0'))) { ... }
/// ```
fn parse_raw_additions(stdout: &[u8]) -> Result<Vec<StagedAddition>, GitFailure> {
    let mut fields = stdout.split(|byte| { return *byte == 0; }).filter(|field| { return !field.is_empty(); });
    let mut additions: Vec<StagedAddition> = Vec::new();
    while let Some(meta) = fields.next() {
        let unreadable = || { return GitFailure(String::from("git diff-index returned a record that could not be read.")); };
        let path: &[u8] = fields.next().ok_or_else(unreadable)?;
        let text: &str = std::str::from_utf8(meta).map_err(|_| { return unreadable(); })?;
        let columns: Vec<&str> = text.trim_start_matches(':').split(' ').collect();
        if columns.len() != 5 || columns[4] != "A" {
            return Err(unreadable());
        }
        additions.push(StagedAddition {
            mode: String::from(columns[1]),
            oid: String::from(columns[3]),
            path: path.to_vec(),
        });
    }
    return Ok(additions);
}

/// What:
///  Carries the additions into the private index with `git update-index --add --cacheinfo`.
/// Why:
///   The entries come from the real index, so the private index tracks the paths and `git add` then refreshes them
///  from the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git update-index --add --cacheinfo <mode>,<oid>,<path> ...
/// ```
pub fn carry_additions(
    context: &GitContext,
    cwd: &Path,
    private_index: &Path,
    object_directory: &Path,
    additions: &[StagedAddition],
) -> Result<(), GitFailure> {
    for chunk in additions.chunks(ENTRIES_PER_START) {
        let mut arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("--add")];
        for addition in chunk {
            let mut entry: OsString = OsString::from(format!("{},{},", addition.mode, addition.oid));
            entry.push(os_from_path_bytes(addition.path.as_slice()));
            arguments.push(OsString::from("--cacheinfo"));
            arguments.push(entry);
        }
        run_git_checked(
            context,
            &indexed_request(cwd, &arguments, Some(private_index), Some(object_directory)),
        )?;
    }
    return Ok(());
}

/// What:
///  The pathspecs a pathspec file names, read the way Git reads it.
/// Why:
///   Git lists additions only for pathspecs given as arguments, so a file's elements have to become arguments;
///  `None` means the file holds something this reader does not take (an empty element or a quote that does not
///  close), and the caller then leaves the selection to Git alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nul ? text.split('\0') : text.split('\n').map(unquoteIfQuoted)
/// ```
pub fn pathspecs_of_file(bytes: &[u8], nul_separated: bool) -> Option<Vec<OsString>> {
    let separator: u8 = if nul_separated { 0 } else { b'\n' };
    let mut records: Vec<&[u8]> = bytes.split(|byte| { return *byte == separator; }).collect();
    // A terminator after the last element leaves one empty piece.
    if records.last().is_some_and(|last| { return last.is_empty(); }) {
        records.pop();
    }
    let mut pathspecs: Vec<OsString> = Vec::new();
    for record in records {
        let element: &[u8] = if nul_separated { record } else { record.strip_suffix(b"\r").unwrap_or(record) };
        if element.is_empty() {
            return None;
        }
        let decoded: Vec<u8> = if !nul_separated && element[0] == b'"' {
            unquote_c_style(element)?
        } else {
            element.to_vec()
        };
        pathspecs.push(os_from_path_bytes(decoded.as_slice()));
    }
    return Some(pathspecs);
}

/// What:
///  Undoes the quoting Git applies to a path with unusual bytes: a leading and a closing double quote, the
///  one-letter escapes, and three-digit octal escapes.
/// Why:
///   A pathspec file may quote any element, and the arguments must carry the bytes the person meant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unquoteCStyle(element)
/// ```
fn unquote_c_style(quoted: &[u8]) -> Option<Vec<u8>> {
    let inner: &[u8] = quoted.strip_prefix(b"\"")?.strip_suffix(b"\"")?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut position: usize = 0;
    while position < inner.len() {
        let current: u8 = inner[position];
        position += 1;
        if current == b'"' {
            return None;
        }
        if current != b'\\' {
            bytes.push(current);
            continue;
        }
        let escape: u8 = *inner.get(position)?;
        position += 1;
        match escape {
            b'a' => bytes.push(7),
            b'b' => bytes.push(8),
            b'f' => bytes.push(12),
            b'n' => bytes.push(b'\n'),
            b'r' => bytes.push(b'\r'),
            b't' => bytes.push(b'\t'),
            b'v' => bytes.push(11),
            b'\\' | b'"' => bytes.push(escape),
            b'0'..=b'3' => {
                let digits: &[u8] = inner.get(position..position + 2)?;
                if !digits.iter().all(|digit| { return (b'0'..=b'7').contains(digit); }) {
                    return None;
                }
                let value: u32 = u32::from(escape - b'0') * 64
                    + u32::from(digits[0] - b'0') * 8
                    + u32::from(digits[1] - b'0');
                bytes.push(u8::try_from(value).ok()?);
                position += 2;
            }
            _ => return None,
        }
    }
    return Some(bytes);
}

/// What:
///  Adds the remedy to a private `git add` failure that names an ignored path.
/// Why:
///   Git's own hint is to force-add, which `git commit` does not accept; the person needs the commit-side way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failure.message.includes('ignored by one of your .gitignore files') ? withRemedy(failure) : failure
/// ```
pub fn explain_ignored_refusal(failure: GitFailure) -> GitFailure {
    if !failure.0.contains("ignored by one of your .gitignore files") {
        return failure;
    }
    return GitFailure(format!(
        "{} cli-git: git commit with explicit paths selects worktree content, and a path an ignore rule matches is not \
tracked until it is staged, so the force-add hint above applies to git add, not to git commit. To commit the path, \
stage it first with git add --force -- <path> and run the same commit again; staged additions are committed from the \
worktree content, as native Git does.",
        failure.0
    ));
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_staged_additions_tests.rs"]
mod tests;
