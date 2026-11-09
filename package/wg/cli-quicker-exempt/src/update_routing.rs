//! The always-on policy that keeps update traffic outside the tunnel.
//! A process that is an update entry point,
//! or runs from the toolchain install roots,
//! is moved into the updates slice,
//! so its sockets and its later children's sockets receive the exemption mark.
//! The policy has no switch.
//! When part of it cannot be enforced,
//! the outcomes here carry a loud report naming the policy,
//! and nothing changes routing on its own.
//! A failed move is retried with a growing wait,
//! never forgotten and never repeated in a tight loop.

/// Slice and cgroup names,
/// and the cgroups that carry the mark themselves.
use crate::application_targets::{is_socket_marked_cgroup, TOOLCHAIN_UPDATES_SLICE};
/// The limits the scope carries.
use crate::inherited_limits::read_inherited_limits;
/// Process facts.
use crate::process_info::{list_pids, read_own_pid_namespace, read_process, read_start_time};
/// Collecting and checking the group of a matched process.
#[path = "routing_group.rs"]
mod routing_group;
/// The functions of that module the engine calls.
use routing_group::{collect_group, recheck_group};
/// The seam to the service manager.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter};
/// The scope of each origin.
use crate::scope_naming::{origin_scope_name, scope_cgroup_path};
/// What is known about the scopes in use.
use crate::scope_registry::ScopeRegistry;
/// Classification of processes.
use crate::update_entry_points::classify;
/// Clock,
/// stop check,
/// retry wait
/// and manager reach.
use crate::routing_support::{adoptable_cgroup, retry_backoff, Clock, StopCheck};
/// Clock and stop check for the test-only rescan.
#[cfg(test)]
use crate::routing_support::{NeverStop, SystemClock};
/// Outcomes and their wording.
use crate::update_report::Outcome;
/// Tracking of processes already handled.
use std::collections::{HashMap, HashSet};
/// Fake procfs roots in tests.
use std::path::PathBuf;
/// Retry timing.
use std::time::{Duration, Instant};

/// Failed moves one pass may spend before it postpones the rest,
/// so a manager that fails slowly cannot stall the watcher for long.
const FAILED_ATTEMPTS_PER_PASS: u32 = 2;
/// Refusals of "No such process" for a living process,
/// in a row,
/// before the policy reports them.
/// A group member that exits makes the manager refuse once;
/// a run of them is not a race any more.
pub const MEMBER_VANISH_LIMIT: u32 = 3;
/// Time one pass may spend on moves before it postpones the rest.
/// A pass can overrun it by the one attempt in progress,
/// which the client timeout bounds.
const PASS_MOVE_BUDGET: Duration = Duration::from_secs(2);
/// How long no move is attempted after the manager did not answer in time.
const BLOCK_AFTER_TIMEOUT: Duration = Duration::from_secs(10);

/// Does nothing,
/// the default step between collecting a group and checking it again.
fn nothing() {}

/// Everything the engine needs for one watched user.
pub struct RoutingSetup {
    /// Watched user.
    pub uid: u32,
    /// Primary group of the watched user.
    pub gid: u32,
    /// Toolchain install roots of that user.
    pub install_roots: Vec<PathBuf>,
    /// Procfs root.
    pub proc_root: PathBuf,
    /// Cgroup filesystem root,
    /// where the limits of the old cgroup are read.
    pub cgroup_root: PathBuf,
    /// Seam to the service manager.
    pub adopter: Box<dyn Adopter>,
}

/// The state of a process whose move failed.
struct Failure {
    /// Failed attempts so far.
    attempts: u32,
    /// Earliest time of the next attempt.
    retry_at: Instant,
}

/// The policy engine for one watched user.
pub struct UpdateRouting {
    /// What the engine was built with.
    setup: RoutingSetup,
    /// The watcher's own pid namespace,
    /// absent when the kernel does not expose it.
    own_pid_namespace: Option<String>,
    /// Processes already handled for good,
    /// by pid and start time.
    handled: HashSet<(i32, u64)>,
    /// The scope of each origin cgroup and the limits it was last given.
    scopes: ScopeRegistry,
    /// Processes whose move failed and will be tried again.
    failures: HashMap<(i32, u64), Failure>,
    /// Moves of a living process that the manager refused with "No such process" because a process of its group exited,
    /// in a row,
    /// by pid and start time.
    member_vanishes: HashMap<(i32, u64), u32>,
    /// Failed moves spent in the current pass.
    failed_attempts_this_pass: u32,
    /// No move is attempted before this time.
    blocked_until: Option<Instant>,
    /// When the current pass began.
    pass_started: Option<Instant>,
}

