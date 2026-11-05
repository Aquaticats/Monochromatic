//! What:
//!  Cloning one directory tree for a worktree repair: every file cloned (or copied when the filesystem cannot clone), every
//!  directory recreated with its mode, every link recreated with its exact target, modes and times kept.
//! Why:
//!  Cargo keeps its build state outside the worktree when the person's configuration says so, so a copy of the worktree's
//!  ignored files does not carry it. The state is reused by the new worktree's registry and git dependencies, and Cargo
//!  decides what is fresh from file times, so the clone keeps them. The destination must not exist: nothing is overwritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cloned = await cloneTree({ source, destination, workers });
//! ```

use crate::scoped_start::start_or_run;
use crate::worktree_copy_stage::copy_file_exclusive;
use std::fs::{FileTimes, Metadata};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

/// What a walk of the source finds at one path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    /// A directory, recreated and entered.
    Directory,
    /// A regular file, cloned.
    File,
    /// A symbolic link, recreated with its exact target.
    Link,
}

/// One entry of the source, relative to its root.
struct Entry {
    /// The path below the root.
    relative: PathBuf,
    /// What it is.
    kind: Kind,
}

/// The entries below which a clone stays on one thread, `PARALLEL_BREAK_EVEN_ENTRIES`; starting threads costs more than a very small
///  clone. Measured with `clone_tree` itself forced to 1, 2, 4 and 8 workers (2026-11-03, five runs each in alternating orders,
///  other sessions running, a debug build): 0.6 ms on every count for 10 entries, 1.2 ms on 8 workers against 1.8 ms on one for 30,
///  8.1 ms against 20.7 ms for 296, and 13.4 ms against 39.2 ms for 555, so the old threshold of 512 left clones of up to 511 leaves
///  on one thread for up to 12 ms more. Picked by the coordinator under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups", option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_MIN_ENTRIES = PARALLEL_BREAK_EVEN_ENTRIES;
/// ```
const PARALLEL_MIN_ENTRIES: usize = crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

/// What a clone made and what it left out.
#[derive(Debug, Eq, PartialEq)]
pub struct Cloned {
    /// The entries of the source that the destination now holds.
    pub made: usize,
    /// The paths below the source that could not be read and so are not in the destination.
    pub left_out: Vec<PathBuf>,
}

/// What:
///  Every entry below the source, parents before children, without following links.
/// Why:
///   Directories must exist before a worker places a file in them, and a link to a directory must be copied as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(source: string): Entry[];
/// ```
fn walk(source: &Path) -> Result<(Vec<Entry>, Vec<PathBuf>), String> {
    let mut found: Vec<Entry> = Vec::new();
    let mut left_out: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let directory: PathBuf = source.join(&relative);
        let listing = match std::fs::read_dir(&directory) {
            Ok(listed_directory) => listed_directory,
            // A directory a test or a tool left without read permission has nothing the clone can take; it is made empty and named.
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied && !relative.as_os_str().is_empty() => {
                left_out.push(relative);
                continue;
            }
            Err(error) => return Err(format!("{} cannot be read ({error})", directory.display())),
        };
        for listed in listing {
            let item = say_where(listed, directory.as_path(), "cannot be listed")?;
            let path: PathBuf = relative.join(item.file_name());
            let file_type = say_where(item.file_type(), item.path().as_path(), "has no type")?;
            if file_type.is_dir() {
                pending.push(path.clone());
                found.push(Entry { relative: path, kind: Kind::Directory });
            } else if file_type.is_symlink() {
                found.push(Entry { relative: path, kind: Kind::Link });
            } else if file_type.is_file() {
                found.push(Entry { relative: path, kind: Kind::File });
            }
        }
    }
    // Parents sort before their children, so directories can be made in list order.
    found.sort_by(compare_by_relative_path);
    left_out.sort();
    return Ok((found, left_out));
}

/// What:
///  Make one file or link in the destination like its source.
/// Why:
///   The clone shares the data; the mode and the times are set after it, because Cargo decides freshness from file times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function place(source: string, destination: string, entry: Entry): void;
/// ```
fn place(source: &Path, destination: &Path, entry: &Entry) -> Result<Option<PathBuf>, String> {
    let from: PathBuf = source.join(&entry.relative);
    let to: PathBuf = destination.join(&entry.relative);
    if entry.kind == Kind::Link {
        let target: PathBuf = say_where(std::fs::read_link(&from), from.as_path(), "cannot be read as a link")?;
        say_where(symlink(&target, &to), to.as_path(), "cannot be made")?;
        return Ok(None);
    }
    let before: Metadata = say_where(std::fs::metadata(&from), from.as_path(), "cannot be read")?;
    if let Err(error) = copy_file_exclusive(&from, &to) {
        // A file without read permission cannot be cloned or copied by anyone but its owner's chmod, which the source is never given:
        //  the file stays out of the destination (a missing file only makes Cargo rebuild what needs it) and is named.
        if matches!(std::fs::File::open(&from), Err(ref denied) if denied.kind() == std::io::ErrorKind::PermissionDenied) {
            // The partial file is this clone's own creation in the new tree.
            let _removed = std::fs::remove_file(&to);
            return Ok(Some(entry.relative.clone()));
        }
        return Err(format!("{} cannot be cloned ({})", from.display(), error.0));
    }
    say_where(std::fs::set_permissions(&to, before.permissions()), to.as_path(), "cannot take its mode")?;
    let times: FileTimes = FileTimes::new().set_accessed(before.accessed().unwrap_or(std::time::UNIX_EPOCH)).set_modified(before.modified().unwrap_or(std::time::UNIX_EPOCH));
    let made: std::fs::File = say_where(std::fs::File::open(&to), to.as_path(), "cannot be opened")?;
    say_where(made.set_times(times), to.as_path(), "cannot take its times")?;
    return Ok(None);
}

