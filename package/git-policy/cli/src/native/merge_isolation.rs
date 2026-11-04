//! What:
//!  A true `git merge <branch>` completed in isolation when the index holds staged changes that make native Git
//!  refuse it: the merge is computed from objects, committed, and applied with Git's own two-way tree update, which
//!  keeps every entry and every file the merge does not change.
//! Why:
//!  Several sessions share one checkout.
//!  Native Git refuses every merge that creates a commit while the index differs from `HEAD` at all (measured in
//!  `doc/planning/cli-git-merge-with-staged-index.md`), although the merge may touch none of the staged paths.
//!  A fast-forward is not refused for unrelated paths, because it is a two-way update that keeps entries it does not
//!  write; a merge commit whose first parent is the current commit is exactly such an update.
//!  So the merge tree comes from `git merge-tree --write-tree`, the commit from `git commit-tree`, and the checkout
//!  moves with `git merge --ff-only <commit>`, which refuses when the branch is no longer where the merge was computed, after a check that no changed path is staged
//!  or modified by someone else.
//!  An intent record, written before Git starts, lets recovery finish or discard a merge whose process died.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runIsolatedMerge({ command, identity, environment }); // 'forward' | { code, stdout, stderr }
//! ```

/// Names and sets of unsaved work.
use super::auto_push_dirty_paths::{diff_paths, overlapping_paths, status_paths};
/// Reading an environment value.
use super::child_environment::{environment_value, environment_with_overlay};
/// Diagnostics for the log.
use super::diagnostic_log::debug;
/// Where Git's words about the wrapper's own fast-forward go now that the run carries no `--quiet`.
use super::inner_git_words::record_streams;
/// The failure rendering for state that cannot be read.
use super::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
/// The resolved command.
use super::forwarded_command::ResolvedCommand;
/// The merge messages.
use super::merge_isolation_message::{clean_message, default_merge_message, describe_merge_source, fetch_head_merge_heads};
/// The word-level second pass over the conflicts of the merge tree, and the attribute tree it reads.
use super::merge_second_pass::{ContextRunner, GitReply, GitRunner, Merged, config_options, conflicts_of_merge_tree};
use super::journald_native::CLIENT_IDENTIFIER;
use super::merge_second_pass_tree::{AttributeProbe, announce, attribute_tree_variables, resolve_tree};
use super::path_bytes::path_message_text;
/// The intent record.
use super::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
/// Recovery of records.
use super::merge_isolation_recovery::{
    MergeRecoveryAction, recover_interrupted_merges, settle_failed_merge,
};
/// The refusal findings.
use super::merge_isolation_report::{conflict_finding, not_applied_finding, overlap_finding};
/// The command-line reading.
use super::merge_options::{FastForwardChoice, MergeShape, ParsedMerge, Signing, parse_merge_shape};
/// The place recovery runs in.
use super::merge_isolation_recovery::RecoveryPlace;
/// Marker phases for the crash controls.
use super::transaction_phase::{Phase, reach_transaction_phase};
/// Real Git.
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// The owner of the intent record, and its identifier.
use super::transaction_owner::current_transaction_owner;
/// Unique identifiers and the clock.
use super::iso_time::now_iso;
use super::random_id::random_uuid;
/// The repository shape.
use super::worktree_identity::WorktreeIdentity;
/// Waiting for any program's hold on the index lock before the checkout changes.
use super::index_holder_wait::wait_for_index_release_here;
/// Owned operating-system text.
use std::ffi::OsString;
/// The notices of a round that is made again.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "isolatedMerge";

/// Files and directories of the Git directory that show another operation is in progress; native Git reports those
/// itself, so the command is forwarded.
const IN_PROGRESS_MARKERS: [&str; 6] = [
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "rebase-merge",
    "rebase-apply",
    "sequencer",
];

/// Everything one isolated merge reads from its invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IsolatedMergeInput = { realGit; overlay; identity; command; environment };
/// ```
pub struct IsolatedMergeInput<'a> {
    /// Real Git executable.
    pub real_git: &'a Path,
    /// Variables added to every Git child.
    pub overlay: &'a [(OsString, OsString)],
    /// The repository shape of the invocation.
    pub identity: &'a WorktreeIdentity,
    /// The merge command after alias resolution.
    pub command: &'a ResolvedCommand,
    /// The environment the invocation inherited.
    pub environment: &'a [(OsString, OsString)],
}

