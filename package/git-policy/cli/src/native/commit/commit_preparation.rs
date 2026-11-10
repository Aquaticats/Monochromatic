//! What:
//!  Native `git commit` in the shadow repository against the private commit index,
//!       the prepared commit read back from the shadow,
//!       and `prepared.json`.
//! Why:
//!  Git owns the hooks,
//! the editor,
//! the templates,
//! the message cleanup and the signing;
//!  this module only points Git at the shadow and the private index.
//!  Every hook is off in the shadow (its config switches the commit events off and the hooks-off option comes
//!  first),
//! so Git starts no external program for a hook and never starts the wrapper.
//!  A failing preparation removes the shadow so no shadow outlives the failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runNativePreparation({ gitPath, cwd, globalArgs, commitArgs, shadowPath, hookPlan, commitIndexPath });
//! ```

/// Private file write for the journal record.
/// Journal record shape and its encoder.
/// Invocation capture facts the shadow was built from.
use crate::capture_invocation::InvocationCapture;
/// The hooks-off value Git is given again after the caller's options.
use crate::hooks_off::NO_HOOKS_DIRECTORY;
/// Shadow removal,
/// so a failed preparation leaves no shadow.
/// The shadow request and the real-Git command builder.
/// The shadow request every shadow command of the wrapper is made of.
use crate::shadow_git::{git_dir_option, shadow_request};
use crate::forwarding::git_command;
use crate::signal_relay::SignalGuard;
/// Context,
/// one required Git start and failure text.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
/// `OsString` is owned operating-system text.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::Path;

/// Variables native Git must not inherit,
/// so every object it writes lands in the shadow store.
pub const NATIVE_UNSET_VARIABLES: [&str; 4] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY"];

/// What:
///  Everything one native preparation needs from its transaction.
/// Why:
///   The incumbent passes these as separate options,
/// and grouping them keeps the call short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runNativePreparation({ gitPath, cwd, globalArgs, commitArgs, shadowPath, worktreeRoot, hooksDirectory, commitIndexPath });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PreparationRequest<'request> {
    /// Invocation capture the shadow was created from.
    pub capture: &'request InvocationCapture,
    /// Shadow repository of the transaction.
    pub shadow: &'request Path,
    /// Private commit index.
    pub commit_index: &'request Path,
    /// The commit's own scratch directory,
    /// which holds the file the native-commit configuration travels in and is removed with the commit.
    pub scratch: &'request Path,
    /// Kept global options of the invocation.
    pub global_args: &'request [String],
    /// Commit arguments without pathspecs or `--only`.
    pub commit_args: &'request [String],
    /// The object directory Git writes its new objects into;
    /// nothing leaves them in the shadow's own store.
    ///  The commit service names the real one,
    /// so the commit it lands is already where the branch can point at it.
    pub object_directory: Option<&'request Path>,
}

/// What:
///  Why a preparation stopped.
/// Why:
///   A nonzero native exit is the invocation's own exit code,
/// while a start failure is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NativeCommitFailedError extends Error { exitCode: number }
/// ```
#[derive(Debug)]
pub enum PreparationError {
    /// A Git step around the commit failed.
    Git(GitFailure),
}

impl std::fmt::Display for PreparationError {
    /// Writes the wording of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Git(error) => write!(formatter, "{error}"),
        };
    }
}

/// What:
///  The prepared commit read back from the shadow `HEAD`.
/// Why:
///   Replay rebuilds every later commit from the exact object bytes,
///  and the tree and signature decide what landing may do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedCommit = { oid: string; treeOid: string; signed: boolean; raw: RawCommit };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCommit {
    /// Prepared commit,
    /// the shadow `HEAD` value.
    pub oid: String,
    /// Prepared commit tree.
    pub tree_oid: String,
    /// Whether the header block carries a `gpgsig` or `gpgsig-sha256` header.
    pub signed: bool,
    /// Exact commit object bytes.
    pub raw: Vec<u8>,
}

