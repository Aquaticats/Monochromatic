//! What:
//!   A `git pull` that merges,
//!   turned into the fetch it starts with and the `git merge FETCH_HEAD` it ends with,
//!   so the merge half goes through the same isolated merge as a typed `git merge`.
//! Why:
//!  Native Git's `git pull` runs its merge as a child of Git (`builtin/pull.c`,
//! `run_merge`,
//! `cmd.git_cmd = 1`),
//! so the wrapper never saw it and the merge kept native Git's failures:
//! a staged index,
//! a held `index.lock`,
//! a late ref update.
//! The owner's rules are "Auto merging should never fail."
//! and "Manual merge should never fail too.";
//! the pull's own decisions are mirrored here only where they can be mirrored exactly,
//! and every other pull is left to native Git.
//! `pull_isolation_options.rs` reads the command line into the two halves,
//! and `pull_isolation_merge_stated.rs` decides whether the pull is stated to merge;
//! this file reads the repository,
//! runs the fetch,
//! and hands the merge on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planPull(region); // undefined for anything not read exactly
//! if (plan && mergeIsStated(plan, settings, rebase)) { await fetch(plan); return merge(['--no-edit', 'FETCH_HEAD']); }
//! return nativePull();
//! ```

/// The inherited environment,
/// which decides whether the reflog message is already set.
use crate::child_environment::environment_value;
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The command a pull becomes.
use crate::forwarded_command::ResolvedCommand;
/// The wait for any program's hold on the index lock,
/// and for the commit service's own hold.
use crate::index_holder_wait::wait_for_index_release_here;
/// The forward being run and how it ended.
use crate::index_writer_forward::{
    CoordinatedForward,
    CoordinatedRun,
};
/// The heads `FETCH_HEAD` lists for merging.
use crate::merge_isolation_message::fetch_head_merge_heads;
/// Whether the pull is stated to merge,
/// and the settings and rebase readings that say so.
use crate::pull_isolation_merge_stated::{
    RebaseStatement,
    ask,
    line_of,
    merge_is_stated,
    pull_settings,
    rebase_statement,
};
/// The plan a pull's command line was read into,
/// and the reading of that line.
use crate::pull_isolation_options::{
    PullPlan,
    plan_pull,
};
/// The untracked files the fetch's branch move would overwrite,
/// kept before the failure is worded,
/// and the words that name every copy.
use crate::pull_untracked_paths_in_the_way::{
    UntrackedInTheWay,
    keep_the_untracked_paths_in_the_way,
    recovery_words,
};
use crate::service_index_wait::wait_for_service_hold_here;
/// Running real Git for the queries.
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
/// Running real Git as a child that owns the terminal,
/// for the fetch.
use crate::worktree_copy_forward::{
    ForwardedRun,
    run_wrapper_own_git,
};
/// The repository shape of the invocation.
use crate::worktree_identity::WorktreeIdentity;
/// Owned operating-system text.
use std::ffi::OsString;
/// Standard error for the pull's own warning.
use std::io::Write;
/// Borrowed filesystem paths.
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "pullIsolation";

/// Files and directories of the Git directory that show another operation is in progress;
/// native Git's pull reports those.
const IN_PROGRESS_MARKERS: &[&str] = &[
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "rebase-merge",
    "rebase-apply",
    "sequencer",
];

/// What:
///  How a pull is handled:
/// native Git's whole,
/// already ended,
/// or the merge that follows its fetch.
/// Why:
///   The caller either forwards the pull unchanged,
/// returns the end,
/// or runs the merge forward it is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PullStep = { kind: 'native' } | { kind: 'ended'; run: CoordinatedRun } | { kind: 'merge'; forward: CoordinatedForward; makesCommit: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PullStep {
    /// Native Git runs the whole pull.
    Native,
    /// The pull ended here (its fetch failed,
    /// or the working tree could not follow the fetch).
    Ended(CoordinatedRun),
    /// The fetch is done and this merge forward finishes the pull;
    /// boxed because a forward is large.
    Merge {
        /// The `git merge FETCH_HEAD` that ends the pull.
        forward: Box<CoordinatedForward>,
        /// Whether the merge creates a commit,
        /// and so is worth the isolated attempt;
        /// a fast-forward or a merge of nothing is not.
        makes_commit: bool,
    },
}

