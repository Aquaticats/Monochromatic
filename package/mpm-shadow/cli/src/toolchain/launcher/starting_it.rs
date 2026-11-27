//! What:
//! Starting what a launcher decided:
//! the check that the program is a file before anything is started,
//! the warning on standard error,
//! the refusal of a record that leads back to this launcher,
//! the rewrite of a record that names a program that is gone,
//! and the replacement of this process with the real binary.
//! Why:
//! Standard output is never written here,
//! since the wrapped tool's own output must reach the caller unchanged,
//! and replacing the process makes exit codes,
//! signals and standard streams the real program's own.
//! A launcher never starts itself:
//! a record that names its own launcher,
//! or two records that name each other,
//! once made each launcher replace itself with the other without end,
//! so a chain that leads back is refused before anything is started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function execProgram(program: string, arguments: string[]): Error
//! function refreshStaleRecord(record: LaunchRecord, program: string): void  // throws
//! function sayWhyTheLockIsUnreadable(lockFile: string): void
//! function missingLines(record: LaunchRecord, program: string, error: Error): [string, string]
//! export function runLauncher(record: LaunchRecord, args: string[]): number
//! function reportProgramNotFound(record: LaunchRecord, program: string, lockText?: string): number
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `Path` and `PathBuf` from `std::path`.
use std::path::{
    Path,
    PathBuf,
};
/// Imports `Command` from `std::process`.
use std::process::Command;

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the check that follows a launcher chain and names a loop or a broken record.
use super::super::launcher_chain::chain_problem;
/// Imports what this module uses from `super::super::launcher_current_link`.
use super::super::launcher_current_link::{
    MissingProgram,
    missing_program_advice,
    repair_command,
};
/// Imports what this module uses from `super::super::repository_link_cause`.
use super::super::repository_link_cause::{
    CHECK_DEADLINE,
    LinkVerdict,
    repository_link_verdict,
};
/// Imports the one atomic write the launcher and the lock share.
use super::super::util::write_atomically;
/// Imports what this module uses from the parent.
use super::{
    Launch,
    LaunchRecord,
    prepare_launch,
    record_path,
    render_record,
};
/// Imports the exit codes the shadow already uses.
use crate::constants::{
    EXEC_FAILED_EXIT_CODE,
    NOT_FOUND_EXIT_CODE,
};
/// Imports the reader that says which interpreter a program names that is not there,
/// and its answers.
use crate::named_interpreter::{
    MissingInterpreter,
    missing_interpreter,
};
/// Imports the shared words for a start refused because the program's file is being written.
use crate::start_failure::being_written_cause_and_remedy;

/// Replaces this process with `program`,
/// keeping `arguments` (argv[0] included) as they were;
/// returns only when that failed.
#[cfg(unix)]
fn exec_program(
    program: &Path,
    arguments: &[OsString],
) -> std::io::Error {
    /// Imports `CommandExt` from `std::os::unix::process`.
    use std::os::unix::process::CommandExt;
    let mut command: Command = Command::new(program);
    if let Some(argv0) = arguments.first() {
        command.arg0(argv0);
    }
    command.args(arguments.iter().skip(1));
    return command.exec();
}

/// Runs `program` and waits (a platform without `exec`),
/// exiting the way it did.
#[cfg(not(unix))]
fn exec_program(
    program: &Path,
    arguments: &[OsString],
) -> std::io::Error {
    return match Command::new(program)
        .args(arguments.iter().skip(1))
        .status()
    {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => error,
    };
}

/// What:
/// Rewrites this launcher's record so that it names `program`,
/// when the record names another path and `program` exists.
/// Why:
/// The shadows a launcher starts (`cargo-shadow`,
/// `pnpm-shadow`,
/// `search-shadow`,
/// `mpm-shadow` in its shim mode) skip a `PATH` candidate
/// that is a launcher whose record leads back to themselves,
/// and they read the path the record names.
/// A record that names a deleted release folder leads nowhere they can resolve,
/// so the shadow took the launcher for the real tool,
/// started it,
/// and its recursion guard stopped the run.
/// Naming the program that ran keeps the record true after every version move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshStaleRecord(record: LaunchRecord, program: string): void  // throws
/// ```
fn refresh_stale_record(
    record: &LaunchRecord,
    program: &Path,
) -> Result<(), ToolchainError> {
    if program == record.real || !program.exists() {
        return Ok(());
    }
    let executable: PathBuf = match std::env::current_exe() {
        Ok(found) => found,
        Err(error) => {
            return Err(ToolchainError::failure(format!(
                "cannot find this launcher's own path: {error}"
            )));
        }
    };
    let refreshed: LaunchRecord = LaunchRecord {
        real: program.to_path_buf(),
        ..record.clone()
    };
    return write_atomically(
        &record_path(&executable),
        render_record(&refreshed).as_bytes(),
    );
}

