//! What:
//!  What ended one bounded query,
//!  as the last status the wait returned showed it,
//!  and the seam a control of that ending arms to put an ending in the instant before the bound's signal.
//! Why:
//!  A wait says which signal ended a child and never who sent it,
//!  so the answer keeps to what a status proves,
//!  and the window before the bound's signal is a few instructions wide,
//!  so no query can arrange to end in it from outside and a control needs a seam to place one there.
//!  Nothing of that seam exists in the release build.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type QueryEnding = 'itself' | 'at-the-bound';
//! // armTheEndingSeam((query) => { ... });   // a control only, in a build with controls
//! ```
/// What:
///  The seam a control of the ending uses:
///  it runs between the last look at the query and the signal that ends it.
/// Why:
///  That window is a few instructions wide,
/// so no query can arrange to end in it from outside,
/// and a control about which ending this helper reports needs a way to put an ending there.
///  Nothing of this seam exists in the release build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeTheSignal?.(query); // absent unless a control armed it
/// ```
#[cfg(test)]
pub(super) fn the_ending_seam(query: libc::pid_t) {
    if let Some(seam) = ENDING_SEAM.lock().expect("the ending seam").take() {
        seam(query);
    }
}

/// The one ending seam a control arms, for the one call it measures.
#[cfg(test)]
static ENDING_SEAM: std::sync::Mutex<Option<Box<dyn FnOnce(libc::pid_t) + Send>>> =
    std::sync::Mutex::new(None);

/// `the_ending_seam` in a build without controls: nothing runs.
#[cfg(not(test))]
pub(super) fn the_ending_seam(_query: libc::pid_t) {}

/// Arm the ending seam for one call of `real_git_within`.
#[cfg(test)]
pub(crate) fn arm_the_ending_seam(seam: impl FnOnce(libc::pid_t) + Send + 'static) {
    *ENDING_SEAM.lock().expect("the ending seam") = Some(Box::new(seam));
}

/// Disarm the ending seam, answering whether it was still armed.
/// Why:
///  a control that did not reach the seam must not leave it armed for another test.
#[cfg(test)]
pub(crate) fn disarm_the_ending_seam() -> bool {
    return ENDING_SEAM
        .lock()
        .expect("the ending seam")
        .take()
        .is_some();
}

/// What ended one bounded query, as the last status the wait returned showed it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryEnding {
    /// The last status the wait returned was not the signal this helper sends at its bound:
    /// the query ended before that bound ran out, or in the instant after it, and this answer does not say which.
    ///  Its status and output are the query's own, whatever status that was:
    /// a code of 124 or 137 or a signal's death included.
    ///  A signal another process sent reads the same here:
    /// a wait says which signal ended a child and nothing about who sent it.
    Itself,
    /// The bound ran out, and the last status the wait returned was the signal this helper sends at the bound
    /// to the process group the query led.
    ///  A query killed by that same signal from elsewhere in that instant reads the same,
    /// because a wait says which signal ended a child and nothing about who sent it.
    AtTheBound,
}
