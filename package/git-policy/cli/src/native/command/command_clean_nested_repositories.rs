//! What:
//!  The nested Git repositories a clean's own dry run says it would delete.
//! Why:
//!  A clean that deletes ignored files is allowed to run by the owner's decision of 2026-11-10,
//!  but a second `-f` also removes a whole nested repository:
//!  its own history,
//!  its uncommitted files and every ignored file inside it.
//!  The outer worktree reports that repository as one ignored directory,
//!  so nothing it or the person can see says the work inside it is about to go.
//!  Git's typed dry run names every path it would remove (`msg_would_remove` in `builtin/clean.c`),
//!  and the repositories are the directories below those paths that are a repository of their own:
//!  one with a `.git` entry, or a bare one whose `HEAD`, `objects` and `refs` sit in the directory itself,
//!  because a second `-f` deletes both shapes and names neither.
//!  The path Git names is sometimes the repository itself and sometimes a directory that holds it.
//!  Git writes a trailing slash on a directory it names at the top level and none on one it names from inside
//!  a directory that survives (`remove_dirs` prints both shapes),
//!  so the name alone says nothing and each one is read as a directory:
//!  a name that is a file holds no repository below it.
//!  Every name is relative to the directory the clean runs in,
//!  so that directory's own prefix is written before it.
//!
//!  Git's one-force answer is no list of the same repositories.
//!  At one force the directory walk skips a nested repository itself (`DIR_SKIP_NESTED_GIT`),
//!  without a word,
//!  and `msg_would_skip_git_dir` reaches the answer only for a repository inside a directory that answer
//!  already lists,
//!  so a repository below an untracked directory was deleted unsaved while that reading named nothing.
//!
//!  The look inside those paths reads a file system,
//!  and a mount whose server stopped answering never returns that read,
//!  so it runs on an awaited thread with the same ceiling a read of Git has:
//!  a look that gives no answer in that time stops the clean instead of holding the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // nestedRepositoriesTheCleanDeletes(['Would remove cache/'], root, Buffer.from('sub/'))
//! //   === ['sub/cache/nested']  when sub/cache/nested/.git exists
//! ```

/// The answer Git gave one dry run,
///  and what a failed one means.
use crate::git_metadata::{MetadataOutput, READ_CEILING};
/// `work_within` looks at a file system that may never answer, on a thread awaited under a ceiling.
use crate::work_awaited_within_a_ceiling::work_within;
/// `plain_components` reads a Git-printed path as the plain components below the worktree it must be.
use crate::fs_beneath::plain_components;
/// `OsStr`/`OsString` are operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `OsStrExt` gives `OsStr::from_bytes`,
/// because a path is bytes and not text.
use std::os::unix::ffi::OsStrExt;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// The words Git writes before a path its dry run would remove (`msg_would_remove` in `builtin/clean.c`).
pub const WOULD_REMOVE: &[u8] = b"Would remove ";