/// What:
/// Says on standard error why an unreadable lock file is unreadable when the cause is the repository's link to the 860 drive.
/// Why:
/// The launcher never blocks a run for an unreadable lock,
/// so without this the version warning is dropped in silence.
/// Nothing is asked unless the identity file exists and the lock file is under the repository it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayWhyTheLockIsUnreadable(lockFile)
/// ```
fn say_why_the_lock_is_unreadable(lock_file: &Path) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    match repository_link_verdict(Path::new(&home), lock_file, CHECK_DEADLINE) {
        LinkVerdict::NotReady {
            message,
        } => {
            eprintln!(
                "mpm-shadow: warning: {} cannot be read, so the version check was skipped and the tool runs anyway: {message}",
                lock_file.display()
            );
        }
        LinkVerdict::CheckUnavailable {
            why,
        } => {
            eprintln!(
                "mpm-shadow: warning: {} cannot be read, so the version check was skipped and the tool runs anyway; the cause is unknown: {why}",
                lock_file.display()
            );
        }
        LinkVerdict::NotConfigured | LinkVerdict::Ready => {}
    }
}

/// What:
/// The two lines a launcher prints for a start the system answered `ENOENT` for:
/// which file is missing,
/// read from the program's own bytes where they say so,
/// and what to do about it.
/// Why:
/// `execve` answers one error for a program that is gone and for the interpreter the program names,
/// and the program has passed this launcher's own check a moment before,
/// so the cause must say which of the two files it can name and claim neither when the program's
/// bytes are gone too;
/// the remedy names this tree's one command first,
/// and the file that the program's own file says is missing after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingLines(record: LaunchRecord, program: string, error: Error): [string, string]
/// ```
fn missing_lines(
    record: &LaunchRecord,
    program: &Path,
    error: &std::io::Error,
) -> (String, String) {
    let repair: String = repair_command(record.scope, &record.lock_file);
    return match missing_interpreter(program) {
        MissingInterpreter::Script {
            interpreter,
        } => (
            format!(
                "the interpreter its first line names (`{}`) is not there ({error}); the program itself was there when this launcher checked the path",
                interpreter.display()
            ),
            format!(
                "{repair}, and install the interpreter its first line names (`{}`) if that is the file that is missing",
                interpreter.display()
            ),
        ),
        MissingInterpreter::DynamicLinker {
            linker,
        } => (
            format!(
                "its dynamic linker (`{}`) is not there ({error}), and a compiled program cannot start without it; the program itself was there when this launcher checked the path",
                linker.display()
            ),
            format!(
                "{repair}, and install or upgrade the package that provides `{}` if that linker is the file that is missing",
                linker.display()
            ),
        ),
        MissingInterpreter::SaysNothing => (
            format!(
                "the program is gone, or the interpreter it names is ({error}); the interpreter a program names is the program after `#!` on a script's first line, or a compiled program's dynamic linker, and this launcher cannot say which of the two files is missing; the program itself was there when this launcher checked the path"
            ),
            format!("{repair}, and if that does not put the program back, install it again"),
        ),
    };
}

