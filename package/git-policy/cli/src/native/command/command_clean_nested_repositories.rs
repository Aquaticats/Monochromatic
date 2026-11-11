//! What:
//!  The nested Git repositories a clean's own dry runs say it would delete.
//! Why:
//!  A clean that deletes ignored files is allowed to run by the owner's decision of 2026-11-10,
//!  but a second `-f` also removes a whole nested repository:
//!  its own history,
//!  its uncommitted files and every ignored file inside it.
//!  The outer worktree reports that repository as one ignored directory,
//!  so nothing it or the person can see says the work inside it is about to go.
//!  Git names both sets itself,
//!  so the policy reads Git's two answers instead of walking the worktree a third time:
//!  the typed dry run names the paths it removes,
//!  and the same dry run written with one `-f` names each nested repository it keeps instead
//!  (`builtin/clean.c`,
//! `msg_would_skip_git_dir`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // nestedRepositoriesTheCleanDeletes(['Would remove cache/'], ['Would skip repository cache/nested'], root)
//! //   === ['cache/nested']
//! ```

use crate::fs_beneath::plain_components;
use crate::git_metadata::MetadataOutput;
/// `OsStr`/`OsString` are operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `OsStrExt` gives `OsStr::from_bytes`,
/// because a path is bytes and not text.
use std::os::unix::ffi::OsStrExt;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// The words Git writes before a path its dry run would remove (`msg_would_remove` in `builtin/clean.c`).
pub const WOULD_REMOVE: &[u8] = b"Would remove ";

/// The words Git writes before a nested repository its dry run keeps (`msg_would_skip_git_dir` in
/// `builtin/clean.c`).
pub const WOULD_SKIP_REPOSITORY: &[u8] = b"Would skip repository ";

