//! What:
//!  The collected elements a pass over unbounded input has looked at on this thread,
//!  counted for the controls and not in the release executable.
//! Why:
//!  `AGENTS.md` `RG2` asks for one linear pass over unbounded input.
//!  The passes that decide "was this already collected" either ask a set,
//!  which looks at one collected element per question,
//!  or scan the collected list,
//!  which looks at every element collected before it.
//!  A control that weighed clock time would report how busy the host is (`AGENTS.md`,
//! `NQH`);
//!  this count reports the shape of the pass instead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const lookedAt = new AsyncLocalStorage<number>(); function noteLookedAt(count: number): void;
//! ```

#[cfg(test)]
thread_local! {
    /// The count of the passes on this thread,
    /// so two controls running beside each other cannot read each other's work.
    static ELEMENTS_LOOKED_AT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What:
///  Count the collected elements one question of a pass looked at.
/// Why:
///   A set answers a question by comparing the candidate with the one element it names;
///  the scan it replaces compares the candidate with every element collected before it,
///  so a control states the work a pass does over its own collected list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteLookedAt(count: number): void
/// ```
#[cfg(test)]
pub(crate) fn note_looked_at(count: u64) {
    ELEMENTS_LOOKED_AT.with(|counted| counted.set(counted.get() + count));
}

/// The release executable counts nothing.
#[cfg(not(test))]
pub(crate) fn note_looked_at(_count: u64) {}

/// The collected elements this thread's passes have looked at so far.
#[cfg(test)]
pub(crate) fn looked_at() -> u64 {
    return ELEMENTS_LOOKED_AT.with(|counted| return counted.get());
}

/// The counting seam's own controls.
#[cfg(test)]
#[path = "collected_elements_looked_at_tests.rs"]
mod tests;
