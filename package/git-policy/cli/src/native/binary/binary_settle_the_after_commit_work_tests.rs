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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleServiceWork(repo); // waits while the queue holds a record, and never answers from a listing it could not make
//! ```

use super::support::{Fixture, fixture, remove, repository};
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
