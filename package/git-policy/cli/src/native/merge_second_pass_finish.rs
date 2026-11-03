//! What:
//!  The finish after the word-level second pass resolved every conflicted file of a merge, rebase, cherry-pick, revert or pull:
//!  the operation ends the way Git ends it after a clean run, through the wrapper's own command path.
//! Why:
//!  The owner decided on 2026-11-03 ("Finish like a clean merge"): when every conflicted file resolves, make the merge
//!  commit with Git's prepared message, or continue the rebase or cherry-pick, and honour `--no-commit`, `--squash`, `--edit`
//!  and similar, which stop as Git would. The continuing step is a command run in this process through `run_process`, so the
//!  commit policies and hooks apply to it and a later pick of a rebase that conflicts gets the second pass again.
//!  What a clean run does, from Git 2.56.0 (`builtin/merge.c`, `sequencer.c`, `builtin/pull.c`):
//!  a merge that is clean and wants a commit runs `prepare_to_commit` and commits (`merge.c` line 1828); with `--no-commit`
//!  or `--squash` it writes the merge state and prints "Automatic merge went well; stopped before committing as requested"
//!  to standard error and exits 0 (`merge.c` lines 1867 to 1873); the editor opens when `--edit` is given, or when nothing is
//!  given, no `-m` was given and `GIT_MERGE_AUTOEDIT` is unset and standard input and output are one terminal
//!  (`default_edit_option`, `merge.c` lines 1158 to 1180). A cherry-pick or revert commits with the picked commit's message and
//!  edits only when `--edit` is given, or for a revert on a terminal (`should_edit`, `sequencer.c` line 2217); after a conflict
//!  `--continue` runs `git commit` (`continue_single_pick`, `sequencer.c` lines 5354 to 5378) and then picks the rest.
//!  A rebase continues with `git rebase --continue`. `git pull` runs `git merge` or `git rebase` as a child
//!  (`builtin/pull.c` lines 525 and 750), so the state files tell which of the two stopped.
//!  The options the person typed are read only by exact spelling; any option this reader does not know leaves the operation
//!  stopped, with the files staged, as Git left it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await finishLikeCleanRun({ forward, gitDir }); // undefined: leave the operation stopped
//! ```

use super::child_environment::{CHILD_SET_PREFIX, environment_value};
use super::entry::run_process;
use super::forwarded_command::ResolvedCommand;
use super::merge_isolation::{editor_would_open, stdin_and_stdout_are_one_terminal};
use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many finishes may be nested in this process. Each conflicting pick of a long rebase or cherry-pick range is one more
/// level of this process (the continuing step runs inside the step before it); past the limit the operation is left stopped,
/// with the files staged, instead of growing the stack. The value is this implementation's choice, not a measured one.
const NESTED_FINISH_LIMIT: usize = 64;

/// The finishes now running in this process, outermost first.
static NESTED_FINISHES: AtomicUsize = AtomicUsize::new(0);

/// The words of a `branch.<name>.mergeoptions` value: the red-control stub, which reads nothing.
#[allow(dead_code)]
fn split_cmdline(_text: &str) -> Option<Vec<OsString>> {
    return None;
}

/// How many finishes one outermost finish may nest: the red-control stub, still the old fixed limit.
#[allow(dead_code)]
fn finish_budget(_operation: Operation, _git_dir: &Path) -> usize {
    return NESTED_FINISH_LIMIT;
}

/// Whether one more finish may be nested inside `running` others.
fn nesting_allows(running: usize) -> bool {
    return running < NESTED_FINISH_LIMIT;
}

/// What stops a clean run before it commits, and what shapes its message.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stops {
    /// `--no-commit` (or `-n` of a pick): stop with the result staged.
    pub no_commit: bool,
    /// `--squash`: stop with the result staged and no merge state.
    pub squash: bool,
    /// `--edit` or `--no-edit`; absent when neither was given.
    pub edit: Option<bool>,
    /// `-m` or `-F` was given, which turns the default editor off.
    pub message: bool,
    /// Signing and sign-off options, handed on to the commit as typed.
    pub passed_on: Vec<OsString>,
}

/// Which operation stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operation {
    /// A merge, with `MERGE_HEAD` (or `--squash`).
    Merge,
    /// A rebase, either backend.
    Rebase,
    /// A cherry-pick or revert.
    Pick,
}

/// The options shared by merge and pull that change nothing the finish needs to know.
const HARMLESS_MERGE_OPTIONS: [&str; 20] = [
    "-v", "--verbose", "--no-verbose", "--progress", "--no-progress", "--no-verify", "--verify", "--summary", "--no-summary",
    "--no-signoff", "--no-verify-signatures", "--ff", "--no-ff", "--ff-only", "--allow-unrelated-histories", "--stat",
    "--no-stat", "--log", "--no-log", "--quiet",
];

/// The options of pull that only fetch, or choose merge or rebase, which the state files decide.
const HARMLESS_PULL_OPTIONS: [&str; 17] = [
    "--rebase", "-r", "--no-rebase", "--rebase=true", "--rebase=false", "--rebase=merges", "--all", "--prune", "-p", "--no-prune",
    "--tags", "-t", "--no-tags", "--force", "-f", "--no-recurse-submodules", "--ipv4",
];