/// How an isolated merge attempt ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IsolatedMerge = { kind: 'forward'; notice?: string } | { kind: 'finished'; code: number; stdout: string; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IsolatedMerge {
    /// The command is not for the wrapper to complete: native Git runs it unchanged, after the notice if any.
    Forward {
        /// A line for standard error before Git runs.
        notice: Option<String>,
    },
    /// The wrapper completed or refused the merge.
    Finished {
        /// Exit status.
        code: i32,
        /// Text for standard output.
        stdout: String,
        /// Text for standard error.
        stderr: String,
    },
}

/// Why the attempt stopped before it finished.
enum Bail {
    /// Forward to native Git.
    Forward(Option<String>),
    /// Done, with this result.
    Done(IsolatedMerge),
    /// The checkout is as it was before the round and the merge is computed again, for this reason (the branch moved, or
    ///  another program took the index lock in the instant before Git did).
    Again(String),
}

/// How many times one merge is computed again on a branch that moved underneath it before the wrapper stops. Each round
/// is a whole merge on the newest head; a branch that moves faster than that for this many rounds in a row is not a
/// race but a loop, and the merge is refused with the cause named, changing nothing.
const MAX_ROUNDS: usize = 20;

/// A failed query is a reason to forward, with the cause in the diagnostic log.
impl From<String> for Bail {
    fn from(reason: String) -> Self {
        debug(TAG, format!("forwarding to native Git: {reason}").as_str());
        return Bail::Forward(None);
    }
}

/// The merge-related settings the isolated path honors or defers on.
#[derive(Debug, Default)]
struct MergeConfig {
    /// `merge.ff`: `Some(false)` is `--no-ff`; `Some(true)` allows; absent is unset.
    fast_forward: Option<bool>,
    /// `merge.ff=only`.
    fast_forward_only: bool,
    /// `merge.stat`: false hides the diffstat.
    stat: bool,
    /// `commit.gpgSign`.
    sign_commits: bool,
    /// Any setting that changes the merge in a way this path does not reproduce.
    deferred: bool,
}

/// What:
///  Read Git's boolean spellings.
/// Why:
///   Settings accept `true`, `yes`, `on`, `1` and their opposites in any letter case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMaybeBool(text: string): boolean | undefined;
/// ```
pub fn parse_maybe_bool(text: &str) -> Option<bool> {
    match text.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => return Some(true),
        "false" | "no" | "off" | "0" | "" => return Some(false),
        _ => return None,
    }
}

/// What:
///  Whether `git merge` would open an editor for the message.
/// Why:
///   `GIT_MERGE_AUTOEDIT` decides when it is set; otherwise the editor opens when standard input and output are one
///  terminal.
///  The editor is not reproduced here, so a merge that would open one is forwarded; `--no-edit` avoids it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editorWouldOpen({ autoEdit, sameTerminal }): boolean | undefined;
/// ```
pub fn editor_would_open(auto_edit: Option<&str>, same_terminal: bool) -> Option<bool> {
    match auto_edit {
        Some(text) => return parse_maybe_bool(text),
        None => return Some(same_terminal),
    }
}

/// What:
///  Whether standard input and standard output are the same terminal.
/// Why:
///   Git's `default_edit_option` asks exactly this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stdin.isTTY && process.stdout.isTTY && sameDevice(0, 1)
/// ```
pub fn stdin_and_stdout_are_one_terminal() -> bool {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        use std::os::unix::fs::MetadataExt;
        let identity = |descriptor: std::os::fd::BorrowedFd<'_>| -> Option<(u64, u64)> {
            let file = std::fs::File::from(descriptor.try_clone_to_owned().ok()?);
            let metadata = file.metadata().ok()?;
            return Some((metadata.dev(), metadata.ino()));
        };
        return identity(std::io::stdin().as_fd()) == identity(std::io::stdout().as_fd());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return true;
    }
}

/// One session of queries against the invocation's repository.
struct Session {
    /// Real Git with the invocation's global options as a prefix.
    context: GitContext,
}

/// Queries.
impl Session {
    /// Run Git and return its output whatever the exit status.
    fn run(&self, arguments: &[&str]) -> Result<GitOutput, String> {
        return run_git(&self.context, &GitRequest::new(Path::new("."), arguments))
            .map_err(|error| { return format!("git {} could not start: {error}", arguments.join(" ")); });
    }

    /// Run Git with bytes on standard input.
    fn run_with_input(&self, arguments: &[&str], input: Vec<u8>) -> Result<GitOutput, String> {
        let mut request: GitRequest = GitRequest::new(Path::new("."), arguments);
        request.input = Some(input);
        return run_git(&self.context, &request)
            .map_err(|error| { return format!("git {} could not start: {error}", arguments.join(" ")); });
    }

