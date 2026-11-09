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
use crate::merge_second_pass::{AttributeSource, Conflict, ContextRunner, GitReply, GitRunner, Left, Merged, REPORT_MARK, Stage, Verdict, config_options, is_attributes_path, resolved_record, second_pass, write_blob};
use crate::merge_second_pass_finish::{finish, plan_finish, split_cmdline};
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass_tree::announce;
use crate::path_bytes::{os_from_path_bytes, path_message_text};
use crate::private_storage::read_regular_file;
use crate::repo_read_index::{IndexEntry, parse_index};
use crate::repo_read_layout::{RepoLayout, read_layout_after_success_for};
use crate::repo_read_object::to_hex;
use crate::transaction_git::GitContext;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Tag used for diagnostics from this module.
const TAG: &str = "wordMergeWorktree";

/// The largest index file this reads.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

/// The line prefix of every line this module writes.
const LINE_PREFIX: &str = "monochromatic-git-policy-cli: ";

/// The commands whose conflicts get the pass;
/// a pull's own merge or rebase stops in the same state the command would.
const OPERATIONS: [&[u8]; 5] = [b"merge", b"rebase", b"cherry-pick", b"revert", b"pull"];

/// The stages in the order `update-index --index-info` takes them back.
const STAGE_NUMBERS: [u8; 3] = [1, 2, 3];

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

/// The records that put a conflict's stages back into the index:
/// remove the path,
/// then each stage it had.
fn restore_records(conflict: &Conflict) -> Vec<u8> {
    let mut width: usize = 0;
    if let Some(stage) = [&conflict.base, &conflict.ours, &conflict.theirs].into_iter().flatten().next() {
        width = stage.oid.len();
    }
    let mut records: Vec<u8> = format!("0 {}\t", "0".repeat(width)).into_bytes();
    records.extend_from_slice(conflict.path.as_slice());
    records.push(0);
    for (number, side) in STAGE_NUMBERS.iter().zip([&conflict.base, &conflict.ours, &conflict.theirs]) {
        if let Some(stage) = side {
            records.extend_from_slice(format!("{:06o} {} {number}\t", stage.mode, stage.oid).as_bytes());
            records.extend_from_slice(conflict.path.as_slice());
            records.push(0);
        }
    }
    return records;
}

