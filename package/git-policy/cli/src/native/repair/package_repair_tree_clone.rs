//! What:
//!  Cloning one directory tree for a worktree repair:
//! every file cloned (or copied when the filesystem cannot clone),
//! every
//!  directory recreated with its mode,
//! every link recreated with its exact target,
//! modes and times kept.
//! Why:
//!  Cargo keeps its build state outside the worktree when the person's configuration says so,
//! so a copy of the worktree's
//!  ignored files does not carry it.
//! The state is reused by the new worktree's registry and git dependencies,
//! and Cargo
//!  decides what is fresh from file times,
//! so the clone keeps them.
//! The destination must not exist:
//! nothing is overwritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cloned = await cloneTree({ source, destination, workers });
//! ```

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::lazy_runtime::run_on_runtime;
use crate::worktree_copy_stage::copy_file_exclusive;
use std::fs::{FileTimes, Metadata, Permissions};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// What a walk of the source finds at one path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    /// A directory,
    /// recreated and entered.
    Directory,
    /// A regular file,
    /// cloned.
    File,
    /// A symbolic link,
    /// recreated with its exact target.
    Link,
}

/// One entry of the source,
/// relative to its root.
struct Entry {
    /// The path below the root.
    relative: PathBuf,
    /// What it is.
    kind: Kind,
}

/// The entries below which a clone stays on one thread,
/// `PARALLEL_BREAK_EVEN_ENTRIES`;
/// starting threads costs more than a very small
///  clone.
/// Measured with `clone_tree` itself forced to 1,
/// 2,
/// 4 and 8 workers (2026-11-03,
/// five runs each in alternating orders,
///  other sessions running,
/// a debug build):
/// 0.6 ms on every count for 10 entries,
/// 1.2 ms on 8 workers against 1.8 ms on one for 30,
///  8.1 ms against 20.7 ms for 296,
/// and 13.4 ms against 39.2 ms for 555,
/// so the old threshold of 512 left clones of up to 511 leaves
///  on one thread for up to 12 ms more.
/// Picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
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
///  Every entry below the source,
/// parents before children,
/// without following links.
/// Why:
///   Directories must exist before a worker places a file in them,
/// and a link to a directory must be copied as a link.
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
///   The clone shares the data;
/// the mode and the times are set after it,
/// because Cargo decides freshness from file times.
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
///  Say which path an input or output step failed on:
/// `<path> <what> (<error>)`.
///  `Result<T, std::io::Error>` is "the value,
/// or an input/output error".
/// Why:
///   Every step of the clone names the entry it could not handle,
/// and a named function stands where a closure that
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

/// Order two entries by their path below the source,
/// so parents sort before their children.
/// `sort_by` takes this
///  function by path;
/// the sort stays stable.
fn compare_by_relative_path(left: &Entry, right: &Entry) -> std::cmp::Ordering {
    return left.relative.cmp(&right.relative);
}

/// What:
///  Place every entry of one slice,
/// and return the paths the slice left out.
/// Why:
///   One worker takes a slice;
/// the first entry that cannot be placed ends the slice with its message.
/// The slice is a list of positions in `entries`,
/// so a job of the blocking pool owns nothing of the walk but the numbers,
///  and the entries stay shared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placeSlice(entries: Entry[], positions: number[], source: string, destination: string): string[] { return positions.flatMap((at) => place(source, destination, entries[at]) ?? []); }
/// ```
fn place_slice(entries: &[Entry], positions: &[usize], source: &Path, destination: &Path) -> Result<Vec<PathBuf>, String> {
    let mut skipped: Vec<PathBuf> = Vec::new();
    for at in positions {
        let Some(entry) = entries.get(*at) else { continue };
        skipped.extend(place(source, destination, entry)?);
    }
    return Ok(skipped);
}

/// What:
///  One slice of leaves as the state of one job:
/// the entries,
/// the positions of this slice,
/// and the two directories.
/// Why:
///   A job of the pool is a function and the state it reads
/// (`blocking_jobs_within_the_budget.rs`),
/// so the slice travels as values the job owns and
///  may be taken again where no thread can start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASliceToPlace = [Entry[], number[], string, string];
/// ```
type ASliceToPlace = (Arc<[Entry]>, Vec<usize>, PathBuf, PathBuf);

