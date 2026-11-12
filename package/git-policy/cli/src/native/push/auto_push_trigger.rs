//! What:
//!  Hands a push to the commit service after a command other than `git commit` added commits to the branch:
//! a merge,
//! a
//!  pull's merge,
//! a cherry-pick,
//! a revert or an applied patch.
//! Why:
//!  A clean `git merge` used to stay local until the next commit pushed it.
//! The owner chose (2026-11-02) that any command
//!  adding commits queues the push as a commit does,
//! when the branch tip moved and the branch is now ahead of its upstream
//!  and not diverged from it,
//! and that a rebase does not,
//! because a rebased branch would need a forced push.
//!  The check reads the branch,
//! its upstream tracking ref and the commit parents from the files,
//! with no Git process:
//!  the tip is read before and after the forwarded command,
//! and the push is handed over only when the old tip and the
//!  upstream tip are both ancestors of the new tip.
//! The old tip being an ancestor is what leaves a rebase (a rewritten
//!  branch) out,
//! `pull --rebase` and `pull.rebase` included,
//! without reading the configuration for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchTip(worktree); await runGit(args); if (exitCode === 0) await watch.handOverIfAhead();
//! ```

#![cfg(unix)]

use crate::git_system_config::with_real_git;
use crate::repo_read_config_list::{ListedEntry, list_config};
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_layout::{RepoLayout, read_layout_after_success};
use crate::repo_read_object::{ObjectKind, from_hex};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_query_refs::tracking_ref;
use crate::repo_read_refs::{RefValue, Refs};
use crate::repo_read_tree::parse_commit;
use crate::service_client::{Activation, ClientConfig, manager_watch_for};
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_handoff::{ServiceHandoff, hand_over_push, write_push_record};
use crate::service_messages::{render};
use crate::push_trigger_log::{Decision, Origin, Outcome, record_decision};
use crate::service_messages::platform_remedy;
use crate::service_provision::platform_without_manager;
use crate::service_route::{Route, route_from_environment};
use crate::wait_ceilings::HAND_OVER_GREETING_WAIT;
use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The most commits the walk from a tip reads before it gives up and hands nothing over:
/// a branch far ahead,
/// or a history
/// that never reaches the upstream,
/// is not worth a long walk after every merge.
/// Measured:
/// the 439 most recent merges on `main` of
///  this repository brought in a median of 8 commits (90th percentile 50,
/// 99th percentile 317,
/// largest 6,542),
/// and only 1 exceeded
///  this budget,
/// so 438 of 439 are walked in full.
/// Kept as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const WALK_BUDGET: usize = 4096;

/// What the branch looked like before the command:
/// where it pointed and where its repository is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TipWatch = { layout: RepoLayout; variables: [string, string][]; realGit: string; branchRef: string; before: string };
/// ```
#[derive(Clone, Debug)]
pub struct TipWatch {
    /// Where the worktree and its repository are.
    layout: RepoLayout,
    /// The environment Git would see,
    /// which the configuration reader needs for its scopes.
    variables: Vec<(OsString, OsString)>,
    /// The real Git the service will push with.
    real_git: PathBuf,
    /// The branch `HEAD` named.
    branch_ref: String,
    /// The tip of that branch before the command.
    before: String,
}

/// Why the trigger did nothing:
/// the step that declined,
/// as a stable word,
/// and the facts a person needs to see why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Declined = { reason: string; detail: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declined {
    /// The step,
    /// in words that do not change (`layout-declined`,
    /// `head-detached`,
    /// `upstream-equals-tip`,
    /// and the others).
    pub reason: &'static str,
    /// What the step saw.
    pub detail: String,
}

/// A decline of `reason` with what was seen.
fn declined(reason: &'static str, detail: impl Into<String>) -> Declined {
    return Declined { reason, detail: detail.into() };
}

impl TipWatch {
    /// The common Git directory of the watched repository,
    /// where the trigger's records are kept.
    pub fn common_dir(&self) -> &Path {
        return self.layout.common_dir.as_path();
    }

    /// The worktree the command runs in,
    /// as a record names it.
    pub fn origin(&self) -> Origin<'_> {
        return Origin { worktree: self.layout.worktree_root.as_path(), git_dir: self.layout.git_dir.as_path() };
    }

    /// The branch `HEAD` named before the command.
    pub fn branch_ref(&self) -> &str {
        return self.branch_ref.as_str();
    }

    /// The tip of that branch before the command.
    pub fn tip_before(&self) -> &str {
        return self.before.as_str();
    }
}

