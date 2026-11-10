//! Running the folder chooser,
//! and saying what it did.
//!
//! What:
//! `choose_folder` opens the desktop's folder chooser and answers with one of four outcomes:
//! the folder the user picked,
//! a cancel,
//! no chooser on this computer,
//! or a chooser that failed and why.
//! Why:
//! the file dialog library answers `None` both for a cancel and for a chooser that could not show a window,
//! and its zenity backend keeps only the program's standard output,
//! so the line that explains a failure was thrown away
//! (`rfd-0.18.0/src/backend/linux/zenity.rs`,
//! `run`).
//! On Linux this module runs `zenity` itself,
//! so a failure keeps its exit status and its own explanation
//! (issue 1010).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = chooseFolder(); // {kind: 'chosen', folder} | {kind: 'cancelled'} | {kind: 'noChooser'} | {kind: 'failed', failure}
//! ```

/// Imports the path type that a chosen folder and the chooser program are.
use std::path::PathBuf;

/// Imports the path type that the chooser program is run from,
/// which only the Linux half does.
#[cfg(target_os = "linux")]
use std::path::Path;

/// Imports the remedy sentence that closes every message about a chooser that will not open.
use crate::dialog_backend::NO_DIALOG_REMEDY;

/// What:
/// `ChooserOutcome` is what one attempt to show the folder chooser did.
/// Why:
/// The four cases are the four things the window can do next,
/// and they are a value,
/// so a test can name each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChooserOutcome = {kind: 'chosen', folder: string} | {kind: 'cancelled'} | {kind: 'noChooser'} | {kind: 'failed', failure: ChooserFailure};
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChooserOutcome {
    /// The user picked a folder.
    Chosen {
        /// The folder the user picked.
        folder: PathBuf,
    },
    /// The user closed the chooser without picking one.
    Cancelled,
    /// This computer has no chooser to open.
    NoChooser,
    /// A chooser exists but could not show a folder.
    Failed {
        /// Why it could not,
        /// and what the program itself said.
        failure: ChooserFailure,
    },
}

/// What:
/// `ChooserFailure` is why a chooser that exists did not show a folder.
/// Why:
/// The two halves are kept apart:
/// the cause says which program ended how,
/// and the standard error is what that program itself said,
/// which is the line that names a cause the player cannot guess
/// (a broken display connection,
/// for instance).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ChooserFailure { cause: string; standardError: string }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChooserFailure {
    /// What happened to the program,
    /// in plain words,
    /// like `zenity exited with status 1`.
    pub cause: String,
    /// The first non-blank line the program wrote to standard error,
    /// empty when it wrote none.
    pub standard_error: String,
}

/// What:
/// `ChooserAdvice` is what the window does about one outcome.
/// Why:
/// The mapping from an outcome to a window action is the whole of the caller's decision,
/// so a test can check it for every outcome without a window and without a chooser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChooserAdvice = {kind: 'open', folder: string} | {kind: 'nothing'} | {kind: 'tellNoChooser'} | {kind: 'tellFailure', message: string};
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChooserAdvice {
    /// Open a folder and leave playback paused.
    Open {
        /// The folder to open.
        folder: PathBuf,
    },
    /// Do nothing:
    /// the user cancelled,
    /// which needs no message.
    Nothing,
    /// Tell the user that no chooser is installed,
    /// and how to go on.
    TellNoChooser,
    /// Tell the user what the chooser program said,
    /// and how to go on.
    TellFailure {
        /// The sentence shown in the window.
        message: String,
    },
}

/// What:
/// `failure_message` is the text shown in the window when the chooser answered with no folder.
/// Why:
/// The user who pressed `Open folder` needs the program's own explanation and the two ways forward,
/// never the folder
/// that was on screen,
/// and the sentence must not claim more than the program said:
/// `zenity` exits 1 both when the
/// user cancels and when it cannot open a window,
/// and it writes GTK warnings to standard error on runs that work,
/// so the
/// window states what happened and lets the program speak for itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failureMessage = (failure) => [`The folder chooser closed without a folder.`, `${failure.cause}.`, failure.standardError && `It said: ${failure.standardError}.`, NO_DIALOG_REMEDY].filter(Boolean).join(' ');
/// ```
pub fn failure_message(failure: &ChooserFailure) -> String {
    let mut sentences: Vec<String> = vec![
        "The folder chooser closed without a folder.".to_string(),
        format!("{}.", failure.cause),
    ];
    if !failure.standard_error.is_empty() {
        sentences.push(format!("It said: {}.", failure.standard_error));
    }
    sentences.push(NO_DIALOG_REMEDY.to_string());
    return sentences.join(" ");
}

