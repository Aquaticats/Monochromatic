//! What:
//!  Hands a push to the commit service after a command other than `git commit` added commits to the branch: a merge, a
//!  pull's merge, a cherry-pick, a revert or an applied patch.
//! Why:
//!  A clean `git merge` used to stay local until the next commit pushed it. The owner chose (2026-11-02) that any command
//!  adding commits queues the push as a commit does, when the branch tip moved and the branch is now ahead of its upstream
//!  and not diverged from it, and that a rebase does not, because a rebased branch would need a forced push.
//!  The check reads the branch, its upstream tracking ref and the commit parents from the files, with no Git process:
//!  the tip is read before and after the forwarded command, and the push is handed over only when the old tip and the
//!  upstream tip are both ancestors of the new tip. The old tip being an ancestor is what leaves a rebase (a rewritten
//!  branch) out, `pull --rebase` and `pull.rebase` included, without reading the configuration for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchTip(worktree); await runGit(args); if (exitCode === 0) await watch.handOverIfAhead();
//! ```

#![cfg(unix)]

use super::git_system_config::with_real_git;
use super::repo_read_config_list::list_config;
use super::repo_read_config_scan::ScopeEnvironment;
use super::repo_read_layout::{RepoLayout, read_layout_in};
use super::repo_read_object::{ObjectKind, from_hex};
use super::repo_read_odb::ObjectDatabase;
use super::repo_read_query_refs::tracking_ref;
use super::repo_read_refs::{RefValue, Refs};
use super::repo_read_tree::parse_commit;
use super::service_client::{Activation, COMMIT_WAIT, ClientConfig, START_WAIT};
use super::service_handoff::{ServiceHandoff, hand_over_push};
use super::service_messages::{ClientFailure, render};
use super::service_provision::platform_without_manager;
use super::service_route::{Route, route_from_environment};
use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The most commits the walk from a tip reads before it gives up and hands nothing over: a branch far ahead, or a history
/// that never reaches the upstream, is not worth a long walk after every merge.
pub const WALK_BUDGET: usize = 4096;

/// What the branch looked like before the command: where it pointed and where its repository is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TipWatch = { layout: RepoLayout; variables: [string, string][]; realGit: string; branchRef: string; before: string };
/// ```
#[derive(Clone, Debug)]
pub struct TipWatch {
    /// Where the worktree and its repository are.
    layout: RepoLayout,
    /// The environment Git would see, which the configuration reader needs for its scopes.
    variables: Vec<(OsString, OsString)>,
    /// The real Git the service will push with.
    real_git: PathBuf,
    /// The branch `HEAD` named.
    branch_ref: String,
    /// The tip of that branch before the command.
    before: String,
}

/// The branch `HEAD` names and its tip, or nothing for a detached `HEAD`, an unborn branch or a layout the reader declines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentBranch(layout): { branchRef: string; tip: string } | undefined;
/// ```
fn current_branch(layout: &RepoLayout) -> Option<(String, String)> {
    let refs: Refs = Refs::new(layout.common_dir.as_path(), layout.object_format.hex_length());
    let RefValue::Symbolic(target) = refs.lookup_head(layout.git_dir.as_path())? else {
        return None;
    };
    if !target.starts_with("refs/heads/") {
        return None;
    }
    let tip: String = refs.resolve(target.as_str())?;
    return Some((target, tip));
}

/// What:
///  Start watching the branch of a worktree before a command runs.
/// Why:
///   The command may be a merge or a pull, which move the tip; what the tip was is what a rebase is told apart by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchTip(worktreeRoot, realGit, overlay, environment): TipWatch | undefined;
/// ```
pub fn watch_tip(
    worktree_root: &Path,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
    environment: &[(OsString, OsString)],
) -> Option<TipWatch> {
    // The readers learn the system configuration file from the real Git the child would run.
    let named: Vec<(OsString, OsString)> = with_real_git(overlay, real_git);
    let layout: RepoLayout = read_layout_in(worktree_root, named.as_slice())?;
    let (branch_ref, before): (String, String) = current_branch(&layout)?;
    let mut variables: Vec<(OsString, OsString)> = environment.to_vec();
    variables.extend(named);
    return Some(TipWatch { layout, variables, real_git: real_git.to_path_buf(), branch_ref, before });
}

