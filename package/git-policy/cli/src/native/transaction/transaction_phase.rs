//! What:
//!  Test-only phase markers of the isolated merge and of a commit headed for the service,
//! for crash and interleaving suites.
//!  An explicit environment variable arms one marker:
//!  `CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill` ends the process with `SIGKILL` at the phase,
//!  `<phase>:kill:<directory>` writes `<directory>/<phase>.reached` first,
//!  and `<phase>:pause:<directory>` writes that marker and waits until `<directory>/<phase>.release` exists.
//!  A repeatable phase numbers its files by occurrence:
//! `<phase>-<n>.reached`.
//! Why:
//!  Crash suites need a deterministic point to kill the wrapper.
//!  The marker is compiled into the release executable,
//! but only the explicit variable arms it,
//!  and a malformed value fails the invocation at every phase it reaches instead of being ignored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await reachTransactionPhase({ phase: 'merge-applied' });
//! ```

/// Diagnostic output to standard error when a marker fires.
use crate::diagnostic_log::warn;
/// Strict JSON string quoting for the refusal message,
/// as `JSON.stringify` writes it.
use serde_json::to_string as json_string;
/// `OsStr` is borrowed operating-system text,
/// and `Path`/`PathBuf` are filesystem paths.
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Environment variable arming one phase marker.
pub const PHASE_SIGNAL_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PHASE_SIGNAL";

/// Poll interval while paused.
use crate::wait_ceilings::PHASE_PATIENCE;

const RELEASE_POLL: Duration = Duration::from_millis(20);

/// What:
///  One phase a marker can name.
/// Why:
///   The isolated merge reaches two of them and a commit headed for the service the other two;
/// the phases of the landing
///  that the commit service replaced (`capture-locked` to `index-installed`,
/// `race-lost`) are gone,
/// and a value naming one
///  is refused like any unknown phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TestPhase = 'merge-carried' | 'merge-cleared' | 'merge-intent' | 'merge-applied' | 'merge-carried-back' | 'service-staged' | 'service-prepared';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// An isolated merge wrote its intent record,
    /// and the bytes of the person's own work in its paths are objects on a ref of their own,
    /// and no byte of the checkout has changed.
    MergeCarried,
    /// The paths that held the person's own work are back to the branch's content,
    /// so Git's own update of the checkout can take them.
    MergeCleared,
    /// An isolated merge wrote its intent record and has not yet asked the checkout to be updated.
    MergeIntent,
    /// The checkout was updated and the branch moved for an isolated merge,
    /// whose intent record is still on disk.
    MergeApplied,
    /// The person's own work is merged with the merge and written back to the index and the worktree,
    /// and the record of the merge is still on disk.
    MergeCarriedBack,
    /// A commit headed for the service has staged its content in a private index and has not yet read it back for the
    ///  policies;
    /// repeatable,
    /// numbered by occurrence (a commit prepared again reaches it again).
    ServiceStaged,
    /// A commit headed for the service is fully prepared and has not yet been sent;
    /// repeatable,
    /// numbered by occurrence.
    ServicePrepared,
}

/// Every phase in the order a command reaches them,
/// which the refusal message lists.
pub const TRANSACTION_TEST_PHASES: [Phase; 7] = [
    Phase::MergeCarried,
    Phase::MergeCleared,
    Phase::MergeIntent,
    Phase::MergeApplied,
    Phase::MergeCarriedBack,
    Phase::ServiceStaged,
    Phase::ServicePrepared,
];

/// Phase names in the order a command reaches them.
impl Phase {
    /// What:
    ///  The phase's name as the marker variable spells it.
    /// Why:
    ///   Marker files and the refusal message use these names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// phase // 'merge-intent'
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Phase::MergeCarried => return "merge-carried",
            Phase::MergeCleared => return "merge-cleared",
            Phase::MergeIntent => return "merge-intent",
            Phase::MergeCarriedBack => return "merge-carried-back",
            Phase::MergeApplied => return "merge-applied",
            Phase::ServiceStaged => return "service-staged",
            Phase::ServicePrepared => return "service-prepared",
        }
    }
}

/// What:
///  What a marker does at its phase.
/// Why:
///   `kill` ends the process;
/// `pause` hands control to the test until the release file exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// action: 'kill' | 'pause'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhaseAction {
    /// Ends the process with `SIGKILL`.
    Kill,
    /// Waits for the release file.
    Pause,
}

/// What:
///  One parsed marker.
/// Why:
///   Parsing happens at every phase reached,
/// so a malformed value fails wherever it is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PhaseSignal = { phase: TransactionTestPhase; action: 'kill' | 'pause'; directory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseSignal {
    /// Armed phase.
    pub phase: Phase,
    /// What happens at the phase.
    pub action: PhaseAction,
    /// Marker directory;
    /// required for `pause`,
    /// absent or empty otherwise.
    pub directory: Option<PathBuf>,
}

