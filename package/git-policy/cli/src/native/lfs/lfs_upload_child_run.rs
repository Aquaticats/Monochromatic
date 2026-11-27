//! What:
//!  One child run of the Git LFS upload:
//! the commands an upload runs,
//! the standard error captured from one of them,
//! and the wait that ends it.
//! Why:
//!  Both faces of the upload (`lfs_upload.rs`) start the same program with the same arguments
//! and read what it printed through the same steps,
//! so those steps live here,
//! beside them and in a file of their own so that each file passes the package's length limit.
//! A child's standard error is where `git-lfs` prints the URL it used,
//! and that URL can carry the `lfs.url` token,
//! so what a caller relays is the masked head of the capture and never the file itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commands = uploadCommands(git, plan);
//! const { ended, relayed } = await runWithCapturedStderr(command);
//! ```

/// The coordinator's Git the commands are made for.
use crate::auto_push_single_flight::PushGit;
/// The grace a stopped child gets,
/// what ended it,
/// the wait a stalled child is ended by,
/// the least traffic the upload's connection must carry,
/// and the line that says how a longer one is allowed.
use crate::bounded_push::{
    STOP_GRACE,
    StopCause,
    command_label,
    connected_window,
    push_minimum_rate_bytes,
    push_patience,
    wait_until_done_noting,
};
/// The child held until it is reaped,
/// awaited on its pidfd.
use crate::child_awaited_through_a_pidfd::{
    OwnedChild,
    adopt,
};
/// The invocation's environment values.
use crate::child_environment::environment_value;
/// How a child ended,
/// as a status.
use crate::forwarding::{
    ChildOutcome,
    outcome_of,
};
/// The process's runtime,
/// which this synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// The masked head of the program's own words.
use crate::lfs_output_mask::{
    MaskedHead,
    mask_url_secrets_within,
};
/// The program lookup,
/// and the refusal a program that could not be started prints.
use crate::lfs_pre_push::{
    find_program,
    start_failed_message,
};
/// The cap on the bytes of a child's standard error that are relayed.
use crate::lfs_upload_explanations::MAX_RELAYED_BYTES;
/// What to upload.
use crate::lfs_upload_plan::UploadPlan;
/// The anonymous file the captured standard error goes to.
use crate::scratch_location::scratch_file;
/// Relays signals to the child,
/// which starts it and hands it over still published.
use crate::signal_relay::SignalGuard;
use crate::signal_relay_relayed_child::RelayedChild;
/// The end of a stopped child.
use crate::stop_of_a_push_and_sweep_of_its_locks::stop_push;
/// The locks a stopped child leaves,
/// none of which the upload claims.
use crate::stopped_git_locks::LockLedger;
/// Owned OS text for arguments.
use std::ffi::OsString;
/// Reading the captured standard error back.
use std::io::{
    Read,
    Seek,
    SeekFrom,
};
/// The real Git whose directory leads `PATH`.
use std::path::Path;
/// The child and its standard error.
use std::process::{
    Command,
    ExitStatus,
    Stdio,
};
/// `PATH` with the directory of the real Git first,
/// so a `git` that `git-lfs` starts is the real Git and never the
///  wrapper (Git does the same for the hooks it starts).
pub(crate) fn path_with_real_git_first(
    environment: &[(OsString, OsString)],
    real_git: &Path,
) -> OsString {
    let existing: OsString = environment_value(environment, "PATH").unwrap_or_default();
    let mut directories: Vec<std::path::PathBuf> = Vec::new();
    if let Some(directory) = real_git.parent() {
        directories.push(directory.to_path_buf());
    }
    directories.extend(std::env::split_paths(existing.as_os_str()));
    return std::env::join_paths(directories).unwrap_or(existing);
}

/// What:
///  The commands of one upload:
/// one `git-lfs push` per remote and per run of it,
/// or nothing when no `git-lfs` is on the caller's `PATH`.
/// Why:
///   Both faces run the same programs with the same arguments,
/// so a value either face reads of the plan reaches the same child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = findProgram(env.PATH); if (!program) return undefined; return plan.remotes.flatMap((remote) => plan.runs(remote).map((args) => [remote, command(program, args)]));
/// ```
pub(crate) fn upload_commands(
    git: &PushGit,
    plan: &UploadPlan,
) -> Option<Vec<(OsString, Command)>> {
    let program = find_program(
        environment_value(git.environment.as_slice(), "PATH")
            .unwrap_or_default()
            .as_os_str(),
        git.cwd.as_path(),
    )?;
    let mut commands: Vec<(OsString, Command)> = Vec::new();
    for remote in &plan.remotes {
        for arguments in plan.runs(remote.as_os_str()) {
            let mut command: Command = Command::new(&program);
            command.current_dir(&git.cwd).env_clear();
            for (name, value) in &git.environment {
                command.env(name, value);
            }
            command
                .env(
                    "PATH",
                    path_with_real_git_first(git.environment.as_slice(), git.real_git.as_path()),
                )
                .args(arguments.as_slice());
            // The destination travels beside its command, so a failure can name the server that asked without searching the
            //  argument list again.
            commands.push((remote.clone(), command));
        }
    }
    return Some(commands);
}

