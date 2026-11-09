//! What:
//!  The finish after the word-level second pass resolved every conflicted file of a merge,
//! rebase,
//! cherry-pick,
//! revert or pull:
//!  the operation ends the way Git ends it after a clean run,
//! through the wrapper's own command path.
//! Why:
//!  The owner decided on 2026-11-03 ("Finish like a clean merge"):
//! when every conflicted file resolves,
//! make the merge
//!  commit with Git's prepared message,
//! or continue the rebase or cherry-pick,
//! and honour `--no-commit`,
//! `--squash`,
//! `--edit`
//!  and similar,
//! which stop as Git would.
//! The continuing step is a command run in this process through `run_process`,
//! so the
//!  commit policies and hooks apply to it and a later pick of a rebase that conflicts gets the second pass again.
//!  What a clean run does,
//! from Git 2.56.0 (`builtin/merge.c`,
//! `sequencer.c`,
//! `builtin/pull.c`):
//!  a merge that is clean and wants a commit runs `prepare_to_commit` and commits (`merge.c` line 1828);
//! with `--no-commit`
//!  or `--squash` it writes the merge state and prints "Automatic merge went well;
//! stopped before committing as requested"
//!  to standard error and exits 0 (`merge.c` lines 1867 to 1873);
//! the editor opens when `--edit` is given,
//! or when nothing is
//!  given,
//! no `-m` was given and `GIT_MERGE_AUTOEDIT` is unset and standard input and output are one terminal
//!  (`default_edit_option`,
//! `merge.c` lines 1158 to 1180).
//! A cherry-pick or revert commits with the picked commit's message and
//!  edits only when `--edit` is given,
//! or for a revert on a terminal (`should_edit`,
//! `sequencer.c` line 2217);
//! after a conflict
//!  `--continue` runs `git commit` (`continue_single_pick`,
//! `sequencer.c` lines 5354 to 5378) and then picks the rest.
//!  A rebase continues with `git rebase --continue`.
//! `git pull` runs `git merge` or `git rebase` as a child
//!  (`builtin/pull.c` lines 525 and 750),
//! so the state files tell which of the two stopped.
//!  The options the person typed are read only by exact spelling;
//! any option this reader does not know leaves the operation
//!  stopped,
//! with the files staged,
//! as Git left it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await finishLikeCleanRun({ forward, gitDir }); // undefined: leave the operation stopped
//! ```

use crate::child_environment::{CHILD_SET_PREFIX, environment_value};
use crate::entry::run_process;
use crate::forwarded_command::ResolvedCommand;
use crate::merge_isolation_git_reads::{editor_would_open, stdin_and_stdout_are_one_terminal};
use crate::merge_second_pass_finish_stops::{Operation, Stops, merge_stops, pick_stops, saved_pick_stops};
use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The finishes now running in this process,
/// outermost first.
static NESTED_FINISHES: AtomicUsize = AtomicUsize::new(0);

/// How many finishes the outermost one of this process may nest,
/// set when it is planned (`finish_budget`).
static FINISH_BUDGET: AtomicUsize = AtomicUsize::new(0);

/// What:
///  The words of a `branch.<name>.mergeoptions` value,
/// split the way Git splits it.
/// Why:
///   `parse_branch_merge_options` (`builtin/merge.c` line 641) hands the value to `split_cmdline` (`alias.c` lines 126 to 180):
///  white space separates words,
/// single and double quotes group (a backslash escapes inside double quotes and outside quotes),
///  and an unclosed quote or a trailing backslash is an error,
/// which Git dies on and this reads as "not read".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitCmdline(text: string): string[] | undefined;
/// ```
pub fn split_cmdline(text: &str) -> Option<Vec<OsString>> {
    let mut words: Vec<String> = Vec::new();
    let mut current: String = String::new();
    let mut started: bool = false;
    let mut quoted: Option<char> = None;
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if quoted.is_none() && character.is_whitespace() {
            if started {
                words.push(std::mem::take(&mut current));
                started = false;
            }
        } else if quoted.is_none() && (character == '\'' || character == '"') {
            quoted = Some(character);
            started = true;
        } else if Some(character) == quoted {
            quoted = None;
        } else {
            let mut kept: char = character;
            if character == '\\' && quoted != Some('\'') {
                kept = characters.next()?;
            }
            current.push(kept);
            started = true;
        }
    }
    if quoted.is_some() {
        return None;
    }
    if started {
        words.push(current);
    }
    return Some(words.into_iter().map(OsString::from).collect());
}

/// The commands of a todo file:
/// its lines that are neither blank nor comments.
fn todo_commands(path: &Path) -> usize {
    let Ok(text) = std::fs::read_to_string(path) else {
        return 0;
    };
    let mut commands: usize = 0;
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            commands += 1;
        }
    }
    return commands;
}

