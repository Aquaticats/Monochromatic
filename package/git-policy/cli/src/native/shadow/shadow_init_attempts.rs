//! What:
//!  How many times one thread's shadow creations started `init`,
//!  counted for the controls only.
//! Why:
//!  A control that counts the starts of the shadow's `init` needs a count the wrapper makes itself:
//!  the program a start runs may be ended at its ceiling before it writes anything.
//!  The count is per thread,
//!  so a control counts the creation it makes and no other test's
//!  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
//!  cause M).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // noteInitAttempt() at each attempt; initAttempts() and resetInitAttempts() in the controls.
//! ```

#[cfg(test)]
thread_local! {
    /// How many `init` starts this thread's shadow creations have made,
    /// counted by the controls only.
    /// It is per thread,
    /// so a control that counts the attempts of its own creation
    ///  is not disturbed by another test's creation.
    static INIT_ATTEMPTS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// What:
///  One attempt at the shadow's `init`,
/// counted for the controls only.
/// Why:
///   The count a control reads from here is the attempt the wrapper made,
/// which exists whether or not the program it starts writes anything
///  before the ceiling ends it.
///  A file the started program writes cannot promise that:
/// the program is ended at the ceiling wherever it has got to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteInitAttempt()
/// ```
pub(crate) fn note_init_attempt() {
    #[cfg(test)]
    INIT_ATTEMPTS.with(|counter| { counter.set(counter.get() + 1); });
}

/// What:
///  How many `init` attempts this thread's shadow creations have made since they were last reset.
/// Why:
///   A control reads it to count the attempts the wrapper made,
/// and never what a program the wrapper started got to write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// initAttempts()
/// ```
#[cfg(test)]
pub(crate) fn init_attempts() -> u32 {
    return INIT_ATTEMPTS.with(std::cell::Cell::get);
}

/// What:
///  Forgets the `init` attempts made so far on this thread.
/// Why:
///   A control starts counting at the creation it makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetInitAttempts()
/// ```
#[cfg(test)]
pub(crate) fn reset_init_attempts() {
    INIT_ATTEMPTS.with(|counter| { counter.set(0); });
}