/// What:
/// `advice` is what the window does about `outcome`.
/// Why:
/// The window's handler then has no decision left to make,
/// and a test can prove every outcome reaches its own action,
/// which is the part of this module a headless test cannot otherwise reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function advice(outcome: ChooserOutcome): ChooserAdvice { switch (outcome.kind) { case 'chosen': return {kind: 'open', folder: outcome.folder}; case 'cancelled': return {kind: 'nothing'}; case 'noChooser': return {kind: 'tellNoChooser'}; case 'failed': return {kind: 'tellFailure', message: failureMessage(outcome.failure)}; } }
/// ```
pub fn advice(outcome: ChooserOutcome) -> ChooserAdvice {
    return match outcome {
        ChooserOutcome::Chosen { folder } => ChooserAdvice::Open { folder },
        ChooserOutcome::Cancelled => ChooserAdvice::Nothing,
        ChooserOutcome::NoChooser => ChooserAdvice::TellNoChooser,
        ChooserOutcome::Failed { failure } => ChooserAdvice::TellFailure {
            message: failure_message(&failure),
        },
    };
}

/// What:
/// `choose_folder` opens the folder chooser and says what it did (Linux).
/// Why:
/// The probes,
/// the choice of way and the run happen here,
/// so the window's handler asks one question and handles one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const chooseFolder = () => { const source = chooserSource(resolveOnPath('zenity', PATH), portalAvailable()); return source.kind === 'missing' ? {kind: 'noChooser'} : source.kind === 'portal' ? chooseThroughTheLibrary() : runChooserProgram(source.program); };
/// ```
#[cfg(target_os = "linux")]
pub fn choose_folder() -> ChooserOutcome {
    let path_variable: std::ffi::OsString = std::env::var_os("PATH").unwrap_or_default();
    let zenity: Option<PathBuf> = crate::dialog_backend::resolve_on_path(crate::dialog_backend::ZENITY, &path_variable);
    let portal_available: bool = crate::dialog_backend::portal_is_available();
    // What: the two probes as one record, so the log says what the choice was made from.
    // Why: a report of a silent chooser otherwise cannot tell "no portal" from "the portal failed".
    tracing::info!(zenity = zenity.is_some(), portal = portal_available, "folder chooser backends");
    return match chooser_source(zenity, portal_available) {
        ChooserSource::Missing => ChooserOutcome::NoChooser,
        ChooserSource::Portal => choose_through_the_library(),
        ChooserSource::Program { program } => run_chooser_program(&program),
    };
}

/// What:
/// `choose_folder` opens the folder chooser and says what it did (macOS and Windows).
/// Why:
/// Both systems have a native chooser,
/// so the dialog library is asked directly and no program is searched for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const chooseFolder = () => chooseThroughTheLibrary();
/// ```
#[cfg(not(target_os = "linux"))]
pub fn choose_folder() -> ChooserOutcome {
    return choose_through_the_library();
}

/// What:
/// `choose_through_the_library` asks the dialog library,
/// which drives the desktop's own chooser.
/// Why:
/// The portal's answer is only "a folder" or "nothing";
/// when the library falls back to a program its own records say so,
/// and those records reach this application's subscriber through the `log` bridge that
/// `src/logging.rs` installs.
fn choose_through_the_library() -> ChooserOutcome {
    return match rfd::FileDialog::new().pick_folder() {
        Some(folder) => ChooserOutcome::Chosen { folder },
        None => {
            tracing::info!("the desktop chooser returned no folder, which the library reports as a cancel");
            ChooserOutcome::Cancelled
        }
    };
}

/// What:
/// `ChooserSource` names the way a chooser would open on this computer.
/// Why:
/// Which of the three ways is taken depends only on two probes,
/// so the decision is a value that a test can give every combination of without opening anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChooserSource = {kind: 'portal'} | {kind: 'program', program: string} | {kind: 'missing'};
/// ```
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChooserSource {
    /// The desktop portal,
    /// which the dialog library drives.
    Portal,
    /// This program,
    /// run here so that its standard error is kept.
    Program {
        /// The chooser program,
        /// as the search path named it.
        program: PathBuf,
    },
    /// Neither the portal nor a program exists.
    Missing,
}

