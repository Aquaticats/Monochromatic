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

/// The limits read from the origin.
use crate::inherited_limits::{read_inherited_limits, InheritedLimits};
/// Where a scope lives.
use crate::scope_naming::{scope_cgroup_directory, scope_cgroup_path};
/// The seam to the manager.
use crate::scope_adoption::{AdoptError, Joined, LimitRequest};
/// What setting the limits of a scope can lead to.
use crate::update_report::Outcome;
/// The slice and the engine's setup.
use crate::update_routing::RoutingSetup;
/// The records by origin.
use std::collections::HashMap;

/// What is known about the scope of one origin.
struct ScopeRecord {
    /// Name of the scope.
    scope_name: String,
    /// The limits the scope was last given by this watcher,
    /// or `None` for a scope found running that this watcher did not create,
    /// whose limits are not known until they are applied once.
    applied: Option<InheritedLimits>,
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
    /// A scope that was already running is given the origin's limits at once if it is not known to have them:
    /// a failure there is left for the periodic follow-up,
    /// which retries and reports it.
    pub fn note_joined(&mut self, setup: &RoutingSetup, slice: &str, origin: &str, scope_name: &str, joined: &Joined, limits: &InheritedLimits) {
        let record = self.records.entry(origin.to_string()).or_insert(ScopeRecord { scope_name: scope_name.to_string(), applied: None });
        if *joined == Joined::Created {
            record.applied = Some(limits.clone());
            return;
        }
        if record.applied.as_ref() == Some(limits) {
            return;
        }
        match apply(setup, slice, scope_name, limits) {
            Ok(()) => record.applied = Some(limits.clone()),
            Err(AdoptError::ScopeGone) => {
                self.records.remove(origin);
            }
            Err(_failed) => {}
        }
    }

    /// Follows every origin's limits:
    /// drops the records of scopes that ended,
    /// and gives a running scope the limits its origin has now when they differ from the ones it was given.
    /// Returns what needs saying.
    pub fn follow_origins(&mut self, setup: &RoutingSetup, slice: &str) -> Vec<Outcome> {
        let mut outcomes: Vec<Outcome> = Vec::new();
        let origins: Vec<String> = self.records.keys().cloned().collect();
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
                    outcomes.push(Outcome::ScopeLimits { scope: record.scope_name.clone(), error: format!("the limits of {origin} could not be read: {error}") });
                    continue;
                }
            };
            if record.applied.as_ref() == Some(&limits) {
                continue;
            }
            match apply(setup, slice, &record.scope_name, &limits) {
                Ok(()) => record.applied = Some(limits),
                Err(AdoptError::ScopeGone) => {
                    self.records.remove(&origin);
                }
                Err(error) => outcomes.push(Outcome::ScopeLimits { scope: record.scope_name.clone(), error: error.to_string() }),
            }
        }
        return outcomes;
    }
}

/// Gives a running scope limits through the manager.
fn apply(setup: &RoutingSetup, slice: &str, scope_name: &str, limits: &InheritedLimits) -> Result<(), AdoptError> {
    return setup.adopter.apply_limits(&LimitRequest { uid: setup.uid, gid: setup.gid, slice, scope_name, limits });
}