/// What:
///  The marker variable is malformed.
/// Why:
///   The invocation fails with this text instead of ignoring the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TestPhaseSignalError(value)
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhaseSignalError {
    /// The rejected value,
    /// as read.
    pub value: String,
}

/// Message text naming the variable,
/// the quoted value and the accepted forms.
impl std::fmt::Display for PhaseSignalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A value that cannot be quoted as JSON text is shown in Rust's debug quoting.
        let quoted: String = match json_string(self.value.as_str()) {
            Ok(text) => text,
            Err(_unquotable) => format!("{:?}", self.value),
        };
        let mut phases: Vec<&str> = Vec::new();
        for phase in TRANSACTION_TEST_PHASES.iter() {
            phases.push(phase.name());
        }
        return write!(
            formatter,
            "{PHASE_SIGNAL_VARIABLE}={quoted} is not <phase>:kill[:<absolute directory>] or <phase>:pause:<absolute directory> with a phase among {}.",
            phases.join(", ")
        )
    }
}

/// Error type of the marker reach.
impl std::error::Error for PhaseSignalError {}

/// What:
///  Why a reached phase could not be completed.
/// Why:
///   A malformed value and a failed marker write are both fatal to the invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// TestPhaseSignalError | NodeJS.ErrnoException
/// ```
#[derive(Debug)]
pub enum PhaseMarkerError {
    /// The marker variable is malformed.
    Malformed(PhaseSignalError),
    /// Writing the reached marker or polling the release file failed.
    Io(std::io::Error),
}

/// Message text of either failure.
impl std::fmt::Display for PhaseMarkerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhaseMarkerError::Malformed(error) => return write!(formatter, "{error}"),
            PhaseMarkerError::Io(error) => return write!(formatter, "transaction phase marker failed: {error}"),
        }
    }
}

/// Error type of the marker reach.
impl std::error::Error for PhaseMarkerError {}

/// What:
///  An operating-system string as text,
/// with bytes that are not valid text replaced.
/// Why:
///  `reach_transaction_phase` reads the marker variable with it,
/// so an odd byte reaches the parser as a replacement character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lossyText = (text: Buffer): string => text.toString('utf8');
/// ```
fn lossy_text(text: std::ffi::OsString) -> String {
    return text.to_string_lossy().into_owned();
}

/// What:
///  The refusal for a marker value that does not parse,
/// carrying the value as read.
/// Why:
///  `parse_phase_signal` refuses at several points,
/// and each refusal is the same error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusalOf = (value: string) => new PhaseSignalError(value);
/// ```
fn refusal_of(value: &str) -> PhaseSignalError {
    return PhaseSignalError { value: value.to_string() };
}

/// What:
///  Parses the marker variable.
/// Why:
///   The phase must match exactly,
/// the action must be `kill` or `pause`,
///  a pause needs a directory,
/// and a directory must be absolute.
///  An empty directory after `kill` counts as no directory,
/// as the incumbent's parser reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parsePhaseSignal(value);
/// ```
pub fn parse_phase_signal(value: &str) -> Result<PhaseSignal, PhaseSignalError> {
    let Some(phase_end) = value.find(':') else {
        return Err(refusal_of(value));
    };
    let rest: &str = &value[phase_end + 1..];
    let (action_text, directory_text) = match rest.find(':') {
        Some(action_end) => (&rest[..action_end], &rest[action_end + 1..]),
        None => (rest, ""),
    };
    // The first phase whose name is the text before the first colon.
    let mut found: Option<Phase> = None;
    for candidate in TRANSACTION_TEST_PHASES.iter().copied() {
        if candidate.name() == &value[..phase_end] {
            found = Some(candidate);
            break;
        }
    }
    let Some(phase) = found else {
        return Err(refusal_of(value));
    };
    let action: PhaseAction = match action_text {
        "kill" => PhaseAction::Kill,
        "pause" => PhaseAction::Pause,
        _ => return Err(refusal_of(value)),
    };
    if (action == PhaseAction::Pause && directory_text.is_empty())
        || (!directory_text.is_empty() && !Path::new(directory_text).is_absolute())
    {
        return Err(refusal_of(value));
    }
    return Ok(PhaseSignal {
        phase,
        action,
        directory: if directory_text.is_empty() {
            None
        } else {
            Some(PathBuf::from(directory_text))
        },
    });
}

