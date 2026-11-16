//! What:
//!  The wait a control makes for the after-commit work the commit service runs:
//! it settles the work of a repository,
//!  and it never answers "settled" from a reading that proved nothing.
//! Why:
//!  A control reads the effect of the service's work (a tool's record,
//! the origin,
//! the packs) after that wait,
//! so a wait that answers while a record is still queued reads the state before the work.
//! The wait had two ways to answer that without evidence:
//! a state directory it could not list counted as no record,
//!  and it spelled the queue's folder with this release's name only,
//! while the wrapper reads a Git directory's data folder under whichever of the two spellings exists.
//!  On 2026-11-15 a package-repair control failed in a whole suite and in the container gate with the record read as
//! absent and the tool run after the control had looked
//! (`doc/troubleshooting/native-binary-settle-wait-reads-an-unread-state-as-settled.md`).
//! Two more were found by the review of 2026-11-16 and are this module's two newest controls:
//! one pass over the three states could find all three empty while an expired claim carried a record from `claimed` to
//! `pending`,
//!  and a repository holding both data-folder spellings kept a record open in a queue the wait never listed
//! (`doc/troubleshooting/native-binary-settle-wait-reads-an-unread-state-as-settled.md`,
//! "A record in motion and a second queue").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleServiceWork(repo); // waits while any queue holds a record, and never answers from a listing it could not make
//! ```

use super::support::{Fixture, LISTINGS_IN_A_LOOK, fixture, records_in, remove, repository, settle_service_work_list_with};
use git_policy_cli::work_item::{CLAIMED, DONE, FAILED, PENDING, RECOVERING, WORK_DIRECTORY};
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// How long a control of the wait looks at a wait that has a record to wait for,
/// in milliseconds.
/// Why this is enough:
///  the wait looks again every 20 ms,
/// so a wait that returned in any of the fifteen looks this covers returned before the record it was waiting for.
const LOOK_SECONDS_AS_MILLIS: u64 = 300;

/// How long a control waits for a wait it has given work to,
/// in seconds:
///  many times the wait's own look,
/// so only a wait that never settles runs past it.
const A_WAIT_THAT_SETTLES_SECONDS: u64 = 30;

/// A work directory's state directory,
/// made with no record in it.
fn state_directory(work: &Path, state: &str) -> PathBuf {
    let directory: PathBuf = work.join(state);
    std::fs::create_dir_all(directory.as_path()).expect("a state directory of the queue");
    return directory;
}

/// The repository of a fixture,
/// and the folder this wait reads the queue of.
fn repository_and_work(fixture: &Fixture) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let work: PathBuf = repo.join(".git").join(git_policy_cli::work_item::WORK_DIRECTORY);
    return (repo, work);
}