/// The branch `HEAD` names and its tip,
/// or the step that found none:
/// a layout the reader declines,
/// a detached `HEAD`,
/// an unborn branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentBranch(layout): { branchRef: string; tip: string } | Declined;
/// ```
fn current_branch(layout: &RepoLayout) -> Result<(String, String), Declined> {
    let refs: Refs = Refs::new(layout.common_dir.as_path(), layout.object_format.hex_length());
    let Some(head) = refs.lookup_head(layout.git_dir.as_path()) else {
        return Err(declined("head-unreadable", format!("HEAD in {} is not a plain symbolic or direct ref", layout.git_dir.display())));
    };
    let RefValue::Symbolic(target) = head else {
        return Err(declined("head-detached", "HEAD names a commit, not a branch"));
    };
    if !target.starts_with("refs/heads/") {
        return Err(declined("head-not-a-branch", format!("HEAD names {target}")));
    }
    let Some(tip) = refs.resolve(target.as_str()) else {
        return Err(declined("branch-has-no-tip", format!("{target} does not resolve")));
    };
    return Ok((target, tip));
}

/// What:
///  Start watching the branch of a worktree before a command runs.
/// Why:
///   The command may be a merge or a pull,
/// which move the tip;
/// what the tip was is what a rebase is told apart by.
///   The layout is read as for a caller that follows a Git run that succeeds (`read_layout_after_success`):
/// a configuration value the reader cannot prove is no reason not to push once the command that moved the tip succeeded
/// (the files are read again,
/// not the snapshot that command saw,
/// and the final push is Git's).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchTip(worktreeRoot, realGit, overlay, environment): TipWatch | Declined;
/// ```
pub fn try_watch_tip(
    worktree_root: &Path,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
) -> Result<TipWatch, Declined> {
    // The readers learn the system configuration file from the real Git the child would run.
    let named: Vec<(OsString, OsString)> = with_real_git(overlay, real_git);
    let Some(layout) = read_layout_after_success(worktree_root, named.as_slice()) else {
        return Err(declined("layout-declined", format!("the files of {} are not a layout the reader takes", worktree_root.display())));
    };
    let (branch_ref, before): (String, String) = current_branch(&layout)?;
    let mut variables: Vec<(OsString, OsString)> = environment.to_vec();
    variables.extend(named);
    return Ok(TipWatch { layout, variables, real_git: real_git.to_path_buf(), branch_ref, before });
}

/// `try_watch_tip` without the reason:
/// nothing when the watch could not start.
pub fn watch_tip(
    worktree_root: &Path,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
) -> Option<TipWatch> {
    return try_watch_tip(worktree_root, real_git, overlay, environment).ok();
}

/// Every readable value of `branch.<name>.<suffix>` in the configuration listing,
/// in order.
fn branch_values(listed: &[ListedEntry], name: &str, suffix: &str) -> Vec<String> {
    let key: String = format!("branch.{name}.{suffix}");
    let mut found: Vec<String> = Vec::new();
    for item in listed {
        if item.entry.key != key {
            continue;
        }
        if let Some(bytes) = item.entry.value.as_deref()
            && let Ok(text) = std::str::from_utf8(bytes)
        {
            found.push(text.to_owned());
        }
    }
    return found;
}

/// Record how a hand-over ended,
/// with the reason,
/// for the watched branch and the command that ran.
fn record_ending(watch: &TipWatch, command: &str, outcome: Outcome, reason: &str, detail: &str) {
    record_decision(&Decision {
        common_dir: Some(watch.layout.common_dir.as_path()),
        origin: Some(watch.origin()),
        command,
        outcome,
        reason,
        detail,
        branch: Some(watch.branch_ref.as_str()),
    });
}

