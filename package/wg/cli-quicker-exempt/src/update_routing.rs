//! The always-on policy that keeps update traffic outside the tunnel.
//! A process that is an update entry point,
//! or runs from the toolchain install roots,
//! is moved into the updates slice,
//! so its sockets and its later children's sockets receive the exemption mark.
//! The policy has no switch.
//! When part of it cannot be enforced,
//! the functions here produce a loud report naming the policy,
//! and nothing changes routing on its own.

/// Slice and cgroup names.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Classification of processes.
use crate::update_entry_points::{classify, MatchReason};
/// Process facts.
use crate::process_info::{list_pids, read_process, ProcessInfo};
/// The seam to the service manager.
use crate::scope_adoption::{AdoptRequest, Adopter};
/// Tracking of processes already handled.
use std::collections::HashSet;
/// Fake procfs roots in tests.
use std::path::PathBuf;

/// Name of the policy this module enforces,
/// used in every report.
pub const POLICY_PROCESS_WATCH: &str = "process watch: update entry points and toolchain install roots stay outside the tunnel";

/// The notice printed when the watcher starts,
/// so anyone using the wg tools knows before traffic flows.
pub fn policy_notice() -> String {
    return [
        "wg-quicker-exempt: POLICY NOTICE. By the repository's policy, and without any switch, this traffic leaves outside the tunnel:",
        "  - processes that are update entry points: mise toolchain tasks that fetch, mpm-shadow install, update and outdated,",
        "    the mpm shadow, and the TypeScript toolchain sync and update scripts, together with the processes they start;",
        "  - every process whose executable is under the toolchain install roots, toolchain and toolchain-global",
        "    in the monochromatic-dev data directory, together with the processes it starts.",
        "  The update-host list is routed around the tunnel for every program by wg-quicker itself.",
    ]
    .join("\n");
}

/// Builds the loud report for a policy that cannot be enforced.
pub fn policy_failure(detail: &str) -> String {
    return format!("wg-quicker-exempt: POLICY NOT IN FORCE: {POLICY_PROCESS_WATCH}: {detail}");
}

/// What considering one process led to.
#[derive(Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The process is not subject to the policy,
    /// vanished,
    /// or was already handled.
    Ignored,
    /// The process already runs in the slice.
    AlreadyInSlice,
    /// The process matched and was moved with its descendants.
    Adopted {
        /// Matched process.
        pid: i32,
        /// Why it matched.
        reason: MatchReason,
        /// Number of processes moved.
        count: usize,
    },
    /// The process matched but cannot be moved.
    Skipped {
        /// Matched process.
        pid: i32,
        /// Why it cannot be moved.
        why: String,
    },
    /// The process matched and the move failed.
    Failed {
        /// Matched process.
        pid: i32,
        /// Failure text.
        error: String,
    },
}

/// Formats an outcome for the watcher log,
/// or `None` when nothing needs saying.
pub fn describe_outcome(outcome: &Outcome) -> Option<String> {
    return match outcome {
        Outcome::Ignored | Outcome::AlreadyInSlice => None,
        Outcome::Adopted { pid, reason, count } => Some(format!(
            "wg-quicker-exempt: moved pid {pid} ({}) and {} descendant(s) into {TOOLCHAIN_UPDATES_SLICE}",
            reason.describe(),
            count.saturating_sub(1)
        )),
        Outcome::Skipped { pid, why } => Some(policy_failure(&format!("pid {pid} matched but was not moved: {why}"))),
        Outcome::Failed { pid, error } => Some(policy_failure(&format!("pid {pid} matched but the move failed: {error}"))),
    };
}

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

/// The policy engine for one watched user.
pub struct UpdateRouting {
    /// Watched user.
    uid: u32,
    /// Primary group of the watched user.
    gid: u32,
    /// Toolchain install roots of that user.
    install_roots: Vec<PathBuf>,
    /// Procfs root.
    proc_root: PathBuf,
    /// Seam to the service manager.
    adopter: Box<dyn Adopter>,
    /// Processes already handled,
    /// by pid and start time.
    handled: HashSet<(i32, u64)>,
}