/// What:
/// `chooser_source` is the way a chooser would open,
/// from the two probes' answers.
/// Why:
/// The portal is the desktop's own chooser and needs no program installed,
/// so it is asked first;
/// only where no portal answers is a program run,
/// which keeps today's behaviour and adds the diagnostics a program run can keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooserSource(zenity: string | undefined, portalAvailable: boolean): ChooserSource { if (noChooserCanOpen(zenity !== undefined, portalAvailable)) return {kind: 'missing'}; return portalAvailable ? {kind: 'portal'} : {kind: 'program', program: zenity}; }
/// ```
#[cfg(target_os = "linux")]
pub fn chooser_source(zenity: Option<PathBuf>, portal_available: bool) -> ChooserSource {
    // The decision that no chooser exists at all stays in one place, so its table of combinations is the only table.
    if crate::dialog_backend::no_chooser_can_open(zenity.is_some(), portal_available) {
        return ChooserSource::Missing;
    }
    // The portal is the desktop's own chooser, and it works without any program installed.
    if portal_available {
        return ChooserSource::Portal;
    }
    return match zenity {
        Some(program) => ChooserSource::Program { program },
        None => ChooserSource::Missing,
    };
}

/// What:
/// `CHOOSER_ARGUMENTS` is what the chooser program is asked for:
/// no markup in any label it could take from us,
/// one folder rather than a file or several.
/// Why:
/// These are the arguments the dialog library's own zenity call passes for `pick_folder`
/// (`rfd-0.18.0/src/backend/linux/zenity.rs`),
/// so running the program here asks the same question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHOOSER_ARGUMENTS = ['--no-markup', '--file-selection', '--directory'];
/// ```
#[cfg(target_os = "linux")]
const CHOOSER_ARGUMENTS: [&str; 3] = ["--no-markup", "--file-selection", "--directory"];

/// What:
/// `run_chooser_program` runs `program` as the folder chooser and classifies what it printed and exited with.
/// Why:
/// Running the program here is what keeps its standard error,
/// which the library's own call discards;
/// the log then names the outcome and the chosen folder is never logged,
/// because it is the user's private path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runChooserProgram(program: string): ChooserOutcome { const output = spawnSync(program, CHOOSER_ARGUMENTS); const outcome = classifyRun(program, output); log(outcome); return outcome; }
/// ```
#[cfg(target_os = "linux")]
pub fn run_chooser_program(program: &Path) -> ChooserOutcome {
    let output: std::io::Result<std::process::Output> = std::process::Command::new(program)
        .args(CHOOSER_ARGUMENTS)
        .output();
    let outcome: ChooserOutcome = classify_run(program, &output);
    let name: String = program_name(program);
    match &outcome {
        ChooserOutcome::Chosen { .. } => tracing::info!(program = %name, "the folder chooser chose a folder"),
        ChooserOutcome::Cancelled => tracing::info!(program = %name, "the folder chooser was cancelled"),
        ChooserOutcome::NoChooser => tracing::warn!(program = %name, "no folder chooser is available"),
        ChooserOutcome::Failed { failure } => tracing::error!(
            program = %name,
            cause = %failure.cause,
            standard_error = %failure.standard_error,
            "the folder chooser failed",
        ),
    };
    return outcome;
}

