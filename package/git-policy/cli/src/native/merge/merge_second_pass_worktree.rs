//! What:
//!  The word-level second pass after a forwarded merge,
//! rebase,
//! cherry-pick or revert that Git stopped with a conflict:
//!  each conflict that command created is re-merged from the index stages,
//! and a file whose every part resolves is
//!  written to the worktree,
//! staged as resolved,
//! and listed on standard error and in the journal.
//! Why:
//!  The owner decided on 2026-11-03 (`doc/planning/cli-git-merge-auto-resolution.md`) that a person's merge gets this pass.
//!  The pass touches only the paths that were not conflicted before the command ran,
//! so a file the person is resolving by
//!  hand (it is still unmerged when a later command fails on it) is never replaced,
//! and it reads the stage blobs,
//! never the
//!  marked-up worktree file,
//! so `merge.conflictStyle` does not matter.
//! Git's exit status is kept:
//! the operation is not finished,
//!  the person finishes it (`git commit`,
//! `git rebase --continue`,
//! ...),
//! and a script that tests the status still stops.
//!  A file that does not resolve,
//! and a file whose staging fails,
//! stays exactly as Git left it.
//!
//! The paths the pass watches, and the conflicts the index holds for them,
//! live in `merge_second_pass_worktree_watch.rs`;
//! the awaited re-merge of those conflicts lives in `merge_second_pass_worktree_remerge.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchBefore(forward); const code = await runGit(forward); if (code !== 0) await resolveAfter(watch, forward, stderr);
//! ```

use crate::conflict_side_state::conflicted_paths;
use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::index_writer_forward::CoordinatedForward;
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass::{
    AttributeSource,
    Conflict,
    ContextRunner,
    GitRunner,
    Merged,
    REPORT_MARK,
    config_options,
    is_attributes_path,
};
use crate::merge_second_pass_finish::{
    finish,
    plan_finish,
    split_cmdline,
};
use crate::merge_second_pass_tree::announce;
use crate::merge_second_pass_worktree_unchanged::{
    Held,
    held_paths,
};
use crate::merge_second_pass_worktree_words::{
    closing_words,
    ending_command,
    moved_words,
    unfinished_words,
    unreadable_words,
};
use crate::path_bytes::path_message_text;
use crate::transaction_git::GitContext;
use std::collections::{
    BTreeMap,
    BTreeSet,
};
use std::io::Write;

