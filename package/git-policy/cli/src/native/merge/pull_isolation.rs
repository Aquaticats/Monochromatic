//! What:
//!  A `git pull` that merges,
//! turned into the fetch it starts with and the `git merge FETCH_HEAD` it ends with,
//! so the merge
//!  half goes through the same isolated merge as a typed `git merge`.
//! Why:
//!  Native Git's `git pull` runs its merge as a child of Git (`builtin/pull.c`,
//! `run_merge`,
//! `cmd.git_cmd = 1`),
//! so the
//!  wrapper never saw it and the merge kept native Git's failures:
//! a staged index,
//! a held `index.lock`,
//! a late ref
//!  update.
//! The owner's rules are "Auto merging should never fail."
//! and "Manual merge should never fail too.";
//! the
//!  pull's own decisions (which options,
//! whether it rebases,
//! whether `HEAD` moved under the fetch,
//! how many heads
//!  `FETCH_HEAD` holds) are mirrored here only where they can be mirrored exactly,
//! and every other pull is left to native Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planPull(region); // undefined for anything not read exactly
//! if (plan && mergeIsStated(plan, config)) { await fetch(plan); return merge(['--no-edit', 'FETCH_HEAD']); }
//! return nativePull();
//! ```

/// The paths a fast-forward writes,
/// and whether anything of the person's stands at them.
use crate::auto_push_dirty_paths::diff_paths;
use crate::merge_isolation_a_fast_forward_in_the_way::paths_in_the_way;
/// The wait for any program's hold on the index lock,
/// and for the commit service's own hold.
use crate::index_holder_wait::wait_for_index_release_here;
use crate::service_index_wait::wait_for_service_hold_here;
/// The inherited environment,
/// which decides whether the reflog message is already set.
use crate::child_environment::environment_value;
/// The untracked files the fetch's branch move would overwrite,
/// kept before the failure is worded,
/// and the words that name every copy.
use crate::pull_untracked_paths_in_the_way::{UntrackedInTheWay, keep_the_untracked_paths_in_the_way, recovery_words};
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The command a pull becomes.
use crate::forwarded_command::ResolvedCommand;
/// The forward being run and how it ended.
use crate::index_writer_forward::{CoordinatedForward, CoordinatedRun};
/// The heads `FETCH_HEAD` lists for merging.
use crate::merge_isolation_message::fetch_head_merge_heads;
/// Running real Git for the queries.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// Running real Git as a child that owns the terminal,
/// for the fetch.
use crate::worktree_copy_forward::{ForwardedRun, run_wrapper_own_git};
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
const IN_PROGRESS_MARKERS: [&str; 6] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply", "sequencer"];

/// The command line of a pull and the halves its options go to,
/// which a file of its own holds because the pull that runs them is at its code-line limit.
pub use crate::pull_isolation_the_command_line::{PullPlan, plan_pull};
use crate::pull_isolation_the_command_line::NATIVE_ONLY_SETTINGS;

/// What:
///  What the rebase settings say about one pull.
/// Why:
///   `pull.rebase` and `branch.<name>.rebase` are booleans or one of `merges` and `interactive`,
/// and only a value Git reads as no is a merge.
/// A value that is a rebase,
/// or one Git refuses,
/// keeps the whole pull native,
/// which words that refusal itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RebaseStatement = 'unstated' | 'merge' | 'notMerge';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebaseStatement {
    /// Neither setting is present,
    /// so a typed `--ff` or `--no-ff` is the only thing that states a merge.
    Unstated,
    /// The setting Git reads says no:
    /// this pull merges.
    Merge,
    /// The setting says a rebase,
    /// a mode this does not mirror,
    /// or a value Git refuses.
    NotMerge,
}