/// What:
/// Runs a launcher:
/// warns when the lock pins an older version than the one that runs,
/// then becomes the real binary.
/// Why:
/// Standard output is never written here,
/// since the wrapped tool's own output (the git wrapper prints JSON lines) must reach the caller unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runLauncher(record: LaunchRecord, args: string[]): number
/// ```
pub fn run_launcher(
    record: &LaunchRecord,
    arguments: &[OsString],
) -> i32 {
    let lock_text: Option<String> = fs::read_to_string(&record.lock_file).ok();
    if lock_text.is_none() {
        say_why_the_lock_is_unreadable(&record.lock_file);
    }
    let launch: Launch = prepare_launch(record, lock_text.as_deref());
    // A launcher starts only a path it has just found to be a file: neither the recorded path nor the one under `current` is
    // tried blind, and no warning says "starting" for a program that does not start.
    if !launch.program.is_file() {
        return report_program_not_found(record, &launch.program, lock_text.as_deref());
    }
    if let Some(warning) = &launch.warning {
        eprintln!("mpm-shadow: warning: {warning}");
    }
    // A launcher never starts itself: a chain of records that leads back to this launcher, or in a circle, is refused before anything is started.
    if let Ok(own) = std::env::current_exe()
        && let Some(problem) = chain_problem(&own, &launch.program)
    {
        eprintln!("mpm-shadow: {problem}");
        return EXEC_FAILED_EXIT_CODE;
    }
    if let Err(error) = refresh_stale_record(record, &launch.program) {
        eprintln!(
            "mpm-shadow: warning: could not update the launcher record, so it keeps naming a program that no longer exists: {error}"
        );
    }
    let error: std::io::Error = exec_program(&launch.program, arguments);
    // What:     A refusal because another process holds a file this start needs open for writing is explained in the
    //           same words the shadow mode uses: what the refusal proves, and both remedies.
    // Why:      `execve` cannot start a file that is being written either, and the bare operating-system text read
    //           as a permanent fault when the same command works a moment later.
    if error.kind() == std::io::ErrorKind::ExecutableFileBusy {
        let (cause, remedy) = being_written_cause_and_remedy(&launch.program, &error);
        eprintln!(
            "mpm-shadow: could not start {}: {cause}",
            launch.program.display()
        );
        eprintln!("mpm-shadow: what to do: {remedy}");
        return EXEC_FAILED_EXIT_CODE;
    }
    if error.kind() == std::io::ErrorKind::NotFound {
        // What:     `execve` answers `ENOENT` both for a program that is gone and for the interpreter its
        //           first line or its own file names, so the cause names the file the program's bytes say is
        //           missing, and both kinds when they say nothing.
        // Why:      The program passed the check a moment ago, so "no file exists there" would be false here,
        //           and a sentence about a first line would be false for a compiled program, which has none.
        let (cause, remedy) = missing_lines(record, &launch.program, &error);
        eprintln!(
            "mpm-shadow: could not start {}: {cause}",
            launch.program.display()
        );
        eprintln!("mpm-shadow: what to do: {remedy}");
        return NOT_FOUND_EXIT_CODE;
    }
    eprintln!(
        "mpm-shadow: could not start {}: {error}",
        launch.program.display()
    );
    return EXEC_FAILED_EXIT_CODE;
}

/// The sentence that says no program was found for `record`,
/// with the tool,
/// both versions and the command that makes the links again.
fn advice_for(
    record: &LaunchRecord,
    lock_text: Option<&str>,
) -> String {
    return missing_program_advice(MissingProgram {
        tool_id: &record.tool_id,
        scope: record.scope,
        lock_file: &record.lock_file,
        real: &record.real,
        lock_text,
    });
}

/// What:
/// Says on standard error that `program` was not started because no file exists there,
/// followed by the advice,
/// and returns the exit status for a missing program.
/// Why:
/// The check comes before any start,
/// so the message names the path that was checked,
/// and the start attempt's operating-system error is not the only evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportProgramNotFound(record: LaunchRecord, program: string, lockText?: string): number
/// ```
fn report_program_not_found(
    record: &LaunchRecord,
    program: &Path,
    lock_text: Option<&str>,
) -> i32 {
    eprintln!(
        "mpm-shadow: not starting {}: no file exists there",
        program.display()
    );
    eprintln!("mpm-shadow: {}", advice_for(record, lock_text));
    return NOT_FOUND_EXIT_CODE;
}

/// What:
/// The unit tests of the two lines a launcher prints for a start that found nothing.
/// Why:
/// The file those lines name is read from the program's own bytes,
/// and the case where the program itself is gone has no process to start in a test.
#[cfg(test)]
#[path = "starting_it_tests.rs"]
mod tests;