/// The reply of a start that must have exited with status 0.
fn succeeded(started: Result<GitReply, String>, what: &str) -> Result<(), String> {
    let reply: GitReply = started?;
    if reply.code == Some(0) {
        return Ok(());
    }
    return Err(format!("{what} failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
}

/// What:
///  Stage the merged files as resolved and write them to the worktree;
/// put the conflicts back if the second step fails.
/// Why:
///   Staging first means a failed write leaves an index that says resolved over a file with markers,
/// so the stages are
///  restored then and the person finds the conflict exactly as Git left it.
/// The checkout applies the repository's own
///  filters (line endings),
/// which a plain file write would skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function install(runner, merged: Merged[], oids: string[]): Promise<void>;
/// ```
fn install(runner: &dyn GitRunner, merged: &[Merged], oids: &[String]) -> Result<(), String> {
    let mut staging: Vec<u8> = Vec::new();
    let mut paths: Vec<u8> = Vec::new();
    for (file, oid) in merged.iter().zip(oids.iter()) {
        staging.extend_from_slice(resolved_record(file.mode, oid.as_str(), file.path.as_slice()).as_slice());
        paths.extend_from_slice(file.path.as_slice());
        paths.push(0);
    }
    succeeded(runner.run(&["update-index", "-z", "--index-info"], Some(staging.as_slice()), None), "git update-index")?;
    let written: Result<(), String> = succeeded(runner.run(&["checkout-index", "--force", "-z", "--stdin"], Some(paths.as_slice()), None), "git checkout-index");
    if let Err(reason) = written {
        let mut restore: Vec<u8> = Vec::new();
        for file in merged {
            restore.extend_from_slice(restore_records(&file.stages).as_slice());
        }
        if let Err(again) = succeeded(runner.run(&["update-index", "-z", "--index-info"], Some(restore.as_slice()), None), "git update-index") {
            return Err(format!("{reason}; putting the conflicts back also failed: {again}"));
        }
        return Err(reason);
    }
    return Ok(());
}

/// What a worktree path held when the conflicts were read.
/// Why:
///   The pass computes a merge from the index stages and then replaces the worktree file.
/// A file that moved in between holds
///  bytes the merge was never computed from,
/// and writing over it discards them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Held {
    /// The file's length in bytes.
    size: u64,
    /// Its last modification time,
    /// when the system reports one.
    modified: Option<SystemTime>,
    /// Its device and inode on a system that numbers them,
    /// so a replacement of the same length at the same time still shows.
    numbers: Option<(u64, u64)>,
}

/// What the worktree holds at `path` now,
/// or nothing when it cannot be read.
fn worktree_state(path: &Path) -> Option<Held> {
    let metadata: std::fs::Metadata = std::fs::metadata(path).ok()?;
    #[cfg(unix)]
    let numbers: Option<(u64, u64)> = {
        use std::os::unix::fs::MetadataExt;
        Some((metadata.dev(), metadata.ino()))
    };
    #[cfg(not(unix))]
    let numbers: Option<(u64, u64)> = None;
    return Some(Held { size: metadata.len(), modified: metadata.modified().ok(), numbers });
}

/// The path a stage names, below the top of the worktree.
fn path_under(root: &Path, path: &[u8]) -> PathBuf {
    return root.join(os_from_path_bytes(path));
}

/// What the worktree holds at each of `paths`.
fn held_paths(root: &Path, paths: &BTreeSet<Vec<u8>>) -> BTreeMap<Vec<u8>, Option<Held>> {
    let mut held: BTreeMap<Vec<u8>, Option<Held>> = BTreeMap::new();
    for path in paths {
        held.insert(path.clone(), worktree_state(path_under(root, path).as_path()));
    }
    return held;
}

/// What the install must find before it replaces anything.
pub struct Before<'a> {
    /// The top of the worktree,
    /// whose paths the stages name.
    worktree_root: &'a Path,
    /// The index as it is now,
    /// read again after the merge was computed.
    entries: &'a [IndexEntry],
    /// What the worktree held at each path when the conflicts were read.
    held: &'a BTreeMap<Vec<u8>, Option<Held>>,
}

/// What the install may replace,
/// and the paths it may not.
struct Kept {
    /// The merged files whose index stages and worktree file are still what the merge was computed from.
    files: Vec<Merged>,
    /// Their blobs,
    /// in the same order.
    oids: Vec<String>,
    /// The paths that moved,
    /// which the install leaves as it finds them.
    moved: Vec<Vec<u8>>,
}

/// What:
///  Whether the index stages and the worktree file of `file` are still what the merge was computed from.
/// Why:
///   Another command that resolved the conflict and staged it leaves a stage 0 entry where the merge read three stages,
/// and a file a person
///  edited is not the file Git left;
/// either way the merged bytes were computed from something else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (file) => stagesOf(index, file.path) === file.stages && stat(file.path) === recorded;
/// ```
fn still_holds(before: &Before<'_>, file: &Merged) -> bool {
    let only: BTreeSet<Vec<u8>> = [file.path.clone()].into_iter().collect();
    let now: Vec<Conflict> = conflicts_of_index(before.entries, &only);
    if now.as_slice() != [file.stages.clone()] {
        return false;
    }
    let Some(recorded) = before.held.get(&file.path) else { return false };
    return *recorded == worktree_state(path_under(before.worktree_root, file.path.as_slice()).as_path());
}