/// Behavior of the policy engine.
impl UpdateRouting {
    /// Builds the engine.
    pub fn new(setup: RoutingSetup) -> UpdateRouting {
        let own_pid_namespace = read_own_pid_namespace(&setup.proc_root);
        return UpdateRouting {
            setup,
            own_pid_namespace,
            handled: HashSet::new(),
            scopes: ScopeRegistry::new(),
            failures: HashMap::new(),
            member_vanishes: HashMap::new(),
            failed_attempts_this_pass: 0,
            blocked_until: None,
            pass_started: None,
        };
    }

    /// Reports whether the watcher could read its own pid namespace.
    /// Without it a process in a container or a sandbox cannot be told apart.
    pub fn pid_namespace_known(&self) -> bool {
        return self.own_pid_namespace.is_some();
    }

    /// Makes sure the updates slice exists below the user's `app.slice`,
    /// so the watcher can attach the mark to it before any process is moved.
    pub fn ensure_slice(&self, app_slice: &std::path::Path) -> std::io::Result<()> {
        return self.setup.adopter.ensure_slice(
            self.setup.uid,
            self.setup.gid,
            TOOLCHAIN_UPDATES_SLICE,
            &app_slice.join(TOOLCHAIN_UPDATES_SLICE),
        );
    }

    /// Starts a pass now:
    /// failed moves spent earlier no longer count against the next ones.
    pub fn begin_pass(&mut self) {
        self.begin_pass_at(Instant::now());
    }

    /// Starts a pass at `now`.
    pub fn begin_pass_at(&mut self, now: Instant) {
        self.failed_attempts_this_pass = 0;
        self.pass_started = Some(now);
    }