/// What:
///  Say which path an input or output step failed on: `<path> <what> (<error>)`.
///  `Result<T, std::io::Error>` is "the value, or an input/output error".
/// Why:
///   Every step of the clone names the entry it could not handle, and a named function stands where a closure that
///  built that text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sayWhere<T>(attempt: () => T, path: string, what: string): T; // throws `${path} ${what} (${error})`
/// ```
fn say_where<T>(result: Result<T, std::io::Error>, path: &Path, what: &str) -> Result<T, String> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{} {what} ({error})", path.display())),
    };
}

/// Order two entries by their path below the source, so parents sort before their children. `sort_by` takes this
///  function by path; the sort stays stable.
fn compare_by_relative_path(left: &Entry, right: &Entry) -> std::cmp::Ordering {
    return left.relative.cmp(&right.relative);
}

/// What:
///  Place each slice of leaf entries on a thread of its own inside one thread scope, and collect what each slice
///  left out.
/// Why:
///   The slices borrow the entries, the source and the destination of the caller for the length of the scope, which a
///  named function cannot express and which owned copies for threads that outlive the caller would only waste. A
///  thread that cannot start runs its slice in turn (`start_or_run`), and a worker that panicked is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcomes = await Promise.all(slices.map(async (slice) => placeAll(source, destination, slice)));
/// ```
#[cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(
        avoidable_closure,
        reason = "std::thread::scope and start_or_run take closures that borrow the caller's entries for the scope only, not 'static; see doc/planning/cli-git-closure-lint-suppressions.md"
    )
)]
fn place_slices_side_by_side(
    leaves: &[&Entry],
    slice_length: usize,
    source: &Path,
    destination: &Path,
) -> Vec<Result<Vec<PathBuf>, String>> {
    return std::thread::scope(|scope| {
        let mut handles: Vec<_> = Vec::new();
        for slice in leaves.chunks(slice_length) {
            handles.push(start_or_run(scope, move || {
                let mut skipped: Vec<PathBuf> = Vec::new();
                for entry in slice {
                    skipped.extend(place(source, destination, entry)?);
                }
                return Ok(skipped);
            }));
        }
        let mut outcomes: Vec<Result<Vec<PathBuf>, String>> = Vec::new();
        for handle in handles {
            outcomes.push(match handle.join() {
                Ok(outcome) => outcome,
                Err(_panic) => Err(String::from("a clone worker stopped unexpectedly")),
            });
        }
        return outcomes;
    });
}

/// What:
///  Clone `source` into the new directory `destination`.
/// Why:
///   The count of entries made is returned for the log; a failure names the entry that could not be made, and leaves what was
///  made in place for the person to remove with the destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cloneTree(args: { source: string; destination: string; workers: number }): Promise<number>;
/// ```
pub fn clone_tree(source: &Path, destination: &Path, workers: usize) -> Result<Cloned, String> {
    let (entries, mut left_out): (Vec<Entry>, Vec<PathBuf>) = walk(source)?;
    let root: Metadata = say_where(std::fs::metadata(source), source, "cannot be read")?;
    if let Some(parent) = destination.parent() {
        say_where(std::fs::create_dir_all(parent), parent, "cannot be made")?;
    }
    // Exclusive creation: an existing destination is refused and left as it was.
    say_where(std::fs::create_dir(destination), destination, "cannot be made")?;
    let mut directories: Vec<&Entry> = Vec::new();
    let mut leaves: Vec<&Entry> = Vec::new();
    for entry in &entries {
        if entry.kind == Kind::Directory {
            directories.push(entry);
        } else {
            leaves.push(entry);
        }
    }
    for directory in &directories {
        let made: PathBuf = destination.join(&directory.relative);
        say_where(std::fs::create_dir(&made), made.as_path(), "cannot be made")?;
    }
    let used: usize = if leaves.len() < PARALLEL_MIN_ENTRIES { 1 } else { workers.max(1) };
    let slice_length: usize = leaves.len().div_ceil(used).max(1);
    let outcomes: Vec<Result<Vec<PathBuf>, String>> = place_slices_side_by_side(&leaves, slice_length, source, destination);
    for outcome in outcomes {
        left_out.extend(outcome?);
    }
    left_out.sort();
    // Modes last, so a read-only directory does not stop its own contents from being placed.
    for directory in directories.iter().rev() {
        let made: PathBuf = destination.join(&directory.relative);
        let source_directory: Metadata = say_where(std::fs::metadata(source.join(&directory.relative)), directory.relative.as_path(), "cannot be read")?;
        say_where(std::fs::set_permissions(&made, source_directory.permissions()), made.as_path(), "cannot take its mode")?;
    }
    say_where(std::fs::set_permissions(destination, root.permissions()), destination, "cannot take its mode")?;
    let mut skipped_files: usize = 0;
    for entry in &entries {
        if entry.kind != Kind::Directory && left_out.contains(&entry.relative) {
            skipped_files += 1;
        }
    }
    return Ok(Cloned { made: entries.len() - skipped_files, left_out });
}

/// Tree clone controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tree_clone_tests.rs"]
mod tests;
