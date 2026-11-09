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
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchBefore(forward); const code = await runGit(forward); if (code !== 0) await resolveAfter(watch, forward, stderr);
//! ```

use crate::child_environment::environment_value;
use crate::conflict_side_state::conflicted_paths;
use crate::diagnostic_log::{debug, warn};
use crate::index_writer_forward::CoordinatedForward;
use crate::merge_second_pass::{AttributeSource, Conflict, ContextRunner, GitRunner, Left, Merged, REPORT_MARK, Stage, Verdict, config_options, is_attributes_path, second_pass, write_blob};
use crate::merge_second_pass_finish::{finish, plan_finish, split_cmdline};
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass_tree::announce;
use crate::path_bytes::path_message_text;
use crate::private_storage::read_regular_file;
use crate::repo_read_index::{IndexEntry, parse_index};
use crate::repo_read_layout::{RepoLayout, read_layout_after_success_for};
use crate::merge_second_pass_worktree_install::install;
/// The records that put a conflict back,
/// which only the controls of this file still ask for.
#[cfg(test)]
use crate::merge_second_pass_worktree_install::restore_records;
use crate::merge_second_pass_worktree_unchanged::{Before, Held, Kept, held_paths, still_the_same};
use crate::merge_second_pass_worktree_words::{closing_words, ending_command, moved_words, unfinished_words, unreadable_words};
use crate::repo_read_object::to_hex;
use crate::transaction_git::GitContext;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeWorktree";

/// The largest index file this reads.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

pub(super) use crate::merge_second_pass_worktree_words::LINE_PREFIX;

/// The commands whose conflicts get the pass;
/// a pull's own merge or rebase stops in the same state the command would.
const OPERATIONS: [&[u8]; 5] = [b"merge", b"rebase", b"cherry-pick", b"revert", b"pull"];

/// What an operation had left unmerged before it ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Watch {
    /// Every path that had conflict stages in the index.
    unmerged_before: BTreeSet<Vec<u8>>,
}

/// The layout of the repository the command runs in,
/// when the in-process readers can describe it.
fn layout_of(forward: &CoordinatedForward) -> Option<RepoLayout> {
    if environment_value(forward.environment.as_slice(), "GIT_INDEX_FILE").is_some() {
        return None;
    }
    return read_layout_after_success_for(&forward.command.arguments[..forward.command.subcommand_index], forward.overlay.as_slice());
}

/// The index entries of the layout's worktree,
/// or nothing when the in-process reader declines the file.
fn index_entries(layout: &RepoLayout) -> Option<Vec<IndexEntry>> {
    let bytes: Vec<u8> = read_regular_file(layout.git_dir.join("index").as_path(), INDEX_LIMIT_BYTES).ok()?;
    return parse_index(bytes.as_slice(), layout.object_format);
}

/// What:
///  The conflicts of `entries` for the paths in `only`,
/// with their stages as object names.
/// Why:
///   The index lists the three stages of a path one after the other;
/// a missing stage stays `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictsOfIndex(entries: IndexEntry[], only: Set<string>): Conflict[];
/// ```
pub fn conflicts_of_index(entries: &[IndexEntry], only: &BTreeSet<Vec<u8>>) -> Vec<Conflict> {
    let mut conflicts: Vec<Conflict> = Vec::new();
    for entry in entries {
        if !(entry.stage != 0 && only.contains(&entry.path)) {
            continue;
        }
        let starts_new: bool = match conflicts.last() {
            Some(last) => last.path != entry.path,
            None => true,
        };
        if starts_new {
            conflicts.push(Conflict { path: entry.path.clone(), base: None, ours: None, theirs: None });
        }
        let Some(conflict) = conflicts.last_mut() else { continue };
        let side: Option<Stage> = Some(Stage { mode: entry.mode, oid: to_hex(entry.id.as_slice()) });
        match entry.stage {
            1 => conflict.base = side,
            2 => conflict.ours = side,
            3 => conflict.theirs = side,
            _ => {}
        }
    }
    return conflicts;
}