/// Behavior of the policy engine.
impl UpdateRouting {
    /// Builds the engine.
    pub fn new(uid: u32, gid: u32, install_roots: Vec<PathBuf>, proc_root: PathBuf, adopter: Box<dyn Adopter>) -> UpdateRouting {
        return UpdateRouting {
            uid,
            gid,
            install_roots,
            proc_root,
            adopter,
            handled: HashSet::new(),
        };
    }

    /// Makes sure the updates slice exists below the user's `app.slice`,
    /// so the watcher can attach the mark to it before any process is moved.
    pub fn ensure_slice(&self, app_slice: &std::path::Path) -> std::io::Result<()> {
        return self.adopter.ensure_slice(self.uid, self.gid, TOOLCHAIN_UPDATES_SLICE, &app_slice.join(TOOLCHAIN_UPDATES_SLICE));
    }

    /// Collects the live descendants of `root` that run as the watched user and can be adopted.
    fn adoptable_descendants(&self, root: &ProcessInfo, all: &[i32]) -> Vec<i32> {
        let mut found: Vec<i32> = vec![root.pid];
        let mut index = 0;
        while index < found.len() {
            let parent = found[index];
            index += 1;
            for candidate in all {
                if found.contains(candidate) {
                    continue;
                }
                let Ok(Some(info)) = read_process(&self.proc_root, *candidate) else {
                    continue;
                };
                if info.parent == parent
                    && info.uid == self.uid
                    && !info.cgroup.contains(TOOLCHAIN_UPDATES_SLICE)
                    && adoptable_cgroup(&info.cgroup, self.uid).is_ok()
                {
                    found.push(info.pid);
                }
            }
        }
        return found;
    }

    /// Considers one process that just executed or was found by a scan.
    pub fn consider(&mut self, pid: i32) -> Outcome {
        let Ok(Some(info)) = read_process(&self.proc_root, pid) else {
            return Outcome::Ignored;
        };
        if info.uid != self.uid {
            return Outcome::Ignored;
        }
        let Some(reason) = classify(&info.exe, &info.args, &self.install_roots) else {
            return Outcome::Ignored;
        };
        if !self.handled.insert((pid, info.start_time)) {
            return Outcome::Ignored;
        }
        if info.cgroup.contains(TOOLCHAIN_UPDATES_SLICE) {
            return Outcome::AlreadyInSlice;
        }
        if let Err(why) = adoptable_cgroup(&info.cgroup, self.uid) {
            return Outcome::Skipped { pid, why };
        }
        let all = list_pids(&self.proc_root).unwrap_or_default();
        let pids = self.adoptable_descendants(&info, &all);
        // The pid must still be the same process when the move is requested.
        match read_process(&self.proc_root, pid) {
            Ok(Some(again)) if again.start_time == info.start_time => {}
            _ => return Outcome::Ignored,
        }
        let request = AdoptRequest {
            pids: &pids,
            uid: self.uid,
            gid: info.gid,
            slice: TOOLCHAIN_UPDATES_SLICE,
        };
        return match self.adopter.adopt(&request) {
            Ok(()) => Outcome::Adopted { pid, reason, count: pids.len() },
            Err(error) => Outcome::Failed { pid, error: error.to_string() },
        };
    }

    /// Considers every live process and forgets handled processes that no longer exist.
    pub fn rescan(&mut self) -> Vec<Outcome> {
        let pids = list_pids(&self.proc_root).unwrap_or_default();
        let mut outcomes: Vec<Outcome> = Vec::new();
        for pid in &pids {
            outcomes.push(self.consider(*pid));
        }
        let mut alive: HashSet<(i32, u64)> = HashSet::new();
        for entry in &self.handled {
            if pids.contains(&entry.0) {
                alive.insert(*entry);
            }
        }
        self.handled = alive;
        return outcomes;
    }
}
