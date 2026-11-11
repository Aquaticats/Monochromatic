//! The words of the update traffic policy:
//! the notice printed on every start,
//! the loud report for a policy that cannot be enforced,
//! and the outcome of considering one process.
//! The policy has no switch,
//! and nothing here changes routing.

/// Slice and cgroup names.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Escaping of untrusted text.
use crate::log_line::sanitize;
/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// The install roots that were left out,
/// and the installed tool names.
use crate::install_roots::{installed_tool_names, RefusedRoot};
/// Why a process matched.
use crate::update_entry_points::MatchReason;
/// Install root paths.
use std::path::{Path, PathBuf};
/// Retry waits.
use std::time::Duration;

/// Name of the policy the process watcher enforces,
/// used in every report,
/// and the same text the TypeScript package prints.
pub const POLICY_PROCESS_WATCH: &str = "process watch: update entry points, toolchain install roots and Pi stay outside the tunnel";

/// The words that mark every loud report,
/// which the starting command also looks for in the watcher log to relay startup failures.
pub const POLICY_NOT_IN_FORCE_MARKER: &str = "POLICY NOT IN FORCE";

/// The notice printed when the watcher starts,
/// so anyone using the wg tools knows before traffic flows.
/// `install_roots` are the watched user's toolchain roots,
/// whose entries the notice lists as installed now.
pub fn policy_notice(install_roots: &[PathBuf], home: &Path) -> String {
    let installed = installed_tool_names(install_roots, home);
    let installed_text = if installed.is_empty() { "nothing yet".to_string() } else { installed.join(", ") };
    return [
        "wg-quicker-exempt: POLICY NOTICE. By the repository's policy, and without any switch, this traffic leaves outside the tunnel".to_string(),
        "wherever wg-quicker installed its bypass rules (with Table = off it installs none, and the socket marks then decide nothing):".to_string(),
        "  - Update entry points: mise toolchain tasks that fetch, mpm-shadow install, update and outdated, the mpm shadow,".to_string(),
        "    and the TypeScript mpm-shadow launcher, together with the processes they start.".to_string(),
        "  - Every program the toolchain installed, not only its updates: any process whose executable is under the toolchain".to_string(),
        "    install roots, toolchain and toolchain-global in the monochromatic-dev data directory, together with the processes it starts.".to_string(),
        "    That is every script and dev server run by the toolchain's node, every build by its compilers, and everything they fetch.".to_string(),
        format!("    Installed on this machine now: {installed_text}."),
        "  - Pi: every process of a pi-<name>.service agent unit, and every node process whose first command-line word is pi,".to_string(),
        "    together with the processes they start. A unit is not moved: the service manager cannot move a service's main".to_string(),
        "    process, so the socket mark the watcher attaches to that cgroup covers it and every process below it,".to_string(),
        "    which is the same rule the terminals have.".to_string(),
        "  - Applications whose own cgroup carries the socket mark: Ghostty, Steam, Helium, Pale Moon, Firefox release,".to_string(),
        "    Firefox Nightly, ChatGPT, Interpreter, Qure and OdyTTY, matched by the file name of their executables anywhere,".to_string(),
        "    which marks the whole cgroup, so every other process in it leaves the tunnel too until the cgroup disappears.".to_string(),
        "  - How a process is recognized, and what can imitate it: the executable's file name (mise, mpm-shadow, mpm, node)".to_string(),
        "    together with its command-line words, the tail of a toolchain script path, any executable named mpm,".to_string(),
        "    and any executable under the install roots, which are inside a home directory the user can write.".to_string(),
        "    Any program the user can run from such a path, or under such a name, leaves outside the tunnel too.".to_string(),
        "    Pi is matched by the first command-line word it sets for itself, together with the node file name,".to_string(),
        "    so `exec -a pi node` imitates it, and by the name of the unit it runs in, which is a name the user can choose.".to_string(),
        "    A toolchain folder that is a link is ignored: links are not followed, wherever they point,".to_string(),
        "    and each such root is reported loudly as not in force every time the watcher starts.".to_string(),
        "  - Not covered: processes in another pid namespace (containers and sandboxes), processes of a login session,".to_string(),
        "    and the main process of a service whose cgroup the watcher does not mark; each is reported when it matches.".to_string(),
        "  - Moved processes from one cgroup share one scope that carries that cgroup's CPU, memory, task and I/O weight limits once,".to_string(),
        "    and follows them while it lives, so together they stay within the cap the cgroup has.".to_string(),
        "  The update-host list and the model provider hosts of Pi are routed around the tunnel for every program by wg-quicker itself,".to_string(),
        "  and each of its rules carries a whole address, so a shared CDN address serves other sites outside the tunnel too.".to_string(),
    ]
    .join("\n");
}