/// The tip the upstream tracking ref of the branch holds, or nothing without a single plain upstream on a remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upstreamTip(watch: TipWatch, refs: Refs): string | undefined;
/// ```
fn upstream_tip(watch: &TipWatch, refs: &Refs) -> Option<String> {
    let name: &str = watch.branch_ref.strip_prefix("refs/heads/")?;
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(watch.variables.as_slice());
    let listed = list_config(watch.layout.common_dir.as_path(), watch.layout.git_dir.as_path(), &scope, watch.variables.as_slice())?;
    let values = |suffix: &str| -> Vec<String> {
        let key: String = format!("branch.{name}.{suffix}");
        return listed
            .iter()
            .filter(|item| { return item.entry.key == key; })
            .filter_map(|item| { return item.entry.value.as_deref().and_then(|bytes| { return std::str::from_utf8(bytes).ok().map(str::to_owned); }); })
            .collect();
    };
    let remotes: Vec<String> = values("remote");
    let merges: Vec<String> = values("merge");
    let ([remote], [merge]) = (remotes.as_slice(), merges.as_slice()) else {
        return None;
    };
    if remote == "." {
        return None;
    }
    let tracking: String = tracking_ref(listed.as_slice(), remote, merge)?;
    return refs.resolve(tracking.as_str());
}

/// What:
///  Whether `target` is `from` or one of its ancestors, within the walk budget; nothing when the walk could not tell.
/// Why:
///   Ahead and not diverged is "the upstream tip is an ancestor of the branch tip"; the walk goes from the branch tip, which
///  reaches the upstream within the commits the branch is ahead by, and stops at the budget on a history that never does.
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
///  The tip to hand over after the command, or nothing when the branch is not a case the owner chose.
/// Why:
///   The branch must be the same one, its tip must have moved forward from the old tip (a rewritten branch is a rebase),
///  and the new tip must be ahead of its upstream and not diverged: the upstream tip is an ancestor of it and not it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tipToPush(watch: TipWatch): string | undefined;
/// ```
pub fn tip_to_push(watch: &TipWatch) -> Option<String> {
    let (branch_ref, after): (String, String) = current_branch(&watch.layout)?;
    if branch_ref != watch.branch_ref || after == watch.before {
        return None;
    }
    let refs: Refs = Refs::new(watch.layout.common_dir.as_path(), watch.layout.object_format.hex_length());
    let upstream: String = upstream_tip(watch, &refs)?;
    if upstream == after {
        return None;
    }
    let database: ObjectDatabase = ObjectDatabase::open(watch.layout.common_dir.as_path(), None, watch.layout.object_format)?;
    let grew: bool = reaches(&database, &watch.layout, after.as_str(), watch.before.as_str(), WALK_BUDGET)?;
    let ahead: bool = reaches(&database, &watch.layout, after.as_str(), upstream.as_str(), WALK_BUDGET)?;
    return (grew && ahead).then_some(after);
}

/// What:
///  After a command that succeeded: hand the push over when the branch grew and is ahead of its upstream.
/// Why:
///   The person's command already finished; a service that cannot be reached is said plainly and changes nothing.
///   The record is the same push record a commit leaves, so the service pushes the branch by name and coalesces it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await watch.handOverIfAhead(stderr);
/// ```
pub fn hand_over_if_ahead(watch: &TipWatch, stderr: &mut dyn Write) {
    let Some(tip) = tip_to_push(watch) else {
        return;
    };
    let Some(Route { address, .. }) = route_from_environment(watch.real_git.as_path(), watch.layout.git_dir.as_path()) else {
        // Terminal output error: the message is the only report of a push that was not handed over.
        let _written = writeln!(stderr, "{}", render(&ClientFailure::NoActivation(platform_without_manager())));
        return;
    };
    let handoff: ServiceHandoff = ServiceHandoff {
        config: ClientConfig { address, activation: Activation::Managed, start_wait: START_WAIT, commit_wait: COMMIT_WAIT },
        repository: watch.layout.worktree_root.clone(),
        git_dir: watch.layout.git_dir.clone(),
        reference: watch.branch_ref.clone(),
        parent: watch.before.clone(),
    };
    if let Err(failure) = hand_over_push(&handoff, tip.as_str(), stderr) {
        // Terminal output error: the message is the only report of a push that was not handed over.
        let _written = writeln!(stderr, "{}", render(&failure));
    }
}

/// The trigger's controls stay out of the release executable.
#[cfg(test)]
#[path = "auto_push_trigger_tests.rs"]
mod tests;
