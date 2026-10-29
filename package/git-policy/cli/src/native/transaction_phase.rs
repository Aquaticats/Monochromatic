//! What:
//!  Test-only transaction phase markers for crash and interleaving suites.
//!  An explicit environment variable arms one marker:
//!  `CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill` ends the process with `SIGKILL` at the phase,
//!  `<phase>:kill:<directory>` writes `<directory>/<phase>.reached` first,
//!  and `<phase>:pause:<directory>` writes that marker and waits until `<directory>/<phase>.release` exists.
//!  A repeatable phase numbers its files by occurrence: `<phase>-<n>.reached`.
//! Why:
//!  Crash suites need a deterministic point to kill the wrapper.
//!  The marker is compiled into the release executable, but only the explicit variable arms it,
//!  and a malformed value fails the invocation at every phase it reaches instead of being ignored
//!  (`src/policy-engine/commit-transaction-test-phase.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await reachTransactionPhase({ phase: 'ref-updated' });
//! ```

/// Diagnostic output to standard error when a marker fires.
use super::diagnostic_log::warn;
/// Strict JSON string quoting for the refusal message, as `JSON.stringify` writes it.
use serde_json::to_string as json_string;
/// `OsStr` is borrowed operating-system text, and `Path`/`PathBuf` are filesystem paths.
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Environment variable arming one phase marker.
pub const PHASE_SIGNAL_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PHASE_SIGNAL";

/// Poll interval while paused.
const RELEASE_POLL: Duration = Duration::from_millis(20);

/// What:
///  One transaction phase a marker can name.
/// Why:
///   The landing reaches four of them; the others are reached by the commit path and the replay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionTestPhase = 'capture-locked' | 'preparation-done' | 'landing-locked' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// The invocation capture is locked.
    CaptureLocked,
    /// Preparation wrote its prepared record.
    PreparationDone,
    /// The landing lock and the real index lock are held.
    LandingLocked,
    /// The shadow's objects are migrated into the kept pack.
    ObjectsMigrated,
    /// The target advanced by compare-and-swap.
    RefUpdated,
    /// The post-index is installed as the real index.
    IndexInstalled,
    /// A landing race was lost; repeatable, numbered by occurrence.
    RaceLost,
    /// An isolated merge wrote its intent record and has not yet asked the checkout to be updated.
    MergeIntent,
    /// The checkout was updated and the branch moved for an isolated merge, whose intent record is still on disk.
    MergeApplied,
}

/// Every phase in transaction order, which the refusal message lists.
pub const TRANSACTION_TEST_PHASES: [Phase; 9] = [
    Phase::CaptureLocked,
    Phase::PreparationDone,
    Phase::LandingLocked,
    Phase::ObjectsMigrated,
    Phase::RefUpdated,
    Phase::IndexInstalled,
    Phase::RaceLost,
    Phase::MergeIntent,
    Phase::MergeApplied,
];

/// Phase names in transaction order.
impl Phase {
    /// What:
    ///  The phase's name as the marker variable spells it.
    /// Why:
    ///   Marker files and the refusal message use these names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// phase // 'landing-locked'
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Phase::CaptureLocked => return "capture-locked",
            Phase::PreparationDone => return "preparation-done",
            Phase::LandingLocked => return "landing-locked",
            Phase::ObjectsMigrated => return "objects-migrated",
            Phase::RefUpdated => return "ref-updated",
            Phase::IndexInstalled => return "index-installed",
            Phase::RaceLost => return "race-lost",
            Phase::MergeIntent => return "merge-intent",
            Phase::MergeApplied => return "merge-applied",
        }
    }
}

/// What:
///  What a marker does at its phase.
/// Why:
///   `kill` ends the process; `pause` hands control to the test until the release file exists.
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
///   Parsing happens at every phase reached, so a malformed value fails wherever it is read.
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
    /// Marker directory; required for `pause`, absent or empty otherwise.
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
    /// The rejected value, as read.
    pub value: String,
}

/// Message text naming the variable, the quoted value and the accepted forms.
impl std::fmt::Display for PhaseSignalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quoted: String = json_string(self.value.as_str()).unwrap_or_else(|_| format!("{:?}", self.value));
        let phases: Vec<&str> = TRANSACTION_TEST_PHASES.iter().map(|phase| { return phase.name(); }).collect();
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
///  Parses the marker variable.
/// Why:
///   The phase must match exactly, the action must be `kill` or `pause`,
///  a pause needs a directory, and a directory must be absolute.
///  An empty directory after `kill` counts as no directory, as the incumbent's parser reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parsePhaseSignal(value);
/// ```
pub fn parse_phase_signal(value: &str) -> Result<PhaseSignal, PhaseSignalError> {
    let refusal = || {
        return PhaseSignalError {
            value: value.to_string(),
        };
    };
    let Some(phase_end) = value.find(':') else {
        return Err(refusal());
    };
    let rest: &str = &value[phase_end + 1..];
    let (action_text, directory_text) = match rest.find(':') {
        Some(action_end) => (&rest[..action_end], &rest[action_end + 1..]),
        None => (rest, ""),
    };
    let Some(phase) = TRANSACTION_TEST_PHASES
        .iter()
        .copied()
        .find(|candidate| { return candidate.name() == &value[..phase_end]; })
    else {
        return Err(refusal());
    };
    let action: PhaseAction = match action_text {
        "kill" => PhaseAction::Kill,
        "pause" => PhaseAction::Pause,
        _ => return Err(refusal()),
    };
    if (action == PhaseAction::Pause && directory_text.is_empty())
        || (!directory_text.is_empty() && !Path::new(directory_text).is_absolute())
    {
        return Err(refusal());
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
///  Ends the process with `SIGKILL`, the signal no handler can catch.
/// Why:
///   A crash suite needs the process to die with no cleanup, as a power loss would leave it.
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
///  Ends the process at once on platforms without `SIGKILL`, running no cleanup.
/// Why:
///   The marker must not run destructors, as an uncatchable kill would not.
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
///  Waits until the release file exists, polling every 20 milliseconds.
/// Why:
///   A paused phase lets a test inspect the repository before the transaction continues.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { try { await access(path); return; } catch { await wait(20); } }
/// ```
fn await_release(path: &Path) {
    while !path.exists() {
        std::thread::sleep(RELEASE_POLL);
    }
}

/// What:
///  Reaches one phase with the marker variable's value, killing or pausing when it is armed for this phase.
/// Why:
///   Tests pass the value directly; the production entry reads the process environment.
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
    let Some(armed) = value.filter(|text| { return !text.is_empty(); }) else {
        return Ok(());
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
            await_release(&directory.join(format!("{stem}.release")));
            return Ok(());
        }
    }
}

/// What:
///  Reaches one phase, reading the marker variable from the process environment.
/// Why:
///   The landing calls this at each marked phase; an unset variable costs one environment lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase: 'landing-locked' });
/// ```
pub fn reach_transaction_phase(phase: Phase, occurrence: Option<u64>) -> Result<(), PhaseMarkerError> {
    let value: Option<String> = std::env::var_os(PHASE_SIGNAL_VARIABLE).map(|text: std::ffi::OsString| { return text.to_string_lossy().into_owned(); });
    return reach_with_value(value.as_deref(), phase, occurrence);
}

/// Phase marker controls stay out of the production code path.
#[cfg(test)]
#[path = "transaction_phase_tests.rs"]
mod tests;