/// What:
///  Whether the command line and the settings state that this pull merges and mirror nothing else the pull would do.
/// Why:
///   Git's pull merges when `--no-rebase` is typed or `pull.rebase` / `branch.<name>.rebase` say no,
/// or,
/// with neither set,
/// when
///  a way of reconciling is typed (`--ff`,
/// `--no-ff`);
/// with nothing stated it refuses divergent branches with its advice,
/// and
///  that refusal stays native Git's.
/// `pull.ff` is left to Git,
/// because with `--no-rebase` it is rewritten by rules of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeIsStated(plan: PullPlan, settings: string[], rebase: RebaseStatement): boolean;
/// ```
pub fn merge_is_stated(plan: &PullPlan, settings: &[String], rebase: RebaseStatement) -> bool {
    // The settings that add steps this does not mirror.
    for key in settings {
        if NATIVE_ONLY_SETTINGS.contains(&key.as_str()) {
            return false;
        }
    }
    // `pull.ff` is Git's to combine with the rest.
    if !plan.fast_forward_stated {
        for key in settings {
            if key == "pull.ff" {
                return false;
            }
        }
    }
    if plan.merge_stated {
        return true;
    }
    return match rebase {
        RebaseStatement::Merge => true,
        RebaseStatement::NotMerge => false,
        RebaseStatement::Unstated => plan.fast_forward_stated,
    };
}

/// What:
///  Run a read-only query against the invocation's repository.
/// Why:
///   Each query is one Git start;
/// the answer is the output whatever its status,
/// so a caller reads the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ask(context: GitContext, args: string[]): Promise<GitOutput | undefined>;
/// ```
fn ask(context: &GitContext, arguments: &[&str]) -> Option<GitOutput> {
    // `.ok()` turns a failed start into nothing, which the callers treat as "leave the pull to native Git".
    return run_git(context, &GitRequest::new(Path::new("."), arguments)).ok();
}

/// What:
///  What one query for a rebase setting answered.
/// Why:
///   The branch's setting is looked at before the general one,
/// and only a setting that is not there lets the next one decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettingAnswer = { kind: 'read'; statement: RebaseStatement } | { kind: 'absent' } | { kind: 'unreadable' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingAnswer {
    /// Git read the setting.
    Read(RebaseStatement),
    /// The setting is not present.
    Absent,
    /// Git refused the value,
    /// or the query could not start.
    Unreadable,
}

/// What:
///  What Git's own reading of one rebase setting says about the pull.
/// Why:
///   `git config --bool --get` answers with the value Git's pull reads:
/// the last of several settings,
/// in every boolean spelling Git accepts (`0`,
/// `00`,
/// `0x0`,
/// `0k`,
/// the empty value,
/// `false`,
/// `no`,
/// `off`).
/// A rebase mode (`merges`,
/// `interactive`) is no boolean,
/// so Git refuses it here and words that refusal itself when the pull runs natively.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rebaseAnswer(context, key): Promise<SettingAnswer>;
/// ```
fn rebase_answer(context: &GitContext, key: &str) -> SettingAnswer {
    // A query Git could not start states nothing this may act on.
    let Some(found) = ask(context, &["config", "--bool", "--get", key]) else {
        return SettingAnswer::Unreadable;
    };
    return match found.code {
        // Git prints `true` or `false` for a value it reads as a boolean and nothing else.
        Some(0) => {
            let text: String = String::from_utf8_lossy(&found.stdout).trim().to_owned();
            let statement: RebaseStatement = if text == "false" { RebaseStatement::Merge } else { RebaseStatement::NotMerge };
            SettingAnswer::Read(statement)
        }
        // Status 1 is Git's answer for a setting that is not there.
        Some(1) => SettingAnswer::Absent,
        // A value Git refuses, or a query that failed: native Git runs the pull and words the refusal.
        _ => SettingAnswer::Unreadable,
    };
}