/// The tip the upstream tracking ref of the branch holds,
/// or the step that found no single plain upstream on a remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upstreamTip(watch: TipWatch, refs: Refs): string | Declined;
/// ```
fn upstream_tip(watch: &TipWatch, refs: &Refs) -> Result<String, Declined> {
    let Some(name) = watch.branch_ref.strip_prefix("refs/heads/") else {
        return Err(declined("upstream-no-branch-name", watch.branch_ref.clone()));
    };
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(watch.variables.as_slice());
    let Some(listed) = list_config(watch.layout.common_dir.as_path(), watch.layout.git_dir.as_path(), &scope, watch.variables.as_slice()) else {
        return Err(declined("upstream-config-unreadable", "the configuration listing was declined by the reader"));
    };
    let remotes: Vec<String> = branch_values(listed.as_slice(), name, "remote");
    let merges: Vec<String> = branch_values(listed.as_slice(), name, "merge");
    let ([remote], [merge]) = (remotes.as_slice(), merges.as_slice()) else {
        return Err(declined("upstream-not-single", format!("branch.{name}.remote has {} values and branch.{name}.merge has {}", remotes.len(), merges.len())));
    };
    if remote == "." {
        return Err(declined("upstream-is-local", format!("branch.{name}.remote is the repository itself")));
    }
    let Some(tracking) = tracking_ref(listed.as_slice(), remote, merge) else {
        return Err(declined("upstream-no-tracking-ref", format!("no remote.{remote}.fetch maps {merge}")));
    };
    let Some(tip) = refs.resolve(tracking.as_str()) else {
        return Err(declined("upstream-tracking-ref-missing", tracking));
    };
    return Ok(tip);
}

/// What:
///  Whether `target` is `from` or one of its ancestors,
/// within the walk budget;
/// nothing when the walk could not tell.
/// Why:
///   Ahead and not diverged is "the upstream tip is an ancestor of the branch tip";
/// the walk goes from the branch tip,
/// which
///  reaches the upstream within the commits the branch is ahead by,
/// and stops at the budget on a history that never does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reaches(database, from: string, target: string, budget: number): boolean | undefined;
/// ```
pub fn reaches(database: &ObjectDatabase, layout: &RepoLayout, from: &str, target: &str, budget: usize) -> Option<bool> {
    let start: Vec<u8> = from_hex(from.as_bytes(), layout.object_format)?;
    let wanted: Vec<u8> = from_hex(target.as_bytes(), layout.object_format)?;
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let mut queue: VecDeque<Vec<u8>> = VecDeque::new();
    seen.insert(start.clone());
    queue.push_back(start);
    let mut read: usize = 0;
    while let Some(name) = queue.pop_front() {
        if name == wanted {
            return Some(true);
        }
        if read >= budget {
            return None;
        }
        read += 1;
        let object = database.read(name.as_slice())?;
        if object.kind != ObjectKind::Commit {
            return None;
        }
        for parent in parse_commit(object.bytes.as_slice(), layout.object_format)?.parents {
            if seen.insert(parent.clone()) {
                queue.push_back(parent);
            }
        }
    }
    return Some(false);
}

/// What:
///  The tip to hand over after the command,
/// or the step that found the branch not a case the owner chose.
/// Why:
///   The branch must be the same one,
/// its tip must have moved forward from the old tip (a rewritten branch is a rebase),
///  and the new tip must be ahead of its upstream and not diverged:
/// the upstream tip is an ancestor of it and not it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tipToPushOrWhy(watch: TipWatch): string | Declined;
/// ```
pub fn tip_to_push_or_why(watch: &TipWatch) -> Result<String, Declined> {
    let (branch_ref, after): (String, String) = current_branch(&watch.layout)?;
    if branch_ref != watch.branch_ref {
        return Err(declined("branch-changed", format!("the command started on {} and ended on {branch_ref}", watch.branch_ref)));
    }
    if after == watch.before {
        return Err(declined("tip-unchanged", format!("{branch_ref} stayed at {after}")));
    }
    let refs: Refs = Refs::new(watch.layout.common_dir.as_path(), watch.layout.object_format.hex_length());
    let upstream: String = upstream_tip(watch, &refs)?;
    if upstream == after {
        return Err(declined("upstream-equals-tip", format!("{after} is already the upstream tip")));
    }
    let Some(database) = ObjectDatabase::open(watch.layout.common_dir.as_path(), None, watch.layout.object_format) else {
        return Err(declined("object-database-unopened", watch.layout.common_dir.display().to_string()));
    };
    let Some(grew) = reaches(&database, &watch.layout, after.as_str(), watch.before.as_str(), WALK_BUDGET) else {
        return Err(declined("walk-undecided-growth", format!("from {after} to the old tip {}", watch.before)));
    };
    if !grew {
        return Err(declined("not-grown", format!("the old tip {} is not an ancestor of {after}: a rewritten branch", watch.before)));
    }
    let Some(ahead) = reaches(&database, &watch.layout, after.as_str(), upstream.as_str(), WALK_BUDGET) else {
        return Err(declined("walk-undecided-ahead", format!("from {after} to the upstream tip {upstream}")));
    };
    if !ahead {
        return Err(declined("not-ahead", format!("the upstream tip {upstream} is not an ancestor of {after}: diverged")));
    }
    return Ok(after);
}

