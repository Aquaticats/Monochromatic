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
//! A claim is one atomic flag taken by compare-and-swap and
//!  given back when its guard drops,
//! a panic included,
//! so no thread that dies holding one leaves a name closed.
//! It is not
//!  lock-free for a thread that finds the flag taken:
//! that thread waits for the holder (the one wait this design needs,
//! so that
//!  the loser can read the winner's file whole afterwards;
//! the lock-inventory guard `source_guard/service_lock_inventory_tests.rs`
//!  names mutexes,
//! read-write locks,
//! condition variables and `flock`,
//! and this is none of them).
//! Two different names that hash
//!  to one slot wait for each other for the few microseconds a check and a rename take;
//! that is slower for that pair and never
//!  wrong.
//!  A wait is bounded ("nothing should ever wait forever"):
//! past the bound the caller gets an error and nothing was published,
//!  which is the one answer that cannot let two threads win.
//! The wait offers its turn a few times and then sleeps briefly between
//!  tries,
//! because a claim lasts a check and a rename,
//! so the wait is over almost at once unless a rename stalls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Single-threaded Node needs no claim; this is a compare-and-swap on a shared flag, as `Atomics.compareExchange` would be.
//! const held = holdNameWhilePublishing(path, waitAtMost); try { checkThenRename(); } finally { held.release(); }
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
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

/// The claims,
/// one flag per slot:
/// `true` while a thread holds a name that hashes to the slot.
static HELD: [AtomicBool; SLOT_COUNT] = [const { AtomicBool::new(false) }; SLOT_COUNT];

/// How many times a waiting thread offers its turn to others before it starts to sleep between tries:
/// a claim lasts a check and a
///  rename,
/// so the holder is usually done after a few turns.
const TURNS_BEFORE_SLEEPING: u32 = 64;

/// How long a waiting thread sleeps between tries once offering its turn did not help:
/// short against a disk that stalls a rename,
///  long enough that a waiting thread does not burn a processor while the holder is stuck.
const SLEEP_BETWEEN_TRIES: Duration = Duration::from_millis(1);

/// A name held by the thread that owns this value;
/// the claim ends when the value drops.
pub struct HeldName {
    /// The flag of the claim.
    slot: &'static AtomicBool,
}

impl Drop for HeldName {
    /// Gives the claim back,
    /// on every way out of the publish,
    /// a panic included.
    fn drop(&mut self) {
        self.slot.store(false, Ordering::Release);
    }
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
/// while (!compareExchange(slot(path), false, true)) { if (elapsed() >= waitAtMost) throw timedOut(path); await turnOrShortSleep(); }
/// ```
pub fn hold_name_while_publishing(path: &Path, wait_at_most: Duration) -> std::io::Result<HeldName> {
    let slot: &'static AtomicBool = &HELD[slot_of(path)];
    let started: Instant = Instant::now();
    let mut turns: u32 = 0;
    loop {
        if slot.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            return Ok(HeldName { slot });
        }
        if started.elapsed() >= wait_at_most {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "another thread of this program has been publishing {} (or a name that shares its claim) for {} ms, so this one published nothing",
                    path.display(),
                    wait_at_most.as_millis()
                ),
            ));
        }
        if turns < TURNS_BEFORE_SLEEPING {
            turns += 1;
            std::thread::yield_now();
        } else {
            std::thread::sleep(SLEEP_BETWEEN_TRIES);
        }
    }
}

/// The controls stay out of the release executable.
#[cfg(test)]
#[path = "name_held_while_one_thread_publishes_it_tests.rs"]
mod tests;