/// What:
///  The merge forward that finishes a pull:
/// `git merge <options> FETCH_HEAD` with the invocation's global options.
/// Why:
///   That is the command `run_merge` starts,
/// so the isolated merge,
/// the lock wait and the second pass handle it as they
///  handle a typed merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeForward(forward: CoordinatedForward, plan: PullPlan, overlay): CoordinatedForward;
/// ```
fn merge_forward(
    forward: &CoordinatedForward,
    plan: &PullPlan,
    overlay: Vec<(OsString, OsString)>,
) -> CoordinatedForward {
    // The global options stay in front of the subcommand, as the caller typed them.
    let mut arguments: Vec<OsString> =
        forward.command.arguments[..forward.command.subcommand_index].to_vec();
    arguments.push(OsString::from("merge"));
    arguments.extend(plan.merge.iter().cloned());
    arguments.push(OsString::from("FETCH_HEAD"));
    let command: ResolvedCommand = ResolvedCommand {
        arguments: arguments.clone(),
        subcommand: Some(OsString::from("merge")),
        ..forward.command.clone()
    };
    return CoordinatedForward {
        arguments,
        command,
        overlay,
        ..forward.clone()
    };
}

/// What:
///  Decide what to do with a `git pull`,
/// and run its fetch when the merge half can be completed here;
/// `real_index` is the index
///  the working tree update waits on.
/// Why:
///   Everything before the fetch decides from the command line,
/// the settings and the repository's state,
/// so a pull this
///  cannot mirror starts nothing;
/// after the fetch,
/// a `FETCH_HEAD` that does not list exactly one head goes back to native
///  Git's whole pull,
/// which fetches again and words the refusal itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pullStep(forward: CoordinatedForward, realIndex: string): Promise<PullStep>;
/// ```
pub fn pull_step(
    forward: &CoordinatedForward,
    real_index: &Path,
) -> PullStep {
    let Some(plan) = plan_pull(forward.command.region()) else {
        debug(
            TAG,
            "native Git runs this pull: an option or operand is not read exactly",
        );
        return PullStep::Native;
    };
    let (git_dir, root): (&Path, &Path) = match &forward.identity {
        WorktreeIdentity::MainWorktree {
            git_dir,
            worktree_root,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            git_dir,
            worktree_root,
            ..
        } => (git_dir.as_path(), worktree_root.as_path()),
        _ => return PullStep::Native,
    };
    let mut operation_in_progress: bool = false;
    for name in IN_PROGRESS_MARKERS {
        if git_dir.join(name).exists() {
            operation_in_progress = true;
            break;
        }
    }
    if operation_in_progress || root.join(".gitmodules").exists() {
        debug(
            TAG,
            "native Git runs this pull: another operation is in progress, or the repository has submodules",
        );
        return PullStep::Native;
    }
    let context: GitContext = GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: forward.command.arguments[..forward.command.subcommand_index].to_vec(),
        common_dir: None,
    };
    let (Some(branch), Some(settings), Some(orig_head)) = (
        line_of(ask(
            &context,
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
        )),
        pull_settings(&context),
        line_of(ask(
            &context,
            &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"],
        )),
    ) else {
        debug(
            TAG,
            "native Git runs this pull: no branch is checked out, or its head is unborn",
        );
        return PullStep::Native;
    };
    // The rebase setting is read only when the command line does not already state a merge, because that start would
    // change nothing then: `--no-rebase` outranks every setting.
    let rebase: RebaseStatement = if plan.merge_stated {
        RebaseStatement::Unstated
    } else {
        rebase_statement(&context, branch.as_str())
    };
    if !merge_is_stated(&plan, settings.as_slice(), rebase) {
        debug(
            TAG,
            "native Git runs this pull: it is not stated to merge, or a setting adds steps this does not mirror",
        );
        return PullStep::Native;
    }
    // An unmerged entry makes native Git's pull refuse with its own advice.
    let unmerged: Option<GitOutput> = ask(&context, &["ls-files", "--unmerged", "-z"]);
    let unmerged_unclear: bool = match unmerged.as_ref() {
        Some(found) => found.code != Some(0) || !found.stdout.is_empty(),
        None => true,
    };
    if unmerged_unclear {
        debug(
            TAG,
            "native Git runs this pull: the index holds unmerged entries",
        );
        return PullStep::Native;
    }
    // Git names the pull in the reflog of every ref the pull moves, unless the caller already did.
    let mut overlay: Vec<(OsString, OsString)> = forward.overlay.clone();
    if environment_value(forward.environment.as_slice(), "GIT_REFLOG_ACTION").is_none() {
        let mut action: OsString = OsString::from("pull");
        for token in forward.command.region() {
            action.push(" ");
            action.push(token);
        }
        overlay.push((OsString::from("GIT_REFLOG_ACTION"), action));
    }
    // `git fetch --update-head-ok <options> <repository> <refspecs>`, as `run_fetch` builds it.
    let mut fetch: Vec<OsString> =
        forward.command.arguments[..forward.command.subcommand_index].to_vec();
    fetch.push(OsString::from("fetch"));
    fetch.push(OsString::from("--update-head-ok"));
    fetch.extend(plan.fetch.iter().cloned());
    fetch.extend(plan.places.iter().cloned());
    debug(
        TAG,
        "the fetch of a pull that merges runs, then the merge goes through the isolated merge",
    );
    match run_wrapper_own_git(
        forward.real_git.as_path(),
        fetch.as_slice(),
        overlay.as_slice(),
    ) {
        ForwardedRun::Ran(0) => {}
        // A failed fetch ends the pull with status 1, as `cmd_pull` does, and a fetch Git was killed in is a failed one.
        ForwardedRun::Ran(_) | ForwardedRun::Killed(_) => {
            return PullStep::Ended(CoordinatedRun::Ran(1));
        }
        ForwardedRun::Failed {
            stderr,
            code,
        } => {
            return PullStep::Ended(CoordinatedRun::Failed {
                stderr,
                code,
            });
        }
    }
    // The fetch can update the branch that is checked out; Git then moves the working tree along with it before it merges.
    let Some(curr_head) = line_of(ask(
        &context,
        &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"],
    )) else {
        return PullStep::Native;
    };
    if curr_head != orig_head {
        let _noted = std::io::stderr().write_all(
            format!("warning: fetch updated the current branch head.\nfast-forwarding your working tree from\ncommit {orig_head}.\n").as_bytes(),
        );
        // The update takes `index.lock` like any writer; the pull waits for a holder, bounded and naming it, where Git's step dies.
        wait_for_service_hold_here(real_index, "pull's merge");
        wait_for_index_release_here(real_index, "pull's merge");
        let moved: Option<GitOutput> = ask(
            &context,
            &[
                "read-tree",
                "-u",
                "-m",
                orig_head.as_str(),
                curr_head.as_str(),
            ],
        );
        let move_failed: bool = match moved {
            Some(found) => found.code != Some(0),
            None => true,
        };
        if move_failed {
            // Git's own recovery words say to save `git diff <old head>`, which holds no untracked file, and the reset
            // they offer deletes the untracked file that made this move fail, so the wrapper keeps those bytes itself
            // and names every copy before it prints the words (`pull_untracked_paths_in_the_way`).
            let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(
                &context,
                root,
                git_dir,
                orig_head.as_str(),
                curr_head.as_str(),
            );
            return PullStep::Ended(CoordinatedRun::Failed {
                stderr: recovery_words(orig_head.as_str(), &in_the_way),
                code: 128,
            });
        }
    }
    // The fetched head is read as a plain file and to a bound, so a named pipe at its name sends this pull to native Git
    //  rather than holding it for a writer that may never come; a file past the bound is no list of heads to merge here.
    let fetched: String = crate::plain_file::read_plain_text_at(
        git_dir.join("FETCH_HEAD").as_path(),
        crate::plain_file::SMALL_FILE_LIMIT,
    )
    .unwrap_or_default();
    let heads: Vec<String> = fetch_head_merge_heads(fetched.as_str());
    if heads.len() != 1 {
        debug(
            TAG,
            "native Git runs the whole pull again: FETCH_HEAD does not list exactly one head to merge",
        );
        return PullStep::Native;
    }
    // One start tells a merge that makes a commit from a fast-forward (the merge base is `HEAD`) and from nothing to merge (the
    // merge base is the head); only the first is worth the isolated attempt, whose queries the other two would only waste.
    let bases: Option<GitOutput> = ask(
        &context,
        &["merge-base", "--all", "HEAD", heads[0].as_str()],
    );
    let makes_commit: bool = match bases {
        Some(found) => {
            let listed: String = String::from_utf8_lossy(&found.stdout).trim().to_owned();
            found.code != Some(0) || (listed != curr_head && listed != heads[0])
        }
        None => true,
    };
    return PullStep::Merge {
        forward: Box::new(merge_forward(forward, &plan, overlay)),
        makes_commit,
    };
}

/// Controls for the pull's reading stay out of the release executable.
#[cfg(test)]
#[path = "pull_isolation_tests.rs"]
mod tests;
