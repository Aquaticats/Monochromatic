//! What:
//!  The wait a caller makes for a copy another caller is writing:
//!  the stored name it looks for,
//!  the copies in progress it reads,
//!  and the three ways the wait can end.
//! Why:
//!  A wave of controls that start together on a fresh store otherwise writes one copy of the whole executable
//!  each,
//!  and that write-back stalls the bounded commands of the controls around it
//!  (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`,
//!  "The first wave on a fresh store");
//!  the wait is its own job, with its own clock a control can move,
//!  so it lives in a file of its own beside the store it looks at.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = waitForPublisher(store, stored); // 'published' | 'nobody-copying' | 'cap-reached'
//! ```

use std::path::Path;
use std::time::Duration;

/// Prefix of a copy in progress;
/// a finished copy is linked to its final name,
/// so a name with this prefix is never used.
pub const PARTIAL_PREFIX: &str = ".partial-";

/// How long a partial copy may go without being written before a caller stops waiting for it and copies for itself:
///  the time a live publisher needs to read its copy back and link it,
/// plus a stall of the volume.
/// A partial copy that a
///  killed run left behind is older than this the next time anyone looks,
/// so it is never waited for.
pub const PROGRESS_WINDOW: Duration = Duration::from_secs(10);

/// The longest a caller waits for another caller's copy in all,
/// however recently that copy was written.
pub const WAIT_CAP: Duration = Duration::from_secs(120);

/// How often a waiting caller looks for the stored copy.
pub const WAIT_POLL: Duration = Duration::from_millis(25);

/// How a wait for another caller's copy ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitEnd {
    /// The stored copy exists.
    Published,
    /// No partial copy was written to within the progress window:
    /// nobody is copying,
    /// so the caller copies.
    NobodyCopying,
    /// A copy kept being written to until the cap:
    /// the caller stops waiting and copies.
    CapReached,
}

/// What the wait asks of time:
/// what it is,
/// and a pause of one poll.
/// The production clock is the system's;
/// a control
/// passes one that moves only when the wait pauses,
/// so the wait's decisions are tested without a wall clock.
pub trait WaitClock {
    /// The time now.
    fn now(&self) -> std::time::SystemTime;
    /// Wait for `poll`.
    fn pause(
        &self,
        poll: Duration,
    );
}

/// The system's clock and sleep.
struct SystemClock;

/// The wait on the system's time.
impl WaitClock for SystemClock {
    /// The system time.
    fn now(&self) -> std::time::SystemTime {
        return std::time::SystemTime::now();
    }

    /// A thread sleep.
    fn pause(
        &self,
        poll: Duration,
    ) {
        std::thread::sleep(poll);
    }
}

/// What:
///  Whether any partial copy in `store` was written to within the `window` before `now`:
/// a publisher is at work.
/// Why:
///  A partial file's modification time moves with every write,
/// so a live copy always has a recent one,
/// and a copy that a
///  killed run left behind does not.
/// A live copy has no second name until it is published,
/// which keeps a leftover that
///  aliases a published file (`reserve_partial`) from counting.
/// A time in the future (a clock that moved back) counts as
///  recent,
/// and the wait it causes ends at the cap.
fn copy_in_progress(
    store: &Path,
    window: Duration,
    now: std::time::SystemTime,
) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir(store) else {
        return false;
    };
    return entries.flatten().any(|entry| {
        let Ok(metadata) = entry.metadata() else {
            return false;
        };
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with(PARTIAL_PREFIX)
            || metadata.nlink() != 1
        {
            return false;
        }
        return match metadata.modified().map(|modified| {
            return now.duration_since(modified);
        }) {
            Ok(Ok(age)) => age < window,
            Ok(Err(_)) => true,
            Err(_) => false,
        };
    });
}

/// What:
///  Wait for another caller's copy to be published,
/// on the system's clock with the production limits.
/// Why:
///  See `wait_for_publisher_on`.
pub fn wait_for_publisher(
    store: &Path,
    stored: &Path,
) -> WaitEnd {
    return wait_for_publisher_on(
        store,
        stored,
        [PROGRESS_WINDOW, WAIT_POLL, WAIT_CAP],
        &SystemClock,
    );
}

/// What:
///  Wait for another caller's copy to be published:
/// `Published` once `stored` exists,
/// `NobodyCopying` when no partial copy
///  was written to within the window,
/// `CapReached` after the cap.
/// `limits` is the window,
/// the poll and the cap,
/// in that order.
/// Why:
///  Copying the build is the only expensive step of a fixture's creation,
/// and callers that start together on a fresh
///  store all did it,
/// each writing the whole executable to the volume the bounded commands of the same wave sync to.
///  The wait asks only for the stored name and for recent writes,
/// so it holds nothing another caller needs;
/// a caller
///  that gives up simply copies,
/// as it always did,
/// and the link of the final name still decides the winner.
/// Time comes
///  from `clock`,
/// so the end of the wait is a decision a control reads from the result.
pub fn wait_for_publisher_on(
    store: &Path,
    stored: &Path,
    limits: [Duration; 3],
    clock: &dyn WaitClock,
) -> WaitEnd {
    let [window, poll, cap] = limits;
    let began: std::time::SystemTime = clock.now();
    loop {
        if stored.exists() {
            return WaitEnd::Published;
        }
        let now: std::time::SystemTime = clock.now();
        if !copy_in_progress(store, window, now) {
            return WaitEnd::NobodyCopying;
        }
        if now.duration_since(began).is_ok_and(|waited| {
            return waited >= cap;
        }) {
            return WaitEnd::CapReached;
        }
        clock.pause(poll);
    }
}