/// What:
///  How many finishes the outermost one may nest,
/// from the input of the operation.
/// Why:
///   Each finish ends one conflicting step and runs the rest,
/// so the steps still to do bound the finishes:
/// a rebase's remaining
///  todo list plus the finish that is running,
/// a cherry-pick or revert sequence's todo list (one for a single pick),
/// and one for
///  a merge,
/// which has no further step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishBudget(operation: Operation, gitDir: string): number;
/// ```
fn finish_budget(operation: Operation, git_dir: &Path) -> usize {
    return match operation {
        Operation::Rebase => todo_commands(git_dir.join("rebase-merge/git-rebase-todo").as_path()) + 1,
        Operation::Pick => todo_commands(git_dir.join("sequencer/todo").as_path()).max(1),
        Operation::Merge => 1,
    };
}

/// Whether one more finish may be nested inside `running` others,
/// under a budget of `budget` finishes.
fn nesting_allows(running: usize, budget: usize) -> bool {
    return running < budget;
}

/// What:
///  The command that ends the operation the Git directory shows in progress,
/// or nothing when no state file names one.
/// Why:
///   The words after a finish that did not end the operation must name a command that works for it,
/// and which operation it is shows
///  in the state files Git writes:
/// a `git pull` runs either a merge or a rebase as a child (`builtin/pull.c` lines 525 and 750),
/// so the
///  word `pull` alone names no ending command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endingCommandOf(gitDir: string): string | undefined;
/// ```
pub fn ending_command_of(git_dir: &Path) -> Option<&'static str> {
    if git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").is_dir() {
        return Some("`git rebase --continue`");
    }
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return Some("`git cherry-pick --continue`");
    }
    if git_dir.join("REVERT_HEAD").exists() {
        return Some("`git revert --continue`");
    }
    if git_dir.join("MERGE_HEAD").exists() {
        return Some("`git commit`");
    }
    return None;
}

/// Whether one byte is the line feed that ends a line of a message.
/// `&u8` borrows the byte,
/// as `split_inclusive` hands it over.
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  `message` without the conflict hint Git appended when it stopped,
/// so it reads as the message of a clean run.
/// Why:
///   A stop appends a blank line,
/// a comment line `Conflicts:` and one comment line with a tab and a path per conflicted file
///  (`append_conflicts_hint`,
/// `sequencer.c` lines 722 to 745).
/// A clean run without an editor keeps the message as prepared,
/// so the
///  hint is cut here;
/// the comment string is whatever precedes `Conflicts:` on the hint's line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutConflictsHint(message: Uint8Array): Uint8Array;
/// ```
pub fn without_conflicts_hint(message: &[u8]) -> Vec<u8> {
    let lines: Vec<&[u8]> = message.split_inclusive(is_line_feed_byte).collect();
    let mut start: usize = lines.len();
    let mut comment: Option<&[u8]> = None;
    // Walk back over the path lines, then expect the `Conflicts:` line.
    while start > 0 {
        let line: &[u8] = lines[start - 1].strip_suffix(b"\n").unwrap_or(lines[start - 1]);
        if let Some(prefix) = line.strip_suffix(b"Conflicts:") {
            let same_comment: bool = match comment {
                Some(known) => known == prefix,
                None => true,
            };
            if same_comment && !prefix.is_empty() {
                comment = Some(prefix);
                start -= 1;
                break;
            }
            return message.to_vec();
        }
        let path_line: Option<&[u8]> = match comment {
            Some(known) => line.strip_prefix(known),
            None => Some(line),
        };
        let Some(tab_path) = path_line else { return message.to_vec() };
        if comment.is_none() && !tab_path.contains(&b'\t') {
            return message.to_vec();
        }
        start -= 1;
    }
    if comment.is_none() {
        return message.to_vec();
    }
    // The blank line the hint starts with goes too.
    if start > 0 && lines[start - 1] == b"\n" {
        start -= 1;
    }
    return lines[..start].concat();
}

/// What:
///  Whether Git's default is to open the editor for a merge that was not told:
/// a message given,
/// `GIT_MERGE_AUTOEDIT`,
/// or one
///  terminal for standard input and output.
/// Why:
///   `default_edit_option`,
/// `merge.c` lines 1158 to 1180.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeEditsByDefault(message: boolean, environment): boolean | undefined;
/// ```
fn merge_edits_by_default(message: bool, environment: &[(OsString, OsString)]) -> Option<bool> {
    if message {
        return Some(false);
    }
    // The value as text, with bytes that are not text replaced; `.map` takes the two steps by path.
    let auto_edit: Option<String> = environment_value(environment, "GIT_MERGE_AUTOEDIT")
        .as_deref()
        .map(std::ffi::OsStr::to_string_lossy)
        .map(std::borrow::Cow::into_owned);
    return editor_would_open(auto_edit.as_deref(), stdin_and_stdout_are_one_terminal());
}