    /// Run Git, require success, and return its trimmed standard output.
    fn text(&self, arguments: &[&str]) -> Result<String, String> {
        let output: GitOutput = self.run(arguments)?;
        if !output.succeeded() {
            return Err(format!("git {} failed: {}", arguments.join(" "), output.error_text()));
        }
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
}

/// What:
///  Read the settings in one Git call.
/// Why:
///   The settings that change what a merge does but that this path does not reproduce defer the whole command to
///  native Git; the others (`merge.ff`, `merge.stat`, `commit.gpgSign`) are honored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readMergeConfig(session): Promise<MergeConfig>;
/// ```
fn read_config(session: &Session) -> Result<MergeConfig, String> {
    let output: GitOutput = session.run(&[
        "config",
        "--null",
        "--get-regexp",
        "^(merge\\.(ff|stat|verifysignatures|autostash)|branch\\..*\\.mergeoptions|commit\\.gpgsign)$",
    ])?;
    if output.code != Some(0) && output.code != Some(1) {
        return Err(format!("git config failed: {}", output.error_text()));
    }
    let mut config: MergeConfig = MergeConfig { stat: true, ..MergeConfig::default() };
    for entry in output.stdout.split(|byte| { return *byte == 0; }).filter(|entry| { return !entry.is_empty(); }) {
        let text: String = String::from_utf8_lossy(entry).into_owned();
        let (key, value) = text.split_once('\n').unwrap_or((text.as_str(), "true"));
        match key {
            "merge.ff" if value.eq_ignore_ascii_case("only") => config.fast_forward_only = true,
            "merge.ff" => config.fast_forward = parse_maybe_bool(value),
            "merge.stat" => config.stat = parse_maybe_bool(value).unwrap_or(true),
            "commit.gpgsign" => config.sign_commits = parse_maybe_bool(value).unwrap_or(false),
            "merge.verifysignatures" | "merge.autostash" => {
                config.deferred |= parse_maybe_bool(value).unwrap_or(true);
            }
            other if other.starts_with("branch.") => config.deferred = true,
            _ => {}
        }
    }
    return Ok(config);
}

/// What:
///  The object IDs on a Git output's lines.
/// Why:
///   `rev-parse` and `merge-base` print one ID per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.stdout.toString().split('\n').filter(Boolean)
/// ```
fn lines_of(output: &GitOutput) -> Vec<String> {
    return String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| { return !line.is_empty(); })
        .map(String::from)
        .collect();
}

/// What:
///  The result of the staged-work check: the paths of everyone else's unsaved work, or a reason to forward.
/// Why:
///   An unmerged path means a conflict is in progress, which native Git reports itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function unsavedPaths(session): Promise<Uint8Array[]>;
/// ```
fn unsaved_paths(session: &Session) -> Result<Vec<Vec<u8>>, Bail> {
    let status: GitOutput = session.run(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ])?;
    if !status.succeeded() {
        return Err(Bail::from(format!("git status failed: {}", status.error_text())));
    }
    let mut entries = status.stdout.split(|byte| { return *byte == 0; });
    while let Some(entry) = entries.next() {
        if entry.len() >= 2 && (entry[0] == b'U' || entry[1] == b'U' || entry[..2] == *b"AA" || entry[..2] == *b"DD") {
            return Err(Bail::Forward(None));
        }
        // A rename or copy is followed by its original path, which carries no status letters.
        if entry.len() >= 2 && (matches!(entry[0], b'R' | b'C') || matches!(entry[1], b'R' | b'C')) {
            entries.next();
        }
    }
    return Ok(status_paths(status.stdout.as_slice()));
}

/// What:
///  Complete or refuse one merge, or hand the command back to native Git.
/// Why:
///   The wrapper acts only on the exact failing case: a supported merge command, a staged index, a true merge.
///  Everything else is native Git's, including its own errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runIsolatedMerge(input: IsolatedMergeInput): Promise<IsolatedMerge>;
/// ```
pub fn run_isolated_merge(input: &IsolatedMergeInput<'_>) -> IsolatedMerge {
    match attempt(input) {
        Ok(done) | Err(Bail::Done(done)) => return done,
        Err(Bail::Forward(notice)) => return IsolatedMerge::Forward { notice },
        // `attempt` makes the rounds itself and never lets one escape; native Git is the safe answer to a bug here.
        Err(Bail::Again(reason)) => {
            debug(TAG, format!("a round escaped the attempt ({reason}); native Git runs the merge").as_str());
            return IsolatedMerge::Forward { notice: None };
        }
    }
}