    /// Why a move must wait now,
    /// or `None` when it may go ahead.
    fn postponement(&self, now: Instant) -> Option<&'static str> {
        if self.is_blocked(now) {
            return Some("the user's service manager did not answer in time, so moves are paused for a few seconds");
        }
        if self.failed_attempts_this_pass >= FAILED_ATTEMPTS_PER_PASS {
            return Some("earlier moves in this pass failed, and a pass spends at most two failed attempts");
        }
        if let Some(started) = self.pass_started
            && now.duration_since(started) >= PASS_MOVE_BUDGET
        {
            return Some("this pass used its time budget for moves");
        }
        return None;
    }

    /// Reports whether moves are paused after a client timeout.
    fn is_blocked(&self, now: Instant) -> bool {
        return match self.blocked_until {
            Some(until) => now < until,
            None => false,
        };
    }

    /// Records the report of a process that could not be inspected,
    /// once per process.
    fn unreadable(&mut self, pid: i32, error: &std::io::Error) -> Outcome {
        let Some(start_time) = read_start_time(&self.setup.proc_root, pid) else {
            return Outcome::Ignored;
        };
        if !self.handled.insert((pid, start_time)) {
            return Outcome::Ignored;
        }
        return Outcome::Unreadable { pid, error: error.to_string() };
    }

    /// Records a failed move and builds its outcome.
    fn register_failure(&mut self, key: (i32, u64), now: Instant, error: String) -> Outcome {
        self.failed_attempts_this_pass += 1;
        let earlier = match self.failures.get(&key) {
            Some(failure) => failure.attempts,
            None => 0,
        };
        let attempt = earlier + 1;
        let retry_in = retry_backoff(attempt);
        self.failures.insert(key, Failure { attempts: attempt, retry_at: now + retry_in });
        return Outcome::Failed { pid: key.0, error, attempt, retry_in };
    }

    /// Considers one process at the current time.
    pub fn consider(&mut self, pid: i32) -> Outcome {
        return self.consider_at(pid, Instant::now());
    }

    /// Considers one process at `now`.
    pub fn consider_at(&mut self, pid: i32, now: Instant) -> Outcome {
        return self.consider_with(pid, now, &mut nothing);
    }

    /// Considers one process at `now`,
    /// calling `before_move` after the group is collected and before it is checked again,
    /// the seam where a test swaps a process for a pid-reusing one.
    pub fn consider_with(&mut self, pid: i32, now: Instant, before_move: &mut dyn FnMut()) -> Outcome {
        let info = match read_process(&self.setup.proc_root, pid, self.setup.uid) {
            Ok(Some(info)) => info,
            Ok(None) => return Outcome::Ignored,
            Err(error) => return self.unreadable(pid, &error),
        };
        let Some(reason) = classify(&info.exe, &info.args, &self.setup.install_roots) else {
            return Outcome::Ignored;
        };
        let key = (pid, info.start_time);
        if self.handled.contains(&key) {
            return Outcome::Ignored;
        }
        if let Some(failure) = self.failures.get(&key)
            && now < failure.retry_at
        {
            return Outcome::Ignored;
        }
        if info.cgroup.contains(TOOLCHAIN_UPDATES_SLICE) {
            self.handled.insert(key);
            return Outcome::AlreadyInSlice;
        }
        match (&self.own_pid_namespace, &info.pid_namespace) {
            (Some(own), Some(theirs)) if own != theirs => {
                self.handled.insert(key);
                return Outcome::Skipped {
                    pid,
                    why: format!("it runs in another pid namespace ({theirs}), such as a container or a sandbox, which this policy does not enter"),
                };
            }
            (Some(_own), None) => {
                self.handled.insert(key);
                return Outcome::Skipped {
                    pid,
                    why: "its pid namespace cannot be read, so it cannot be told apart from a process in a container or a sandbox, which this policy does not enter".to_string(),
                };
            }
            _ => {}
        }
        // A cgroup the watcher attaches the mark to itself needs no move,
        // which is how a service's main process, and every Pi process in its unit, is covered.
        if is_socket_marked_cgroup(&info.cgroup, self.setup.uid) {
            self.handled.insert(key);
            return Outcome::Covered { pid };
        }
        if let Err(why) = adoptable_cgroup(&info.cgroup, self.setup.uid) {
            self.handled.insert(key);
            return Outcome::Skipped { pid, why };
        }
        if let Some(why) = self.postponement(now) {
            return Outcome::Deferred { pid, why };
        }
        let (group, unreadable) = match collect_group(&self.setup, &info) {
            Ok(collected) => collected,
            Err(error) => return self.register_failure(key, now, error),
        };
        before_move();
        let (pids, dropped) = recheck_group(&self.setup, &group, &info.cgroup);
        // The matched process must still be the same process when the move is requested.
        if pids.first() != Some(&pid) {
            return Outcome::Ignored;
        }
        let scope_name = origin_scope_name(&info.cgroup, self.setup.uid);
        let destination = scope_cgroup_path(self.setup.uid, TOOLCHAIN_UPDATES_SLICE, &scope_name);
        let limits = match read_inherited_limits(&self.setup.cgroup_root, &info.cgroup, &destination) {
            Ok(limits) => limits,
            Err(error) => {
                let text = format!("the limits of {} could not be read, and a process is never moved without them: {error}", info.cgroup);
                return self.register_failure(key, now, text);
            }
        };
        let request = AdoptRequest {
            pids: &pids,
            uid: self.setup.uid,
            gid: self.setup.gid,
            slice: TOOLCHAIN_UPDATES_SLICE,
            limits: &limits,
            scope_name: &scope_name,
            first_start_time: Some(info.start_time),
        };
        return match self.setup.adopter.adopt(&request) {
            Ok(joined) => {
                self.handled.insert(key);
                self.failures.remove(&key);
                self.member_vanishes.remove(&key);
                // A join that finds the manager hung pauses the moves like a hung move does.
                if self.scopes.note_joined(&self.setup, TOOLCHAIN_UPDATES_SLICE, &info.cgroup, &scope_name, &joined, &limits, now) {
                    self.blocked_until = Some(now + BLOCK_AFTER_TIMEOUT);
                }
                Outcome::Adopted { pid, reason, count: pids.len(), left_behind: unreadable + dropped }
            }
            Err(AdoptError::Vanished | AdoptError::ScopeGone) => {
                self.failures.remove(&key);
                self.member_vanishes.remove(&key);
                Outcome::Ignored
            }
            Err(AdoptError::MemberVanished) => {
                let count = match self.member_vanishes.get(&key) {
                    Some(earlier) => earlier + 1,
                    None => 1,
                };
                self.member_vanishes.insert(key, count);
                if count < MEMBER_VANISH_LIMIT {
                    return Outcome::Ignored;
                }
                let text = format!(
                    "the user's service manager answered \"No such process\" {count} times in a row while a process of its group was exiting, so the process is still outside the policy"
                );
                self.register_failure(key, now, text)
            }
            Err(AdoptError::Failed(error)) => self.register_failure(key, now, error),
            Err(AdoptError::TimedOut(error)) => {
                self.blocked_until = Some(now + BLOCK_AFTER_TIMEOUT);
                let text = format!("{error}; no move is attempted for {BLOCK_AFTER_TIMEOUT:?}");
                self.register_failure(key, now, text)
            }
        };
    }

    /// Forgets the handled and failed processes whose pids are no longer listed.
    fn forget_exited(&mut self, listed: &[i32]) {
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
    pub fn rescan_with(&mut self, clock: &dyn Clock, stop: &dyn StopCheck) -> Vec<Outcome> {
        self.begin_pass_at(clock.now());
        let pids = match list_pids(&self.setup.proc_root) {
            Ok(pids) => pids,
            Err(error) => return vec![Outcome::ScanFailed { error: error.to_string() }],
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
        if !self.is_blocked(clock.now()) {
            let followed = self.scopes.follow_origins(&self.setup, TOOLCHAIN_UPDATES_SLICE, clock);
            if followed.timed_out {
                self.blocked_until = Some(clock.now() + BLOCK_AFTER_TIMEOUT);
            }
            outcomes.extend(followed.outcomes);
        }
        return outcomes;
    }
}