/// What:
///  Note which paths are unmerged before the command runs,
/// when the command is one the pass serves.
/// Why:
///   Only a conflict this command creates is the pass's to resolve.
/// When the index cannot be read in process the pass is off
///  for this run:
/// it never guesses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchBefore(forward: CoordinatedForward): Watch | undefined;
/// ```
pub fn watch_before(forward: &CoordinatedForward) -> Option<Watch> {
    if !forward.command.is_builtin() || !OPERATIONS.contains(&forward.command.word()) {
        return None;
    }
    let layout: RepoLayout = layout_of(forward)?;
    let entries: Vec<IndexEntry> = index_entries(&layout)?;
    return Some(Watch { unmerged_before: conflicted_paths(entries.as_slice()) });
}

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
pub fn resolve_after(watched: Option<&Watch>, forward: &CoordinatedForward, code: i32, stderr: &mut dyn Write) -> Option<i32> {
    let watch = watched?;
    if code == 0 {
        return None;
    }
    let Some(layout) = layout_of(forward) else {
        debug(TAG, "the layout cannot be read in process after the command; no second pass");
        return None;
    };
    let Some(entries) = index_entries(&layout) else {
        debug(TAG, "the index cannot be read in process after the command; no second pass");
        return None;
    };
    let created: BTreeSet<Vec<u8>> = conflicted_paths(entries.as_slice()).difference(&watch.unmerged_before).cloned().collect();
    if created.is_empty() {
        return None;
    }
    let conflicts: Vec<Conflict> = conflicts_of_index(entries.as_slice(), &created);
    // What the worktree held at the moment the command left these conflicts, read before the merge looks at any stage: an edit made while the merge runs must not become the baseline it is compared with.
    let held: BTreeMap<Vec<u8>, Held> = held_paths(layout.worktree_root.as_path(), &created);
    let context: GitContext = GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: config_options(&forward.command.arguments[..forward.command.subcommand_index]),
        common_dir: Some(layout.common_dir.clone()),
    };
    // From the top of the worktree, so the paths of the stages mean what they say.
    let runner: ContextRunner<'_> = ContextRunner { context: &context, cwd: layout.worktree_root.as_path(), without_prefix: false, object_directory: None, environment: Vec::new() };
    // A rule file the merge left conflicted is Git's marker file in the worktree, so no rule of it can be read: the pass re-merges nothing rather than trust a line between the markers.
    let mut attributes_conflict: bool = false;
    for path in conflicted_paths(entries.as_slice()) {
        if is_attributes_path(path.as_slice()) {
            attributes_conflict = true;
        }
    }
    let source: AttributeSource<'_> = if attributes_conflict { AttributeSource::Unknown } else { AttributeSource::Invocation };
    // The pass and the write of its files await every Git start they make, so the person's command reaches them through one entry of the runtime.
    let parts: SecondPassParts = match crate::lazy_runtime::run_on_runtime(second_pass_parts(&runner, conflicts.as_slice(), source, &layout, &held, stderr)) {
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
    for line in announce(LINE_PREFIX, (CLIENT_IDENTIFIER, operation.as_str()), installed.as_slice()) {
        text.push_str(line.as_str());
        text.push('\n');
    }
    let remaining: usize = conflicted_paths(entries.as_slice()).len().saturating_sub(installed.len());
    // Every conflicted file resolved: end the operation the way a clean run does, if the options say how.
    let planned = if remaining == 0 {
        match crate::lazy_runtime::run_on_runtime(branch_merge_options(&runner, layout.git_dir.as_path(), &forward.command)) {
            Ok(Some(configured)) => plan_finish(&forward.command, forward.environment.as_slice(), layout.git_dir.as_path(), configured.as_slice()),
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
    text.push_str(format!("{LINE_PREFIX}{REPORT_MARK}: staged as resolved by the second merge pass: {listed}. ").as_str());
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
        text.push_str(format!("The wrapper finishes the {operation} below, the way Git finishes a clean run.\n").as_str());
    }
    if let Err(error) = stderr.write_all(text.as_bytes()) {
        debug(TAG, format!("standard error did not take the list of merged files: {error}").as_str());
    }
    let plan = planned?;
    let ended: i32 = finish(&plan, &forward.command, forward.environment.as_slice(), stderr);
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
        let after: String = format!("{}\n", unfinished_words(operation.as_str(), listed.as_str(), conflicted_now.as_deref(), ending));
        if let Err(error) = stderr.write_all(after.as_bytes()) {
            debug(TAG, format!("standard error did not take the line about the unfinished operation: {error}").as_str());
        }
    }
    return Some(ended);
}

/// What the awaited part of a forwarded pass came to.
/// Why:
///   The files it installed and the paths it left are what the report names,
/// and `None` says the conflicts stay as Git left them,
/// with the reason already given.
struct SecondPassParts {
    /// The files staged as resolved and written to the worktree.
    files: Vec<Merged>,
    /// The paths another command moved while the pass computed its merge,
    /// in words.
    moved: Vec<String>,
    /// The paths nobody could compare,
    /// in words.
    unreadable: Vec<String>,
}