/// The text a finding or failure puts on standard error, as a finished merge.
fn refused(code: i32, stderr: String) -> Bail {
    return Bail::Done(IsolatedMerge::Finished { code, stdout: String::new(), stderr });
}

/// What:
///  The whole attempt, with `Bail` standing for every early ending.
/// Why:
///   Early endings are many and mostly mean "forward"; the question mark keeps the sequence readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attempt(input): Promise<IsolatedMerge>; // throws Bail
/// ```
fn attempt(input: &IsolatedMergeInput<'_>) -> Result<IsolatedMerge, Bail> {
    for round in 1..=MAX_ROUNDS {
        match attempt_round(input) {
            Err(Bail::Again(reason)) => {
                debug(TAG, format!("round {round} of {MAX_ROUNDS}: {reason}").as_str());
                let _noted = std::io::stderr().write_all(format!("cli-git: git merge: {reason}; merging again on the current branch.\n").as_bytes());
            }
            finished => return finished,
        }
    }
    return Err(refused(
        2,
        format!(
            "cli-git: git merge: the branch moved or the index lock was taken in {MAX_ROUNDS} merges in a row, so the wrapper \
             stopped; nothing was changed. Run the merge again.\n"
        ),
    ));
}

/// What:
///  One round of the attempt: everything from the recovery of dead merges to the application, on the branch as it is now.
/// Why:
///   A round that finds the branch moved by the time it applies ends with the checkout as it was, and the next round
///  starts from the new head.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attemptRound(input): Promise<IsolatedMerge>; // throws Bail
/// ```
fn attempt_round(input: &IsolatedMergeInput<'_>) -> Result<IsolatedMerge, Bail> {
    if !input.command.is_builtin() || input.command.word() != b"merge" {
        return Err(Bail::Forward(None));
    }
    let (git_dir, root, common_dir): (&Path, &Path, &Path) = match input.identity {
        WorktreeIdentity::MainWorktree { git_dir, worktree_root, common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { git_dir, worktree_root, common_dir, .. } => {
            (git_dir.as_path(), worktree_root.as_path(), common_dir.as_path())
        }
        _ => return Err(Bail::Forward(None)),
    };
    recover_dead_merges(input, git_dir, root)?;
    let shape: MergeShape = match parse_merge_shape(input.command.region()) {
        ParsedMerge::Supported(found) => found,
        ParsedMerge::Unsupported(reason) => {
            debug(TAG, format!("native Git runs this merge: {reason}").as_str());
            return Err(Bail::Forward(None));
        }
    };
    if IN_PROGRESS_MARKERS.iter().any(|name| { return git_dir.join(name).exists(); }) {
        return Err(Bail::Forward(None));
    }
    let session: Session = Session {
        context: GitContext {
            real_git: input.real_git.to_path_buf(),
            overlay: input.overlay.to_vec(),
            global_prefix: input.command.arguments[..input.command.subcommand_index].to_vec(),
            // The merge message passes through `git stripspace` and `git fmt-merge-msg` as input.
            common_dir: Some(common_dir.to_path_buf()),
        },
    };
    let Some(reference) = shape.reference.to_str().map(String::from) else {
        return Err(Bail::Forward(None));
    };
    // Exit 1 means another session has staged work, 0 means the index equals `HEAD`; the merge is completed here in both
    //  cases, because native Git's merge stashes first (`git stash create`), dies when `index.lock` is held, and moves the
    //  branch only after it changed the checkout. What native Git does better stays its own: a conflict that the person
    //  resolves in the worktree, and an overlap with unsaved work, which it words itself.
    let staged: GitOutput = session.run(&["diff-index", "--cached", "--quiet", "HEAD", "--"])?;
    let staged_work: bool = match staged.code {
        Some(1) => true,
        Some(0) => false,
        _ => return Err(Bail::Forward(None)),
    };
    let auto_edit: Option<String> = environment_value(input.environment, "GIT_MERGE_AUTOEDIT")
        .map(|value| { return value.to_string_lossy().into_owned(); });
    let opens_editor: bool = !shape.no_edit
        && editor_would_open(auto_edit.as_deref(), stdin_and_stdout_are_one_terminal()).unwrap_or(true);
    if opens_editor && !staged_work {
        return Err(Bail::Forward(None));
    }
    if opens_editor {
        return Err(Bail::Forward(Some(String::from(
            "cli-git: this merge would open an editor, so the wrapper does not complete it in isolation while the \
             index holds staged changes; native Git refuses it. Run `git merge --no-edit` (or set \
             GIT_MERGE_AUTOEDIT=no) and the wrapper completes the merge without touching the staged changes.\n",
        ))));
    }
    let config: MergeConfig = read_config(&session)?;
    if config.deferred {
        return Err(Bail::Forward(None));
    }
    // `FETCH_HEAD` is the merge of a pull: the one head the file lists for merging, and the file's lines name it in the message.
    let fetched: Option<(String, String)> = if reference == "FETCH_HEAD" {
        Some(read_fetch_head(git_dir)?)
    } else {
        None
    };
    let target_spec: String = match &fetched {
        Some((oid, _)) => format!("{oid}^{{commit}}"),
        None => format!("{reference}^{{commit}}"),
    };
    let both = session.run(&["rev-parse", "HEAD^{commit}", target_spec.as_str()])?;
    let ids: Vec<String> = lines_of(&both);
    if both.code != Some(0) || ids.len() != 2 {
        return Err(Bail::Forward(None));
    }
    let (head, target): (&str, &str) = (ids[0].as_str(), ids[1].as_str());
    let bases: Vec<String> = lines_of(&session.run(&["merge-base", "--all", head, target])?);
    let fast_forwardable: bool = bases.len() == 1 && bases[0] == head;
    if bases.len() == 1 && bases[0] == target {
        return Err(Bail::Forward(None));
    }
    if bases.is_empty() && !shape.allow_unrelated_histories {
        return Err(Bail::Forward(None));
    }
    let merge_commit_wanted: bool = match shape.fast_forward {
        FastForwardChoice::Never => true,
        FastForwardChoice::Allowed => false,
        FastForwardChoice::Default => config.fast_forward == Some(false),
    };
    if config.fast_forward_only && shape.fast_forward == FastForwardChoice::Default {
        return Err(Bail::Forward(None));
    }
    if fast_forwardable && !merge_commit_wanted {
        return Err(Bail::Forward(None));
    }
    let description: Option<String> = if shape.message.is_some() {
        None
    } else if let Some((_, lines)) = fetched {
        Some(lines)
    } else {
        let full_name: String = session.text(&["rev-parse", "--symbolic-full-name", reference.as_str()])?;
        Some(describe_merge_source(reference.as_str(), full_name.as_str(), target).ok_or(Bail::Forward(None))?)
    };
    let (tree, second_pass_files): (String, Vec<Merged>) = merge_tree(
        &session,
        &shape,
        (head, target, reference.as_str()),
        (common_dir, root, input.environment),
        staged_work,
    )?;
    let incoming: Vec<Vec<u8>> = diff_paths(
        session
            .run(&["diff-tree", "-r", "--name-only", "-z", "--no-renames", head, tree.as_str()])?
            .stdout
            .as_slice(),
    );
    let overlap: Vec<Vec<u8>> = overlapping_paths(unsaved_paths(&session)?.as_slice(), incoming.as_slice());
    if !overlap.is_empty() && !staged_work {
        // Native Git words a refusal over unsaved work in the worktree itself.
        return Err(Bail::Forward(None));
    }
    if !overlap.is_empty() {
        return Err(refused(2, overlap_finding(reference.as_str(), overlap.as_slice())));
    }
    let message: Vec<u8> = match (&shape.message, description) {
        (Some(given), _) => clean_message(&session.context, given.as_slice())?,
        (None, Some(line)) => {
            let subject: Vec<u8> = default_merge_message(&session.context, line.as_str(), shape.log)?;
            clean_message(&session.context, subject.as_slice())?
        }
        (None, None) => return Err(Bail::Forward(None)),
    };
    let new_oid: String = match commit_merge(&session, &shape, &config, (head, target, tree.as_str()), message) {
        Ok(created) => created,
        Err(reason) => {
            return Err(refused(
                128,
                format!("cli-git: git merge {reference}: the merge commit could not be created, and nothing was changed: {reason}\n"),
            ));
        }
    };
    let plan: Plan<'_> = Plan {
        shape: &shape,
        config: &config,
        git_dir,
        root,
        old_oid: head,
        new_oid: new_oid.as_str(),
    };
    return apply(input, &session, &plan).map(|done| { return with_second_pass_report(done, second_pass_files.as_slice()); });
}

