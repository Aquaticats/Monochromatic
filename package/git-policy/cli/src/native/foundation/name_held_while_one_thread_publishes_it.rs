//! What:
//!  A claim,
//! kept in memory and taken with one compare-and-swap,
//! on a file name that a thread of this process is publishing by the
//!  checked rename;
//! a second thread that wants the same name waits for the claim to end,
//! for a bounded time.
//! Why:
//!  The checked rename (`private_storage.rs`,
//! `publish_by_checked_rename`) is a check that the name is free and then a rename,
//! and
//!  a filesystem that has neither `renameat2` with `RENAME_NOREPLACE` nor hard links (an S3 bucket through `s3fs-fuse`) gives
//!  nothing better.
//! Two threads of one service,
//! the landing thread and a connection thread deciding one request,
//! could both
//!  pass the check and both rename,
//! each then believing it made the file.
//! The owner kept that fallback for two programs on such a
//!  disk ("Keep the fallback (Recommended)",
//! 2026-11-07),
//! and the coordinator settled the same-process case from the owner's
//!  rules ("Use as few locks as possible.",
//! an atomic claim first;
//! `doc/planning/git-policy-cli-full-tokio.md`,
//! "Safe file
//!  saving:
//! the publish replaced",
//! "Settled by the coordinator",
//! item 1):
//! the race is inside one process,
//! so the process can
//!  settle it by itself,
//! in memory and without a file.
//! Two programs on such a disk can still overwrite each other;
//! nothing
//!  in memory reaches them.
//!  The claims are slots of a fixed table picked by a hash of the path.
//! A claim is one atomic word taken by compare-and-swap and
//!  given back when its guard drops,
//! a panic included,
//! so no thread that dies holding one leaves a name closed.
//! It is not
//!  lock-free for a thread that finds the word taken:
//! that thread sleeps on the word until the holder gives the name back and
//!  wakes it (a futex wait on Linux,
//! so no timer,
//! no polling and no `thread::sleep`),
//! the one wait this design needs,
//! so that
//!  the loser can read the winner's file whole afterwards;
//! the lock-inventory guard `source_guard/service_lock_inventory_tests.rs`
//!  names mutexes,
//! read-write locks,
//! condition variables and `flock`,
//! and this is none of them.
//! Two different names that hash
//!  to one slot wait for each other for the few microseconds a check and a rename take;
//! that is slower for that pair and never
//!  wrong.
//!  A wait is bounded ("nothing should ever wait forever"):
//! past the bound the caller gets an error and nothing was published,
//!  which is the one answer that cannot let two threads win.
//! The sleep on the word carries the time that is left of the bound,
//!  so the bound holds with no one waking the sleeper.
//! Where the platform has no futex (macOS and Windows,
//! which the owner said to
//!  ignore until told otherwise) the sleeper parks for a short slice and looks at the word again,
//! still inside the bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Single-threaded Node needs no claim; this is a compare-and-swap on a shared flag, as `Atomics.compareExchange` would be.
//! const held = holdNameWhilePublishing(path, waitAtMost); try { checkThenRename(); } finally { held.release(); }
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// How many names can be claimed independently:
/// a power of two far above the threads that publish at once (the service answers
///  bursts of 128 clients,
/// and each decision holds its claim for a check and a rename),
/// so two names seldom share a slot.
/// No
///  measurement picked it;
/// it only has to exceed the threads a service has at once,
/// and a shared slot costs a wait,
/// not a
///  wrong answer.
const SLOT_COUNT: usize = 256;

/// The word of a slot when no thread holds a name that hashes to it.
const FREE: u32 = 0;

/// The word of a slot while one thread holds a name and no other thread is known to be waiting.
const HELD: u32 = 1;

/// The word of a slot while one thread holds a name and another thread has gone to sleep on the word,
/// or is about to;
/// the holder's
///  release then wakes one sleeper.
const HELD_WITH_A_SLEEPER: u32 = 2;

/// The claims,
/// one word per slot,
/// `FREE`,
/// `HELD` or `HELD_WITH_A_SLEEPER`.
static CLAIMS: [AtomicU32; SLOT_COUNT] = [const { AtomicU32::new(FREE) }; SLOT_COUNT];

/// How long a thread parks before it looks at the word again where the platform has no futex.
#[cfg(not(target_os = "linux"))]
const PARK_SLICE: Duration = Duration::from_millis(1);

/// A name held by the thread that owns this value;
/// the claim ends when the value drops.
pub struct HeldName {
    /// The word of the claim.
    slot: &'static AtomicU32,
}

impl Drop for HeldName {
    /// Gives the claim back,
    /// on every way out of the publish,
    /// a panic included,
    /// and wakes one sleeper if any went to sleep.
    fn drop(&mut self) {
        if self.slot.swap(FREE, Ordering::Release) == HELD_WITH_A_SLEEPER {
            wake_one_sleeper(self.slot);
        }
    }
}