/// The options of a cherry-pick or revert that change nothing the finish needs to know.
const HARMLESS_PICK_OPTIONS: [&str; 13] = [
    "-x", "--ff", "--allow-empty", "--allow-empty-message", "--keep-redundant-commits", "--rerere-autoupdate",
    "--no-rerere-autoupdate", "--reference", "-q", "--quiet", "-v", "--verbose", "--no-verbose",
];

/// The options that take the next token as their value and change nothing the finish needs to know.
const VALUE_OPTIONS: [&str; 4] = ["-s", "--strategy", "-X", "--strategy-option"];

/// What:
///  Read the arguments after the subcommand of a merge or pull, by exact spelling.
/// Why:
///   An option this does not know might be an abbreviation of `--no-commit`; the finish must not guess, so it declines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeStops(region: string[], pull: boolean): Stops | undefined;
/// ```
pub fn merge_stops(region: &[OsString], pull: bool) -> Option<Stops> {
    let mut stops: Stops = Stops::default();
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let text: &str = token.to_str()?;
        match text {
            "--no-commit" => stops.no_commit = true,
            "--commit" => stops.no_commit = false,
            "--squash" => stops.squash = true,
            "--no-squash" => stops.squash = false,
            "-e" | "--edit" => stops.edit = Some(true),
            "--no-edit" => stops.edit = Some(false),
            "-m" | "--message" | "-F" | "--file" => {
                tokens.next()?;
                stops.message = true;
            }
            "--signoff" | "--no-gpg-sign" => stops.passed_on.push(token.clone()),
            other if other.starts_with("--message=") || other.starts_with("--file=") => stops.message = true,
            other if other.starts_with("-S") || other.starts_with("--gpg-sign") => stops.passed_on.push(token.clone()),
            other if VALUE_OPTIONS.contains(&other) => {
                tokens.next()?;
            }
            other if other.starts_with("--strategy=") || other.starts_with("--strategy-option=") => {}
            other if HARMLESS_MERGE_OPTIONS.contains(&other) => {}
            other if pull && HARMLESS_PULL_OPTIONS.contains(&other) => {}
            other if other.starts_with('-') && other != "-" => return None,
            _ => {}
        }
    }
    return Some(stops);
}

/// What:
///  Read the arguments after the subcommand of a cherry-pick or revert, by exact spelling, and say whether they continue a
///  sequence (`--continue`, `--skip`), whose options Git saved.
/// Why:
///   Same reason as `merge_stops`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pickStops(region: string[]): { stops: Stops; continuing: boolean } | undefined;
/// ```
pub fn pick_stops(region: &[OsString]) -> Option<(Stops, bool)> {
    let mut stops: Stops = Stops::default();
    let mut continuing: bool = false;
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let text: &str = token.to_str()?;
        match text {
            "-n" | "--no-commit" => stops.no_commit = true,
            "-e" | "--edit" => stops.edit = Some(true),
            "--no-edit" => stops.edit = Some(false),
            "--continue" | "--skip" => continuing = true,
            "-s" | "--signoff" => stops.passed_on.push(OsString::from("--signoff")),
            "-m" | "--mainline" => {
                tokens.next()?;
            }
            other if other.starts_with("--mainline=") || other.starts_with("--empty=") => {}
            other if other.starts_with("-S") || other.starts_with("--gpg-sign") || other == "--no-gpg-sign" => stops.passed_on.push(token.clone()),
            other if VALUE_OPTIONS.contains(&other) => {
                tokens.next()?;
            }
            other if other.starts_with("--strategy=") || other.starts_with("--strategy-option=") => {}
            other if HARMLESS_PICK_OPTIONS.contains(&other) => {}
            other if other.starts_with('-') && other != "-" => return None,
            _ => {}
        }
    }
    return Some((stops, continuing));
}

/// What:
///  The options Git saved for a cherry-pick or revert sequence, read from `sequencer/opts`.
/// Why:
///   `--continue` takes the options of the first command (`save_opts`, `sequencer.c` lines 3734 to 3760); the lines of the
///  file are `name = value` under `[options]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedPickStops(gitDir: string): Stops | undefined;
/// ```
pub fn saved_pick_stops(git_dir: &Path) -> Option<Stops> {
    let text: String = std::fs::read_to_string(git_dir.join("sequencer/opts")).ok()?;
    let mut stops: Stops = Stops::default();
    for line in text.lines() {
        let Some((name, value)) = line.split_once('=') else { continue };
        match (name.trim(), value.trim()) {
            ("no-commit", "true") => stops.no_commit = true,
            ("edit", "true") => stops.edit = Some(true),
            ("edit", "false") => stops.edit = Some(false),
            ("signoff", "true") => stops.passed_on.push(OsString::from("--signoff")),
            _ => {}
        }
    }
    return Some(stops);
}

