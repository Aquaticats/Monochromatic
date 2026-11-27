//! What:
//!  The bounds one real-Git query of a control is held to:
//!  how long the query may run before this helper ends it,
//!  how long each of its pipes is given to reach its end,
//!  how long a reader may take to hand over what it read,
//!  how often the wait looks whether the query ended,
//!  and how long a query this helper killed may take to be reaped.
//! Why:
//!  Every number here is the promise a control's answer rests on,
//!  so each one stands beside the measurement or the bound of this crate that sizes it,
//!  and a change to one is weighed against that reason rather than guessed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const FIXTURE_GIT_BOUND = 60_000;   // ms, one query of a control
//! export const PIPE_GRACE = 250;             // ms, a pipe older than the query getting its end
//! export const REPORTING_THE_END = 60_000;   // ms, a reader handing over what it read
//! const LOOK_EVERY = 1;                      // ms, one look at the query
//! const REAPING = 5_000;                     // ms, a killed query being reaped
//! ```

/// How long one real-Git query of a control may run before this helper ends it.
/// Why sixty:
///  The same bound the binary-level controls give one fixture Git run
/// (`NATIVE_GIT_BOUND_SECONDS`),
/// and for the same measurement:
/// the longest disk stall one `fsync` of a trivial command showed on this host is 4.74 s
/// (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
/// so sixty is more than ten times it.
pub(crate) const FIXTURE_GIT_BOUND: std::time::Duration = std::time::Duration::from_secs(60);

/// How long the answer gives one of the query's pipes to reach its end once the query is gone,
/// before it says that something outside the query still holds that pipe.
/// Why 250 ms:
///  the query's own process is gone when this window opens,
/// and the kernel reports a pipe's end the instant its last writing end closes,
/// so this window is spent only by a holder that is not the query:
/// a descendant that inherited the pipe,
/// or a program that was forked with it open.
///  The window is asked of the pipe itself (`has_ended_within`) and not of a thread of this process,
/// so a host that wakes such a thread late cannot spend it:
/// measured 2026-11-27 on this host at a load average of 86.0 to 142.5,
/// the thread that read a query's standard output handed it over up to 110.4 ms after the pipe's end,
/// 2.3 times inside this window,
/// which is how close that window came to failing for a reason that was never about the query.
pub(crate) const PIPE_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

/// How long the reader of one of the query's pipes may take to hand over what it read,
/// once the kernel says that pipe has no writing end left.
/// Why sixty seconds:
///  this bound only stops a hang:
/// the kernel has already said that every byte the query wrote is in the pipe,
/// so what is left is one thread's wakes,
/// which no honest host holds for a minute.
///  It is 543 times the worst hand-over measured on this host at a load average of 86.0 to 142.5
/// (110.4 ms),
/// and the same figure this crate's run watchdog allows a working run to show nothing at all
/// (`STILL_WORKING_WINDOW`,
/// which rests on a measured 8.0 s of silence).
pub(crate) const REPORTING_THE_END: std::time::Duration = std::time::Duration::from_secs(60);

/// How often the wait looks whether the query ended.
/// Why one millisecond:
///  this helper owns the wait,
/// so no second thread may collect the query while the bound's signal is on its way;
/// the standard library has no timed wait,
/// so the wait looks,
/// and a query that ends in a millisecond costs about a millisecond.
///  The typed-value sweep,
/// which makes thousands of these calls,
/// measured 143 s with a ten-millisecond look and 25 s
/// with a wait in another thread that this replaces.
pub(super) const LOOK_EVERY: std::time::Duration = std::time::Duration::from_millis(1);

/// How long a query this helper has killed may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once,
/// so this window only covers the kernel and the scheduler;
/// a query that still has not been reaped in it is a defect of this helper,
/// not a slow host.
pub(super) const REAPING: std::time::Duration = std::time::Duration::from_secs(5);