/// The loud reports for install roots that were left out of the matching.
/// Each names the root as spelled and why,
/// and says what that leaves.
pub fn refused_root_failures(refused: &[RefusedRoot]) -> Vec<String> {
    return refused.iter().map(refused_root_failure).collect();
}

/// The loud report for one install root that was left out.
fn refused_root_failure(entry: &RefusedRoot) -> String {
    return policy_failure(&format!(
        "the install root {} was left out of the matching: {}; programs run from there are not treated as toolchain programs, \
and links are never followed",
        quote_name(&entry.root.display().to_string()),
        entry.why
    ));
}

/// Builds the loud report for a policy that cannot be enforced.
/// Every character that could start an escape sequence or end the line is escaped,
/// because the detail often embeds names a user chose,
/// such as a cgroup path,
/// and the line reaches a log,
/// the journal
/// and a terminal.
pub fn policy_failure(detail: &str) -> String {
    return format!("wg-quicker-exempt: {POLICY_NOT_IN_FORCE_MARKER}: {POLICY_PROCESS_WATCH}: {}", sanitize(detail));
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
    /// The process matched but the attempt was postponed.
    /// It is tried again on a later pass.
    Deferred {
        /// Matched process.
        pid: i32,
        /// Why the attempt was postponed.
        why: &'static str,
    },
    /// The process matched and its own cgroup carries the socket mark,
    /// so it needs no move:
    /// a service's main process cannot be moved,
    /// and the watcher attaches the mark to every cgroup it names.
    Covered {
        /// Matched process.
        pid: i32,
    },
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
    /// A scope could not be given the current limits of its origin cgroup.
    ScopeLimits {
        /// The scope.
        scope: String,
        /// The failure text.
        error: String,
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
    /// The failure kind when the outcome means the policy is not in force for some process.
    /// The kind,
    /// not the text,
    /// decides what counts as a repeat of a report,
    /// because the text embeds names a user chose.
    pub fn failure_kind(&self) -> Option<&'static str> {
        return match self {
            Outcome::Skipped { .. } => Some("move skipped"),
            Outcome::Failed { .. } => Some("move failed"),
            Outcome::Deferred { .. } => Some("move postponed"),
            Outcome::ScopeLimits { .. } => Some("scope limits not applied"),
            Outcome::Unreadable { .. } => Some("process unreadable"),
            Outcome::ScanFailed { .. } => Some("process list unreadable"),
            Outcome::Ignored | Outcome::AlreadyInSlice | Outcome::Covered { .. } | Outcome::Adopted { .. } => None,
        };
    }

}

/// Formats an outcome for the watcher log,
/// or `None` when nothing needs saying.
pub fn describe_outcome(outcome: &Outcome) -> Option<String> {
    return match outcome {
        Outcome::Ignored | Outcome::AlreadyInSlice => None,
        Outcome::Covered { pid } => Some(format!(
            "wg-quicker-exempt: pid {pid} matched and its own cgroup carries the socket mark, so it is covered without a move"
        )),
        Outcome::Deferred { pid, why } => Some(policy_failure(&format!(
            "pid {pid} matched but its move was postponed ({why}); it is tried again on a later pass, and until then its traffic uses the tunnel"
        ))),
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
        Outcome::ScopeLimits { scope, error } => Some(policy_failure(&format!(
            "the scope {} could not be given the current limits of its origin cgroup, so the processes in it keep the limits it has now, which may be older than the cgroup's: {error}",
            quote_name(scope)
        ))),
        Outcome::Unreadable { pid, error } => Some(policy_failure(&format!(
            "pid {pid} could not be inspected, so if it is an update entry point it was not moved: {error}"
        ))),
        Outcome::ScanFailed { error } => Some(policy_failure(&format!(
            "the list of processes could not be read, so only exec events find update entry points until it can: {error}"
        ))),
    };
}