/// What:
///  Builds the arguments of native `git commit` in the shadow.
/// Why:
///   The kept global options come first,
/// then the shadow,
/// then the file the native-commit configuration travels in,
/// then a hooks directory that cannot exist,
/// which also beats a
///  `-c core.hooksPath` of the caller.
///  The shadow's own config switches every commit event off,
/// so no external program runs for a hook.
///  No `--work-tree` is passed:
/// the shadow's own config names the work tree.
///  Every configuration key native `git commit` reads travels in that file,
/// because an `includeIf` condition that named the real Git directory no longer matches in the shadow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...globalArgs, `--git-dir=${shadowPath}`, ...nativeConfiguration, '-c', `core.hooksPath=${NO_HOOKS}`, 'commit', ...commitArgs]
/// ```
///
/// # Errors
///
/// The error of writing the file the configuration travels in.
fn native_commit_arguments(request: &PreparationRequest) -> Result<Vec<OsString>, PreparationError> {
    // The caller's directory changes already applied when the wrapper placed the shadow, and the shadow names its own repository:
    //  forwarding a `-C`, `--git-dir`, `--work-tree` or `--bare` would move Git away from the shadow or name the real repository
    //  beside it. The configuration options the caller wrote stay, because they are the person's settings for the commit.
    let own_globals: Vec<OsString> = request.global_args.iter().map(OsString::from).collect();
    let mut arguments: Vec<OsString> = crate::repository_selection::without_repository_selection(own_globals.as_slice());
    arguments.push(git_dir_option(request.shadow));
    let configuration: Vec<OsString> = crate::native_commit_configuration::native_commit_configuration(
        request.capture.layout.config_snapshot.as_ref(),
        request.scratch,
    )
    .map_err(configuration_write_failure)?;
    arguments.extend(configuration);
    arguments.push(OsString::from("-c"));
    arguments.push(OsString::from(format!("core.hooksPath={NO_HOOKS_DIRECTORY}")));
    arguments.push(OsString::from("commit"));
    arguments.extend(request.commit_args.iter().map(OsString::from));
    return Ok(arguments);
}

/// What:
///  The preparation failure of a configuration file the native commit reads that could not be written.
/// Why:
///   A command that cannot be given the person's configuration must not run with it missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = (error: Error) => new GitFailedError(`writing the configuration the native commit reads failed: ${error}`);
/// ```
fn configuration_write_failure(error: std::io::Error) -> PreparationError {
    return PreparationError::Git(GitFailure(format!("writing the configuration the native commit reads failed: {error}")));
}

/// What:
///  Runs native `git commit` in the shadow with the terminal attached and returns its exit code,
/// stopping it at `SCRATCH_COMMIT_STEP_CEILING` when the message came with the command.
/// Why:
///   The editor output reaches the user unchanged,
///  so the child inherits standard streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args, indexPath, unsetEnvironment, stdio: 'inherit', allowFailure: true });
/// ```
///
/// # Errors
///
/// `PreparationError::Git` when Git could not be started,
/// or when the step lasted to its ceiling and was ended (nothing was committed).
pub fn run_native_commit(
    context: &GitContext,
    cwd: &Path,
    request: &PreparationRequest,
) -> Result<i32, PreparationError> {
    return run_native_commit_within(context, cwd, request, crate::native_commit_waiting_notice::scratch_ceiling());
}

/// What:
///  `run_native_commit` with the ceiling given.
/// Why:
///   The owner's answer (2026-11-07,
///   "limit unless editor"):
/// the ceiling applies only when the message came with the command;
/// with an editor open the step is waited for,
/// and the notice says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runNativeCommitWithin({ context, cwd, request, ceiling });
/// ```
pub fn run_native_commit_within(
    context: &GitContext,
    cwd: &Path,
    request: &PreparationRequest,
    ceiling: std::time::Duration,
) -> Result<i32, PreparationError> {
    let arguments: Vec<OsString> = native_commit_arguments(request)?;
    let mut command = git_command(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice());
    command
        .current_dir(cwd)
        .env("GIT_INDEX_FILE", request.commit_index);
    for name in NATIVE_UNSET_VARIABLES {
        command.env_remove(name);
    }
    if let Some(directory) = request.object_directory {
        command.env("GIT_OBJECT_DIRECTORY", directory);
    }
    // This step runs with the terminal attached and no watch, so no other record knows it: its own entry says how long it took,
    //  signing and an editor included.
    let step: String = crate::add_timing::git_step_label(arguments.as_slice());
    let cadence: crate::wait_progress::Cadence = crate::child_watch::WatchTerms::for_class(crate::child_class::ChildClass::Object).cadence;
    let snapshot: Option<&crate::config_snapshot::ConfigSnapshot> = request.capture.layout.config_snapshot.as_ref();
    let notice: crate::native_commit_waiting_notice::NativeCommitNotice = crate::native_commit_waiting_notice::NativeCommitNotice::start_within(
        crate::native_commit_waiting_notice::NoticeStart {
            snapshot,
            commit_arguments: request.commit_args,
            cadence,
            ceiling,
            sink: Box::new(std::io::stderr()),
        },
    );
    let began: std::time::Instant = std::time::Instant::now();
    // Two enforcers of one ceiling: the notice's thread, which prints the lines and ends the step through the descriptor
    //  taken at its start, and the wait that owns the child, which needs no thread and no number a reuse can reach.
    let duty: crate::signal_relay::CeilingDuty<'_> = crate::signal_relay::CeilingDuty {
        told: notice.step_number(),
        bound: notice.ceiling_that_applies(),
    };
    let finished = run_guarded_child(&mut command, duty);
    // The step's own figure is the child alone. The notice's own end is reported beside it in `scratchSteps`, and the time
    // the signing program ran below the step is there too, so a long step names no work the person's terminal did and no
    // line the notice could not hand to a reader that stopped reading.
    crate::transaction_step_durations::note_step(step.as_str(), began, began.elapsed());
    let mut facts: crate::scratch_commit_step::ScratchStepFacts = notice.finish();
    // One record of a stop, whoever ended the step first: the notice's own thread, or the wait that owns the child.
    if let Ok((_outcome, Some(stop))) = &finished
        && facts.stopped_after.is_none()
    {
        facts.armed = true;
        facts.stopped_after = Some(stop.waited);
        facts.below_unlisted = !stop.below_listed;
        facts.below_unended = stop.below_unended;
    }
    let stopped_after: Option<std::time::Duration> = facts.stopped_after;
    let below_unlisted: bool = facts.below_unlisted;
    let below_unended: u32 = facts.below_unended;
    crate::scratch_commit_step::note_scratch_step(facts);
    if let Some(waited) = stopped_after {
        let signs: bool = crate::native_commit_waiting_notice::signs_commit(snapshot, request.commit_args);
        crate::diagnostic_log::warn("runNativeCommit", format!("the scratch git commit was stopped after {} ms at its ceiling of {} ms", waited.as_millis(), ceiling.as_millis()).as_str());
        let stop: crate::scratch_step_stop_words::ScratchStepStop = crate::scratch_step_stop_words::ScratchStepStop { waited, ceiling, signs, below_unlisted, below_unended };
        return Err(PreparationError::Git(GitFailure(crate::scratch_step_stop_words::stop_words(stop))));
    }
    let (outcome, _no_ceiling): (crate::forwarding::ChildOutcome, Option<crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop>) = finished?;
    return Ok(crate::forwarding::exit_code(outcome));
}