/// What:
/// `classify_run` reads one run of a chooser program as an outcome.
/// Why:
/// This split is the part worth proving.
/// `zenity` uses exit 1 for a cancel AND for a window it could not open,
/// and it writes GTK warnings to standard
/// error on runs that do work,
/// so the exit status alone decides nothing and the standard error only says what the program
/// thought:
/// a run that printed a folder answered,
/// a run that never exited cannot have been cancelled (a cancel is exit 1),
/// a
/// clean exit with no folder chose nothing,
/// and anything else is a failure whose explanation is kept.
/// Measured on zenity 4.2.2:
/// with no display it exited 1 and wrote `Failed to open display`;
/// inside a headless
/// compositor it wrote `Failed to measure available space` and never exited;
/// with `--timeout=2` it exited 5 with
/// nothing on either stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyRun(program: string, output: Result<Output>): ChooserOutcome { if (output.isErr()) return failed(`${program} could not be started: ${output.error}`); const folder = trimNewline(output.stdout); if (folder) return chosen(folder); const reported = firstLine(output.stderr); if (output.status.code === undefined) return failed(exitCause(program, output.status), reported); return output.status.code === 0 || !reported ? cancelled() : failed(exitCause(program, output.status), reported); }
/// ```
#[cfg(target_os = "linux")]
fn classify_run(program: &Path, output: &std::io::Result<std::process::Output>) -> ChooserOutcome {
    let finished: &std::process::Output = match output {
        Ok(finished) => finished,
        // The program was found by the search but could not start: no interpreter, no permission bit, and so on.
        Err(error) => {
            return ChooserOutcome::Failed {
                failure: ChooserFailure {
                    cause: format!("{} could not be started: {error}", program_name(program)),
                    standard_error: String::new(),
                },
            };
        }
    };
    if let Some(folder) = chosen_folder(&finished.stdout) {
        return ChooserOutcome::Chosen { folder };
    }
    let reported: String = first_reported_line(&finished.stderr);
    // What: a run that never exited has no status code, and a cancel is exit 1, so this is a failure.
    // Why: a chooser killed with nothing on standard error would otherwise read as a cancel.
    if finished.status.code().is_none() {
        return ChooserOutcome::Failed {
            failure: ChooserFailure {
                cause: exit_cause(program, finished.status),
                standard_error: reported,
            },
        };
    }
    if finished.status.success() || reported.is_empty() {
        return ChooserOutcome::Cancelled;
    }
    return ChooserOutcome::Failed {
        failure: ChooserFailure {
            cause: exit_cause(program, finished.status),
            standard_error: reported,
        },
    };
}

/// What:
/// `chosen_folder` is the folder the program printed,
/// without the newline it writes after it.
/// Why:
/// A folder name on Linux is bytes and not necessarily valid text,
/// so the answer is read as an operating-system string and no name is lost to a decoding failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const chosenFolder = (stdout: Buffer) => { const trimmed = stdout.toString().replace(/[\r\n]+$/, ''); return trimmed.length > 0 ? trimmed : undefined; };
/// ```
#[cfg(target_os = "linux")]
fn chosen_folder(standard_out: &[u8]) -> Option<PathBuf> {
    let mut bytes: &[u8] = standard_out;
    while let [rest @ .., last] = bytes {
        if *last != b'\n' && *last != b'\r' {
            break;
        }
        bytes = rest;
    }
    if bytes.is_empty() {
        return None;
    }
    return Some(PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec())));
}

/// What:
/// `first_reported_line` is the first line of `standard_err` that is not blank.
/// Why:
/// A program's explanation is its first line
/// (`Failed to open display: `),
/// and the later lines are usually a stack of library detail the window cannot show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstReportedLine = (stderr: Buffer) => stderr.toString().split('\n').map((line) => line.trim()).find((line) => line.length > 0) ?? '';
/// ```
#[cfg(target_os = "linux")]
fn first_reported_line(standard_err: &[u8]) -> String {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(standard_err);
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    return String::new();
}

/// What:
/// `exit_cause` says in plain words how the program ended.
/// Why:
/// The window states the status,
/// so a user can tell a cancel's status from a crash's,
/// and a program ended by a signal has no status at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exitCause = (program: string, status: number | null) => status === null ? `${program} was ended before it exited` : `${program} exited with status ${status}`;
/// ```
#[cfg(target_os = "linux")]
fn exit_cause(program: &Path, status: std::process::ExitStatus) -> String {
    return match status.code() {
        Some(code) => format!("{} exited with status {code}", program_name(program)),
        None => format!("{} was ended before it exited", program_name(program)),
    };
}

/// What:
/// `program_name` is the name of the chooser program without its folders.
/// Why:
/// The message names the program a reader would recognize and not the folder it was found in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const programName = (program: string) => program.split('/').at(-1) ?? program;
/// ```
#[cfg(target_os = "linux")]
fn program_name(program: &Path) -> String {
    return match program.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => program.to_string_lossy().into_owned(),
    };
}

/// What:
/// `OsString::from_vec` reads a folder name that is not valid text.
/// Why:
/// A Linux path is bytes;
/// the extension trait is how a chosen folder keeps bytes CString would reject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fromVec } from 'os-strings';
/// ```
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStringExt;

/// Unit tests of `folder_chooser`,
/// in the sidecar file `folder_chooser_tests.rs`.
/// Role of the module:
/// run the folder chooser and say what it did.
#[cfg(test)]
#[path = "folder_chooser_tests.rs"]
mod tests;