/// What:
///  Wakes one thread that sleeps on `slot`,
/// if there is one.
/// Why:
///  The holder that gives the name back is the only one who knows the name is free;
/// the sleeper looks at the word again when woken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Atomics.notify(words, index, 1);
/// ```
#[cfg(target_os = "linux")]
fn wake_one_sleeper(slot: &AtomicU32) {
    // No sleeper may be there any more (it timed out); a wake of nobody is harmless, and a failed wake leaves the sleeper to its
    //  own bound, which ends it.
    let _woken: std::io::Result<usize> = rustix::thread::futex::wake(slot, rustix::thread::futex::Flags::PRIVATE, 1).map_err(std::io::Error::from);
}

/// Where the platform has no futex the sleepers look at the word on their own;
/// nobody is woken.
#[cfg(not(target_os = "linux"))]
fn wake_one_sleeper(_slot: &AtomicU32) {}

/// What:
///  Sleeps while `slot` holds `HELD_WITH_A_SLEEPER`,
/// for at most `at_most`.
/// Why:
///  A wake,
/// the word changing before the sleep,
/// a signal and the bound all return alike;
/// the caller looks at the word again and at
///  the clock,
/// so none of them needs telling apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Atomics.wait(words, index, HELD_WITH_A_SLEEPER, atMostMs);
/// ```
#[cfg(target_os = "linux")]
fn sleep_while_a_sleeper_is_marked(slot: &AtomicU32, at_most: Duration) {
    let remaining: rustix::thread::futex::Timespec = rustix::thread::futex::Timespec {
        tv_sec: i64::try_from(at_most.as_secs()).unwrap_or(i64::MAX),
        tv_nsec: i64::from(at_most.subsec_nanos()),
    };
    // `EAGAIN` (the word changed), `EINTR` and `ETIMEDOUT` are all answered by the caller's next look.
    let _slept: std::io::Result<()> =
        rustix::thread::futex::wait(slot, rustix::thread::futex::Flags::PRIVATE, HELD_WITH_A_SLEEPER, Some(&remaining)).map_err(std::io::Error::from);
}

/// Where the platform has no futex the sleeper parks for a short slice (never past the bound) and looks again.
#[cfg(not(target_os = "linux"))]
fn sleep_while_a_sleeper_is_marked(_slot: &AtomicU32, at_most: Duration) {
    std::thread::park_timeout(at_most.min(PARK_SLICE));
}

/// What:
///  The slot a path's claim lives in.
/// Why:
///   The same path always lands in the same slot,
/// so two threads that want one name meet at one flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hash(path) % SLOT_COUNT
/// ```
fn slot_of(path: &Path) -> usize {
    let mut hasher: DefaultHasher = DefaultHasher::new();
    path.hash(&mut hasher);
    return usize::try_from(hasher.finish() % u64::try_from(SLOT_COUNT).unwrap_or(1)).unwrap_or(0);
}

/// What:
///  Hold the name `path` against every other thread of this process that publishes it by the checked rename,
/// waiting at most
///  `wait_at_most` for a holder that is there already.
/// Why:
///   The check and the rename that follow are then one step to any other thread of the process.
/// The error is `TimedOut` and means
///  nothing was claimed,
/// so the caller publishes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (compareExchange(slot(path), FREE, HELD)) return held;
/// while (true) { if (elapsed() >= waitAtMost) throw timedOut(path); if (exchange(slot(path), SLEEPER) === FREE) return held; wait(slot, SLEEPER, waitAtMost - elapsed()); }
/// ```
pub fn hold_name_while_publishing(path: &Path, wait_at_most: Duration) -> std::io::Result<HeldName> {
    let slot: &'static AtomicU32 = &CLAIMS[slot_of(path)];
    let started: Instant = Instant::now();
    if slot.compare_exchange(FREE, HELD, Ordering::Acquire, Ordering::Relaxed).is_ok() {
        return Ok(HeldName { slot });
    }
    loop {
        let waited: Duration = started.elapsed();
        if waited >= wait_at_most {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "another thread of this program has been publishing {} (or a name that shares its claim) for {} ms, so this one published nothing",
                    path.display(),
                    wait_at_most.as_millis()
                ),
            ));
        }
        // Say that a thread sleeps, so the holder's release wakes one. A thread that finds the name free here takes it with the word
        //  still saying a sleeper may exist, and never with `HELD`: other sleepers may be there, and the release of the name it takes
        //  must wake the next of them or that one sleeps out its whole bound. A release that wakes nobody is harmless.
        if slot.swap(HELD_WITH_A_SLEEPER, Ordering::Acquire) == FREE {
            return Ok(HeldName { slot });
        }
        sleep_while_a_sleeper_is_marked(slot, wait_at_most - waited);
    }
}

/// The controls stay out of the release executable.
#[cfg(test)]
#[path = "name_held_while_one_thread_publishes_it_tests.rs"]
mod tests;