/// Which operation left its state in the Git directory,
/// from the state files Git writes.
fn operation_in_progress(git_dir: &Path, command: &ResolvedCommand, squash: bool) -> Option<Operation> {
    if git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").is_dir() {
        return Some(Operation::Rebase);
    }
    if git_dir.join("CHERRY_PICK_HEAD").exists() || git_dir.join("REVERT_HEAD").exists() {
        return Some(Operation::Pick);
    }
    if git_dir.join("MERGE_HEAD").exists() || (squash && command.word() != b"cherry-pick" && command.word() != b"revert") {
        return Some(Operation::Merge);
    }
    return None;
}

/// A finish the second pass has planned and can run.
pub struct Finish {
    /// Which operation to end.
    operation: Operation,
    /// What the person typed that stops or shapes it.
    stops: Stops,
    /// The Git directory of the worktree,
    /// for the message file and the sequencer files.
    git_dir: std::path::PathBuf,
}

/// What:
///  Decide whether the stopped operation can be finished like a clean run,
/// and how.
/// Why:
///   Anything this does not know exactly leaves the operation stopped with the files staged:
/// an option it cannot read,
/// an
///  operation whose state files are not the ones it knows,
/// an editor value Git would refuse,
/// or a finish already
///  nested as deep as the budget the outermost one set from its input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planFinish({ command, environment, gitDir, mergeOptionsSet }): Finish | undefined;
/// ```
pub fn plan_finish(command: &ResolvedCommand, environment: &[(OsString, OsString)], git_dir: &Path, configured_options: &[OsString]) -> Option<Finish> {
    let region: &[OsString] = command.region();
    let word: &[u8] = command.word();
    // `branch.<name>.mergeoptions` is parsed before the command line (`merge.c` lines 1407 to 1409).
    let merge_region: Vec<OsString> = configured_options.iter().chain(region.iter()).cloned().collect();
    let (mut stops, continuing): (Stops, bool) = match word {
        b"merge" => (merge_stops(merge_region.as_slice(), false)?, false),
        b"pull" => (merge_stops(merge_region.as_slice(), true)?, false),
        b"rebase" => (Stops::default(), false),
        b"cherry-pick" | b"revert" => pick_stops(region)?,
        _ => return None,
    };
    // A pick told `--no-commit` leaves no pick head: Git applies the change and stops (`do_pick_commit`), with the index unmerged.
    let pick_without_commit: bool = matches!(word, b"cherry-pick" | b"revert") && stops.no_commit;
    let operation: Operation = match operation_in_progress(git_dir, command, stops.squash) {
        Some(found) => found,
        None if pick_without_commit => Operation::Pick,
        None => return None,
    };
    // The state must be the one this command makes: a merge's, a rebase's (also a pull's), a pick's.
    if !matches!((word, operation), (b"merge" | b"pull", Operation::Merge) | (b"pull" | b"rebase", Operation::Rebase) | (b"cherry-pick" | b"revert", Operation::Pick)) {
        return None;
    }
    // The outermost finish sets the budget from its input; the finishes nested in it spend it.
    let running: usize = NESTED_FINISHES.load(Ordering::SeqCst);
    if running == 0 {
        FINISH_BUDGET.store(finish_budget(operation, git_dir), Ordering::SeqCst);
    }
    if !nesting_allows(running, FINISH_BUDGET.load(Ordering::SeqCst)) {
        return None;
    }
    if continuing {
        stops = Stops { passed_on: stops.passed_on.clone(), ..saved_pick_stops(git_dir)? };
    }
    if operation == Operation::Rebase && todo_needs_editor(git_dir) {
        return None;
    }
    if operation == Operation::Merge && stops.edit.is_none() {
        stops.edit = Some(merge_edits_by_default(stops.message, environment)?);
    }
    if operation == Operation::Pick {
        // A clean pick edits when asked to, and a revert also on a terminal (`should_edit`).
        let reverting: bool = word == b"revert" || git_dir.join("REVERT_HEAD").exists();
        stops.edit = Some(stops.edit.unwrap_or(reverting && std::io::stdin().is_terminal()));
        // A range applied with `--no-commit` cannot be continued by Git either (`error_dirty_index`).
        if stops.no_commit && git_dir.join("sequencer/todo").exists() {
            return None;
        }
    }
    return Some(Finish { operation, stops, git_dir: git_dir.to_path_buf() });
}

/// The invocation's global options followed by `words`.
fn invocation(command: &ResolvedCommand, words: &[&str]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = command.arguments[..command.subcommand_index].to_vec();
    arguments.extend(words.iter().map(OsString::from));
    return arguments;
}