/// `tip_to_push_or_why` without the reason.
pub fn tip_to_push(watch: &TipWatch) -> Option<String> {
    return tip_to_push_or_why(watch).ok();
}

/// What:
///  After a command that succeeded:
/// hand the push over when the branch grew and is ahead of its upstream.
/// Why:
///   The person's command already finished;
/// a service that cannot be reached is said plainly and changes nothing.
///   The record is the same push record a commit leaves,
/// so the service pushes the branch by name and coalesces it.
///   Every ending is recorded with its reason (`push_trigger_log.rs`),
/// so a push that did not happen can be explained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await watch.handOverIfAhead(command, stderr);
/// ```
pub fn hand_over_if_ahead(watch: &TipWatch, command: &str, stderr: &mut dyn Write) {

    let branch: &str = watch.branch_ref.as_str();
    let tip: String = match tip_to_push_or_why(watch) {
        Ok(tip) => tip,
        Err(Declined { reason, detail }) => {
            record_ending(watch, command, Outcome::Declined, reason, detail.as_str());
            return;
        }
    };
    // The record is written before the route is read: a context with no route (a non-systemd session, a container, another
    //  `XDG_RUNTIME_DIR`) then leaves the push on disk for the next wake instead of losing it with the message.
    if let Err(failure) = write_push_record(watch.layout.worktree_root.as_path(), watch.layout.git_dir.as_path(), &crate::ref_name::RefName::from_text(branch), tip.as_str(), watch.before.as_str()) {
        record_ending(watch, command, Outcome::Declined, "handover-failed", render(&failure).as_str());
        // Terminal output error: the message is the only report of a push that was not handed over.
        let _written = writeln!(stderr, "{}", render(&failure));
        return;
    }
    let Some(Route { address, provision }) = route_from_environment(watch.real_git.as_path(), watch.layout.git_dir.as_path()) else {
        record_ending(watch, command, Outcome::Declined, "no-route", "no way to reach the commit service");
        // Terminal output error: the message is the only report of a push that waits for a service.
        // This is a merge or a pull that already happened, so the words say what became of its push, never that a commit was not made.
        let _written = writeln!(
            stderr,
            "monochromatic-git-policy-cli: {branch} was queued to be pushed, and no way to reach the commit service was found now. {} The push stays queued: the next command that reaches the service pushes it.",
            platform_remedy(platform_without_manager())
        );
        return;
    };
    let handoff: ServiceHandoff = ServiceHandoff {
        config: ClientConfig {
            address,
            activation: Activation::Managed,
            // The push record is durable, so the wake waits the hand-over's own greeting bound.
            start_wait: HAND_OVER_GREETING_WAIT,
            heartbeat_silence: HEARTBEAT_SILENCE,
            start_watch: provision.as_ref().map(manager_watch_for),
            keep_fresh: None, answer_ceiling: crate::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING, extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: HAND_OVER_GREETING_WAIT, waiting_notice_after: crate::wait_ceilings::waiting_notice_after(),
            units_record_dir: provision.as_ref().map(crate::service_provision_host_and_manager::units_record_dir_of),
        },
        repository: watch.layout.worktree_root.clone(),
        git_dir: watch.layout.git_dir.clone(),
        reference: crate::ref_name::RefName::from_text(watch.branch_ref.as_str()),
        parent: watch.before.clone(),
    };
    if let Err(failure) = hand_over_push(&handoff, tip.as_str(), stderr) {
        record_ending(watch, command, Outcome::Declined, "handover-failed", render(&failure).as_str());
        // Terminal output error: the message is the only report of a push that was not handed over.
        let _written = writeln!(stderr, "{}", render(&failure));
        return;
    }
    record_ending(watch, command, Outcome::HandedOver, "ahead-of-upstream", format!("pushed {tip} of {branch}").as_str());
}

/// The trigger's controls stay out of the release executable.
#[cfg(test)]
#[path = "auto_push_trigger_tests.rs"]
mod tests;