/// What:
///  Keep only the merged files whose path still holds what the merge was computed from,
/// and name the paths that moved.
/// Why:
///   The person's own resolution,
/// a commit another session made,
/// or any other command may touch the conflict between the read and the
///  install;
/// the pass then leaves that path exactly as it is and says which one it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stillTheSame(before, merged, oids) { ... } // { files, oids, moved }
/// ```
fn still_the_same(before: &Before<'_>, merged: Vec<Merged>, oids: Vec<String>) -> Kept {
    let mut kept: Kept = Kept { files: Vec::new(), oids: Vec::new(), moved: Vec::new() };
    for (file, oid) in merged.into_iter().zip(oids) {
        if still_holds(before, &file) {
            kept.files.push(file);
            kept.oids.push(oid);
        } else {
            kept.moved.push(file.path.clone());
        }
    }
    return kept;
}

/// The sentence that names the paths the pass left because they moved.
fn moved_words(moved: &[String]) -> String {
    return format!(
        "{LINE_PREFIX}{REPORT_MARK}: changed while the second merge pass computed its merge, so it is left as it is now: {}. ",
        moved.join(", ")
    );
}

/// The sentence after the list of staged files.
fn closing_words(operation: &str, remaining: usize) -> String {
    let rest: String = if remaining == 0 {
        format!("no conflicted file is left; finish the {operation} the way Git said above")
    } else {
        format!("resolve the files Git still lists as conflicted, then finish the {operation} the way Git said above")
    };
    return format!("Review a merged file with `git diff --cached -- <file>`; {rest}.");
}

/// What:
///  The command that ends an operation the wrapper did not finish.
/// Why:
///   The line after a failed finish must name a way on that works for the operation at hand;
/// Git ends a merge with `git commit`,
/// and a
///  rebase,
/// a cherry-pick and a revert with their own `--continue`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endingCommand = (operation: string) => ({rebase: 'git rebase --continue', 'cherry-pick': 'git cherry-pick --continue', revert: 'git revert --continue'}[operation] ?? 'git commit');
/// ```
fn ending_command(operation: &str) -> &'static str {
    return match operation {
        "rebase" => "`git rebase --continue`",
        "cherry-pick" => "`git cherry-pick --continue`",
        "revert" => "`git revert --continue`",
        _ => "`git commit`",
    };
}

