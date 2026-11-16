//! Remembers the scope of each origin cgroup and the limits it was last given,
//! and keeps the scope's limits equal to its origin's while the scope lives.
//! The cap daemon rewrites the `cpu.max` of `claude-code-bash` as the machine heats and cools
//! (the `CAP-APPLIED` lines of its log),
//! so a scope that kept the limits of its creation would let its processes escape a cap the origin tightened,
//! or stay under one the origin relaxed.
//! Once the origin's limits are read again,
//! a different value is applied to the live scope through the manager.
//! When the origin's directory is gone the scope keeps the limits it has,
//! and when the scope is gone the record is dropped.
//! Each application is a call to the user's manager that may take as long as the client timeout,
//! on the watcher's own thread,
//! so a scan spends at most a budget on them,
//! stops at the first call that times out and tells the engine so it pauses its moves,
//! and an origin whose application failed waits a growing time before it is tried again.

/// The limits read from the origin.
use crate::inherited_limits::{read_inherited_limits, InheritedLimits};
/// The clock and the retry wait.
use crate::routing_support::{retry_backoff, Clock};
/// The seam to the manager.
use crate::scope_adoption::{AdoptError, Joined, LimitRequest};
/// Where a scope lives.
use crate::scope_naming::{scope_cgroup_directory, scope_cgroup_path};
/// What setting the limits of a scope can lead to.
use crate::update_report::Outcome;
/// The slice and the engine's setup.
use crate::update_routing::RoutingSetup;
/// The records by origin.
use std::collections::HashMap;
/// Budget and backoff.
use std::time::{Duration, Instant};

/// Time one scan may spend applying limits before it leaves the rest for the next scan.
const FOLLOW_BUDGET: Duration = Duration::from_secs(1);

/// What is known about the scope of one origin.
struct ScopeRecord {
    /// Name of the scope.
    scope_name: String,
    /// The limits the scope was last given by this watcher,
    /// or `None` for a scope found running that this watcher did not create,
    /// whose limits are not known until they are applied once.
    applied: Option<InheritedLimits>,
    /// Applications that failed in a row.
    failures: u32,
    /// Earliest time of the next application after a failure.
    retry_at: Option<Instant>,
}

/// Backoff of a record.
impl ScopeRecord {
    /// Reports whether the record is waiting out the backoff of a failed application.
    fn waiting(&self, now: Instant) -> bool {
        return match self.retry_at {
            Some(until) => now < until,
            None => false,
        };
    }

    /// Notes a failed application at `now` and sets the wait before the next one.
    fn failed(&mut self, now: Instant) {
        self.failures += 1;
        self.retry_at = Some(now + retry_backoff(self.failures));
    }

    /// Notes an application that worked.
    fn succeeded(&mut self, limits: &InheritedLimits) {
        self.applied = Some(limits.clone());
        self.failures = 0;
        self.retry_at = None;
    }
}

/// What one follow-up scan found.
#[derive(Debug, Default)]
pub struct FollowResult {
    /// What needs saying.
    pub outcomes: Vec<Outcome>,
    /// Whether a call to the manager ran past its timeout,
    /// so the engine should pause its moves.
    pub timed_out: bool,
}

/// The scopes the watcher uses,
/// by the cgroup path of their origin.
#[derive(Default)]
pub struct ScopeRegistry {
    /// Records by origin.
    records: HashMap<String, ScopeRecord>,
}

/// Recording and following the origins' limits.
impl ScopeRegistry {
    /// Builds an empty registry.
    pub fn new() -> ScopeRegistry {
        return ScopeRegistry { records: HashMap::new() };
    }

    /// The number of origins with a scope.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        return self.records.len();
    }

    /// Notes that processes joined the scope of `origin`.
    /// A scope this watcher created has the limits it was created with.
    /// A scope that was already running is given the origin's limits at once if it is not known to have them
    /// and its record is not waiting out a failure:
    /// a failure there is left for the periodic follow-up,
    /// which retries after the backoff and reports it.
    /// Returns whether the manager did not answer in time.
    pub fn note_joined(&mut self, setup: &RoutingSetup, slice: &str, origin: &str, scope_name: &str, joined: &Joined, limits: &InheritedLimits, now: Instant) -> bool {
        let record = self.records.entry(origin.to_string()).or_insert(ScopeRecord { scope_name: scope_name.to_string(), applied: None, failures: 0, retry_at: None });
        if *joined == Joined::Created {
            record.succeeded(limits);
            return false;
        }
        if record.applied.as_ref() == Some(limits) || record.waiting(now) {
            return false;
        }
        match apply(setup, slice, scope_name, limits) {
            Ok(()) => {
                record.succeeded(limits);
                return false;
            }
            Err(AdoptError::ScopeGone) => {
                self.records.remove(origin);
                return false;
            }
            Err(error) => {
                record.failed(now);
                return matches!(error, AdoptError::TimedOut(_));
            }
        }
    }

    /// Follows every origin's limits:
    /// drops the records of scopes that ended,
    /// and gives a running scope the limits its origin has now when they differ from the ones it was given.
    /// A scan spends at most the budget on calls to the manager,
    /// stops at the first call that times out,
    /// and leaves an origin alone until its backoff after a failure is over.
    pub fn follow_origins(&mut self, setup: &RoutingSetup, slice: &str, clock: &dyn Clock) -> FollowResult {
        let started = clock.now();
        let mut result = FollowResult::default();
        let mut origins: Vec<String> = self.records.keys().cloned().collect();
        origins.sort();
        for origin in origins {
            let Some(record) = self.records.get_mut(&origin) else {
                continue;
            };
            let directory = scope_cgroup_directory(&setup.cgroup_root, setup.uid, slice, &record.scope_name);
            if !directory.is_dir() {
                self.records.remove(&origin);
                continue;
            }
            if !setup.cgroup_root.join(origin.trim_start_matches('/')).is_dir() {
                continue;
            }
            let destination = scope_cgroup_path(setup.uid, slice, &record.scope_name);
            let limits = match read_inherited_limits(&setup.cgroup_root, &origin, &destination) {
                Ok(limits) => limits,
                Err(error) => {
                    result.outcomes.push(Outcome::ScopeLimits { scope: record.scope_name.clone(), error: format!("the limits of {origin} could not be read: {error}") });
                    continue;
                }
            };
            if record.applied.as_ref() == Some(&limits) {
                continue;
            }
            let now = clock.now();
            if record.waiting(now) {
                continue;
            }
            if now.duration_since(started) >= FOLLOW_BUDGET {
                break;
            }
            match apply(setup, slice, &record.scope_name, &limits) {
                Ok(()) => record.succeeded(&limits),
                Err(AdoptError::ScopeGone) => {
                    self.records.remove(&origin);
                }
                Err(error) => {
                    record.failed(clock.now());
                    result.outcomes.push(Outcome::ScopeLimits { scope: record.scope_name.clone(), error: error.to_string() });
                    if matches!(error, AdoptError::TimedOut(_)) {
                        result.timed_out = true;
                        break;
                    }
                }
            }
        }
        return result;
    }
}

/// Gives a running scope limits through the manager.
fn apply(setup: &RoutingSetup, slice: &str, scope_name: &str, limits: &InheritedLimits) -> Result<(), AdoptError> {
    return setup.adopter.apply_limits(&LimitRequest { uid: setup.uid, slice, scope_name, limits });
}