/// Why a run with captured standard error did not produce an outcome.
pub(crate) enum CaptureError {
    /// The file for the standard error could not be made;
    /// the text says why.
    Scratch(String),
    /// The program was found and could not be started;
    /// the text names the file and the operating system's error.
    Start(String),
    /// The wait for the program could not be made at all:
    /// no signal handler could be installed,
    /// or the process's runtime refused.
    /// The text says why.
    Guard(String),
}

/// What:
///  Why one bounded run of the upload produced no end.
/// Why:
///   A program the lookup found and the system refused to start is explained by the words of that refusal,
///  which name the file and the operating system's error,
/// while a wait that could not be made is a defect of this process and is named as one,
/// so the two never share words,
/// and neither claims a program that is there is absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunError = { kind: 'start'; why: string } | { kind: 'wait'; why: string };
/// ```
enum RunError {
    /// The program was found and could not be started;
    /// the text names the file and the operating system's error.
    Start(String),
    /// The wait failed,
    /// or no relay could be published;
    /// the text says why.
    Wait(String),
}

/// What:
///  How one run of the upload ended:
///  the program's own end,
/// or the end the wrapper gave it for showing no sign of work.
/// Why:
///   The two are explained in different words,
/// and a stopped run's status is the wrapper's `timeout(1)` convention rather than a status the program chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UploadEnd = { kind: 'finished'; outcome: ChildOutcome } | { kind: 'stopped'; cause: StopCause };
/// ```
pub(crate) enum UploadEnd {
    /// The program ended by itself.
    Finished(ChildOutcome),
    /// The wrapper ended it,
    /// and the cause says which limit came first.
    Stopped(StopCause),
}

/// What:
///  Run the command with its standard error in an anonymous file and return how it ended with that text,
/// the
///  credentials of every URL in it hidden.
/// Why:
///   Standard output stays the caller's,
/// so the progress meter is unchanged.
/// The standard error is where `git-lfs`
///  prints its errors,
/// which can carry the `lfs.url` token,
/// so it is relayed only after its URLs are masked.
/// A file needs no reader thread that a
///  stuck helper could keep alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { outcome, stderr } = await spawnCapturingStderr(command); return [outcome, maskUserinfo(stderr)];
/// ```
pub(crate) fn run_with_captured_stderr(
    command: &mut Command,
    environment: &[(OsString, OsString)],
) -> Result<(UploadEnd, Vec<u8>), CaptureError> {
    let mut captured: std::fs::File = match scratch_file(environment) {
        Ok(file) => file,
        Err(error) => return Err(CaptureError::Scratch(error.to_string())),
    };
    let for_child: std::fs::File = match captured.try_clone() {
        Ok(file) => file,
        Err(error) => return Err(CaptureError::Scratch(error.to_string())),
    };
    command.stderr(Stdio::from(for_child));
    // What: The program reads no update lines (its refs are arguments), so its input is the null device, as the bounded
    //  upload of the auto-push gives it.
    // Why:  An inherited input of a command nobody typed could be a pipe another program waits on.
    command.stdin(Stdio::null());
    let guard: SignalGuard = match SignalGuard::install() {
        Ok(installed) => installed,
        Err(why) => return Err(CaptureError::Guard(why.to_string())),
    };
    let ended: UploadEnd =
        match run_on_runtime(await_upload(command, &guard, &captured, environment)) {
            Ok(Ok(finished)) => finished,
            Ok(Err(RunError::Start(why))) => return Err(CaptureError::Start(why)),
            Ok(Err(RunError::Wait(why))) => return Err(CaptureError::Guard(why)),
            Err(refusal) => return Err(CaptureError::Guard(refusal.to_string())),
        };
    let relayed: Vec<u8> = match relayed_head(&mut captured) {
        Ok(text) => text,
        Err(error) => return Err(CaptureError::Scratch(error.to_string())),
    };
    return Ok((ended, relayed));
}