/// What:
///  Ends the process with `SIGKILL`,
/// the signal no handler can catch.
/// Why:
///   A crash suite needs the process to die with no cleanup,
/// as a power loss would leave it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(process.pid, 'SIGKILL');
/// ```
#[cfg(unix)]
fn kill_self() -> ! {
    // SAFETY: `getpid` and `kill` take no pointers, and signalling this process with `SIGKILL` cannot return.
    unsafe {
        libc::kill(libc::getpid(), libc::SIGKILL);
    }
    std::process::abort()
}

/// What:
///  Ends the process at once on platforms without `SIGKILL`,
/// running no cleanup.
/// Why:
///   The marker must not run destructors,
/// as an uncatchable kill would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(process.pid, 'SIGKILL');
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn kill_self() -> ! {
    std::process::exit(137)
}

/// What:
///  Waits until the release file exists,
/// polling every 20 milliseconds,
/// and gives up with an error when the directory the
///  test named for its markers is gone.
/// Why:
///   A paused phase lets a test inspect the repository before the transaction continues.
/// The directory is the test's fixture,
///  which a test removes as it ends,
/// however it ended;
/// once it is gone nobody can write the release file,
/// and a process that
///  went on polling would run for ever in a directory that no longer exists.
/// The wait watches the fixture first,
/// and a clock
///  second:
/// the patience is the longest a test may run,
/// so a slow host never cuts a pause short and a pause never outlives its test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) {
///   if (await exists(release)) return;
///   if (!(await exists(directory))) throw new Error('the directory of the pause is gone');
///   await wait(20);
/// }
/// ```
///
/// # Errors
/// An I/O error of kind `NotFound` when the directory of the pause no longer exists.
fn await_release(directory: &Path, release: &Path) -> Result<(), PhaseMarkerError> {
    return await_release_within(directory, release, PHASE_PATIENCE);
}

/// `await_release` that gives up with an error of kind `TimedOut` after `patience`:
/// the owner's rule is that nothing waits for ever
///  (2026-11-03),
/// and a control that armed a pause and then died leaves a directory that still exists.
///
/// # Errors
/// As `await_release`,
/// and `TimedOut` when the release file did not come in time.
fn await_release_within(directory: &Path, release: &Path, patience: Duration) -> Result<(), PhaseMarkerError> {
    let began: std::time::Instant = std::time::Instant::now();
    while !release.exists() {
        if began.elapsed() >= patience {
            return Err(PhaseMarkerError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("nothing released the pause in {} within {} seconds", directory.display(), patience.as_secs()),
            )));
        }
        if !directory.exists() {
            return Err(PhaseMarkerError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("the directory {} of the pause is gone, so nothing can release it", directory.display()),
            )));
        }
        std::thread::sleep(RELEASE_POLL);
    }
    return Ok(());
}

/// What:
///  Reaches one phase with the marker variable's value,
/// killing or pausing when it is armed for this phase.
/// Why:
///   Tests pass the value directly;
/// the production entry reads the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase, occurrence, environment });
/// ```
pub(crate) fn reach_with_value(
    value: Option<&str>,
    phase: Phase,
    occurrence: Option<u64>,
) -> Result<(), PhaseMarkerError> {
    // An empty value arms nothing.
    let armed: &str = match value {
        Some(text) if !text.is_empty() => text,
        _ => return Ok(()),
    };
    let signal: PhaseSignal = parse_phase_signal(armed).map_err(PhaseMarkerError::Malformed)?;
    if signal.phase != phase {
        return Ok(());
    }
    let stem: String = match occurrence {
        Some(number) => format!("{}-{number}", phase.name()),
        None => phase.name().to_string(),
    };
    warn(
        "transactionPhase",
        &format!("{PHASE_SIGNAL_VARIABLE} reached {stem}: {:?}", signal.action),
    );
    if let Some(directory) = signal.directory.as_deref() {
        std::fs::write(directory.join(format!("{stem}.reached")), std::process::id().to_string())
            .map_err(PhaseMarkerError::Io)?;
    }
    match signal.action {
        PhaseAction::Kill => kill_self(),
        PhaseAction::Pause => {
            let directory: &Path = signal.directory.as_deref().unwrap_or(Path::new(""));
            return await_release(directory, &directory.join(format!("{stem}.release")));
        }
    }
}

/// What:
///  Reaches one phase,
/// reading the marker variable from the process environment.
/// Why:
///   A command calls this at each marked phase;
/// an unset variable costs one environment lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase: 'merge-intent' });
/// ```
pub fn reach_transaction_phase(phase: Phase, occurrence: Option<u64>) -> Result<(), PhaseMarkerError> {
    let value: Option<String> = std::env::var_os(PHASE_SIGNAL_VARIABLE).map(lossy_text);
    return reach_with_value(value.as_deref(), phase, occurrence);
}

/// Phase marker controls stay out of the production code path.
#[cfg(test)]
#[path = "transaction_phase_tests.rs"]
mod tests;
