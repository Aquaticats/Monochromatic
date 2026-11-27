//! The decision about one process:
//! whether the policy moves it,
//! and what a refused move is tried again after.

/// Collecting the group of a matched process and checking it again.
use super::routing_group::{
    collect_group,
    recheck_group,
};
/// The engine this decision belongs to,
/// and the state it reads.
use super::{
    Failure,
    MEMBER_VANISH_LIMIT,
    UpdateRouting,
};
/// The slice a process is moved into,
/// and the test for a process that already sits in it.
use crate::application_targets::{
    TOOLCHAIN_UPDATES_SLICE,
    is_in_updates_slice,
};
/// The limits the scope carries.
use crate::inherited_limits::read_inherited_limits;
/// How long the moves stay paused after a manager that did not answer.
use crate::pass_gate::BLOCK_AFTER_TIMEOUT;
/// Process facts.
use crate::process_info::{
    read_process,
    read_start_time,
};
/// The retry wait,
/// and whether the manager may move a process out of its cgroup.
use crate::routing_support::{
    adoptable_cgroup,
    retry_backoff,
};
/// The seam to the service manager,
/// and what a refused move reports.
use crate::scope_adoption::{
    ADOPTION_DEFERRED,
    AdoptError,
    AdoptRequest,
};
/// The name and the path of the scope an origin receives.
use crate::scope_naming::{
    origin_scope_name,
    scope_cgroup_path,
};
/// Why a process matched,
/// which the scope description names.
use crate::update_entry_points::classify;
/// What one decision reports.
use crate::update_report::Outcome;
/// The time of the decision and of its retry.
use std::time::Instant;

/// Does nothing,
/// the default step between collecting a group and checking it again.
fn nothing() {}

/// The decisions about one process.
impl UpdateRouting {
    /// Records the report of a process that could not be inspected,
    /// once per process.
    fn unreadable(
        &mut self,
        pid: i32,
        error: &std::io::Error,
    ) -> Outcome {
        let Some(start_time) = read_start_time(&self.setup.proc_root, pid) else {
            return Outcome::Ignored;
        };
        if !self.handled.insert((pid, start_time)) {
            return Outcome::Ignored;
        }
        return Outcome::Unreadable {
            pid,
            error: error.to_string(),
        };
    }

    /// Records a failed move and builds its outcome.
    fn register_failure(
        &mut self,
        key: (i32, u64),
        now: Instant,
        error: String,
    ) -> Outcome {
        self.gate.note_failed_attempt();
        let earlier = match self.failures.get(&key) {
            Some(failure) => failure.attempts,
            None => 0,
        };
        let attempt = earlier + 1;
        let retry_in = retry_backoff(attempt);
        self.failures.insert(
            key,
            Failure {
                attempts: attempt,
                retry_at: now + retry_in,
            },
        );
        return Outcome::Failed {
            pid: key.0,
            error,
            attempt,
            retry_in,
        };
    }

    /// Considers one process at the current time.
    pub fn consider(
        &mut self,
        pid: i32,
    ) -> Outcome {
        return self.consider_at(pid, Instant::now());
    }

    /// Considers one process at `now`.
    pub fn consider_at(
        &mut self,
        pid: i32,
        now: Instant,
    ) -> Outcome {
        return self.consider_with(pid, now, &mut nothing);
    }

    /// Considers one process at `now`,
    /// calling `before_move` after the group is collected and before it is checked again,
    /// the seam where a test swaps a process for a pid-reusing one.
    pub fn consider_with(
        &mut self,
        pid: i32,
        now: Instant,
        before_move: &mut dyn FnMut(),
    ) -> Outcome {
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
        if is_in_updates_slice(&info.cgroup, self.setup.uid) {
            self.handled.insert(key);
            return Outcome::AlreadyInSlice;
        }
        match (&self.own_pid_namespace, &info.pid_namespace) {
            (Some(own), Some(theirs)) if own != theirs => {
                self.handled.insert(key);
                return Outcome::Skipped {
                    pid,
                    why: format!(
                        "it runs in another pid namespace ({theirs}), such as a container or a sandbox, which this policy does not enter"
                    ),
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
        // A cgroup the watcher attached the mark to needs no move,
        // which is how a service's main process, and every Pi process in its unit, is covered.
        if self.is_covered(&info.cgroup) {
            self.handled.insert(key);
            return Outcome::Covered {
                pid,
            };
        }
        if let Err(why) = adoptable_cgroup(&info.cgroup, self.setup.uid) {
            self.handled.insert(key);
            return Outcome::Skipped {
                pid,
                why,
            };
        }
        if let Some(why) = self.gate.postponement(now) {
            return Outcome::Deferred {
                pid,
                why,
            };
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
        // The origin's scope may be running under an earlier spelling of its name,
        // and its later processes join it there instead of one more scope with the same cap.
        let scope_name = match self.setup.adopter.existing_scope_name(
            self.setup.uid,
            TOOLCHAIN_UPDATES_SLICE,
            &info.cgroup,
        ) {
            Some(name) => name,
            None => origin_scope_name(&info.cgroup, self.setup.uid),
        };
        let destination = scope_cgroup_path(self.setup.uid, TOOLCHAIN_UPDATES_SLICE, &scope_name);
        let limits = match read_inherited_limits(
            &self.setup.cgroup_root,
            &info.cgroup,
            &destination,
        ) {
            Ok(limits) => limits,
            Err(error) => {
                let text = format!(
                    "the limits of {} could not be read, and a process is never moved without them: {error}",
                    info.cgroup
                );
                return self.register_failure(key, now, text);
            }
        };
        let request = AdoptRequest {
            pids: &pids,
            uid: self.setup.uid,
            slice: TOOLCHAIN_UPDATES_SLICE,
            limits: &limits,
            scope_name: &scope_name,
            origin: &info.cgroup,
            reason,
            first_start_time: Some(info.start_time),
        };
        return match self.setup.adopter.adopt(&request) {
            Ok(joined) => {
                self.handled.insert(key);
                self.failures.remove(&key);
                self.member_vanishes.remove(&key);
                // A join that finds the manager hung pauses the moves like a hung move does.
                if self.scopes.note_joined(
                    &self.setup,
                    TOOLCHAIN_UPDATES_SLICE,
                    &info.cgroup,
                    &scope_name,
                    &joined,
                    &limits,
                    now,
                ) {
                    self.gate.block(now);
                }
                Outcome::Adopted {
                    pid,
                    reason,
                    count: pids.len(),
                    left_behind: unreadable + dropped,
                }
            }
            Err(AdoptError::Deferred) => Outcome::Deferred {
                pid,
                why: ADOPTION_DEFERRED,
            },
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
                self.gate.block(now);
                let text = format!("{error}; no move is attempted for {BLOCK_AFTER_TIMEOUT:?}");
                self.register_failure(key, now, text)
            }
        };
    }
}
