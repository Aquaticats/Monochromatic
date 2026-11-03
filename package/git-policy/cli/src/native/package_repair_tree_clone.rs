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

use super::worktree_copy_stage::copy_file_exclusive;
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

/// The entries per worker below which a clone stays on one thread; starting threads costs more than a small clone.
const PARALLEL_MIN_ENTRIES: usize = 512;

/// What:
///  Every entry below the source, parents before children, without following links.
/// Why:
///   Directories must exist before a worker places a file in them, and a link to a directory must be copied as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(source: string): Entry[];
/// ```
fn walk(source: &Path) -> Result<Vec<Entry>, String> {
    let mut found: Vec<Entry> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let directory: PathBuf = source.join(&relative);
        let listing = std::fs::read_dir(&directory).map_err(|error| { return format!("{} cannot be read ({error})", directory.display()); })?;
        for item in listing {
            let item = item.map_err(|error| { return format!("{} cannot be listed ({error})", directory.display()); })?;
            let path: PathBuf = relative.join(item.file_name());
            let file_type = item.file_type().map_err(|error| { return format!("{} has no type ({error})", item.path().display()); })?;
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
    found.sort_by(|left: &Entry, right: &Entry| { return left.relative.cmp(&right.relative); });
    return Ok(found);
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
fn place(source: &Path, destination: &Path, entry: &Entry) -> Result<(), String> {
    let from: PathBuf = source.join(&entry.relative);
    let to: PathBuf = destination.join(&entry.relative);
    if entry.kind == Kind::Link {
        let target: PathBuf = std::fs::read_link(&from).map_err(|error| { return format!("{} cannot be read as a link ({error})", from.display()); })?;
        return symlink(&target, &to).map_err(|error| { return format!("{} cannot be made ({error})", to.display()); });
    }
    let before: Metadata = std::fs::metadata(&from).map_err(|error| { return format!("{} cannot be read ({error})", from.display()); })?;
    copy_file_exclusive(&from, &to).map_err(|error| { return format!("{} cannot be cloned ({})", from.display(), error.0); })?;
    std::fs::set_permissions(&to, before.permissions()).map_err(|error| { return format!("{} cannot take its mode ({error})", to.display()); })?;
    let times: FileTimes = FileTimes::new().set_accessed(before.accessed().unwrap_or(std::time::UNIX_EPOCH)).set_modified(before.modified().unwrap_or(std::time::UNIX_EPOCH));
    let made: std::fs::File = std::fs::File::open(&to).map_err(|error| { return format!("{} cannot be opened ({error})", to.display()); })?;
    return made.set_times(times).map_err(|error| { return format!("{} cannot take its times ({error})", to.display()); });
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
pub fn clone_tree(source: &Path, destination: &Path, workers: usize) -> Result<usize, String> {
    let entries: Vec<Entry> = walk(source)?;
    let root: Metadata = std::fs::metadata(source).map_err(|error| { return format!("{} cannot be read ({error})", source.display()); })?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| { return format!("{} cannot be made ({error})", parent.display()); })?;
    }
    // Exclusive creation: an existing destination is refused and left as it was.
    std::fs::create_dir(destination).map_err(|error| { return format!("{} cannot be made ({error})", destination.display()); })?;
    let directories: Vec<&Entry> = entries.iter().filter(|entry: &&Entry| { return entry.kind == Kind::Directory; }).collect();
    for directory in &directories {
        let made: PathBuf = destination.join(&directory.relative);
        std::fs::create_dir(&made).map_err(|error| { return format!("{} cannot be made ({error})", made.display()); })?;
    }
    let leaves: Vec<&Entry> = entries.iter().filter(|entry: &&Entry| { return entry.kind != Kind::Directory; }).collect();
    let used: usize = if leaves.len() < PARALLEL_MIN_ENTRIES { 1 } else { workers.max(1) };
    let slice_length: usize = leaves.len().div_ceil(used).max(1);
    let outcomes: Vec<Result<(), String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = leaves
            .chunks(slice_length)
            .map(|slice: &[&Entry]| {
                return scope.spawn(move || {
                    for entry in slice {
                        place(source, destination, entry)?;
                    }
                    return Ok(());
                });
            })
            .collect();
        return handles.into_iter().map(|handle| { return handle.join().unwrap_or_else(|_panic| { return Err(String::from("a clone worker stopped unexpectedly")); }); }).collect();
    });
    for outcome in outcomes {
        outcome?;
    }
    // Modes last, so a read-only directory does not stop its own contents from being placed.
    for directory in directories.iter().rev() {
        let made: PathBuf = destination.join(&directory.relative);
        let mode: std::fs::Permissions = std::fs::metadata(source.join(&directory.relative)).map_err(|error| { return format!("{} cannot be read ({error})", directory.relative.display()); })?.permissions();
        std::fs::set_permissions(&made, mode).map_err(|error| { return format!("{} cannot take its mode ({error})", made.display()); })?;
    }
    std::fs::set_permissions(destination, root.permissions()).map_err(|error| { return format!("{} cannot take its mode ({error})", destination.display()); })?;
    return Ok(entries.len());
}

/// Tree clone controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tree_clone_tests.rs"]
mod tests;