/// What:
///  Whether the rest of a rebase asks for an editor:
/// a later `reword`,
/// `squash` or `fixup -c`/`-C` of its todo list.
/// Why:
///   The continuing step of a rebase edits the message of the commit it makes after a conflict,
/// which a clean pick does not;
/// it is
///  run with the editor switched off (`:`),
/// which would also switch off the editor of a later `reword`,
/// so a todo list that wants
///  one is not finished here and the files stay staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function todoNeedsEditor(gitDir: string): boolean;
/// ```
fn todo_needs_editor(git_dir: &Path) -> bool {
    let Ok(todo) = std::fs::read_to_string(git_dir.join("rebase-merge/git-rebase-todo")) else { return false };
    for line in todo.lines() {
        let mut words = line.split_whitespace();
        let command: &str = words.next().unwrap_or_default();
        let fixup_edits: bool = match words.next() {
            Some(option) => option == "-c" || option == "-C",
            None => false,
        };
        if matches!(command, "reword" | "r" | "squash" | "s") || (matches!(command, "fixup" | "f") && fixup_edits) {
            return true;
        }
    }
    return false;
}

/// A command of this invocation run through the wrapper's own path,
/// nested one level deeper.
fn run_nested(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32 {
    NESTED_FINISHES.fetch_add(1, Ordering::SeqCst);
    let code: i32 = run_process(arguments, environment);
    NESTED_FINISHES.fetch_sub(1, Ordering::SeqCst);
    return code;
}

/// The commit that ends a stopped merge or pick,
/// run through the wrapper.
fn commit_step(finish: &Finish, command: &ResolvedCommand, environment: &[(OsString, OsString)]) -> i32 {
    let mut words: Vec<&str> = vec!["commit"];
    if finish.stops.edit != Some(true) {
        // A pick passes the same two Git's own `--continue` does (`continue_single_pick`, `sequencer.c` line 5371); a merge
        // keeps its prepared message as it is, comment lines included, as a clean merge without an editor does.
        words.extend(["--no-edit", if finish.operation == Operation::Merge { "--cleanup=whitespace" } else { "--cleanup=strip" }]);
    }
    let mut arguments: Vec<OsString> = invocation(command, words.as_slice());
    arguments.extend(finish.stops.passed_on.iter().cloned());
    return run_nested(arguments.as_slice(), environment);
}

/// What:
///  End the stopped operation the way a clean run ends,
/// and return the exit status that run gives.
/// Why:
///   See the module comment.
/// The message file is cleaned of the conflict hint before the commit,
/// and a stop requested with
///  `--no-commit` or `--squash` prints Git's own line and exits 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finish(plan, command, environment, stderr): Promise<number>;
/// ```
pub fn finish(plan: &Finish, command: &ResolvedCommand, environment: &[(OsString, OsString)], stderr: &mut dyn Write) -> i32 {
    let message_file = plan.git_dir.join("MERGE_MSG");
    if plan.operation != Operation::Rebase
        && let Ok(message) = std::fs::read(&message_file)
    {
        let cleaned: Vec<u8> = without_conflicts_hint(message.as_slice());
        if cleaned != message {
            let result = if cleaned.is_empty() { std::fs::remove_file(&message_file) } else { std::fs::write(&message_file, cleaned) };
            if let Err(error) = result {
                crate::diagnostic_log::warn("wordMergeFinish", format!("the conflict hint could not be cut from {}: {error}", message_file.display()).as_str());
            }
        }
    }
    match plan.operation {
        Operation::Rebase => {
            let mut quiet: Vec<(OsString, OsString)> = environment.to_vec();
            quiet.push((OsString::from(format!("{CHILD_SET_PREFIX}GIT_EDITOR")), OsString::from(":")));
            return run_nested(invocation(command, &["rebase", "--continue"]).as_slice(), quiet.as_slice());
        }
        Operation::Merge if plan.stops.no_commit || plan.stops.squash => {
            let _ = stderr.write_all(b"Automatic merge went well; stopped before committing as requested\n");
            return 0;
        }
        Operation::Pick if plan.stops.no_commit => return 0,
        _ => {}
    }
    let committed: i32 = commit_step(plan, command, environment);
    if committed != 0 || plan.operation != Operation::Pick || !plan.git_dir.join("sequencer/todo").exists() {
        return committed;
    }
    // The rest of the sequence, which also ends it (`sequencer_continue`, `sequencer.c` lines 5617 to 5630).
    let word: &str = if command.word() == b"revert" || plan.git_dir.join("REVERT_HEAD").exists() { "revert" } else { "cherry-pick" };
    return run_nested(invocation(command, &[word, "--continue"]).as_slice(), environment);
}

/// Controls for the option readers,
/// the hint cutting and the planning.
#[cfg(test)]
#[path = "merge_second_pass_finish_tests.rs"]
mod tests;
