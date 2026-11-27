//! One pass over the process list:
//! where it starts,
//! what it forgets,
//! and when it stops early.

/// The engine whose pass this is,
/// and the state of a process whose move failed.
use super::{
    Failure,
    UpdateRouting,
};
/// The slice whose origins are followed on every pass.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// The processes of this host.
use crate::process_info::list_pids;
/// The clock and the stop check of a pass.
use crate::routing_support::{
    Clock,
    StopCheck,
};
/// The clock and the stop check of the test-only pass.
#[cfg(test)]
use crate::routing_support::{
    NeverStop,
    SystemClock,
};
/// What a pass reports.
use crate::update_report::Outcome;
/// The processes already handled and the ones whose move failed.
use std::collections::{
    HashMap,
    HashSet,
};
/// The instants of a pass.
use std::time::Instant;

/// The steps of one pass over the process list.
impl UpdateRouting {
    /// Starts a pass now:
    /// failed moves spent earlier no longer count against the next ones.
    pub fn begin_pass(&mut self) {
        self.begin_pass_at(Instant::now());
    }

    /// Starts a pass at `now`.
    pub fn begin_pass_at(
        &mut self,
        now: Instant,
    ) {
        self.gate.begin(now);
    }

    /// Forgets the handled and failed processes whose pids are no longer listed.
    fn forget_exited(
        &mut self,
        listed: &[i32],
    ) {
        let alive: HashSet<i32> = listed.iter().copied().collect();
        let mut kept_handled: HashSet<(i32, u64)> = HashSet::new();
        for key in &self.handled {
            if alive.contains(&key.0) {
                kept_handled.insert(*key);
            }
        }
        self.handled = kept_handled;
        let mut kept_failures: HashMap<(i32, u64), Failure> = HashMap::new();
        for (key, failure) in self.failures.drain() {
            if alive.contains(&key.0) {
                kept_failures.insert(key, failure);
            }
        }
        self.failures = kept_failures;
        let mut kept_vanishes: HashMap<(i32, u64), u32> = HashMap::new();
        for (key, count) in self.member_vanishes.drain() {
            if alive.contains(&key.0) {
                kept_vanishes.insert(key, count);
            }
        }
        self.member_vanishes = kept_vanishes;
    }

    /// Considers every live process now and forgets handled processes that no longer exist.
    #[cfg(test)]
    pub fn rescan(&mut self) -> Vec<Outcome> {
        return self.rescan_with(&SystemClock, &NeverStop);
    }

    /// Considers every live process,
    /// reading the time from `clock` before each one,
    /// and stops early when `stop` asks to,
    /// so a shutdown signal is not left waiting behind a long pass.
    /// Processes that need no report are left out of the result.
    pub fn rescan_with(
        &mut self,
        clock: &dyn Clock,
        stop: &dyn StopCheck,
    ) -> Vec<Outcome> {
        self.begin_pass_at(clock.now());
        let pids = match list_pids(&self.setup.proc_root) {
            Ok(pids) => pids,
            Err(error) => {
                return vec![Outcome::ScanFailed {
                    error: error.to_string(),
                }];
            }
        };
        let mut outcomes: Vec<Outcome> = Vec::new();
        for pid in &pids {
            if stop.stop_requested() {
                return outcomes;
            }
            let outcome = self.consider_at(*pid, clock.now());
            if outcome != Outcome::Ignored {
                outcomes.push(outcome);
            }
        }
        self.forget_exited(&pids);
        // Following the origins asks the manager too: not while its calls are paused,
        // for at most a budget, and a call that times out pauses the moves.
        if !self.gate.is_blocked(clock.now()) {
            let followed = self
                .scopes
                .follow_origins(&self.setup, TOOLCHAIN_UPDATES_SLICE, clock);
            if followed.timed_out {
                self.gate.block(clock.now());
            }
            outcomes.extend(followed.outcomes);
        }
        return outcomes;
    }
}