use crate::merge_second_pass_worktree_remerge::{
    SecondPassParts,
    second_pass_parts,
};
/// The paths an operation had already left unmerged when it started,
/// and the conflicts the index holds for the paths it created,
/// reachable at this module's path as they were before the split.
pub use crate::merge_second_pass_worktree_watch::{
    Watch,
    conflicts_of_index,
    watch_before,
};
use crate::merge_second_pass_worktree_watch::{
    index_entries,
    layout_of,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeWorktree";

pub(super) use crate::merge_second_pass_worktree_words::LINE_PREFIX;

/// What:
///  After a forwarded operation that exited with a non-zero status,
/// re-merge the conflicts it created.
/// Why:
///   See the module comment.
/// Every outcome that is not a resolution goes to the diagnostic log with its reason,
/// and the
///  operation's own output and status stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveAfter(watch: Watch | undefined, forward: CoordinatedForward, code: number, stderr: Writable): Promise<void>;
/// ```
pub fn resolve_after(
    watched: Option<&Watch>,
    forward: &CoordinatedForward,
    code: i32,
    stderr: &mut dyn Write,
) -> Option<i32> {
    let watch = watched?;
    if code == 0 {
        return None;
    }
    let Some(layout) = layout_of(forward) else {
        debug(
            TAG,
            "the layout cannot be read in process after the command; no second pass",
        );
        return None;
    };
    let Some(entries) = index_entries(&layout) else {
        debug(
            TAG,
            "the index cannot be read in process after the command; no second pass",
        );
        return None;
    };
    let created: BTreeSet<Vec<u8>> = conflicted_paths(entries.as_slice())
        .difference(&watch.unmerged_before)
        .cloned()
        .collect();
    if created.is_empty() {
        return None;
    }
    let conflicts: Vec<Conflict> = conflicts_of_index(entries.as_slice(), &created);
    // What the worktree held at the moment the command left these conflicts, read before the merge looks at any stage: an edit made while the merge runs must not become the baseline it is compared with.
    let held: BTreeMap<Vec<u8>, Held> = held_paths(layout.worktree_root.as_path(), &created);
    let context: GitContext = GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: config_options(
            &forward.command.arguments[..forward.command.subcommand_index],
        ),
        common_dir: Some(layout.common_dir.clone()),
    };
    // From the top of the worktree, so the paths of the stages mean what they say.
    let runner: ContextRunner<'_> = ContextRunner {
        context: &context,
        cwd: layout.worktree_root.as_path(),
        without_prefix: false,
        repository: Vec::new(),
        object_directory: None,
        environment: Vec::new(),
    };
    // A rule file the merge left conflicted is Git's marker file in the worktree, so no rule of it can be read: the pass re-merges nothing rather than trust a line between the markers.
    let mut attributes_conflict: bool = false;
    for path in conflicted_paths(entries.as_slice()) {
        if is_attributes_path(path.as_slice()) {
            attributes_conflict = true;
        }
    }
    let source: AttributeSource<'_> = if attributes_conflict {
        AttributeSource::Unknown
    } else {
        AttributeSource::Invocation
    };
    // The pass and the write of its files await every Git start they make, so the person's command reaches them through one entry of the runtime.
    let parts: SecondPassParts = match crate::lazy_runtime::run_on_runtime(second_pass_parts(
        &runner,
        conflicts.as_slice(),
        source,
        &layout,
        &held,
        stderr,
    )) {
        Ok(Some(found)) => found,
        Ok(None) => return None,
        Err(refusal) => {
            warn(TAG, format!("the second merge pass could not be run, so the conflicts stay as Git left them: {refusal}").as_str());
            return None;
        }
    };
    let installed: Vec<Merged> = parts.files;
    let moved_names: Vec<String> = parts.moved;
    let unreadable_names: Vec<String> = parts.unreadable;
    let operation: String = String::from_utf8_lossy(forward.command.word()).into_owned();
    let mut text: String = String::new();
    for line in announce(
        LINE_PREFIX,
        (CLIENT_IDENTIFIER, operation.as_str()),
        installed.as_slice(),
    ) {
        text.push_str(line.as_str());
        text.push('\n');
    }
    let remaining: usize = conflicted_paths(entries.as_slice())
        .len()
        .saturating_sub(installed.len());
    // Every conflicted file resolved: end the operation the way a clean run does, if the options say how.
    let planned = if remaining == 0 {
        match crate::lazy_runtime::run_on_runtime(branch_merge_options(
            &runner,
            layout.git_dir.as_path(),
            &forward.command,
        )) {
            Ok(Some(configured)) => plan_finish(
                &forward.command,
                forward.environment.as_slice(),
                layout.git_dir.as_path(),
                configured.as_slice(),
            ),
            // A setting that cannot be read is no set of options, and the finish is then not made.
            Ok(None) | Err(_) => None,
        }
    } else {
        None
    };
    let mut names: Vec<String> = Vec::new();
    for file in &installed {
        names.push(path_message_text(file.path.as_slice()));
    }
    let listed: String = names.join(", ");
    // What the pass staged is said whether or not a finish follows: a finish that fails must not be the only account of it.
    text.push_str(
        format!(
            "{LINE_PREFIX}{REPORT_MARK}: staged as resolved by the second merge pass: {listed}. "
        )
        .as_str(),
    );
    if !moved_names.is_empty() {
        // A path the pass left is said in the same breath, so the person knows why it still conflicts.
        text.push_str(moved_words(moved_names.as_slice()).as_str());
    }
    if !unreadable_names.is_empty() {
        // A path nobody could compare is its own reason, never the one about an edit.
        text.push_str(unreadable_words(unreadable_names.as_slice()).as_str());
    }
    if planned.is_none() {
        text.push_str(format!("{}\n", closing_words(operation.as_str(), remaining)).as_str());
    } else {
        text.push_str(
            format!(
                "The wrapper finishes the {operation} below, the way Git finishes a clean run.\n"
            )
            .as_str(),
        );
    }
    if let Err(error) = stderr.write_all(text.as_bytes()) {
        debug(
            TAG,
            format!("standard error did not take the list of merged files: {error}").as_str(),
        );
    }
    let plan = planned?;
    let ended: i32 = finish(
        &plan,
        &forward.command,
        forward.environment.as_slice(),
        stderr,
    );
    if ended != 0 {
        // The ending step's words are its own; the merge changed before it ran, and the person must be told what the index holds now.
        // An index that cannot be read again is not a clean one, so the line says the state could not be read.
        let mut conflicted_now: Option<Vec<String>> = None;
        if let Some(entries_after) = index_entries(&layout) {
            let mut left_now: Vec<String> = Vec::new();
            for path in conflicted_paths(entries_after.as_slice()) {
                left_now.push(path_message_text(path.as_slice()));
            }
            conflicted_now = Some(left_now);
        }
        let ending: &'static str = ending_command(layout.git_dir.as_path(), operation.as_str());
        let after: String = format!(
            "{}\n",
            unfinished_words(
                operation.as_str(),
                listed.as_str(),
                conflicted_now.as_deref(),
                ending
            )
        );
        if let Err(error) = stderr.write_all(after.as_bytes()) {
            debug(
                TAG,
                format!(
                    "standard error did not take the line about the unfinished operation: {error}"
                )
                .as_str(),
            );
        }
    }
    return Some(ended);
}