/// What:
///  The finished merge with the lines that list the files the second pass merged by word, ahead of Git's own words.
/// Why:
///   The journal entries and the lines are produced only now, after the merge is in the checkout, so neither says a merge
///  happened that was refused or undone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withSecondPassReport(done: IsolatedMerge, merged: Merged[]): IsolatedMerge;
/// ```
fn with_second_pass_report(done: IsolatedMerge, merged: &[Merged]) -> IsolatedMerge {
    match done {
        IsolatedMerge::Finished { code: 0, stdout, stderr } if !merged.is_empty() => {
            let mut report: String = String::new();
            for line in announce("cli-git: ", (CLIENT_IDENTIFIER, "merge"), merged) {
                report.push_str(line.as_str());
                report.push('\n');
            }
            return IsolatedMerge::Finished { code: 0, stdout, stderr: report + stderr.as_str() };
        }
        other => return other,
    }
}

/// What:
///  Recover merges that died, before this merge reads the checkout.
/// Why:
///   A merge killed between Git's checkout update and the branch move leaves the checkout half done, and the
///  checks that follow would read it as another session's staged work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInterruptedMerges({ place, gitDir });
/// ```
fn recover_dead_merges(input: &IsolatedMergeInput<'_>, git_dir: &Path, root: &Path) -> Result<(), Bail> {
    let common_dir: PathBuf = match input.identity {
        WorktreeIdentity::MainWorktree { common_dir, .. } | WorktreeIdentity::LinkedWorktree { common_dir, .. } => {
            common_dir.clone()
        }
        _ => return Ok(()),
    };
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext {
            real_git: input.real_git.to_path_buf(),
            overlay: input.overlay.to_vec(),
            global_prefix: Vec::new(),
            common_dir: Some(common_dir.clone()),
        },
        cwd: root.to_path_buf(),
        common_dir,
    };
    match recover_interrupted_merges(&place, git_dir) {
        Ok(_) => return Ok(()),
        Err(error) => {
            return Err(refused(
                2,
                render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.0.as_str(), None),
            ));
        }
    }
}

