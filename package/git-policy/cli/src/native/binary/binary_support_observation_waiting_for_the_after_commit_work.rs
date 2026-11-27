//! What:
//!  The wait for the after-commit work a landed commit leaves to the commit service:
//!  the folders a record of its queue is in flight in,
//!  the step a control installs to move one between two reads,
//!  the Git directory of the worktree a commit was made in,
//!  and the listing a failure names.
//! Why:
//!  A commit that landed through the service returns before its push and its upkeep run,
//!  so a control that inspects the origin reads the queue until it is empty;
//!  the queue and the order its records move in are one job,
//!  and they live in a file of their own beside the faces that call them,
//!  which keeps those faces inside the repository's per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! settleServiceWork(repo); // returns once no record of the after-commit queue is in flight
//! ```

use git_policy_cli::work_item::{
    CLAIMED,
    PENDING,
    RECOVERING,
};
use std::path::{
    Path,
    PathBuf,
};

/// How long a control waits for the after-commit work (the push,
/// the upkeep) the commit service runs once the commit returned.
pub const SERVICE_WORK_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// The pause between two samples of the work queue the wait reads.
const SAMPLE_PAUSE: std::time::Duration = std::time::Duration::from_millis(20);

/// What:
///  The folders a record of this queue is in flight in,
///  in the order a record moves through them.
/// Why:
///   `take_entry` (`work_item.rs`)
/// claims a record in three steps:
/// a rename from `pending` into `recovering`,
/// a stamp there,
/// and a rename into `claimed`,
/// and a claim settles from `claimed` into `done` or `failed`.
/// A sample that reads these in the order a record moves through them cannot miss a record moving forward
/// during that sample,
/// because a forward move lands it in a folder the sample has not read yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inFlight = ['pending', 'recovering', 'claimed'] as const;
/// ```
pub const A_RECORD_IN_FLIGHT_FOLDERS: &[&str] = &[PENDING, RECOVERING, CLAIMED];

/// What:
///  A step a control runs before each folder of one sample is read,
///  and no release run installs one.
/// Why:
///  The window this wait closes is one record moving between two folders while a sample reads them,
/// and only a step between the reads can move the record there,
/// so the control of the wait installs one (`binary_support_observation_tests.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BeforeAFolderRead = (folder: string) => void
/// ```
type BeforeAFolderRead = fn(folder: &str);