/// What:
///  What the rebase settings of one branch say about a pull.
/// Why:
///   `config_get_rebase` reads `branch.<name>.rebase` before `pull.rebase`,
/// so the branch's own setting decides whenever it is present;
/// asking Git for the value keeps the last of several settings and every spelling of no,
/// which a reader of the raw text got wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rebaseStatement(context, branch): Promise<RebaseStatement>;
/// ```
fn rebase_statement(context: &GitContext, branch: &str) -> RebaseStatement {
    let branch_key: String = format!("branch.{branch}.rebase");
    for key in [branch_key.as_str(), "pull.rebase"] {
        match rebase_answer(context, key) {
            SettingAnswer::Read(statement) => return statement,
            SettingAnswer::Absent => {}
            SettingAnswer::Unreadable => return RebaseStatement::NotMerge,
        }
    }
    return RebaseStatement::Unstated;
}

/// What:
///  The settings that add steps the split pull does not mirror,
/// as the keys present.
/// Why:
///   One `git config` call answers for all of them,
/// and only their presence decides anything,
/// so no value is read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pullSettings(context): Promise<string[] | undefined>;
/// ```
fn pull_settings(context: &GitContext) -> Option<Vec<String>> {
    let output: GitOutput = ask(
        context,
        &[
            "config",
            "--null",
            "--get-regexp",
            "^(pull\\.(ff|autostash)|submodule\\.recurse|fetch\\.recursesubmodules)$",
        ],
    )?;
    // Status 1 means no setting matched.
    if output.code != Some(0) && output.code != Some(1) {
        return None;
    }
    // The keys present, built entry by entry.
    let mut settings: Vec<String> = Vec::new();
    for entry in output.stdout.split(is_nul_byte) {
        if entry.is_empty() {
            continue;
        }
        let text: String = String::from_utf8_lossy(entry).into_owned();
        let key: &str = match text.split_once('\n') {
            // A value-less setting is present too, and only the key is read.
            Some((key, _value)) => key,
            None => text.as_str(),
        };
        settings.push(key.to_owned());
    }
    return Some(settings);
}

/// What:
///  The first line of a query's output,
/// trimmed,
/// when the query succeeded.
/// Why:
///   Names and object names are one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (output) => output.code === 0 ? output.stdout.trim() : undefined;
/// ```
fn line_of(output: Option<GitOutput>) -> Option<String> {
    let found: GitOutput = output?;
    if found.code != Some(0) {
        return None;
    }
    return Some(String::from_utf8_lossy(&found.stdout).trim().to_owned());
}

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
/// type PullStep = { kind: 'native' } | { kind: 'ended'; run: CoordinatedRun } | { kind: 'merge'; forward: CoordinatedForward; isolate: boolean };
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
        /// Whether the merge goes through the isolated merge,
        /// which carries the person's own work over it:
        /// a merge that makes a commit always does,
        /// and a fast-forward does when something of the person's stands at the paths it writes.
        isolate: bool,
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
fn merge_forward(forward: &CoordinatedForward, plan: &PullPlan, overlay: Vec<(OsString, OsString)>) -> CoordinatedForward {
    // The global options stay in front of the subcommand, as the caller typed them.
    let mut arguments: Vec<OsString> = forward.command.arguments[..forward.command.subcommand_index].to_vec();
    arguments.push(OsString::from("merge"));
    arguments.extend(plan.merge.iter().cloned());
    arguments.push(OsString::from("FETCH_HEAD"));
    let command: ResolvedCommand = ResolvedCommand {
        arguments: arguments.clone(),
        subcommand: Some(OsString::from("merge")),
        ..forward.command.clone()
    };
    return CoordinatedForward { arguments, command, overlay, ..forward.clone() };
}