/// What:
///  The head a pull's `FETCH_HEAD` lists for merging, with the file's text that names it in the merge message.
/// Why:
///   `git merge FETCH_HEAD` merges the lines marked for merging and words the message from the whole file
///  (`handle_fetch_head`, `builtin/merge.c`); this completes the merge only for exactly one such head and leaves several, or
///  none, to native Git, which words that refusal itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readFetchHead(gitDir: string): { oid: string; text: string }; // throws Bail
/// ```
fn read_fetch_head(git_dir: &Path) -> Result<(String, String), Bail> {
    // A missing or unreadable file is native Git's to report.
    let Ok(text) = std::fs::read_to_string(git_dir.join("FETCH_HEAD")) else {
        return Err(Bail::Forward(None));
    };
    let mut heads: Vec<String> = fetch_head_merge_heads(text.as_str());
    if heads.len() != 1 {
        return Err(Bail::Forward(None));
    }
    return Ok((heads.remove(0), text));
}

/// What:
///  Compute the merge from objects and return its tree and the files the word pass merged, or refuse a conflict.
/// Why:
///   `merge-tree --write-tree` exits 1 for a conflicted merge and prints the tree with the conflicted files' stages; nothing
///  is written to the worktree or the index. The conflicts get the word-level second pass, and the merge continues only when
///  every conflicted file resolved; otherwise it is refused as before, naming the paths. The attributes of the first side's
///  tip apply, because no worktree is read. A conflict with a clean index is not computed here at all: native Git writes
///  the markers for the person, and the second pass then runs on the stopped merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeTree({ session, shape, ends, places, stagedWork }): Promise<{ tree: string; merged: Merged[] }>;
/// ```
fn merge_tree(
    session: &Session,
    shape: &MergeShape,
    ends: (&str, &str, &str),
    places: (&Path, &Path, &[(OsString, OsString)]),
    staged_work: bool,
) -> Result<(String, Vec<Merged>), Bail> {
    let (head, target, reference): (&str, &str, &str) = ends;
    let (common_dir, root, environment): (&Path, &Path, &[(OsString, OsString)]) = places;
    let mut arguments: Vec<&str> = vec!["merge-tree", "--write-tree", "-z", "--no-messages"];
    if shape.allow_unrelated_histories {
        arguments.push("--allow-unrelated-histories");
    }
    arguments.push(head);
    arguments.push(target);
    let probing: ContextRunner<'_> = ContextRunner { context: &session.context, cwd: Path::new("."), without_prefix: false, object_directory: None, environment: Vec::new() };
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(environment, session.context.overlay.as_slice());
    let probe: AttributeProbe<'_> = AttributeProbe { directory: root, global_prefix: session.context.global_prefix.as_slice(), variables: variables.as_slice(), runner: &probing };
    let runner: ContextRunner<'_> = ContextRunner {
        context: &session.context,
        cwd: Path::new("."),
        without_prefix: false,
        object_directory: None,
        environment: attribute_tree_variables(head, &probe),
    };
    let merged: GitReply = runner.run(arguments.as_slice(), None, None)?;
    let (tree, conflicts) = conflicts_of_merge_tree(merged.stdout.as_slice());
    match merged.code {
        Some(0) if !tree.is_empty() => return Ok((tree, Vec::new())),
        Some(0) => return Err(Bail::from(String::from("merge-tree printed no tree"))),
        // A conflict with a clean index is left to native Git, which writes the markers for the person to resolve; the
        //  second pass then runs on the stopped merge in the worktree.
        Some(1) if !staged_work => return Err(Bail::Forward(None)),
        Some(1) => {
            // The second pass runs from the top of the worktree with the person's configuration options only.
            let top_context: GitContext = GitContext { global_prefix: config_options(&session.context.global_prefix), ..session.context.clone() };
            let top_runner: ContextRunner<'_> = ContextRunner { context: &top_context, cwd: root, ..runner };
            if let Some(resolved) = resolve_tree(&top_runner, common_dir, tree.as_str(), conflicts.as_slice()) {
                return Ok((resolved.tree, resolved.merged));
            }
            let paths: Vec<String> = conflicts.iter().map(|conflict| { return path_message_text(conflict.path.as_slice()); }).collect();
            return Err(refused(2, conflict_finding(reference, paths.as_slice())));
        }
        _ => return Err(Bail::from(format!("git merge-tree failed: {}", String::from_utf8_lossy(&merged.stderr).trim()))),
    }
}

