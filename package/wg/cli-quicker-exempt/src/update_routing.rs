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

/// Slice and cgroup names.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Process facts.
use crate::process_info::{list_pids, read_own_pid_namespace, read_process, read_start_time, ProcessInfo};
/// The seam to the service manager.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter};
/// Classification of processes.
use crate::update_entry_points::classify;
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
const FAILED_ATTEMPTS_PER_PASS: u32 = 3;
/// Wait before the second attempt;
/// each later wait doubles.
const RETRY_BACKOFF_BASE: Duration = Duration::from_secs(1);
/// Longest wait between attempts for one process.
const RETRY_BACKOFF_MAX: Duration = Duration::from_secs(60);
/// How long no move is attempted after the manager did not answer in time.
const BLOCK_AFTER_TIMEOUT: Duration = Duration::from_secs(30);

/// Reports whether a cgroup path can be adopted by the user's manager.
/// The path must lie in the user manager's subtree,
/// and must not be the cgroup of a service's main process.
pub fn adoptable_cgroup(cgroup: &str, uid: u32) -> Result<(), String> {
    let prefix = format!("/user.slice/user-{uid}.slice/user@{uid}.service/");
    if !cgroup.starts_with(&prefix) {
        return Err(format!(
            "its cgroup {cgroup} is outside the user manager's subtree, which the manager cannot move processes out of"
        ));
    }
    let leaf = cgroup.rsplit('/').next().unwrap_or("");
    if leaf.ends_with(".service") {
        return Err(format!("it is the main process of the service {leaf}, and moving it would empty that service"));
    }
    return Ok(());
}

/// The wait before attempt number `attempt` plus one:
/// one second after the first failure,
/// doubling,
/// capped.
pub fn retry_backoff(attempt: u32) -> Duration {
    let doublings = attempt.saturating_sub(1).min(6);
    return RETRY_BACKOFF_BASE.saturating_mul(1 << doublings).min(RETRY_BACKOFF_MAX);
}

/// Reports whether a collected group already holds `pid`.
fn contains_pid(group: &[(i32, u64)], pid: i32) -> bool {
    for (member, _start_time) in group {
        if *member == pid {
            return true;
        }
    }
    return false;
}

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
    /// Processes whose move failed and will be tried again.
    failures: HashMap<(i32, u64), Failure>,
    /// Failed moves spent in the current pass.
    failed_attempts_this_pass: u32,
    /// No move is attempted before this time.
    blocked_until: Option<Instant>,
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
            failures: HashMap::new(),
            failed_attempts_this_pass: 0,
            blocked_until: None,
        };
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

    /// Starts a pass:
    /// failed moves spent earlier no longer count against the next ones.
    pub fn begin_pass(&mut self) {
        self.failed_attempts_this_pass = 0;
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

    /// Collects the live descendants of `root` that run as the watched user in the same cgroup,
    /// with their start times,
    /// and the number of processes of this user that could not be inspected.
    fn collect_group(&self, root: &ProcessInfo) -> Result<(Vec<(i32, u64)>, usize), String> {
        let pids = match list_pids(&self.setup.proc_root) {
            Ok(pids) => pids,
            Err(error) => return Err(format!("listing the processes failed: {error}")),
        };
        let mut snapshot: Vec<ProcessInfo> = Vec::new();
        let mut unreadable = 0;
        for pid in pids {
            match read_process(&self.setup.proc_root, pid, self.setup.uid) {
                Ok(Some(info)) => snapshot.push(info),
                Ok(None) => {}
                Err(_error) => unreadable += 1,
            }
        }
        let mut found: Vec<(i32, u64)> = vec![(root.pid, root.start_time)];
        let mut index = 0;
        while index < found.len() {
            let parent = found[index].0;
            index += 1;
            for candidate in &snapshot {
                if candidate.parent == parent && candidate.cgroup == root.cgroup && !contains_pid(&found, candidate.pid) {
                    found.push((candidate.pid, candidate.start_time));
                }
            }
        }
        return Ok((found, unreadable));
    }

    /// Keeps the processes that are still the same ones,
    /// and counts the rest.
    /// A pid that exited and was reused has another start time and is dropped.
    fn recheck_group(&self, group: &[(i32, u64)], cgroup: &str) -> (Vec<i32>, usize) {
        let mut kept: Vec<i32> = Vec::new();
        let mut dropped = 0;
        for (pid, start_time) in group {
            match read_process(&self.setup.proc_root, *pid, self.setup.uid) {
                Ok(Some(again)) if again.start_time == *start_time && again.cgroup == cgroup => kept.push(*pid),
                _ => dropped += 1,
            }
        }
        return (kept, dropped);
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
        if let (Some(own), Some(theirs)) = (&self.own_pid_namespace, &info.pid_namespace)
            && own != theirs
        {
            self.handled.insert(key);
            return Outcome::Skipped {
                pid,
                why: format!("it runs in another pid namespace ({theirs}), such as a container or a sandbox, which this policy does not enter"),
            };
        }
        if let Err(why) = adoptable_cgroup(&info.cgroup, self.setup.uid) {
            self.handled.insert(key);
            return Outcome::Skipped { pid, why };
        }
        if self.is_blocked(now) || self.failed_attempts_this_pass >= FAILED_ATTEMPTS_PER_PASS {
            return Outcome::Deferred;
        }
        let (group, unreadable) = match self.collect_group(&info) {
            Ok(collected) => collected,
            Err(error) => return self.register_failure(key, now, error),
        };
        before_move();
        let (pids, dropped) = self.recheck_group(&group, &info.cgroup);
        // The matched process must still be the same process when the move is requested.
        if pids.first() != Some(&pid) {
            return Outcome::Ignored;
        }
        let request = AdoptRequest {
            pids: &pids,
            uid: self.setup.uid,
            gid: self.setup.gid,
            slice: TOOLCHAIN_UPDATES_SLICE,
        };
        return match self.setup.adopter.adopt(&request) {
            Ok(()) => {
                self.handled.insert(key);
                self.failures.remove(&key);
                Outcome::Adopted { pid, reason, count: pids.len(), left_behind: unreadable + dropped }
            }
            Err(AdoptError::Vanished) => {
                self.failures.remove(&key);
                Outcome::Ignored
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
    }

    /// Considers every live process now and forgets handled processes that no longer exist.
    pub fn rescan(&mut self) -> Vec<Outcome> {
        return self.rescan_at(Instant::now());
    }

    /// Considers every live process at `now` and forgets handled processes that no longer exist.
    /// Processes that need no report are left out of the result.
    pub fn rescan_at(&mut self, now: Instant) -> Vec<Outcome> {
        self.begin_pass();
        let pids = match list_pids(&self.setup.proc_root) {
            Ok(pids) => pids,
            Err(error) => return vec![Outcome::ScanFailed { error: error.to_string() }],
        };
        let mut outcomes: Vec<Outcome> = Vec::new();
        for pid in &pids {
            let outcome = self.consider_at(*pid, now);
            if outcome != Outcome::Ignored {
                outcomes.push(outcome);
            }
        }
        self.forget_exited(&pids);
        return outcomes;
    }
}