/// What:
///  Run the native commit under the signal guard,
///  tell the notice which process is the step as soon as it starts,
///  and answer how it ended,
///  with the ceiling's stop when the wait had to end it.
/// Why:
///   A signal sent to the wrapper while Git runs reaches Git when a process sent it;
/// Git's ending decides the status.
///  The number is what lets the notice's ceiling reach this step and no other child of this process,
/// which the landing service runs beside it,
/// and `duty` is what keeps a ceiling when nothing the notice holds names the step for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using guard = installSignalGuard(); return await guard.runChildNaming(command, { told: notice.stepNumber(), bound, anotherEndsIt });
/// ```
fn run_guarded_child(
    command: &mut std::process::Command,
    duty: crate::signal_relay::CeilingDuty<'_>,
) -> Result<(crate::forwarding::ChildOutcome, Option<crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop>), PreparationError> {
    let guard: SignalGuard = match SignalGuard::install() {
        Ok(installed) => installed,
        Err(error) => {
            return Err(PreparationError::Git(GitFailure(format!("installing the signal guard failed: {error}"))));
        }
    };
    let outcome = match guard.run_child_telling_its_number(command, duty) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(PreparationError::Git(GitFailure(format!("git commit could not start: {error}"))));
        }
    };
    return Ok(outcome);
}

/// What:
///  Reads the prepared commit from the shadow `HEAD`:
///  its object,
/// its tree and whether it is signed.
/// Why:
///   The header block ends at the first empty line;
/// a signature header in it marks the commit signed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readPreparedCommit({ gitPath, shadowPath });
/// ```
pub fn read_prepared_commit(context: &GitContext, shadow: &Path) -> Result<PreparedCommit, GitFailure> {
    let oid_output: GitOutput = run_git_checked(
        context,
        &shadow_request(shadow, shadow, &["rev-parse", "--verify", "HEAD^{commit}"]),
    )?;
    let oid: String = String::from_utf8_lossy(oid_output.stdout.as_slice()).trim().to_string();
    let raw: Vec<u8> = run_git_checked(
        context,
        &shadow_request(shadow, shadow, &["cat-file", "commit", oid.as_str()]),
    )?
    .stdout;
    let mut header_end: usize = raw.len();
    for (position, window) in raw.windows(2).enumerate() {
        if window == b"\n\n" {
            header_end = position;
            break;
        }
    }
    let header: String = String::from_utf8_lossy(&raw[..header_end]).into_owned();
    // The first `tree ` line names the tree; any `gpgsig` line marks the commit as signed.
    let mut tree_oid: String = String::new();
    let mut found_tree: bool = false;
    let mut signed: bool = false;
    for line in header.lines() {
        if !found_tree && let Some(named) = line.strip_prefix("tree ") {
            tree_oid = named.to_string();
            found_tree = true;
        }
        if line.starts_with("gpgsig ") || line.starts_with("gpgsig-sha256 ") {
            signed = true;
        }
    }
    return Ok(PreparedCommit {
        oid,
        tree_oid,
        signed,
        raw,
    });
}

/// Native preparation controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_preparation_tests.rs"]
mod tests;