/// What the job of one slice reads,
/// placed as `place_slice` places it.
/// Why:
///   The budget runner takes a function and its state,
/// never a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placeASliceOf = ([entries, positions, source, destination]) => placeSlice(entries, positions, source, destination);
/// ```
fn place_a_slice_of((entries, positions, source, destination): ASliceToPlace) -> Result<Vec<PathBuf>, String> {
    return place_slice(entries.as_ref(), positions.as_slice(), source.as_path(), destination.as_path());
}

/// What:
///  The walk and the making of the directories as the state of one job.
/// Why:
///   Same as `place_a_slice_of`:
/// the walk is storage work,
/// and its two paths travel as values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const walkAndMake = ([source, destination]) => walkAndMakeDirectories(source, destination);
/// ```
fn walk_and_make_of((source, destination): (PathBuf, PathBuf)) -> Result<TheWalk, String> {
    return walk_and_make_directories(source.as_path(), destination.as_path());
}

/// What:
///  The modes of a finished clone as the state of one job.
/// Why:
///   Same as `place_a_slice_of`:
/// the modes are storage work,
/// and everything the job reads travels as values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setTheModesOf = ([source, destination, entries, directories, rootMode]) => setTheModes(source, destination, entries, directories, rootMode);
/// ```
fn set_the_modes_of((source, destination, entries, directories, root_mode): (PathBuf, PathBuf, Arc<[Entry]>, Vec<usize>, Permissions)) -> Result<(), String> {
    return set_the_modes(source.as_path(), destination.as_path(), entries.as_ref(), directories.as_slice(), root_mode);
}

/// What:
///  The leaves of a clone,
/// shared with the jobs that place them,
/// and the two directories they go between.
/// Why:
///   A job of the blocking pool must own what it reads,
/// so the entries a walk made are shared instead of lent,
/// and each
///  job takes its own slice of them as one more share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThePlacing = { entries: Entry[]; source: string; destination: string };
/// ```
struct ThePlacing {
    /// Every entry of the source,
    /// in path order.
    entries: Arc<[Entry]>,
    /// The directory the entries come from.
    source: PathBuf,
    /// The directory they are made in.
    destination: PathBuf,
}

/// No answer yet:
/// the value a slot of the report holds until its slice ends.
/// Why:
///   A `Vec` of these is made without a closure,
/// as the concurrent runner's own is
/// (`package_repair_cargo_concurrent.rs`,
/// `empty_answer`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const emptyAnswer = () => undefined;
/// ```
fn empty_answer() -> Option<Result<Vec<PathBuf>, String>> {
    return None;
}