/// What:
///  The line that says what is left after a finish that did not end the operation.
/// Why:
///   The ending step's own text is about that step alone:
/// a commit that could not be made ends with
///  "Nothing in the repository changed.",
/// which is true of the commit and false of the merge,
/// whose conflicts the pass
///  replaced with staged files before the commit started.
/// The last word a person reads must name the state they will find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${LINE_PREFIX}${REPORT_MARK}: the ${operation} did not finish: ${listed} stays staged as resolved and the conflict is gone; run ${endingCommand(operation)} to end the ${operation}.`
/// ```
fn unfinished_words(operation: &str, listed: &str) -> String {
    return format!(
        "{LINE_PREFIX}{REPORT_MARK}: the {operation} did not finish: {listed} stays staged as resolved and the conflict is gone; run {} to end the {operation}.",
        ending_command(operation)
    );
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
    let context: GitContext = GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: config_options(&forward.command.arguments[..forward.command.subcommand_index]),
        common_dir: Some(layout.common_dir.clone()),
    };
    // From the top of the worktree, so the paths of the stages mean what they say.
    let runner: ContextRunner<'_> = ContextRunner { context: &context, cwd: layout.worktree_root.as_path(), without_prefix: false, object_directory: None, environment: Vec::new() };
    // A rule file the merge left conflicted is Git's marker file in the worktree, so no rule of it can be read: the pass re-merges nothing rather than trust a line between the markers.
    let source: AttributeSource<'_> = if conflicted_paths(entries.as_slice()).iter().any(|path| { return is_attributes_path(path.as_slice()); }) {
        AttributeSource::Unknown
    } else {
        AttributeSource::Invocation
    };
    let verdicts: Vec<Verdict> = match second_pass(&runner, conflicts.as_slice(), source) {
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
    // What the worktree held when the stages were read, so a file that moved before the install is left alone.
    let held: BTreeMap<Vec<u8>, Option<Held>> = held_paths(layout.worktree_root.as_path(), &created);
    let mut oids: Vec<String> = Vec::new();
    for file in &merged {
        match write_blob(&runner, file.bytes.as_slice()) {
            Ok(oid) => oids.push(oid),
            Err(reason) => {
                warn(TAG, format!("the merged blob of {} could not be written, so the conflicts stay: {reason}", path_message_text(file.path.as_slice())).as_str());
                return None;
            }
        }
    }
    // The index is read again after the merge was computed: a path another command resolved and staged holds no stages of the merge's.
    let Some(entries_now) = index_entries(&layout) else {
        warn(TAG, "the index cannot be read again before the merged files are installed, so the conflicts stay as they are");
        return None;
    };
    let before: Before<'_> = Before { worktree_root: layout.worktree_root.as_path(), entries: entries_now.as_slice(), held: &held };
    let kept: Kept = still_the_same(&before, merged, oids);
    let mut moved_names: Vec<String> = Vec::new();
    for path in &kept.moved {
        moved_names.push(path_message_text(path.as_slice()));
    }
    if kept.files.is_empty() {
        warn(TAG, format!("every merged file moved while the pass computed its merge, so they stay as they are now: {}", moved_names.join(", ")).as_str());
        // The person must still be told which file the pass did not touch.
        let alone: String = format!("{}\n", moved_words(moved_names.as_slice()).trim_end());
        if let Err(error) = stderr.write_all(alone.as_bytes()) {
            debug(TAG, format!("standard error did not take the line about the files the pass left alone: {error}").as_str());
        }
        return None;
    }
    let merged: Vec<Merged> = kept.files;
    let oids: Vec<String> = kept.oids;
    if let Err(reason) = install(&runner, merged.as_slice(), oids.as_slice()) {
        warn(TAG, format!("the merged files could not be installed, so the conflicts stay as Git left them: {reason}").as_str());
        return None;
    }
    let operation: String = String::from_utf8_lossy(forward.command.word()).into_owned();
    let mut text: String = String::new();
    for line in announce(LINE_PREFIX, (CLIENT_IDENTIFIER, operation.as_str()), merged.as_slice()) {
        text.push_str(line.as_str());
        text.push('\n');
    }
    let remaining: usize = conflicted_paths(entries.as_slice()).len().saturating_sub(merged.len());
    // Every conflicted file resolved: end the operation the way a clean run does, if the options say how.
    let planned = if remaining == 0 {
        match branch_merge_options(&runner, layout.git_dir.as_path(), &forward.command) {
            Some(configured) => plan_finish(&forward.command, forward.environment.as_slice(), layout.git_dir.as_path(), configured.as_slice()),
            None => None,
        }
    } else {
        None
    };
    let mut names: Vec<String> = Vec::new();
    for file in &merged {
        names.push(path_message_text(file.path.as_slice()));
    }
    let listed: String = names.join(", ");
    // What the pass staged is said whether or not a finish follows: a finish that fails must not be the only account of it.
    text.push_str(format!("{LINE_PREFIX}{REPORT_MARK}: staged as resolved by the second merge pass: {listed}. ").as_str());
    if !moved_names.is_empty() {
        // A path the pass left is said in the same breath, so the person knows why it still conflicts.
        text.push_str(moved_words(moved_names.as_slice()).as_str());
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
        // The ending step's words are its own; the merge changed before it ran, and the person must be told.
        let after: String = format!("{}\n", unfinished_words(operation.as_str(), listed.as_str()));
        if let Err(error) = stderr.write_all(after.as_bytes()) {
            debug(TAG, format!("standard error did not take the line about the unfinished operation: {error}").as_str());
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
fn branch_merge_options(runner: &ContextRunner<'_>, git_dir: &std::path::Path, command: &crate::forwarded_command::ResolvedCommand) -> Option<Vec<std::ffi::OsString>> {
    if !matches!(command.word(), b"merge" | b"pull") {
        return Some(Vec::new());
    }
    let head: String = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let Some(branch) = head.trim().strip_prefix("ref: refs/heads/") else { return Some(Vec::new()) };
    let key: String = format!("branch.{branch}.mergeoptions");
    return match runner.run(&["config", "--get", key.as_str()], None, None) {
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