/// Whether a wait of `repo` has returned yet,
/// and the record it was given is taken away once the wait has shown it waits for it.
struct Waiting {
    /// Raised when the wait returns.
    returned: Arc<AtomicBool>,
    /// The thread that waits.
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Waiting {
    /// A wait of `repo` started on a thread of its own.
    fn start(repo: &Path) -> Self {
        let returned: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let raised: Arc<AtomicBool> = Arc::clone(&returned);
        let directory: PathBuf = repo.to_path_buf();
        let thread = std::thread::spawn(move || {
            super::support::settle_service_work(directory.as_path());
            raised.store(true, Ordering::Release);
        });
        return Self { returned, thread: Some(thread) };
    }

    /// Whether the wait has returned.
    fn returned(&self) -> bool {
        return self.returned.load(Ordering::Acquire);
    }

    /// Waits for the wait itself to return.
    fn join(mut self) {
        let Some(thread) = self.thread.take() else {
            return;
        };
        let began: Instant = Instant::now();
        while !self.returned() {
            assert!(
                began.elapsed() < Duration::from_secs(A_WAIT_THAT_SETTLES_SECONDS),
                "the wait for the after-commit work did not return within {A_WAIT_THAT_SETTLES_SECONDS} s of its record being taken away"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        if let Err(payload) = thread.join() {
            std::panic::resume_unwind(payload);
        }
    }
}

/// A queue a repository's wrapper still keeps under the earlier release's folder name is waited for.
#[test]
fn a_queue_left_under_the_earlier_folder_name_is_waited_for() {
    let fixture: Fixture = fixture("settle-earlier-folder");
    let (repo, current): (PathBuf, PathBuf) = repository_and_work(&fixture);
    // The wrapper reads the data folder under whichever spelling exists, so a repository whose folder an earlier release
    // made holds its queue there.
    let earlier: PathBuf = repo.join(".git").join(git_policy_cli::wrapper_names_old_and_new::DATA_FOLDER.earlier);
    // The work directory's spelling begins with the data folder's own name, which the earlier release spelled differently.
    let within_the_data_folder: &str = git_policy_cli::work_item::WORK_DIRECTORY.rsplit('/').next().expect("the work directory's last segment");
    let work: PathBuf = earlier.join(within_the_data_folder);
    let pending: PathBuf = state_directory(work.as_path(), "pending");
    let record: PathBuf = pending.join("relink-0123456789abcdef0123456789abcdef01234567");
    std::fs::write(record.as_path(), b"{\"kind\":\"relink\"}\n").expect("the record an earlier release queued");
    assert!(!current.exists(), "this control needs the earlier spelling alone: {}", current.display());
    let waiting: Waiting = Waiting::start(repo.as_path());
    std::thread::sleep(Duration::from_millis(LOOK_SECONDS_AS_MILLIS));
    let returned_in_time: bool = waiting.returned();
    std::fs::remove_file(record.as_path()).expect("the record was taken away");
    waiting.join();
    assert!(!returned_in_time, "the wait answered settled while {} held a record", work.display());
    remove(&fixture);
}

/// A state directory the wait cannot list is reported:
///  it is never read as "no record there".
#[test]
fn a_state_directory_that_cannot_be_listed_is_reported() {
    let fixture: Fixture = fixture("settle-unreadable-state");
    let (repo, work) = repository_and_work(&fixture);
    let pending: PathBuf = state_directory(work.as_path(), "pending");
    let mode_before: u32 = std::fs::metadata(pending.as_path()).expect("the state directory").permissions().mode();
    std::fs::set_permissions(pending.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a state directory nothing may list");
    let panicked: Option<String> = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        super::support::settle_service_work(repo.as_path());
    }))
    .err()
    .map(|payload| {
        return payload.downcast_ref::<String>().cloned().unwrap_or_else(|| { return String::from("a check failed without a message"); });
    });
    std::fs::set_permissions(pending.as_path(), std::fs::Permissions::from_mode(mode_before)).expect("the state directory's own mode back");
    let said: String = panicked.expect("a state directory nothing can list must be reported, not read as settled");
    assert!(said.contains("could not be listed"), "{said}");
    assert!(said.contains(pending.to_str().expect("a utf-8 fixture path")), "{said}");
    remove(&fixture);
}

/// The record the two new controls put in a queue,
/// named as a push names its own.
const A_QUEUED_RECORD: &str = "push-0123456789abcdef0123456789abcdef01234567";

/// How many looks a record's recovery can span,
/// for a control that places its two renames at the gaps between a look's listings.
/// Why two:
///  the wait pauses between two looks,
/// so a recovery whose renames fall on either side of that pause is in motion across both of them.
const LOOKS_A_RECOVERY_CAN_SPAN: usize = 2;

/// A record a control moves through a queue while the wait lists its state directories,
/// as an expired claim returns it from `claimed` to `pending` through `recovering`
/// (`landing/work_item_recovery.rs`).
struct MovingRecord {
    /// The queue the record moves in.
    work: PathBuf,
    /// The state the record is in now.
    state: RefCell<&'static str>,
    /// How many listings have been answered.
    listings: Cell<usize>,
    /// What happens in the gap after each listing,
    /// by the number of the listing it follows:
    /// the state the record is moved to.
    moves_after: Vec<(usize, &'static str)>,
}

impl MovingRecord {
    /// What:
    ///  The listing the wait asks for,
    /// with the record's next move done in the gap after it.
    /// Why:
    ///   A control cannot make a rename straddle a listing on a real filesystem,
    ///  so it moves the record in the gap the wait itself leaves between two listings,
    /// and the count it answers is the one the directory really held when the wait asked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const listing = (work, state) => { const count = recordsIn(work, state); listings += 1; moveAfter(listings); return count };
    /// ```
    fn listing(&self) -> impl Fn(&Path, &str) -> Result<usize, String> + '_ {
        return move |work: &Path, state: &str| -> Result<usize, String> {
            let count: usize = records_in(work, state)?;
            let taken: usize = self.listings.get() + 1;
            self.listings.set(taken);
            for (after, to) in self.moves_after.iter() {
                if *after == taken {
                    self.move_to(to);
                }
            }
            return Ok(count);
        };
    }