/// What:
///  The head of the captured standard error:
/// the whole lines the cap holds,
/// with the credentials of every URL in them
///  masked,
///  and the notice that more was printed than is relayed when there was.
/// Why:
///   This is the read the audit's credential leak went through:
///  the mask has to run before the cut and on one byte past the cap,
///  because a URL cut before its `@` shows no userinfo and a read of the cap alone cannot say that more was printed.
///  It takes the file rather than a process,
///  so a control drives these shipped steps from prepared bytes and cannot race a child's ending;
///  the upload's own run above and this read stay one path.
///  A run that ends between two looks of the wait prints its output past the cap without the flood stop seeing it,
///  so the notice belongs here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// file.seek(0); const bytes = file.take(MAX_RELAYED_BYTES + 1); const head = maskUrlSecretsWithin(bytes, MAX_RELAYED_BYTES); return head.cut ? head.text + notice() : head.text
/// ```
pub(crate) fn relayed_head(captured: &mut std::fs::File) -> std::io::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    // The file is read only after the seek to its start succeeded, as `.and_then(...)` ran the read.
    // One byte past the cap is read, so the read itself says whether more was printed than is relayed.
    captured.seek(SeekFrom::Start(0))?;
    captured
        .by_ref()
        .take(MAX_RELAYED_BYTES + 1)
        .read_to_end(&mut bytes)?;
    let head: MaskedHead = mask_url_secrets_within(bytes.as_slice(), MAX_RELAYED_BYTES);
    let mut relayed: Vec<u8> = head.text;
    if head.cut {
        relayed.extend_from_slice(shortened_relay_notice().as_bytes());
    }
    return Ok(relayed);
}

/// What:
///  The line a relayed standard error gains when the program printed more than the cap keeps.
/// Why:
///   A person reading the text must know it is the head of it,
///  and that what is missing was dropped rather than not printed;
///  the line is written after the masked head and ends in a line break,
///  so the explanation after it starts on a line of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: \`git-lfs push\` wrote more than ${MAX_RELAYED_BYTES} bytes ...\n`
/// ```
pub(crate) fn shortened_relay_notice() -> String {
    return format!(
        "monochromatic-git-policy-cli: `git-lfs push` wrote more than {MAX_RELAYED_BYTES} bytes of standard error, so only the whole \
         lines of it that fit in that many bytes are relayed here; the rest was dropped and the upload itself was not stopped.\n"
    );
}

/// What:
///  One `git-lfs push` run to its end,
/// or until it shows no sign of work for the push's stall window,
/// with the wrapper's signals relayed to it and its standard error already captured.
/// Why:
///   A server that accepts the connection and never answers held a typed push for good before this
/// (`package/git-policy/cli/SPEC.md`,
/// once "LFS is paused");
/// the auto-push's upload was bounded and this face was not.
/// The child is stopped and its whole tree ended when the window passes,
/// and a stop is reported as a stop,
/// never as the program's answer.
/// The relay stays published for the whole wait,
/// so a signal sent to the wrapper still ends the upload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relayed = await guard.startRelayed(command); const child = relayed.takeChild();
/// const ended = await waitUntilDoneOrStalled(child, captured, patience); if (ended.stopped) await stopPush(child, grace);
/// return ended;
/// ```
async fn await_upload(
    command: &mut Command,
    guard: &SignalGuard,
    captured: &std::fs::File,
    environment: &[(OsString, OsString)],
) -> Result<UploadEnd, RunError> {
    let mut relayed: RelayedChild = match guard.start_relayed(command).await {
        // A program the lookup found and the system refused to start is not a missing program: the operating system's own
        //  error says what to fix, and the file's name says which file it is.
        Err(error) => {
            let program: String = command.get_program().to_string_lossy().into_owned();
            return Err(RunError::Start(start_failed_message(
                program.as_str(),
                error.to_string().as_str(),
            )));
        }
        Ok(started) => started,
    };
    let started: std::process::Child = relayed
        .take_child()
        .expect("the child was handed over once");
    let mut child: OwnedChild = adopt(started);
    let label: String = command_label(command);
    let mut waits: Vec<crate::push_progress::WaitReport> = Vec::new();
    // The upload leaves no reference lock, so the ledger the stop sweeps is empty.
    let locks: LockLedger = LockLedger::none();
    let patience = push_patience(environment);
    let ended: Result<ExitStatus, StopCause> = match wait_until_done_noting(
        &mut child,
        &[captured],
        patience,
        label.as_str(),
        &mut waits,
        None,
        connected_window(environment),
        push_minimum_rate_bytes(environment),
    )
    .await
    {
        // A wait that could not be made at all is this process's own defect, and its words say which.
        Err(wait) => return Err(RunError::Wait(wait.to_string())),
        Ok(finished) => finished,
    };
    let finished: UploadEnd = match ended {
        // `outcome_of` maps the status the way the wrapper's exit and its notices read it, a death by a signal included.
        Ok(status) => UploadEnd::Finished(outcome_of(status)),
        Err(cause) => {
            // The upload is ended with the grace every stopped child gets, and what it started goes with it.
            stop_push(child, STOP_GRACE, locks).await;
            UploadEnd::Stopped(cause)
        }
    };
    // The token stops the relay pointing at the child here, once the wait no longer owns a living one.
    drop(relayed);
    return Ok(finished);
}
