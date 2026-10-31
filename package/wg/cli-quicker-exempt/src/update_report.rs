//! The words of the update traffic policy:
//! the notice printed on every start,
//! the loud report for a policy that cannot be enforced,
//! and the outcome of considering one process.
//! The policy has no switch,
//! and nothing here changes routing.

/// Slice and cgroup names.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Why a process matched,
/// and the installed tool names.
use crate::update_entry_points::{installed_tool_names, MatchReason};
/// Install root paths.
use std::path::PathBuf;
/// Retry waits.
use std::time::Duration;

/// Name of the policy the process watcher enforces,
/// used in every report.
pub const POLICY_PROCESS_WATCH: &str = "process watch: update entry points and toolchain install roots stay outside the tunnel";

/// The words that mark every loud report,
/// which the starting command also looks for in the watcher log to relay startup failures.
pub const POLICY_NOT_IN_FORCE_MARKER: &str = "POLICY NOT IN FORCE";

/// The notice printed when the watcher starts,
/// so anyone using the wg tools knows before traffic flows.
/// `install_roots` are the watched user's toolchain roots,
/// whose entries the notice lists as installed now.
pub fn policy_notice(install_roots: &[PathBuf]) -> String {
    let installed = installed_tool_names(install_roots);
    let installed_text = if installed.is_empty() { "nothing yet".to_string() } else { installed.join(", ") };
    return [
        "wg-quicker-exempt: POLICY NOTICE. By the repository's policy, and without any switch, this traffic leaves outside the tunnel".to_string(),
        "wherever wg-quicker installed its bypass rules (with Table = off it installs none, and the socket marks then decide nothing):".to_string(),
        "  - Update entry points: mise toolchain tasks that fetch, mpm-shadow install, update and outdated, the mpm shadow,".to_string(),
        "    and the TypeScript toolchain sync and update scripts, together with the processes they start.".to_string(),
        "  - Every program the toolchain installed, not only its updates: any process whose executable is under the toolchain".to_string(),
        "    install roots, toolchain and toolchain-global in the monochromatic-dev data directory, together with the processes it starts.".to_string(),
        "    That is every script and dev server run by the toolchain's node, every build by its compilers, and everything they fetch.".to_string(),
        format!("    Installed on this machine now: {installed_text}."),
        "  - How a process is recognized, and what can imitate it: the executable's file name (mise, mpm-shadow, mpm, node)".to_string(),
        "    together with its command-line words, the tail of a toolchain script path, any executable named mpm,".to_string(),
        "    and any executable under the install roots, which are inside a home directory the user can write.".to_string(),
        "    Any program the user can run from such a path, or under such a name, leaves outside the tunnel too.".to_string(),
        "  - Not covered: processes in another pid namespace (containers and sandboxes), and processes the user's service manager".to_string(),
        "    cannot move (login sessions and the main process of a service); each is reported when it matches.".to_string(),
        "  - Moved processes keep the CPU, memory and task limits of the cgroup they left, as their own limits.".to_string(),
        "  The update-host list is routed around the tunnel for every program by wg-quicker itself,".to_string(),
        "  and each of its rules carries a whole address, so a shared CDN address serves other sites outside the tunnel too.".to_string(),
    ]
    .join("\n");
}

/// Builds the loud report for a policy that cannot be enforced.
pub fn policy_failure(detail: &str) -> String {
    return format!("wg-quicker-exempt: {POLICY_NOT_IN_FORCE_MARKER}: {POLICY_PROCESS_WATCH}: {detail}");
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
    /// The process matched but the attempt was postponed,
    /// because earlier attempts in this pass failed or the manager did not answer in time.
    /// It is tried again on a later pass.
    Deferred,
    /// The process matched and was moved with its descendants.
    Adopted {
        /// Matched process.
        pid: i32,
        /// Why it matched.
        reason: MatchReason,
        /// Number of processes moved.
        count: usize,
        /// Number of processes that could not be moved or inspected with it.
        left_behind: usize,
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
        /// Which attempt this was.
        attempt: u32,
        /// How long until the next attempt.
        retry_in: Duration,
    },
    /// A process of the watched user could not be inspected.
    Unreadable {
        /// The process.
        pid: i32,
        /// The reading failure.
        error: String,
    },
    /// The list of processes could not be read.
    ScanFailed {
        /// The reading failure.
        error: String,
    },
}

/// Classification of outcomes.
impl Outcome {
    /// Reports whether the outcome means the policy is not in force for some process.
    pub fn is_failure(&self) -> bool {
        return matches!(self, Outcome::Skipped { .. } | Outcome::Failed { .. } | Outcome::Unreadable { .. } | Outcome::ScanFailed { .. });
    }
}

/// Formats an outcome for the watcher log,
/// or `None` when nothing needs saying.
pub fn describe_outcome(outcome: &Outcome) -> Option<String> {
    return match outcome {
        Outcome::Ignored | Outcome::AlreadyInSlice | Outcome::Deferred => None,
        Outcome::Adopted { pid, reason, count, left_behind } => {
            let mut line = format!(
                "wg-quicker-exempt: moved pid {pid} ({}) and {} descendant(s) into {TOOLCHAIN_UPDATES_SLICE}",
                reason.describe(),
                count.saturating_sub(1)
            );
            if *left_behind > 0 {
                line.push_str(&format!("; {left_behind} other process(es) of this user could not be moved or inspected with it"));
            }
            Some(line)
        }
        Outcome::Skipped { pid, why } => Some(policy_failure(&format!("pid {pid} matched but was not moved: {why}"))),
        Outcome::Failed { pid, error, attempt, retry_in } => Some(policy_failure(&format!(
            "pid {pid} matched but the move failed (attempt {attempt}, next attempt in {retry_in:?}; the limits of its old cgroup are never dropped to make it succeed): {error}"
        ))),
        Outcome::Unreadable { pid, error } => Some(policy_failure(&format!(
            "pid {pid} could not be inspected, so if it is an update entry point it was not moved: {error}"
        ))),
        Outcome::ScanFailed { error } => Some(policy_failure(&format!(
            "the list of processes could not be read, so only exec events find update entry points until it can: {error}"
        ))),
    };
}