/// What:
///  Place each slice of leaf entries as one job of the blocking pool,
/// awaited on this task,
///  and give one answer per slice in slice order.
/// Why:
///   The entries,
/// the source and the destination are shared with the jobs,
/// which own what they read,
/// and the process's one bound
///  on blocking jobs covers every slice.
/// The slices are started as tasks and collected in slice order:
/// the ordered runners take their
///  consumer as a borrow that is not `Send`
/// (`parallel_runner_on_the_runtime_awaiting.rs`),
/// so neither runner can be awaited inside a task of another
///  run,
/// and this clone is one item of the Cargo repair's run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await Promise.all(slices.map((slice) => runBlockingJobWithinTheBudget(() => placeSlice(entries, slice, source, destination))));
/// ```
async fn place_slices_awaiting(placing: ThePlacing, slices: Vec<Vec<usize>>) -> Vec<Result<Vec<PathBuf>, String>> {
    let count: usize = slices.len();
    let entries: Arc<[Entry]> = placing.entries;
    let mut started: tokio::task::JoinSet<(usize, Result<Vec<PathBuf>, String>)> = tokio::task::JoinSet::new();
    for (index, positions) in slices.into_iter().enumerate() {
        let theirs: Arc<[Entry]> = Arc::clone(&entries);
        let (source, destination): (PathBuf, PathBuf) = (placing.source.clone(), placing.destination.clone());
        started.spawn(async move {
            let placed = run_a_job_or_in_turn((theirs, positions, source, destination), place_a_slice_of).await;
            let answer: Result<Vec<PathBuf>, String> = match placed {
                Ok(outcome) => outcome,
                // The entries of a slice that did not run are not in the destination, and the words say only that.
                Err(BlockingJobError::Panicked) => Err(String::from("a clone worker stopped unexpectedly")),
                Err(refused) => Err(format!("a clone worker did not run ({refused})")),
            };
            return (index, answer);
        });
    }
    let mut answers: Vec<Option<Result<Vec<PathBuf>, String>>> = std::iter::repeat_with(empty_answer).take(count).collect();
    while let Some(joined) = started.join_next().await {
        match joined {
            Ok((index, answer)) => answers[index] = Some(answer),
            // A task that panicked outside the guarded job placed nothing and must be read as such below.
            Err(_panicked) => {}
        }
    }
    let mut outcomes: Vec<Result<Vec<PathBuf>, String>> = Vec::new();
    for (index, answer) in answers.into_iter().enumerate() {
        outcomes.push(match answer {
            Some(given) => given,
            None => Err(format!("a clone worker gave no answer (slice {index})")),
        });
    }
    return outcomes;
}

/// What:
///  Everything a clone does before and after its slices,
/// as the pool does it.
/// Why:
///   The walk,
/// the directory creation and the modes are storage waits,
/// so they run as jobs of the blocking pool rather than on the task
///  that awaits them (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheWalk = { entries: Entry[]; leftOut: string[]; directories: number[]; rootMode: Permissions };
/// ```
struct TheWalk {
    /// Every entry below the source,
    /// parents before children.
    entries: Vec<Entry>,
    /// The paths that could not be read and so are not in the destination.
    left_out: Vec<PathBuf>,
    /// The positions of the entries that are directories,
    /// which are made now and whose modes are set at the end.
    directories: Vec<usize>,
    /// The mode the source itself has,
    /// which the destination takes last.
    root_mode: Permissions,
}

/// What:
///  Walk the source and make every directory of the destination,
/// so the slices have somewhere to place their files.
/// Why:
///   Directories must exist before a slice places a file in them,
/// and the walk names the entry it could not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkAndMakeDirectories(source: string, destination: string): TheWalk;
/// ```
fn walk_and_make_directories(source: &Path, destination: &Path) -> Result<TheWalk, String> {
    let (entries, left_out): (Vec<Entry>, Vec<PathBuf>) = walk(source)?;
    let root: Metadata = say_where(std::fs::metadata(source), source, "cannot be read")?;
    if let Some(parent) = destination.parent() {
        say_where(std::fs::create_dir_all(parent), parent, "cannot be made")?;
    }
    // Exclusive creation: an existing destination is refused and left as it was.
    say_where(std::fs::create_dir(destination), destination, "cannot be made")?;
    let mut directories: Vec<usize> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.kind == Kind::Directory {
            let made: PathBuf = destination.join(&entry.relative);
            say_where(std::fs::create_dir(&made), made.as_path(), "cannot be made")?;
            directories.push(index);
        }
    }
    return Ok(TheWalk { entries, left_out, directories, root_mode: root.permissions() });
}

/// What:
///  Give the destination and its directories the modes of the source,
/// the deepest directory first.
/// Why:
///   Modes go last,
/// so a read-only directory does not stop its own contents from being placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setTheModes(source: string, destination: string, entries: Entry[], directories: number[], rootMode: Permissions): void;
/// ```
fn set_the_modes(source: &Path, destination: &Path, entries: &[Entry], directories: &[usize], root_mode: Permissions) -> Result<(), String> {
    for at in directories.iter().rev() {
        let Some(entry) = entries.get(*at) else { continue };
        let made: PathBuf = destination.join(&entry.relative);
        let source_directory: Metadata = say_where(std::fs::metadata(source.join(&entry.relative)), entry.relative.as_path(), "cannot be read")?;
        say_where(std::fs::set_permissions(&made, source_directory.permissions()), made.as_path(), "cannot take its mode")?;
    }
    say_where(std::fs::set_permissions(destination, root_mode), destination, "cannot take its mode")?;
    return Ok(());
}