/// What:
///  Create the merge commit with its parents in Git's order, signing it as the options and settings ask.
/// Why:
///   `commit-tree` signs only when given `-S` and never reads `commit.gpgSign`; a merge on a signed branch must not
///  be the one unsigned commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ session, shape, config, parents, tree, message }): Promise<string>;
/// ```
fn commit_merge(
    session: &Session,
    shape: &MergeShape,
    config: &MergeConfig,
    ends: (&str, &str, &str),
    message: Vec<u8>,
) -> Result<String, String> {
    let (head, target, tree): (&str, &str, &str) = ends;
    let signing: Option<String> = match &shape.signing {
        Signing::Never => None,
        Signing::Sign(None) => Some(String::from("-S")),
        Signing::Sign(Some(key)) => Some(format!("-S{}", String::from_utf8_lossy(key))),
        Signing::Default => if config.sign_commits { Some(String::from("-S")) } else { None },
    };
    let mut arguments: Vec<&str> = vec!["commit-tree", tree, "-p", head, "-p", target];
    if let Some(flag) = signing.as_deref() {
        arguments.push(flag);
    }
    let created: GitOutput = session.run_with_input(arguments.as_slice(), message)?;
    if !created.succeeded() {
        return Err(created.error_text());
    }
    return Ok(String::from_utf8_lossy(&created.stdout).trim().to_owned());
}

