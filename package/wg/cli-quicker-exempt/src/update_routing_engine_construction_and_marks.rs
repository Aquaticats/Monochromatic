//! Building the policy engine,
//! and the short questions it answers:
//! the slice it moves into,
//! its own pid namespace,
//! and which cgroups already carry the socket mark.

/// What the engine is built with,
/// and the engine itself.
use super::{
    RoutingSetup,
    UpdateRouting,
};
/// The slice a process is moved into,
/// and the names that need no evidence to be marked.
use crate::application_targets::{
    TOOLCHAIN_UPDATES_SLICE,
    is_unconditionally_marked_cgroup,
};
/// What one pass has spent so far.
use crate::pass_gate::PassGate;
/// The watcher's own pid namespace.
use crate::process_info::read_own_pid_namespace;
/// The scope of each origin.
use crate::scope_registry::ScopeRegistry;
/// The processes already handled and the ones whose move failed.
use std::collections::{
    HashMap,
    HashSet,
};
/// Cgroups as filesystem paths.
use std::path::PathBuf;

/// Construction and the questions the engine answers about itself.
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
            gate: PassGate::new(),
            marked: Vec::new(),
        };
    }

    /// Records the cgroups the watcher holds the mark on,
    /// which is what the process watch treats as covered.
    pub fn note_marked(
        &mut self,
        marked: &[PathBuf],
    ) {
        self.marked = marked.to_vec();
    }

    /// Reports whether a process's cgroup carries the socket mark:
    /// one of the marks the watcher holds,
    /// or a cgroup below one,
    /// or a name that needs no evidence,
    /// because a Pi agent's unit and the updates slice are marked whoever filled them.
    pub(super) fn is_covered(
        &self,
        cgroup: &str,
    ) -> bool {
        if is_unconditionally_marked_cgroup(cgroup, self.setup.uid) {
            return true;
        }
        let full = self.setup.cgroup_root.join(cgroup.trim_start_matches('/'));
        return self.marked.iter().any(|mark| return full.starts_with(mark));
    }

    /// Reports whether the watcher could read its own pid namespace.
    /// Without it a process in a container or a sandbox cannot be told apart.
    pub fn pid_namespace_known(&self) -> bool {
        return self.own_pid_namespace.is_some();
    }

    /// Makes sure the updates slice exists below the user's `app.slice`,
    /// so the watcher can attach the mark to it before any process is moved.
    pub fn ensure_slice(
        &self,
        app_slice: &std::path::Path,
    ) -> std::io::Result<()> {
        return self.setup.adopter.ensure_slice(
            self.setup.uid,
            TOOLCHAIN_UPDATES_SLICE,
            &app_slice.join(TOOLCHAIN_UPDATES_SLICE),
        );
    }
}