/// What:
///  The lines of a `git clean --dry-run` answer,
///  each as Git printed it,
///  or why Git gave none.
/// Why:
///  A dry run prints one item per line and nothing at all when it would remove nothing,
///  and a clean Git refuses says nothing either,
///  so a failure is an error and never "nothing to delete".
///  Two failures are told apart:
///  Git ran and refused the clean,
///  which deletes nothing and is Git's own answer to the person,
///  and a read that ended without an exit status,
///  which says nothing about what Git would delete and therefore stops the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dryRunLinesFromAnswer(answer, what: string): Buffer[]; // throws
/// ```
pub fn dry_run_lines_from_answer(answer: &MetadataOutput, what: &str) -> Result<Vec<Vec<u8>>, String> {
    if !answer.success {
        // A child ended by a signal reports no exit status: it never said what it does, so its silence is not an
        //  answer about the clean and nothing may be judged from it.
        if answer.code.is_none() {
            return Err(format!(
                "Git never answered {what}: the read ended without an exit status, so the clean was not judged"
            ));
        }
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
///  The paths a dry run prints for the entries it would remove,
///  each without its prefix and its trailing slash.
/// Why:
///   Git prints `Would remove <path>`,
///  and a path it cannot print as text is C-quoted,
///  so the quoted form is decoded before anything reads the name as a path.
///  Git writes the trailing slash of a directory it names at the top level and leaves it off one it names from
///  inside a directory that survives,
///  so both spellings are read and the caller decides which one is a directory:
///  a name that is a file holds no repository below it.
///  Every other line names nothing:
///  Git also prints `Would refuse to remove current working directory`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removedPathsFromDryRun(lines: Buffer[]): Buffer[];
/// ```
pub fn removed_paths_from_dry_run(lines: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut removed: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for line in lines {
        let Some(printed) = line.strip_prefix(WOULD_REMOVE) else {
            continue;
        };
        let Some(path) = unquoted(printed) else {
            continue;
        };
        let name: &[u8] = path.strip_suffix(b"/").unwrap_or(path.as_slice());
        if name.is_empty() {
            continue;
        }
        removed.push(name.to_vec());
    }
    return removed;
}

/// What:
///  Every nested repository a typed clean would delete,
///  as its path below the worktree root,
///  with no repository that another path already holds.
///  `&Path` borrows the worktree top level Git printed those paths from,
///  and `&[u8]` the prefix Git printed for the directory the command runs in
///  (`git rev-parse --show-prefix`).
/// Why:
///   A second `-f` removes each of these whole,
///  with its history and its uncommitted files,
///  and only a look inside the paths Git names finds them,
///  because Git names the directory it removes and not the repository inside it.
///  A name Git prints is relative to the directory the clean runs in,
///  not to the worktree top level (`relative_path` in `builtin/clean.c`),
///  so the command's own prefix is written before it:
///  read from the top level instead, a clean started in a subdirectory named a directory that is
///  not the one Git looked at, and the repository it held was deleted unsaved.
///  Git writes a `.` in front of a path it prints from inside a directory it will not remove itself
///  (`remove_dirs` prints the cwd's own entries from a `./` path),
///  so a `.` component is dropped and a parent component steps up inside the worktree.
///  A path that is not a plain relative path names nothing this policy may look inside,
///  and a path reached through a symbolic link is removed as the link alone,
///  so neither is one a repository is found below.
///  A directory the wrapper cannot read is a failure and never a guess:
///  what it holds was not seen.
///  Only the outermost of two repositories is returned,
///  because moving a repository takes everything inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedRepositoriesTheCleanDeletes(deleted: Buffer[], worktreeRoot: string, commandPrefix: Buffer): Buffer[]; // throws
/// ```
pub fn nested_repositories_the_clean_deletes(
    deleted_lines: &[Vec<u8>],
    worktree_root: &Path,
    command_prefix: &[u8],
) -> Result<Vec<Vec<u8>>, String> {
    let command: Vec<OsString> = match plain_components(Path::new(OsStr::from_bytes(command_prefix))) {
        Ok(names) => names,
        Err(error) => {
            return Err(format!(
                "Git printed `{}` as the directory the clean runs in, and that is not a plain path below the worktree, so \
                 the clean was not judged ({error}).",
                String::from_utf8_lossy(command_prefix)
            ));
        }
    };
    let mut found: Vec<(Vec<u8>, Vec<OsString>)> = Vec::<(Vec<u8>, Vec<OsString>)>::new();
    for path in removed_paths_from_dry_run(deleted_lines) {
        let Some(names) = names_below_the_command(path.as_slice(), command.as_slice()) else {
            continue;
        };
        // The look inside the path reads its every component, so the check that it is a directory of its own happens
        //  under the same ceiling as the walk below it.
        for nested in repositories_beneath_within_a_ceiling(worktree_root, names.as_slice(), READ_CEILING, walk_beneath)? {
            found.push((joined_names(nested.as_slice()), nested));
        }
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
///  The directory one name Git printed reaches,
///  as components below the worktree root,
///  or nothing when it names a path outside the worktree or an absolute one.
///  `&[OsString]` borrows the components of the directory the clean runs in.
/// Why:
///   Git prints a path relative to the directory the command runs in,
///  with the `.` of a path named from inside a directory it keeps,
///  and a pathspec may name a directory above that one,
///  so a parent component steps up the components already read.
///  A path that steps above the worktree names nothing this policy may look inside,
///  because Git removes nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesBelowTheCommand(printed: Buffer, command: string[]): string[] | undefined;
/// ```
fn names_below_the_command(printed: &[u8], command: &[OsString]) -> Option<Vec<OsString>> {
    let mut names: Vec<OsString> = command.to_vec();
    for part in Path::new(OsStr::from_bytes(printed)).components() {
        match part {
            std::path::Component::Normal(name) => names.push(name.to_os_string()),
            std::path::Component::CurDir => continue,
            // A path that steps above the worktree names nothing Git removes.
            std::path::Component::ParentDir => {
                names.pop()?;
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => return None,
        }
    }
    // The command's own directory is no path below it.
    if names.is_empty() {
        return None;
    }
    return Some(names);
}

/// What:
///  Every directory at or below `start` that is a Git repository of its own,
///  read on a thread the caller waits for only until `ceiling`.
///  `walk` is the look itself, so a control can hold it past the ceiling without a mount whose server stopped.
/// Why:
///  A directory read or a metadata lookup on a mount whose server stopped answering can wait in the kernel with no end,
///  and the thread it holds cannot be ended while that call runs;
///  only work on another thread lets the command stop waiting,
///  so a look that gives no answer in the ceiling stops the clean with a sentence naming the directory
///  instead of holding the command for as long as the mount takes.
///  The thread is abandoned, never cancelled:
///  it ends when the kernel lets the call go, or at the process exit where the kernel lets the task be ended then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoriesBeneathWithinACeiling(root, start, ceiling, walk): string[][]; // throws
/// ```
pub(crate) fn repositories_beneath_within_a_ceiling(
    worktree_root: &Path,
    start: &[OsString],
    ceiling: std::time::Duration,
    walk: fn(&Path, &[OsString]) -> Result<Vec<Vec<OsString>>, String>,
) -> Result<Vec<Vec<OsString>>, String> {
    let root: PathBuf = worktree_root.to_path_buf();
    let names: Vec<OsString> = start.to_vec();
    let Some(answer) = work_within(ceiling, move || { return walk(root.as_path(), names.as_slice()); }) else {
        return Err(did_not_answer(start, ceiling));
    };
    return answer;
}

/// What:
///  Every directory at or below `start` that is a Git repository of its own,
///  as its components below the worktree root.
/// Why:
///  A nested repository is a directory with a `.git` entry of its own,
///  whether that entry is a directory or a file,
///  or a bare repository, which has no `.git` entry at all,
///  and moving the outermost one takes every repository inside it,
///  so the walk opens no directory below one.
///  Every component of `start` is checked as a directory of its own first,
///  so a file there holds nothing and a link is removed as the link.
///  It opens no symbolic link either:
///  a link is removed as the link,
/// so nothing behind it is deleted.
///  A directory that went away between Git's answer and this look holds nothing,
///  and a directory the wrapper cannot open is a failure naming that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkBeneath(root: string, start: string[]): string[][]; // throws
/// ```
fn walk_beneath(worktree_root: &Path, start: &[OsString]) -> Result<Vec<Vec<OsString>>, String> {
    let mut found: Vec<Vec<OsString>> = Vec::<Vec<OsString>>::new();
    // Every component of the removed path must be a directory of its own: a file holds nothing below it, and a
    //  link is removed as a link.
    let Some(start_path) = directory_beneath(worktree_root, start) else {
        return Ok(found);
    };
    let mut pending: Vec<(PathBuf, Vec<OsString>)> = vec![(start_path, start.to_vec())];
    while let Some((directory, names)) = pending.pop() {
        if a_repository_is(directory.as_path()) {
            found.push(names);
            continue;
        }
        let listing: std::fs::ReadDir = match std::fs::read_dir(directory.as_path()) {
            Ok(listed) => listed,
            // The path went away after Git named it, and a path that is not there holds nothing.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(cannot_look_inside(names.as_slice(), &error)),
        };
        for listed in listing {
            // A directory entry the system could not read is one that is gone, and a gone entry holds nothing.
            let Ok(entry) = listed else {
                continue;
            };
            // The entry's own type, never the target of a link: a link to a directory is not walked.
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if !kind.is_dir() {
                continue;
            }
            let mut child: Vec<OsString> = names.clone();
            child.push(entry.file_name());
            pending.push((directory.join(entry.file_name()), child));
        }
    }
    return Ok(found);
}

/// What:
///  Whether one directory is a Git repository of its own,
///  with a `.git` entry or as a bare repository.
/// Why:
///   A second `-f` deletes both shapes whole,
///  without naming the repository it removes,
///  and a bare repository has no `.git` entry to find.
///  `.git` is read as the entry itself, so a file (a worktree's or a submodule's) counts as much as a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRepositoryIs(directory: string): boolean;
/// ```
fn a_repository_is(directory: &Path) -> bool {
    if std::fs::symlink_metadata(directory.join(".git")).is_ok() {
        return true;
    }
    return a_bare_repository_is(directory);
}

/// What:
///  Whether one directory is a bare repository:
///  a `HEAD` file, an `objects` directory and a `refs` directory,
///  which is what Git's own `is_git_directory` reads before it calls a directory a repository.
/// Why:
///  A bare repository keeps its whole history in the directory itself,
///  so nothing else marks it as a repository,
///  and Git's clean deletes it as an ordinary ignored directory.
///  A `HEAD` whose content no longer names a head still counts:
///  the directory is a person's repository that Git would delete, and saving it is never deleting it.
///  The markers are read through links, as Git opens them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aBareRepositoryIs(directory: string): boolean; // HEAD file && objects/ && refs/
/// ```
fn a_bare_repository_is(directory: &Path) -> bool {
    let Ok(head) = std::fs::metadata(directory.join("HEAD")) else {
        return false;
    };
    if !head.is_file() {
        return false;
    }
    if !a_directory_is(directory.join("objects").as_path()) {
        return false;
    }
    return a_directory_is(directory.join("refs").as_path());
}

/// What:
///  Whether one path is a directory,
///  as the marker of a bare repository must be.
/// Why:
///  A marker this cannot read is one that is not there,
///  and a path that is not a directory is no repository's `objects` or `refs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (path: string) => stat(path, { throwIfNoEntry: false })?.isDirectory() ?? false;
/// ```
fn a_directory_is(path: &Path) -> bool {
    return std::fs::metadata(path).map(|found| { return found.is_dir(); }).unwrap_or(false);
}

/// What:
///  The sentence for a removed directory the wrapper's look inside did not answer for,
///  naming it and the ceiling.
/// Why:
///  What the directory holds was not seen within the time a read has,
///  so a repository inside it may be about to be deleted unsaved,
///  and the person is told which path stopped the clean and for how long it was read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const didNotAnswer = (names: string[], ceiling: Duration) => `... ${names.join('/')} ... not judged.`;
/// ```
fn did_not_answer(names: &[OsString], ceiling: std::time::Duration) -> String {
    return format!(
        "Git named `{}` among what the clean would delete, and the wrapper's look inside it did not answer within {} seconds, \
         so the clean was not judged.",
        String::from_utf8_lossy(joined_names(names).as_slice()),
        ceiling.as_secs()
    );
}

/// What:
///  The sentence for a removed directory the wrapper could not open,
///  naming it and what the system said.
/// Why:
///  What the directory holds was not seen,
///  so a repository inside it may be about to be deleted unsaved,
///  and the person is told which path stopped the clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotLookInside = (names: string[], error: Error) => `... ${names.join('/')} ... (${error}) ...`;
/// ```
fn cannot_look_inside(names: &[OsString], error: &std::io::Error) -> String {
    return format!(
        "Git named `{}` among what the clean would delete, and the wrapper could not look inside it for the Git \
         repositories it holds ({error}), so the clean was not judged.",
        String::from_utf8_lossy(joined_names(names).as_slice())
    );
}

/// What:
///  One printed or walked path's components as the bytes Git prints for that path.
/// Why:
///  A path below a worktree is its components separated by `/`,
///  which is exactly what every answer and every notice names,
///  and the components themselves may hold bytes that are not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (names: string[]) => Buffer.concat(names.map((n, i) => Buffer.concat([i ? Buffer.from('/') : Buffer.alloc(0), n])));
/// ```
fn joined_names(names: &[OsString]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            bytes.push(b'/');
        }
        bytes.extend_from_slice(name.as_bytes());
    }
    return bytes;
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
