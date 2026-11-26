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

/// Building the engine and the short questions it answers.
mod engine_construction_and_marks;
/// The decision about one process,
/// and what a refused move is tried again after.
mod one_process_decision;
/// One pass over the process list.
mod policy_pass;

/// Collecting and checking the group of a matched process.
#[path = "routing_group.rs"]
mod routing_group;
/// What one pass has spent so far.
use crate::pass_gate::PassGate;
/// The seam to the service manager.
use crate::scope_adoption::Adopter;
/// What is known about the scopes in use.
use crate::scope_registry::ScopeRegistry;
/// The processes already handled and the ones whose move failed.
use std::collections::{
    HashMap,
    HashSet,
};
/// Cgroups as filesystem paths.
use std::path::PathBuf;
/// The instants of the pass and of retries.
use std::time::Instant;

/// Refusals of "No such process" for a living process,
/// in a row,
/// before the policy reports them.
/// A group member that exits makes the manager refuse once;
/// a run of them is not a race any more.
pub const MEMBER_VANISH_LIMIT: u32 = 3;

/// Everything the engine needs for one watched user.
pub struct RoutingSetup {
    /// Watched user.
    pub uid: u32,
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
    /// What the current pass has spent,
    /// and when moves may go ahead again.
    gate: PassGate,
    /// Cgroups the watcher attached the socket mark to,
    /// as its last scan found them.
    /// A process in one of them,
    /// or in a cgroup below one,
    /// needs no move:
    /// its sockets already leave the tunnel.
    marked: Vec<PathBuf>,
}