    /// Moves the record from the state it is in to `to`,
    /// and says so if it was not there.
    fn move_to(&self, to: &'static str) {
        let from: &'static str = *self.state.borrow();
        if from == to {
            return;
        }
        let source: PathBuf = self.work.join(from).join(A_QUEUED_RECORD);
        let target: PathBuf = state_directory(self.work.as_path(), to).join(A_QUEUED_RECORD);
        std::fs::rename(source.as_path(), target.as_path()).unwrap_or_else(|error| {
            panic!("the control's record could not move from `{from}` to `{to}`: {error}");
        });
        *self.state.borrow_mut() = to;
    }
}

/// Puts the control's record back where an expired claim left it,
/// and takes every copy of it out of the four states it can be in.
fn the_record_starts_in_claimed(work: &Path) {
    for state in [CLAIMED, RECOVERING, PENDING, DONE] {
        let record: PathBuf = state_directory(work, state).join(A_QUEUED_RECORD);
        let _gone = std::fs::remove_file(record.as_path());
    }
    let claimed: PathBuf = work.join(CLAIMED).join(A_QUEUED_RECORD);
    std::fs::write(claimed.as_path(), b"{\"kind\":\"push\"}\n").expect("the record an expired claim left in claimed");
}

/// What:
///  A record an expired claim returns from `claimed` to `pending` through `recovering`,
///  moved at every two gaps of a look,
///  is never read as settled work while it is in one of those three folders.
/// Why:
///   A record is in exactly one of the three states until its work is settled,
/// and one pass over them is three listings at three moments:
/// a record that leaves `recovering` for `pending` between two of a look's listings,
/// after another listing saw `recovering` empty,
/// is in none of them although it was queued throughout
/// (review of 2026-11-16,
/// `one_recovery_missed_by_two_consecutive_empty_readings.rs`).
/// What this control proves:
/// whatever the wait's look reads and in whatever order,
/// a record moved at any two of the gaps of the looks it spans is in a state the wait counts when it answers,
/// so an answer of "settled" cannot be given while the record is open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const first of gaps) for (const second of gaps.slice(first)) { moving = aRecordMoving({movesAfter: [[first, RECOVERING], [second, PENDING], [settlesAfter, DONE]]}); settleServiceWorkListWith(repo, moving.listing); expect(moving.state).toBe(DONE); }
/// ```
#[test]
fn a_recovery_moved_at_every_gap_is_never_read_as_settled() {
    let fixture: Fixture = fixture("settle-recovery-gaps");
    let (repo, work): (PathBuf, PathBuf) = repository_and_work(&fixture);
    let gaps: usize = LOOKS_A_RECOVERY_CAN_SPAN * LISTINGS_IN_A_LOOK;
    // The record's work settles one gap after the last one this control moves it at,
    // so a wait that answered before it was unsettled would have to answer with the record still in a counted state.
    let settles_after: usize = gaps + 1;
    for first in 1..=gaps {
        for second in first..=gaps {
            the_record_starts_in_claimed(work.as_path());
            let moving = MovingRecord {
                work: work.clone(),
                state: RefCell::new(CLAIMED),
                listings: Cell::new(0),
                moves_after: vec![(first, RECOVERING), (second, PENDING), (settles_after, DONE)],
            };
            settle_service_work_list_with(repo.as_path(), &moving.listing());
            for state in [PENDING, RECOVERING, CLAIMED] {
                assert!(
                    !work.join(state).join(A_QUEUED_RECORD).exists(),
                    "the wait answered settled while its record was still in `{state}`: the recovery's renames fell at the gaps after listings {first} and {second} of the {gaps} this control places them at, and the record's work settles only after listing {settles_after}"
                );
            }
            assert_eq!(*moving.state.borrow(), DONE, "the wait answered settled although the record never reached `done`");
        }
    }
    remove(&fixture);
}

/// The queue folder a repository keeps under one data-folder spelling,
/// with every state directory of it made and nothing in them.
fn an_empty_queue(repo: &Path, data_folder: &str) -> PathBuf {
    let within_the_data_folder: &str = WORK_DIRECTORY.rsplit('/').next().expect("the work directory's last segment");
    let work: PathBuf = repo.join(".git").join(data_folder).join(within_the_data_folder);
    for state in [CLAIMED, PENDING, RECOVERING, DONE, FAILED] {
        state_directory(work.as_path(), state);
    }
    return work;
}

/// What:
///  A record open in a queue of a repository is waited for,
///  whichever of the two data-folder spellings holds that queue,
///  including a repository that holds both spellings at once.
/// Why:
///   A repository can hold both spellings at once while the folder is folded or after a process was killed during a move,
/// and the wrapper's own reader of those folders says reading the first alone hides the other
/// (`foundation/kept_names_in_the_git_directories.rs`,
/// `relative_paths_for_reading`).
/// Until 2026-11-16 this wait resolved one folder,
/// so a record open under the earlier name was never listed
/// (review of 2026-11-16,
/// `current_folder_selected_while_earlier_queue_is_open.rs`).
/// What this control proves:
/// every combination of the two spellings waits for a record open in any queue it has,
/// and settles when nothing is open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const spellings of [[current], [earlier], [current, earlier]]) for (const spelling of spellings) { emptyQueues(spellings); put(recordIn(spelling)); expectWaits(); }
/// ```
#[test]
fn a_queue_under_either_name_is_waited_for_when_both_exist() {
    let fixture: Fixture = fixture("settle-both-data-folders");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let current: &str = git_policy_cli::wrapper_names_old_and_new::DATA_FOLDER.current;
    let earlier: &str = git_policy_cli::wrapper_names_old_and_new::DATA_FOLDER.earlier;
    let both: [&str; 2] = [current, earlier];
    let combinations: [&[&str]; 3] = [&[current], &[earlier], &both];
    for spellings in combinations {
        for data_folder in both {
            let _gone = std::fs::remove_dir_all(repo.join(".git").join(data_folder).as_path());
        }
        let works: Vec<PathBuf> = spellings.iter().map(|data_folder: &&str| { return an_empty_queue(repo.as_path(), data_folder); }).collect();
        // Nothing open settles: the wait has nothing to wait for whichever spellings the repository holds.
        Waiting::start(repo.as_path()).join();
        for work in works {
            let record: PathBuf = work.join(PENDING).join(A_QUEUED_RECORD);
            std::fs::write(record.as_path(), b"{\"kind\":\"push\"}\n").expect("an open record in this queue");
            let waiting: Waiting = Waiting::start(repo.as_path());
            std::thread::sleep(Duration::from_millis(LOOK_SECONDS_AS_MILLIS));
            let returned_in_time: bool = waiting.returned();
            std::fs::remove_file(record.as_path()).expect("the record was taken away");
            waiting.join();
            assert!(!returned_in_time, "the wait answered settled while {} held a record", work.display());
        }
    }
    remove(&fixture);
}
