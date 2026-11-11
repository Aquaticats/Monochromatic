//! What:
//!  A control's hold on the check that names which instance a Git directory belongs to.
//! Why:
//!  The check resolves a directory's real path on the blocking pool,
//!  and under a busy disk that resolution took thirty-two seconds
//!  (the record of 2026-11-10T17:50:19Z).
//!  A control cannot hand the connection task a future for a check the shipped code
//!  creates itself,
//!  so the hold sits inside the blocking job,
//!  which is the wait itself,
//!  and the control needs no clock to say "while the check waits".
//!  The hold is a file of its own so the shipped module keeps no test code
//!  (`rust/tests-in-sidecars`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdAwhile('/elsewhere/repo/gitdir'); // ... releaseAwhile(); while (held) await changed;
//! ```

use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// The longest a held check waits for its control:
/// far beyond any beat a control watches for,
/// and short enough that a release a control never makes cannot hold a blocking thread.
pub(crate) const LONGEST_HOLD: Duration = Duration::from_secs(30);

/// Whether a check is held,
/// and which directory's.
struct Hold {
    /// The directory whose check waits.
    directory: PathBuf,
    /// Whether the hold has been let go.
    released: bool,
}

/// The hold in force.
static HELD: Mutex<Hold> = Mutex::new(Hold { directory: PathBuf::new(), released: true });

/// What a release or a hold is announced on.
static CHANGED: Condvar = Condvar::new();

/// What:
///  Holds the check of `directory` still until `release` or `LONGEST_HOLD`.
/// Why:
///   The control arms the hold before it writes the request,
/// so the check that resolves the directory is the one that waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdAwhile(directory)
/// ```
pub(crate) fn hold(directory: &Path) {
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.directory = directory.to_path_buf();
    held.released = false;
    CHANGED.notify_all();
}

/// Lets a held check go on,
/// and every later check with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// releaseAwhile()
/// ```
pub(crate) fn release() {
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.released = true;
    CHANGED.notify_all();
}

/// What:
///  Waits while a control holds the check of `directory` still,
///  and goes on by itself at `LONGEST_HOLD`.
/// Why:
///   This runs inside the blocking job,
/// so a control that never releases costs that thread the bound and no more,
///  and every other check and every other test is unaffected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whileHeld(directory, LONGEST_HOLD)
/// ```
pub(crate) fn wait_while_held(directory: &Path) {
    let began: Instant = Instant::now();
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    while !held.released && held.directory == directory {
        let Some(left) = LONGEST_HOLD.checked_sub(began.elapsed()) else {
            return;
        };
        let (next, _waited) = CHANGED.wait_timeout(held, left).unwrap_or_else(PoisonError::into_inner);
        held = next;
    }
}
