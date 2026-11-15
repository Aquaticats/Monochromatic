//! What:
//!  The instant before a removal's thread starts deleting,
//!  which a control fills with whatever the finding needs:
//!  a claim taken in that instant, or a writer that goes on after the caller stopped waiting.
//! Why:
//!  `remove_within` asks for its removal on a thread and stops waiting at the ceiling,
//!  so a control of what happens to the tree afterwards has to hold that thread at one known point:
//!  this seam holds it before the first entry is deleted, until the control releases it.
//!  A seam of the test build only (`rust/tests-in-sidecars`), so no release executable carries it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test builds only
//! export const removalPoint = { hold: () => release, wait: () => void }
//! ```
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// Whether the next removal waits before it deletes.
static HELD: AtomicBool = AtomicBool::new(false);

/// The release of the removal that is waiting, once one is.
static WAITING: Mutex<Option<Receiver<()>>> = Mutex::new(None);

/// How long the waiting removal waits when a control never releases it,
/// so a control that fails cannot leave a thread blocked for the whole run.
const CONTROL_PATIENCE: Duration = Duration::from_secs(5);

/// What:
///  Holds the next removal's thread before it deletes anything,
///  and answers the value that releases it.
/// Why:
///  The caller of a removal stops waiting at its ceiling while the thread goes on;
///  a control of what happens next needs the deletion to be after its own writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const release = holdTheNextRemoval(); // the next removal waits until release.send(undefined)
/// ```
pub(crate) fn hold_the_next_removal_for_a_control() -> Sender<()> {
    let (release, waited): (Sender<()>, Receiver<()>) = channel();
    *WAITING.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(waited);
    HELD.store(true, Ordering::SeqCst);
    return release;
}

/// What:
///  Waits where a control holds this removal, and returns at once when none does.
/// Why:
///  It is called by the removal's own thread, before the first entry goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitHereWhenAControlHoldsTheRemoval()
/// ```
pub(crate) fn wait_before_the_deletion() {
    if !HELD.swap(false, Ordering::SeqCst) {
        return;
    }
    let waited: Option<Receiver<()>> = WAITING.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
    if let Some(waited) = waited {
        let _released = waited.recv_timeout(CONTROL_PATIENCE);
    }
}
