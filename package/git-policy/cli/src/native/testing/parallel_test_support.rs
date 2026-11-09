//! What:
//!  A point where several threads must all arrive,
//! for controls that prove work really overlapped.
//! Why:
//!  A bare barrier hangs a run that is serial;
//! this one gives up after a stated patience and says so,
//! so a serial run fails
//!  the control instead of stalling it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const point = new Rendezvous(4, 5_000);
//! const everyoneCame = await point.arrive();
//! ```

use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// How long an arrival waits for the others when the control expects them to come.
pub(crate) const PATIENCE: Duration = Duration::from_secs(10);

/// How long the control for a serial run waits at a point it can never pass.
pub(crate) const SERIAL_PATIENCE: Duration = Duration::from_millis(200);

/// A point where a number of threads must all arrive:
/// it can be passed only if they overlap in time.
pub(crate) struct Rendezvous {
    /// How many threads have arrived.
    arrived: Mutex<usize>,
    /// Woken when one arrives.
    changed: Condvar,
    /// How many must arrive.
    target: usize,
    /// How long an arrival waits for the others.
    patience: Duration,
}

impl Rendezvous {
    /// A point for `target` threads.
    pub(crate) fn new(target: usize) -> Rendezvous {
        return Rendezvous { arrived: Mutex::new(0), changed: Condvar::new(), target, patience: PATIENCE };
    }

    /// A point for `target` threads whose arrivals give up after `patience`.
    pub(crate) fn impatient(target: usize, patience: Duration) -> Rendezvous {
        return Rendezvous { patience, ..Rendezvous::new(target) };
    }

    /// Arrive and wait for the others;
    /// false when they did not all come within the patience,
    /// which is what a serial run sees.
    pub(crate) fn arrive(&self) -> bool {
        let mut count = self.arrived.lock().expect("lock");
        *count += 1;
        self.changed.notify_all();
        return self.wait_while_short(count);
    }

    /// What:
    ///  Arrive at a point that the work under test reaches with one place of its bound each,
    ///  and fail the control when the work never brought `bound` of its jobs together.
    /// Why:
    ///  A bound control claims that the overlap reaches the bound and no more.
    ///  A caller that throws the arrival's answer away holds the early jobs for the patience and cannot tell work that
    /// never reached the bound from work that did,
    /// so the places of such a run are given back on a clock.
    ///  The patience then bounds only how long a failing run waits, never whether the claim is witnessed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// point.arriveAtTheBound(allowed); // throws unless the bound was reached
    /// ```
    pub(crate) fn arrive_at_the_bound(&self, bound: usize) {
        assert!(
            self.arrive(),
            "the work under test never brought {bound} of its jobs together inside the same work, so the overlap this control measures against a bound of {bound} was not reached"
        );
    }

    /// Wait without arriving until the target number have arrived;
    /// false after the patience.
    pub(crate) fn wait_for_target(&self) -> bool {
        let count = self.arrived.lock().expect("lock");
        return self.wait_while_short(count);
    }

    /// Count an arrival without waiting for the others.
    pub(crate) fn arrive_without_waiting(&self) {
        let mut count = self.arrived.lock().expect("lock");
        *count += 1;
        self.changed.notify_all();
    }

    /// Wait on a held count until it reaches the target;
    /// false when the patience ran out first.
    fn wait_while_short(&self, count: std::sync::MutexGuard<'_, usize>) -> bool {
        let (guard, timeout) = self.changed.wait_timeout_while(count, self.patience, |seen: &mut usize| { return *seen < self.target; }).expect("wait");
        drop(guard);
        return !timeout.timed_out();
    }
}