/// What:
///  Clone `source` into the new directory `destination` with every wait awaited on the runtime.
/// Why:
///  `clone_tree` enters the runtime,
/// which a caller that is itself a task cannot do
/// (`RuntimeError::Nested`),
/// and the clone of a build directory is one
///  of the waits a task of the Cargo workspace repair makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cloneTreeAwaiting(source: string, destination: string, workers: number): Promise<Cloned>;
/// ```
pub(crate) async fn clone_tree_awaiting(source: &Path, destination: &Path, workers: usize) -> Result<Cloned, String> {
    let walked = run_a_job_or_in_turn((source.to_path_buf(), destination.to_path_buf()), walk_and_make_of).await;
    let TheWalk { entries, mut left_out, directories, root_mode }: TheWalk = match walked {
        Ok(Ok(found)) => found,
        // The walk itself failed and named the entry it could not read; nothing of the clone was made.
        Ok(Err(reason)) => return Err(reason),
        // Nothing of the clone was made, and the runtime's own reason is why.
        Err(refused) => return Err(format!("{} could not be cloned ({refused})", source.display())),
    };
    let mut leaves: Vec<usize> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.kind != Kind::Directory {
            leaves.push(index);
        }
    }
    let used: usize = if leaves.len() < PARALLEL_MIN_ENTRIES { 1 } else { workers.max(1) };
    let slice_length: usize = leaves.len().div_ceil(used).max(1);
    let slices: Vec<Vec<usize>> = leaves.chunks(slice_length).map(<[usize]>::to_vec).collect();
    let shared: Arc<[Entry]> = Arc::from(entries);
    let placing: ThePlacing = ThePlacing { entries: Arc::clone(&shared), source: source.to_path_buf(), destination: destination.to_path_buf() };
    let outcomes: Vec<Result<Vec<PathBuf>, String>> = place_slices_awaiting(placing, slices).await;
    for outcome in outcomes {
        left_out.extend(outcome?);
    }
    left_out.sort();
    // Modes last, so a read-only directory does not stop its own contents from being placed.
    let modes = run_a_job_or_in_turn((source.to_path_buf(), destination.to_path_buf(), Arc::clone(&shared), directories, root_mode), set_the_modes_of).await;
    match modes {
        Ok(Ok(())) => {}
        // A directory or the destination could not take its mode, and the message names which.
        Ok(Err(reason)) => return Err(reason),
        // Every entry is in the destination and the modes are the ones its creation gave, which the words say.
        Err(refused) => return Err(format!("{} could not take its modes ({refused})", destination.display())),
    }
    let mut skipped_files: usize = 0;
    for entry in shared.iter() {
        if entry.kind != Kind::Directory && left_out.contains(&entry.relative) {
            skipped_files += 1;
        }
    }
    return Ok(Cloned { made: shared.len() - skipped_files, left_out });
}

/// What:
///  Clone `source` into the new directory `destination`.
/// Why:
///   The count of entries made is returned for the log;
/// a failure names the entry that could not be made,
/// and leaves what was
///  made in place for the person to remove with the destination.
/// The callers of this face are synchronous (the Cargo repair of a worktree inside the repair the add runs),
/// so it
///  enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
/// and the Cargo repair's own
///  items await `clone_tree_awaiting` instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneTree(args: { source: string; destination: string; workers: number }): Cloned;
/// ```
pub fn clone_tree(source: &Path, destination: &Path, workers: usize) -> Result<Cloned, String> {
    return match run_on_runtime(clone_tree_awaiting(source, destination, workers)) {
        Ok(cloned) => cloned,
        // Nothing of the clone was made, and the runtime's own reason is why.
        Err(refusal) => Err(format!("{} could not be cloned ({refusal})", source.display())),
    };
}

/// Tree clone controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tree_clone_tests.rs"]
mod tests;
