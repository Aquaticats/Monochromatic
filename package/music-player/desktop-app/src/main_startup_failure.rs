//! What the entry point does when a startup step fails.
//!
//! What:
//! `forced_failure` is the debug-build hook that fails a named step on purpose,
//! `startup_failed` reports the failed step and hands the error on,
//! and `finish_step` is the guard each fallible startup step goes through.
//! Why:
//! a failure before the window exists has to reach a person who may have no
//! terminal,
//! so the report is one place,
//! split from `src/main.rs` to stay within the line budget.

/// What:
///      `use anyhow::Result` brings the success-or-error return type the guard returns.
/// Why:
///      `finish_step` hands either the step's value or the startup error to `main`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS throws instead of returning a Result type.
/// ```
use anyhow::Result;
/// What:
///      `use music_player::startup_failure::StartupStep` brings the name of each startup
///           step.
/// Why:
///      A failure is reported by the step that failed,
///  and that name is the library's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { StartupStep } from "music_player/startup_failure";
/// ```
use music_player::startup_failure::StartupStep;

/// What:
///      `pub(super) fn forced_failure(step: StartupStep) -> Result<(), slint::PlatformError>`.
///  In a Windows debug build,
/// fails `step` on purpose when `MUSIC_PLAYER_FORCE_STARTUP_FAILURE` names it.
/// Why:
///       The Windows runner proves the failure report by forcing each step and reading the message box back
///           (`.github/workflows/music-player-release.yml`,
/// job `test-windows`).
/// The error goes through the same
///           path as a real one.
#[cfg(all(windows, debug_assertions))]
pub(super) fn forced_failure(step: StartupStep) -> Result<(), slint::PlatformError> {
    /// Imports the startup failure injection variable and the helpers that read it.
    use music_player::startup_failure::{
        FORCE_STARTUP_FAILURE_ENV,
        forced_failure_detail,
        forced_startup_failure,
    };
    let value = std::env::var(FORCE_STARTUP_FAILURE_ENV).ok();
    if forced_startup_failure(value.as_deref()) == Some(step) {
        tracing::warn!(
            ?step,
            "failing a startup step on purpose ({FORCE_STARTUP_FAILURE_ENV})"
        );
        return Err(slint::PlatformError::from(forced_failure_detail(step)));
    }
    return Ok(());
}

/// What:
///      `pub(super) fn forced_failure(_step: StartupStep) -> Result<(), slint::PlatformError>`.
///  Release builds and non-Windows builds:
/// never fails.
/// Why:
///       The test hook is compiled out of the release;
/// the call sites stay the same on every build.
#[cfg(not(all(windows, debug_assertions)))]
pub(super) fn forced_failure(_step: StartupStep) -> Result<(), slint::PlatformError> {
    return Ok(());
}

/// What:
///      `fn startup_failed(step: StartupStep, error: slint::PlatformError) -> anyhow::Error`.
///  Windows:
/// reports the failed step (a message box when stderr reaches nobody,
/// the text on stderr otherwise),
/// then
///  hands the error on to `main`.
/// Why:
///       A double-click start has no console,
/// so the error `main` returns would be written nowhere.
#[cfg(windows)]
fn startup_failed(
    step: StartupStep,
    error: slint::PlatformError,
) -> anyhow::Error {
    music_player::startup_failure_win32::report_startup_failure(step, &error.to_string());
    tracing::debug!(
        ?step,
        "the startup failure was reported; main returns the error and drops its state"
    );
    return anyhow::Error::from(error);
}

/// What:
///      `fn startup_failed(_step: StartupStep, error: slint::PlatformError) -> anyhow::Error`.
///  Linux and macOS:
/// hands the error on to `main` unchanged,
/// which prints it as before.
/// Why:
///       Those builds always have a terminal or a session log for stderr.
#[cfg(not(windows))]
fn startup_failed(
    _step: StartupStep,
    error: slint::PlatformError,
) -> anyhow::Error {
    return anyhow::Error::from(error);
}

/// What:
/// `finish_step` gives the value of a startup step,
/// or the startup failure for that step.
/// Why:
/// each step that can fail names itself in the failure the user sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishStep<T>(step: StartupStep, outcome: T | PlatformError): T { if (outcome instanceof PlatformError) throw startupFailed(step, outcome); return outcome; }
/// ```
pub(super) fn finish_step<T>(
    step: StartupStep,
    outcome: Result<T, slint::PlatformError>,
) -> Result<T> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(startup_failed(step, error)),
    };
}