/// What:
///  `message` without the conflict hint Git appended when it stopped, so it reads as the message of a clean run.
/// Why:
///   A stop appends a blank line, a comment line `Conflicts:` and one comment line with a tab and a path per conflicted file
///  (`append_conflicts_hint`, `sequencer.c` lines 722 to 745). A clean run without an editor keeps the message as prepared, so the
///  hint is cut here; the comment string is whatever precedes `Conflicts:` on the hint's line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutConflictsHint(message: Uint8Array): Uint8Array;
/// ```
pub fn without_conflicts_hint(message: &[u8]) -> Vec<u8> {
    let lines: Vec<&[u8]> = message.split_inclusive(|byte| { return *byte == b'\n'; }).collect();
    let mut start: usize = lines.len();
    let mut comment: Option<&[u8]> = None;
    // Walk back over the path lines, then expect the `Conflicts:` line.
    while start > 0 {
        let line: &[u8] = lines[start - 1].strip_suffix(b"\n").unwrap_or(lines[start - 1]);
        if let Some(prefix) = line.strip_suffix(b"Conflicts:") {
            if comment.is_none_or(|known| { return known == prefix; }) && !prefix.is_empty() {
                comment = Some(prefix);
                start -= 1;
                break;
            }
            return message.to_vec();
        }
        let Some(tab_path) = comment.map_or_else(|| { return Some(line); }, |known| { return line.strip_prefix(known); }) else { return message.to_vec() };
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
///  Whether Git's default is to open the editor for a merge that was not told: a message given, `GIT_MERGE_AUTOEDIT`, or one
///  terminal for standard input and output.
/// Why:
///   `default_edit_option`, `merge.c` lines 1158 to 1180.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeEditsByDefault(message: boolean, environment): boolean | undefined;
/// ```
fn merge_edits_by_default(message: bool, environment: &[(OsString, OsString)]) -> Option<bool> {
    if message {
        return Some(false);
    }
    let auto_edit: Option<String> = environment_value(environment, "GIT_MERGE_AUTOEDIT").map(|value| { return value.to_string_lossy().into_owned(); });
    return editor_would_open(auto_edit.as_deref(), stdin_and_stdout_are_one_terminal());
}

/// Which operation left its state in the Git directory, from the state files Git writes.
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
    /// The Git directory of the worktree, for the message file and the sequencer files.
    git_dir: std::path::PathBuf,
}

/// What:
///  Decide whether the stopped operation can be finished like a clean run, and how.
/// Why:
///   Anything this does not know exactly leaves the operation stopped with the files staged: an option it cannot read, an
///  operation whose state files are not the ones it knows, a merge setting that may carry `--no-commit`, an editor value Git
///  would refuse, or a finish already nested as deep as the limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planFinish({ command, environment, gitDir, mergeOptionsSet }): Finish | undefined;
/// ```
pub fn plan_finish(command: &ResolvedCommand, environment: &[(OsString, OsString)], git_dir: &Path, merge_options_set: bool) -> Option<Finish> {
    if !nesting_allows(NESTED_FINISHES.load(Ordering::SeqCst)) {
        return None;
    }
    let region: &[OsString] = command.region();
    let word: &[u8] = command.word();
    let (mut stops, continuing): (Stops, bool) = match word {
        b"merge" => (merge_stops(region, false)?, false),
        b"pull" => (merge_stops(region, true)?, false),
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
    if matches!(word, b"merge" | b"pull") && operation == Operation::Merge && merge_options_set {
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
///  Whether the rest of a rebase asks for an editor: a later `reword`, `squash` or `fixup -c`/`-C` of its todo list.
/// Why:
///   The continuing step of a rebase edits the message of the commit it makes after a conflict, which a clean pick does not; it is
///  run with the editor switched off (`:`), which would also switch off the editor of a later `reword`, so a todo list that wants
///  one is not finished here and the files stay staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function todoNeedsEditor(gitDir: string): boolean;
/// ```
fn todo_needs_editor(git_dir: &Path) -> bool {
    let Ok(todo) = std::fs::read_to_string(git_dir.join("rebase-merge/git-rebase-todo")) else { return false };
    return todo.lines().any(|line| {
        let mut words = line.split_whitespace();
        let command: &str = words.next().unwrap_or_default();
        return matches!(command, "reword" | "r" | "squash" | "s") || (matches!(command, "fixup" | "f") && words.next().is_some_and(|option| { return option == "-c" || option == "-C"; }));
    });
}

/// A command of this invocation run through the wrapper's own path, nested one level deeper.
fn run_nested(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32 {
    NESTED_FINISHES.fetch_add(1, Ordering::SeqCst);
    let code: i32 = run_process(arguments, environment);
    NESTED_FINISHES.fetch_sub(1, Ordering::SeqCst);
    return code;
}

/// The commit that ends a stopped merge or pick, run through the wrapper.
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
///  End the stopped operation the way a clean run ends, and return the exit status that run gives.
/// Why:
///   See the module comment. The message file is cleaned of the conflict hint before the commit, and a stop requested with
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
                super::diagnostic_log::warn("wordMergeFinish", format!("the conflict hint could not be cut from {}: {error}", message_file.display()).as_str());
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

/// Controls for the option readers, the hint cutting and the planning.
#[cfg(test)]
#[path = "merge_second_pass_finish_tests.rs"]
mod tests;