thread_local! {
    /// The step the running control installed,
    /// and nothing in an ordinary run.
    static BEFORE_A_FOLDER_READ: std::cell::Cell<Option<BeforeAFolderRead>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Install the step a sample runs before it reads each folder,
///  or take the step away with `None`.
/// Why:
///  A control that means to move a record between two folders of one sample has to move it between two reads,
///  and this is the only point between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeEachFolderRead(step);
/// ```
pub fn run_before_each_folder_read(step: Option<BeforeAFolderRead>) {
    BEFORE_A_FOLDER_READ.with(|installed| {
        installed.set(step);
    });
}

/// What:
///  Whether a record of the queue is in one of the folders it is in flight in,
///  read in the order a record moves through them.
/// Why:
///   See `A_RECORD_IN_FLIGHT_FOLDERS`,
/// and `settle_service_work` for why a second such sample is asked for as well.
/// A folder that cannot be read holds nothing,
/// as before.
fn a_record_is_in_flight(work: &Path) -> bool {
    for folder in A_RECORD_IN_FLIGHT_FOLDERS {
        let installed = BEFORE_A_FOLDER_READ.with(std::cell::Cell::get);
        if let Some(step) = installed {
            step(folder);
        }
        if std::fs::read_dir(work.join(folder)).map_or(0, Iterator::count) > 0 {
            return true;
        }
    }
    return false;
}

/// What:
///  The Git directory of the worktree `directory` is in:
///  the first `.git` on the way up,
///  followed to the directory it names when that `.git` is a file.
/// Why:
///   A linked worktree has no `.git` directory of its own;
/// it holds a file with one `gitdir:` line naming its administrative directory,
///  where its own queue of after-commit work lives.
/// A wait that looked for a directory alone returned at once for such a worktree,
///  so a control that committed in one read the origin before its push landed (issue 6336).
///  The named path is read as Git wrote it:
///  a trailing space belongs to the directory's name
///  (`a_commit_lands_in_a_worktree_whose_git_directory_name_ends_with_a_space`),
///  and a relative path is relative to the directory that holds the file.
fn git_directory_of(directory: &Path) -> Option<PathBuf> {
    for candidate in directory.ancestors() {
        let marker: PathBuf = candidate.join(".git");
        if marker.is_dir() {
            return Some(marker);
        }
        if !marker.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(marker.as_path()) else {
            return None;
        };
        let named: &str = text.lines().next().and_then(|line| {
            return line.strip_prefix("gitdir: ");
        })?;
        if named.is_empty() {
            return None;
        }
        let path: PathBuf = PathBuf::from(named);
        return Some(if path.is_absolute() {
            path
        } else {
            candidate.join(path)
        });
    }
    return None;
}

/// Every state directory of a work directory with the names it holds,
/// for the failure message of a wait that ran out.
pub fn work_listing(work: &Path) -> String {
    return ["pending", "claimed", "recovering", "done", "failed"]
        .iter()
        .map(|state| {
            let mut names: Vec<String> = std::fs::read_dir(work.join(state))
                .map(|listing| {
                    return listing
                        .flatten()
                        .map(|entry| {
                            return entry.file_name().to_string_lossy().into_owned();
                        })
                        .collect();
                })
                .unwrap_or_default();
            names.sort();
            return format!("{state}={names:?}");
        })
        .collect::<Vec<String>>()
        .join(" ");
}

/// What:
///  Waits until the commit service has settled the after-commit work of the repository around `directory`.
/// Why:
///   A commit that landed through the service returns before its push and its upkeep run,
/// so a control that inspects the
///  origin,
/// the packs or the push's failure after the command must first wait for the records to leave `pending`,
///  `claimed` and `recovering`;
/// it waits only where a work directory exists.
/// The repository is the first `.git` on the way up,
/// which a linked worktree keeps as a file that names its own Git directory
/// (`git_directory_of`),
/// so the wait covers a commit made in any worktree of the repository.
///  This wait is `SERVICE_WORK_PATIENCE`,
///  which is shorter than one claim's lease (`CLAIM_LEASE`,
///  120 seconds):
/// a record whose handler answered with a retry keeps its claim until that lease ends,
/// so a control that waits here cannot see such a record settle.
/// A control that means to read one attempt of such a record reads that instead
/// (`observe_without_waiting_for_the_after_commit_work`).
///  The folders are read in the order a record moves through them,
/// and an empty reading must be confirmed by a second one,
/// because a record being claimed moves between two of them while a sample reads them
/// (issue 6345,
///  and `A_RECORD_IN_FLIGHT_FOLDERS` for the moves the code makes).
pub fn settle_service_work(directory: &Path) {
    let Some(git_dir) = git_directory_of(directory) else {
        return;
    };
    let work: PathBuf = git_dir.join(git_policy_cli::work_item::WORK_DIRECTORY);
    let began: std::time::Instant = std::time::Instant::now();
    // The flow order alone closes the moves that go forward, and the second sample the ones that go back;
    // the third closes a recovery whose two renames straddle two samples.
    // Every move a record makes between these folders, from the code:
    // `take_entry` (`work_item.rs`) moves `pending` into `recovering` and then into `claimed`,
    // and moves a record it could not publish from `recovering` back into `pending`,
    // takes a `claimed` record of a later release back into `pending`, and settles one in `failed`;
    // `quarantine_pending` moves `pending` into `failed`;
    // `settle_named` moves a claim into `done` or `failed`, and `renew_claim` renames a claim inside `claimed`;
    // `recover` (`work_item_recovery.rs`) moves a `claimed` or `recovering` record into `failed` or, under a new name,
    // into `recovering`, and from there into `pending` or `failed`;
    // `service_handoff` moves a held or a pending record into `done`,
    // `promote` moves a held record into `pending`, and `requeue_after_giving_up` moves a failed one into `pending`.
    // Arrivals into `pending` from outside are written by commands (`enqueue_all`), never by a service round,
    // and a record in `held` is not waited for.
    // What three samples do not close, over schedules of one record's moves placed among the wait's reads,
    // counted by a reviewer's run kept outside the repository and reproduced by the pass after it:
    // they closed all 3350 schedules of up to three moves, and all but 4 of 18365 of up to four.
    // So a chain of four or more moves that a thread's stall stretches across every sample of this wait is still missed.
    // A record renamed inside one folder while that folder is read was not tested.
    while a_record_is_in_flight(work.as_path())
        || a_record_is_in_flight(work.as_path())
        || a_record_is_in_flight(work.as_path())
    {
        assert!(
            began.elapsed() < SERVICE_WORK_PATIENCE,
            "the service never settled the after-commit work in {}: {}",
            work.display(),
            work_listing(work.as_path())
        );
        std::thread::sleep(SAMPLE_PAUSE);
    }
}