/// Whether one byte is the NUL byte that ends each entry of a `-z` listing.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
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
pub fn pull_step(forward: &CoordinatedForward, real_index: &Path) -> PullStep {
    let Some(plan) = plan_pull(forward.command.region()) else {
        debug(TAG, "native Git runs this pull: an option or operand is not read exactly");
        return PullStep::Native;
    };
    let (git_dir, root): (&Path, &Path) = match &forward.identity {
        WorktreeIdentity::MainWorktree { git_dir, worktree_root, .. } | WorktreeIdentity::LinkedWorktree { git_dir, worktree_root, .. } => {
            (git_dir.as_path(), worktree_root.as_path())
        }
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
        debug(TAG, "native Git runs this pull: another operation is in progress, or the repository has submodules");
        return PullStep::Native;
    }
    let context: GitContext = GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: forward.command.arguments[..forward.command.subcommand_index].to_vec(),
        common_dir: None,
    };
    let (Some(branch), Some(settings), Some(orig_head)) = (
        line_of(ask(&context, &["symbolic-ref", "--quiet", "--short", "HEAD"])),
        pull_settings(&context),
        line_of(ask(&context, &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])),
    ) else {
        debug(TAG, "native Git runs this pull: no branch is checked out, or its head is unborn");
        return PullStep::Native;
    };
    // The rebase setting is read only when the command line does not already state a merge, because that start would
    // change nothing then: `--no-rebase` outranks every setting.
    let rebase: RebaseStatement = if plan.merge_stated { RebaseStatement::Unstated } else { rebase_statement(&context, branch.as_str()) };
    if !merge_is_stated(&plan, settings.as_slice(), rebase) {
        debug(TAG, "native Git runs this pull: it is not stated to merge, or a setting adds steps this does not mirror");
        return PullStep::Native;
    }
    // An unmerged entry makes native Git's pull refuse with its own advice.
    let unmerged: Option<GitOutput> = ask(&context, &["ls-files", "--unmerged", "-z"]);
    let unmerged_unclear: bool = match unmerged.as_ref() {
        Some(found) => found.code != Some(0) || !found.stdout.is_empty(),
        None => true,
    };
    if unmerged_unclear {
        debug(TAG, "native Git runs this pull: the index holds unmerged entries");
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
    let mut fetch: Vec<OsString> = forward.command.arguments[..forward.command.subcommand_index].to_vec();
    fetch.push(OsString::from("fetch"));
    fetch.push(OsString::from("--update-head-ok"));
    fetch.extend(plan.fetch.iter().cloned());
    fetch.extend(plan.places.iter().cloned());
    debug(TAG, "the fetch of a pull that merges runs, then the merge goes through the isolated merge");
    match run_wrapper_own_git(forward.real_git.as_path(), fetch.as_slice(), overlay.as_slice()) {
        ForwardedRun::Ran(0) => {}
        // A failed fetch ends the pull with status 1, as `cmd_pull` does, and a fetch Git was killed in is a failed one.
        ForwardedRun::Ran(_) | ForwardedRun::Killed(_) => return PullStep::Ended(CoordinatedRun::Ran(1)),
        ForwardedRun::Failed { stderr, code } => return PullStep::Ended(CoordinatedRun::Failed { stderr, code }),
    }
    // The fetch can update the branch that is checked out; Git then moves the working tree along with it before it merges.
    let Some(curr_head) = line_of(ask(&context, &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])) else {
        return PullStep::Native;
    };
    if curr_head != orig_head {
        let _noted = std::io::stderr().write_all(
            format!("warning: fetch updated the current branch head.\nfast-forwarding your working tree from\ncommit {orig_head}.\n").as_bytes(),
        );
        // The update takes `index.lock` like any writer; the pull waits for a holder, bounded and naming it, where Git's step dies.
        wait_for_service_hold_here(real_index, "pull's merge");
        wait_for_index_release_here(real_index, "pull's merge");
        let moved: Option<GitOutput> = ask(&context, &["read-tree", "-u", "-m", orig_head.as_str(), curr_head.as_str()]);
        let move_failed: bool = match moved {
            Some(found) => found.code != Some(0),
            None => true,
        };
        if move_failed {
            // Git's own recovery words say to save `git diff <old head>`, which holds no untracked file, and the reset
            // they offer deletes the untracked file that made this move fail, so the wrapper keeps those bytes itself
            // and names every copy before it prints the words (`pull_untracked_paths_in_the_way`).
            let in_the_way: UntrackedInTheWay = keep_the_untracked_paths_in_the_way(&context, root, git_dir, orig_head.as_str(), curr_head.as_str());
            return PullStep::Ended(CoordinatedRun::Failed { stderr: recovery_words(orig_head.as_str(), &in_the_way), code: 128 });
        }
    }
    let fetched: String = std::fs::read_to_string(git_dir.join("FETCH_HEAD")).unwrap_or_default();
    let heads: Vec<String> = fetch_head_merge_heads(fetched.as_str());
    if heads.len() != 1 {
        debug(TAG, "native Git runs the whole pull again: FETCH_HEAD does not list exactly one head to merge");
        return PullStep::Native;
    }
    // One start tells a merge that makes a commit from a fast-forward (the merge base is `HEAD`) and from nothing to merge (the
    // merge base is the head). A merge that makes a commit is worth the isolated attempt. A fast-forward is native Git's own
    // while nothing of the person's stands at the paths it writes, and is carried here when something does, as a typed merge
    // is (`SPEC.md`, "Carry edits forward"), so the paths it writes are what decide that.
    let bases: Option<GitOutput> = ask(&context, &["merge-base", "--all", "HEAD", heads[0].as_str()]);
    let fast_forward: bool = match bases {
        Some(found) => {
            let listed: String = String::from_utf8_lossy(&found.stdout).trim().to_owned();
            found.code == Some(0) && listed == curr_head
        }
        // A question that could not be answered leaves the whole pull to native Git.
        None => return PullStep::Native,
    };
    // `--ff-only` states that the branch may move only by a fast-forward, and a merge that cannot fast-forward is native Git's,
    //  whose refusal and advice (`fatal: Not possible to fast-forward, aborting.`) are its own; the pull then runs again as a
    //  whole, and its fetch is idempotent.
    if !fast_forward && plan.fast_forward_only {
        debug(TAG, "native Git runs the whole pull again: --ff-only was typed and the merge cannot fast-forward");
        return PullStep::Native;
    }
    let isolate: bool = !fast_forward || match the_paths_the_pull_writes(&context, curr_head.as_str(), heads[0].as_str()) {
        // Nothing of the person's stands there, so the `git merge FETCH_HEAD` child is the whole answer and no isolated
        //  attempt is wasted on the ordinary case.
        Some(written) => !matches!(paths_in_the_way(&context, root, written.as_slice()), Some(found) if found.is_empty()),
        // A difference that could not be read is no answer, so the merge is carried and asks again.
        None => true,
    };
    return PullStep::Merge { forward: Box::new(merge_forward(forward, &plan, overlay)), isolate };
}

/// What:
///  The paths a pull's merge writes,
///  from the difference between the branch and the fetched head.
/// Why:
///   A fast-forward writes exactly the paths the two commits differ in,
///  and the question whether anything of the
///  person's stands at those paths is what decides whether native Git's own merge is the whole answer.
///  A difference that gave no answer is
///  reported as such,
///  so the caller carries the merge rather than leave the person's file to whatever Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = diffPaths((await git(['diff-tree', '-r', '--name-only', '-z', '--no-renames', head, fetched])).stdout);
/// ```
fn the_paths_the_pull_writes(context: &GitContext, head: &str, fetched: &str) -> Option<Vec<Vec<u8>>> {
    let found: GitOutput = ask(context, &["diff-tree", "-r", "--name-only", "-z", "--no-renames", head, fetched])?;
    if !found.succeeded() {
        return None;
    }
    return Some(diff_paths(found.stdout.as_slice()));
}

/// Controls for the pull's reading stay out of the release executable.
#[cfg(test)]
#[path = "pull_isolation_tests.rs"]
mod tests;