/// What:
///  The words of `branch.<name>.mergeoptions` for the branch that is checked out,
/// which Git parses before the command line of a
///  merge (`builtin/merge.c` lines 1387 to 1409),
/// or nothing when the value cannot be read as Git reads it.
/// Why:
///   A merge or a pull applies the setting to the merge it runs,
/// so the finish reads the same words and applies them to the
///  options it reads;
/// `--no-commit` there stops the finished merge as a clean one stops.
///   A detached `HEAD` has no branch and so no setting (`merge.c` lines 1388 to 1394).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function branchMergeOptions(runner, gitDir, command): Promise<string[] | undefined>;
/// ```
async fn branch_merge_options(
    runner: &ContextRunner<'_>,
    git_dir: &std::path::Path,
    command: &crate::forwarded_command::ResolvedCommand,
) -> Option<Vec<std::ffi::OsString>> {
    if !matches!(command.word(), b"merge" | b"pull") {
        return Some(Vec::new());
    }
    let head: String = crate::plain_file::read_plain_text_at(
        git_dir.join("HEAD").as_path(),
        crate::plain_file::SMALL_FILE_LIMIT,
    )
    .ok()?;
    let Some(branch) = head.trim().strip_prefix("ref: refs/heads/") else {
        return Some(Vec::new());
    };
    let key: String = format!("branch.{branch}.mergeoptions");
    return match runner
        .run(&["config", "--get", key.as_str()], None, None)
        .await
    {
        Ok(reply) if reply.code == Some(1) => Some(Vec::new()),
        Ok(reply) if reply.code == Some(0) => {
            split_cmdline(String::from_utf8_lossy(&reply.stdout).trim_end_matches('\n'))
        }
        Ok(_) => None,
        Err(reason) => {
            debug(
                TAG,
                format!("{key} could not be read ({reason}), so the finish is not made").as_str(),
            );
            None
        }
    };
}

/// Controls for the index reading and the restore records.
#[cfg(test)]
#[path = "merge_second_pass_worktree_tests.rs"]
mod tests;
