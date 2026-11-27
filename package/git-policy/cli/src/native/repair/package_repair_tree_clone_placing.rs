//! What:
//!  Placing the entries of one clone:
//! one file or link made like its source,
//! one slice of entries as one job of the blocking pool,
//! and every slice of one clone awaited together,
//! each answering the paths it left out.
//! Why:
//!  The clone shares the data instead of copying it where the filesystem allows a clone,
//! sets the mode and the times because Cargo decides freshness from them,
//! and splits the leaves over as many workers as the clone's size pays for.
//! The entries are shared with those jobs and never lent,
//! so a job owns what it reads and the walk is not held open by one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function place(source: string, destination: string, entry: Entry): string | undefined;
//! function placeSlice(entries: Entry[], positions: number[], source: string, destination: string): string[];
//! async function placeSlicesAwaiting(placing: ThePlacing, slices: number[][], places: PlacesOfAClone): Promise<string[][]>;
//! ```

/// Import the refusal a job of the blocking pool answers with.
use crate::blocking_job::BlockingJobError;
/// Import the places a clone's steps take,
/// the one-step runner,
/// the entry it places,
/// and the two words a failed step is said in.
use crate::package_repair_tree_clone_entries_and_places::{
    Entry,
    Kind,
    PlacesOfAClone,
    run_a_clone_step,
    say_where,
};
/// Import the exclusive copy a file of the destination is made by.
use crate::worktree_copy_stage::copy_file_exclusive;
/// Import the file times and metadata a placed file keeps.
use std::fs::{
    FileTimes,
    Metadata,
};
/// Import the link a source's link is recreated as.
use std::os::unix::fs::symlink;
/// Import the borrowed and owned filesystem paths a slice places between.
use std::path::{
    Path,
    PathBuf,
};
/// Import the shared entry list the jobs read from.
use std::sync::Arc;

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
pub(super) fn place(
    source: &Path,
    destination: &Path,
    entry: &Entry,
) -> Result<Option<PathBuf>, String> {
    let from: PathBuf = source.join(&entry.relative);
    let to: PathBuf = destination.join(&entry.relative);
    if entry.kind == Kind::Link {
        let target: PathBuf = say_where(
            std::fs::read_link(&from),
            from.as_path(),
            "cannot be read as a link",
        )?;
        say_where(symlink(&target, &to), to.as_path(), "cannot be made")?;
        return Ok(None);
    }
    let before: Metadata = say_where(std::fs::metadata(&from), from.as_path(), "cannot be read")?;
    if let Err(error) = copy_file_exclusive(&from, &to) {
        // A file without read permission cannot be cloned or copied by anyone but its owner's chmod, which the source is never given:
        //  the file stays out of the destination (a missing file only makes Cargo rebuild what needs it) and is named.
        if matches!(std::fs::File::open(&from), Err(ref denied) if denied.kind() == std::io::ErrorKind::PermissionDenied)
        {
            // The partial file is this clone's own creation in the new tree.
            let _removed = std::fs::remove_file(&to);
            return Ok(Some(entry.relative.clone()));
        }
        return Err(format!("{} cannot be cloned ({error})", from.display()));
    }
    say_where(
        std::fs::set_permissions(&to, before.permissions()),
        to.as_path(),
        "cannot take its mode",
    )?;
    let times: FileTimes = FileTimes::new()
        .set_accessed(before.accessed().unwrap_or(std::time::UNIX_EPOCH))
        .set_modified(before.modified().unwrap_or(std::time::UNIX_EPOCH));
    let made: std::fs::File =
        say_where(std::fs::File::open(&to), to.as_path(), "cannot be opened")?;
    say_where(made.set_times(times), to.as_path(), "cannot take its times")?;
    return Ok(None);
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
pub(super) fn place_slice(
    entries: &[Entry],
    positions: &[usize],
    source: &Path,
    destination: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut skipped: Vec<PathBuf> = Vec::new();
    for at in positions {
        let Some(entry) = entries.get(*at) else {
            continue;
        };
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
pub(super) type ASliceToPlace = (Arc<[Entry]>, Vec<usize>, PathBuf, PathBuf);

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
pub(super) fn place_a_slice_of(
    (entries, positions, source, destination): ASliceToPlace
) -> Result<Vec<PathBuf>, String> {
    return place_slice(
        entries.as_ref(),
        positions.as_slice(),
        source.as_path(),
        destination.as_path(),
    );
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
pub(super) struct ThePlacing {
    /// Every entry of the source,
    /// in path order.
    pub(super) entries: Arc<[Entry]>,
    /// The directory the entries come from.
    pub(super) source: PathBuf,
    /// The directory they are made in.
    pub(super) destination: PathBuf,
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
pub(super) fn empty_answer() -> Option<Result<Vec<PathBuf>, String>> {
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
pub(super) async fn place_slices_awaiting(
    placing: ThePlacing,
    slices: Vec<Vec<usize>>,
    places: PlacesOfAClone,
) -> Vec<Result<Vec<PathBuf>, String>> {
    let count: usize = slices.len();
    let entries: Arc<[Entry]> = placing.entries;
    let mut started: tokio::task::JoinSet<(usize, Result<Vec<PathBuf>, String>)> =
        tokio::task::JoinSet::new();
    for (index, positions) in slices.into_iter().enumerate() {
        let theirs: Arc<[Entry]> = Arc::clone(&entries);
        let (source, destination): (PathBuf, PathBuf) =
            (placing.source.clone(), placing.destination.clone());
        started.spawn(async move {
            let placed = run_a_clone_step(
                places,
                (theirs, positions, source, destination),
                place_a_slice_of,
            )
            .await;
            let answer: Result<Vec<PathBuf>, String> = match placed {
                Ok(outcome) => outcome,
                // The entries of a slice that did not run are not in the destination, and the words say only that.
                Err(BlockingJobError::Panicked) => {
                    Err(String::from("a clone worker stopped unexpectedly"))
                }
                Err(refused) => Err(format!("a clone worker did not run ({refused})")),
            };
            return (index, answer);
        });
    }
    let mut answers: Vec<Option<Result<Vec<PathBuf>, String>>> =
        std::iter::repeat_with(empty_answer).take(count).collect();
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