/// What:
///  Re-merge the conflicts this command created and write the files whose every part resolves,
/// awaiting every Git start the pass makes.
/// Why:
///   The pass awaits its starts (`merge_second_pass_git_runner.rs`),
/// so a person's command reaches it through one entry of the runtime,
/// and the reconcile's pass,
/// which runs as a task,
/// needs no entry at all.
/// The order of the steps and every reason a path is left are the synchronous face's:
/// the merge is computed first,
/// the index is read again after it,
/// and a path another command moved is named and never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parts = await secondPassParts({ runner, conflicts, source, layout, held, stderr }); // { files, moved, unreadable } | undefined
/// ```
async fn second_pass_parts(
    runner: &ContextRunner<'_>,
    conflicts: &[Conflict],
    source: AttributeSource<'_>,
    layout: &RepoLayout,
    held: &BTreeMap<Vec<u8>, Held>,
    stderr: &mut dyn Write,
) -> Option<SecondPassParts> {
    let verdicts: Vec<Verdict> = match second_pass(runner, conflicts, source).await {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("the second merge pass could not run, so the conflicts stay as Git left them: {reason}").as_str());
            return None;
        }
    };
    let mut merged: Vec<Merged> = Vec::new();
    for verdict in verdicts {
        match verdict {
            Verdict::Merged(file) => merged.push(*file),
            Verdict::Left { path, why } => debug(TAG, format!("{} stays conflicted: {}", path_message_text(path.as_slice()), left_words(&why)).as_str()),
        }
    }
    if merged.is_empty() {
        return None;
    }
    let mut oids: Vec<String> = Vec::new();
    for file in &merged {
        match write_blob(runner, file.bytes.as_slice()).await {
            Ok(oid) => oids.push(oid),
            Err(reason) => {
                warn(TAG, format!("the merged blob of {} could not be written, so the conflicts stay: {reason}", path_message_text(file.path.as_slice())).as_str());
                return None;
            }
        }
    }
    // The index is read again after the merge was computed: a path another command resolved and staged holds no stages of the merge's.
    let Some(entries_now) = index_entries(layout) else {
        warn(TAG, "the index cannot be read again before the merged files are installed, so the conflicts stay as they are");
        return None;
    };
    let before: Before<'_> = Before { worktree_root: layout.worktree_root.as_path(), entries: entries_now.as_slice(), held };
    let kept: Kept = still_the_same(&before, merged, oids);
    let mut moved_names: Vec<String> = Vec::new();
    for path in &kept.moved {
        moved_names.push(path_message_text(path.as_slice()));
    }
    let mut unreadable_names: Vec<String> = Vec::new();
    for path in &kept.unreadable {
        unreadable_names.push(path_message_text(path.as_slice()));
    }
    if kept.files.is_empty() {
        warn(TAG, format!("every merged file moved or could not be compared while the pass computed its merge, so they stay as they are now: {}{}", moved_names.join(", "), unreadable_names.join(", ")).as_str());
        // The person must still be told which file the pass did not touch, and which of the two reasons it was.
        let mut alone: String = String::new();
        if !moved_names.is_empty() {
            alone.push_str(moved_words(moved_names.as_slice()).as_str());
        }
        if !unreadable_names.is_empty() {
            alone.push_str(unreadable_words(unreadable_names.as_slice()).as_str());
        }
        let told: String = format!("{}\n", alone.trim_end());
        if let Err(error) = stderr.write_all(told.as_bytes()) {
            debug(TAG, format!("standard error did not take the line about the files the pass left alone: {error}").as_str());
        }
        return None;
    }
    let files: Vec<Merged> = kept.files;
    let file_oids: Vec<String> = kept.oids;
    if let Err(reason) = install(runner, files.as_slice(), file_oids.as_slice()).await {
        warn(TAG, format!("the merged files could not be installed, so the conflicts stay as Git left them: {reason}").as_str());
        return None;
    }
    return Some(SecondPassParts { files, moved: moved_names, unreadable: unreadable_names });
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
async fn branch_merge_options(runner: &ContextRunner<'_>, git_dir: &std::path::Path, command: &crate::forwarded_command::ResolvedCommand) -> Option<Vec<std::ffi::OsString>> {
    if !matches!(command.word(), b"merge" | b"pull") {
        return Some(Vec::new());
    }
    let head: String = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let Some(branch) = head.trim().strip_prefix("ref: refs/heads/") else { return Some(Vec::new()) };
    let key: String = format!("branch.{branch}.mergeoptions");
    return match runner.run(&["config", "--get", key.as_str()], None, None).await {
        Ok(reply) if reply.code == Some(1) => Some(Vec::new()),
        Ok(reply) if reply.code == Some(0) => split_cmdline(String::from_utf8_lossy(&reply.stdout).trim_end_matches('\n')),
        Ok(_) => None,
        Err(reason) => {
            debug(TAG, format!("{key} could not be read ({reason}), so the finish is not made").as_str());
            None
        }
    };
}

/// Why a path stayed conflicted,
/// in words for the diagnostic log.
fn left_words(why: &Left) -> String {
    return format!("{why:?}");
}

/// Controls for the index reading and the restore records.
#[cfg(test)]
#[path = "merge_second_pass_worktree_tests.rs"]
mod tests;