/// What:
///  The lines of a `git clean --dry-run` answer,
///  each as Git printed it,
///  or why Git could not give one.
/// Why:
///  A dry run prints one item per line and nothing at all when it would remove nothing,
///  and a clean Git refuses says nothing either,
///  so a failure is an error and never "nothing to delete".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dryRunLinesFromAnswer(answer, what: string): Buffer[]; // throws
/// ```
pub fn dry_run_lines_from_answer(answer: &MetadataOutput, what: &str) -> Result<Vec<Vec<u8>>, String> {
    if !answer.success {
        let detail: String = String::from_utf8_lossy(&answer.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not read {what}: {detail}"));
    }
    let mut lines: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for line in answer.stdout.split(is_newline) {
        // A dry run that would delete nothing prints nothing, and a line of whitespace alone names no path.
        if line.iter().all(is_ascii_whitespace) {
            continue;
        }
        lines.push(line.to_vec());
    }
    return Ok(lines);
}

/// What:
///  The names a typed dry run prints for the paths it would remove,
///  each without its prefix and its trailing slash.
/// Why:
///   Git prints `Would remove <path>` and writes a directory with a trailing `/`;
///  a path it cannot print as text is C-quoted,
///  so the quoted form is decoded before anything reads the name as a path.
///  Every other line is left out:
///  Git also prints `Would refuse to remove current working directory`,
///  which names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deletedDirectoriesFromDryRun(lines: Buffer[]): Buffer[];
/// ```
pub fn deleted_directories_from_dry_run(lines: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut removed: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for line in lines {
        let Some(printed) = line.strip_prefix(WOULD_REMOVE) else {
            continue;
        };
        let Some(path) = unquoted(printed) else {
            continue;
        };
        // A directory is printed with a trailing slash; Git never writes one on a file whose name ends there,
        //  because a file name cannot hold a slash.
        let name: &[u8] = path.strip_suffix(b"/").unwrap_or(path.as_slice());
        if name.is_empty() {
            continue;
        }
        removed.push(name.to_vec());
    }
    return removed;
}

/// What:
///  The nested repositories a dry run written with one `-f` says it keeps,
///  each without its prefix and its quoting.
/// Why:
///   Git writes `Would skip repository <path>` for every repository its walk finds below a path it would
///  remove,
///  and that same walk removes each of them at a second `-f`;
///  a path it cannot print as text is C-quoted,
///  so the quoted form is decoded.
///  Anything else is not a repository Git named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptRepositoriesFromDryRun(lines: Buffer[]): Buffer[];
/// ```
pub fn kept_repositories_from_dry_run(lines: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut kept: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for line in lines {
        let Some(printed) = line.strip_prefix(WOULD_SKIP_REPOSITORY) else {
            continue;
        };
        let Some(path) = unquoted(printed) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        kept.push(path);
    }
    return kept;
}

/// What:
///  Every nested repository the typed clean would delete,
///  as its path below the worktree root,
///  with no repository that another answer already keeps whole.
///  `&Path` borrows the worktree top level Git printed those paths from.
/// Why:
///   Two answers together cover both shapes Git removes:
///  a repository that is itself the removed path is named by the typed dry run and found through its own
///  `.git`,
///  and a repository below a removed directory is named by the one-force dry run.
///  A path that is not a plain relative path names nothing this policy may move,
///  so it is a failure and never a guess,
///  except on the typed answer,
///  whose every entry is only a candidate:
///  one that does not name a directory holding a `.git` entry is not a repository at all.
///  Only the outermost of two repositories is returned,
///  because moving a repository takes everything inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedRepositoriesTheCleanDeletes(deleted: Buffer[], kept: Buffer[], worktreeRoot: string): Buffer[]; // throws
/// ```
pub fn nested_repositories_the_clean_deletes(
    deleted_lines: &[Vec<u8>],
    kept_lines: &[Vec<u8>],
    worktree_root: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    let mut found: Vec<(Vec<u8>, Vec<OsString>)> = Vec::<(Vec<u8>, Vec<OsString>)>::new();
    for path in kept_repositories_from_dry_run(kept_lines) {
        let names: Vec<OsString> = names_of(path.as_slice())?;
        if names.is_empty() {
            return Err(String::from("Git named a nested repository with no path, so the clean was not judged."));
        }
        found.push((path, names));
    }
    for path in deleted_directories_from_dry_run(deleted_lines) {
        let Ok(names) = names_of(path.as_slice()) else {
            continue;
        };
        let Some(directory) = directory_beneath(worktree_root, names.as_slice()) else {
            continue;
        };
        if std::fs::symlink_metadata(directory.join(".git")).is_err() {
            continue;
        }
        found.push((path, names));
    }
    // The shortest path first, so an ancestor is always kept before anything it holds.
    found.sort_by_key(fewest_components_first);
    let mut outermost: Vec<(Vec<u8>, Vec<OsString>)> = Vec::<(Vec<u8>, Vec<OsString>)>::new();
    for (path, names) in found {
        if !is_at_or_below_another(names.as_slice(), outermost.as_slice()) {
            outermost.push((path, names));
        }
    }
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for (path, _names) in outermost {
        paths.push(path);
    }
    return Ok(paths);
}

/// What:
///  How many components a path has,
///  which orders a list by how deep each path is.
///  `&(Vec<u8>, Vec<OsString>)` borrows one candidate and its components.
/// Why:
///   The deepest path is the one inside another,
///  so the shortest first is what makes one pass of the list enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (entry: [Buffer, string[]]) => entry[1].length;
/// ```
fn fewest_components_first(entry: &(Vec<u8>, Vec<OsString>)) -> usize {
    return entry.1.len();
}

/// What:
///  Whether one list of components names a path at or below a path already kept.
/// Why:
///   A path already kept holds everything below it,
///  so a second move of a path inside it would fail on a directory that is no longer there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (names: string[], kept: [Buffer, string[]][]) => kept.some(([, known]) => names.startsWith(known));
/// ```
fn is_at_or_below_another(names: &[OsString], kept: &[(Vec<u8>, Vec<OsString>)]) -> bool {
    for (_path, known) in kept {
        if names.starts_with(known.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  A printed path's components,
///  or the sentence saying why it names no path below the worktree.
/// Why:
///   The policy moves or asks about a path below the worktree root,
///  so an absolute path or one holding `..` would name something else;
///  Git never prints one,
///  and a guess would move a directory nobody asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesOf(path: Buffer): string[]; // throws
/// ```
fn names_of(path: &[u8]) -> Result<Vec<OsString>, String> {
    return match plain_components(Path::new(OsStr::from_bytes(path))) {
        Ok(names) => Ok(names),
        Err(error) => Err(format!(
            "Git named `{}` among what the clean would keep, and that is not a plain path below the worktree, so the clean \
             was not judged ({error}).",
            String::from_utf8_lossy(path)
        )),
    };
}

/// What:
///  The directory `names` reaches below `root`,
///  or nothing when one component is not a directory or is reached through a link.
/// Why:
///   A symbolic link is removed as the link itself and never descended into,
///  so a repository behind one is not deleted with it;
///  checking each component as it is walked answers that without following any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directoryBeneath(root: string, names: string[]): string | undefined;
/// ```
fn directory_beneath(root: &Path, names: &[OsString]) -> Option<PathBuf> {
    let mut current: PathBuf = root.to_path_buf();
    for name in names {
        current.push(name);
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(current.as_path()).ok()?;
        if !metadata.is_dir() {
            return None;
        }
    }
    return Some(current);
}

/// What:
///  A name Git printed,
///  decoded when Git had to quote it.
/// Why:
///   Git wraps a name it cannot print as text in `"` and escapes it (`quote.c`,
/// `quote_c_style`),
///  with `\a`,
///  `\b`,
///  `\f`,
///  `\n`,
///  `\r`,
///  `\t`,
///  `\v`,
///  `\\`,
///  `\"` and `\ooo` for every other byte;
///  reading the quoted form as the name itself would find no repository there,
///  and a repository whose name needs quoting would be deleted unsaved.
///  A quoted form this cannot read is nothing,
///  never a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unquoted(printed: Buffer): Buffer | undefined;
/// ```
fn unquoted(printed: &[u8]) -> Option<Vec<u8>> {
    if !printed.starts_with(b"\"") {
        return Some(printed.to_vec());
    }
    let inner: &[u8] = printed.get(1..)?;
    let (last, body) = inner.split_last()?;
    if *last != b'"' {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::<u8>::with_capacity(body.len());
    let mut index: usize = 0;
    while index < body.len() {
        let byte: u8 = body[index];
        index += 1;
        if byte != b'\\' {
            bytes.push(byte);
            continue;
        }
        let escape: u8 = *body.get(index)?;
        index += 1;
        let decoded: u8 = match escape {
            b'a' => 0x07,
            b'b' => 0x08,
            b'f' => 0x0c,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'v' => 0x0b,
            b'\\' => b'\\',
            b'"' => b'"',
            b'0'..=b'3' => {
                let second: u8 = *body.get(index)?;
                let third: u8 = *body.get(index + 1)?;
                index += 2;
                if !second.is_ascii_digit() || !third.is_ascii_digit() || second > b'7' || third > b'7' {
                    return None;
                }
                (escape - b'0') * 64 + (second - b'0') * 8 + (third - b'0')
            }
            _ => return None,
        };
        bytes.push(decoded);
    }
    return Some(bytes);
}

/// What:
///  Whether one byte of a dry run's answer ends a line.
/// Why:
///   The answer names one item per line,
///  and a named function keeps that pass free of a closure that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a
/// ```
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one byte of the answer is whitespace.
/// Why:
///   A dry run that would do nothing prints nothing,
///  and a line of whitespace alone names no path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => /\s/.test(String.fromCharCode(byte))
/// ```
fn is_ascii_whitespace(byte: &u8) -> bool {
    return byte.is_ascii_whitespace();
}

/// The two readings,
/// their quoting and the shapes Git removes.
#[cfg(test)]
#[path = "command_clean_nested_repositories_tests.rs"]
mod tests;