/// What:
///  Record the intent, let Git move the checkout to the merge commit, and report.
/// Why:
///   Git's fast-forward is the two-way update that keeps every other staged entry and unstaged change, refuses an
///  untracked file in the way, and moves the branch only after the checkout is updated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function apply({ session, shape, locations, commits }): Promise<IsolatedMerge>;
/// ```
fn apply(input: &IsolatedMergeInput<'_>, session: &Session, plan: &Plan<'_>) -> Result<IsolatedMerge, Bail> {
    let (git_dir, root): (&Path, &Path) = (plan.git_dir, plan.root);
    let (old_oid, new_oid): (&str, &str) = (plan.old_oid, plan.new_oid);
    let shape: &MergeShape = plan.shape;
    let reference: String = shape.reference.to_string_lossy().into_owned();
    let target_ref: Option<String> = {
        let named: GitOutput = session.run(&["symbolic-ref", "--quiet", "HEAD"])?;
        if named.succeeded() { Some(String::from_utf8_lossy(&named.stdout).trim().to_owned()) } else { None }
    };
    // Git's fast-forward takes `index.lock` before it changes anything and dies when another program holds it. The merge waits
    //  for the holder here, bounded and naming it; a lock taken in the instant between is met by the round made again.
    wait_for_index_release_here(git_dir.join("index").as_path(), "merge");
    let transaction_id: String = random_uuid().map_err(|error| { return Bail::from(error.0); })?;
    let owner = current_transaction_owner(transaction_id.as_str(), now_iso().as_str())
        .map_err(|error| { return Bail::from(error.to_string()); })?;
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: root.to_string_lossy().into_owned(),
        target_ref,
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
    };
    let record: PathBuf = write_merge_intent(git_dir, &intent)
        .map_err(|error| { return Bail::from(format!("the merge record could not be written: {error}")); })?;
    reach_phase(Phase::MergeIntent)?;
    let mut overlay: Vec<(OsString, OsString)> = input.overlay.to_vec();
    overlay.push((OsString::from("GIT_REFLOG_ACTION"), OsString::from(format!("merge {reference}"))));
    let mover: Session = Session {
        context: GitContext { overlay, ..session.context.clone() },
    };
    // A start failure is handled like a refusal: the record settles whatever Git began.
    let moved: GitOutput = mover.run(&["merge", "--ff-only", "--no-edit", new_oid]).unwrap_or_else(|reason| {
        return GitOutput { code: None, stdout: Vec::new(), stderr: reason.into_bytes() };
    });
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext { global_prefix: Vec::new(), ..session.context.clone() },
        cwd: root.to_path_buf(),
        common_dir: git_dir.to_path_buf(),
    };
    if !moved.succeeded() {
        let detail: String = moved.error_text();
        let lock_taken: bool = detail.contains("index.lock");
        return Err(match settle_failed_merge(&place, record.as_path(), &intent) {
            Ok(MergeRecoveryAction::BranchMoved) => Bail::Again(format!("{reference} was merged on a branch that moved meanwhile")),
            Ok(MergeRecoveryAction::Discarded) if lock_taken => Bail::Again(String::from("another program took the index lock first")),
            Ok(MergeRecoveryAction::Completed) => Bail::Done(IsolatedMerge::Finished {
                code: 0,
                stdout: String::new(),
                stderr: format!(
                    "cli-git: git merge {reference}: Git reported an error ({detail}) after it had started, \
                     and the wrapper completed the merge.\n"
                ),
            }),
            Ok(_) => refused(
                moved.code.unwrap_or(2),
                format!("{}{}", String::from_utf8_lossy(&moved.stderr), not_applied_finding(reference.as_str(), detail.as_str())),
            ),
            Err(error) => refused(2, render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.0.as_str(), None)),
        });
    }
    reach_phase(Phase::MergeApplied)?;
    if let Err(error) = remove_merge_intent(record.as_path()) {
        debug(TAG, format!("the merge record could not be removed: {error}").as_str());
    }
    // The fast-forward carries no `--quiet`, and what Git says of it ("Updating a..b", "Fast-forward", its diffstat) is about
    //  the wrapper's own step, not about the merge the person asked for. The person's streams carry exactly what native Git's
    //  merge prints (the wrapper's lines below), so what Git wrote to standard output about the step is kept in the journal. What
    //  it wrote to standard error (a warning) still reaches the person, as it did before the option was dropped.
    record_streams(&["merge", "--ff-only", "--no-edit", new_oid], &moved.stdout, &[]);
    let mut stdout: String = String::new();
    if !shape.quiet {
        stdout.push_str("Merge made by the 'ort' strategy.\n");
        if shape.stat.unwrap_or(plan.config.stat)
            && let Ok(stat) = session.run(&["diff", "--stat", "--summary", old_oid, new_oid])
        {
            stdout.push_str(String::from_utf8_lossy(&stat.stdout).as_ref());
        }
    }
    return Ok(IsolatedMerge::Finished {
        code: 0,
        stdout,
        stderr: String::from_utf8_lossy(&moved.stderr).into_owned(),
    });
}

/// What the apply step needs to know about the merge it applies.
struct Plan<'a> {
    /// The command line.
    shape: &'a MergeShape,
    /// The settings read for the merge.
    config: &'a MergeConfig,
    /// The worktree's Git directory.
    git_dir: &'a Path,
    /// The worktree top level.
    root: &'a Path,
    /// The commit the branch holds before the merge.
    old_oid: &'a str,
    /// The merge commit.
    new_oid: &'a str,
}

/// What:
///  Reach a marker phase, failing the invocation when the marker variable is malformed.
/// Why:
///   The crash controls arm a phase by environment; a malformed value must not be ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase });
/// ```
fn reach_phase(phase: Phase) -> Result<(), Bail> {
    return reach_transaction_phase(phase, None).map_err(|error| {
        return refused(2, format!("cli-git: {error}\n"));
    });
}

/// Boolean settings, the editor choice and the markers of other operations, which need no repository.
#[cfg(test)]
#[path = "merge_isolation_tests.rs"]
mod tests;

/// The settings, the staged-work check and the first decisions of an attempt, each in a real repository.
#[cfg(test)]
#[path = "merge_isolation_repository_tests.rs"]
mod repository_tests;
