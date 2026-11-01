//! What:
//! The shadow mode: when the binary is started as `pnpm`, it warns about manifest problems in the workspace of the
//! current directory, finds the real `pnpm` on PATH, runs it as a child with the arguments untouched, waits for it,
//! and exits with the child's own status.
//! Why:
//! The warning never blocks. A scan problem, a missing workspace and a closed standard error all lead to the same
//! thing: pnpm runs. Only a missing or unstartable pnpm is the shadow's own failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.ts
//! export function runShim(args: string[]): number {
//!   warnAboutWorkspace();
//!   const real = findInPath('pnpm', process.env.PATH ?? '', ownPath);
//!   const result = spawnSync(real, args, { stdio: 'inherit' });
//!   return result.status ?? 128 + signalNumber(result.signal);
//! }
//! ```

/// Imports the process-environment module (PATH, the current directory).
use std::env;

/// Imports the owned operating-system text type: arguments may be invalid UTF-8 and pass byte for byte.
use std::ffi::OsString;

/// Imports the file-system module (resolves the shadow's own path).
use std::fs;

/// Imports `Write`, the trait that adds `write_all` to standard error.
use std::io::Write;

/// What:
/// `use std::os::unix::process::ExitStatusExt;` imports a trait that adds `.signal()` to a process's exit status.
/// Why:
/// A child killed by a signal has no exit code, and the shadow reports it the way a shell does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node reports `result.signal` directly.
/// ```
use std::os::unix::process::ExitStatusExt;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports `Command` (build and start a program) and `ExitStatus` (how a finished program ended).
use std::process::{Command, ExitStatus};

/// Imports the fixed values.
use crate::constants::{EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE, SHADOWED_NAME, SIGNAL_EXIT_CODE_OFFSET};
/// Imports the PATH lookup.
use crate::real_binary::find_in_path;
/// Imports the report renderer.
use crate::report::render_report;
/// Imports the workspace scan.
use crate::scan::{find_workspace_root, scan_workspace};
/// Imports the notice that follows pnpm's version and help.
use crate::shadow_notice::{TopLevelRequest, notice_text, top_level_request};

/// What:
/// `pub fn exit_code_of(status: ExitStatus) -> i32` turns a finished child's status into the number the shadow exits with.
/// Why:
/// A normal exit passes its code through; death by signal becomes `128 + signal`, the shell convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function exitCodeOf(result: SpawnSyncReturns): number { return result.status ?? 128 + os.constants.signals[result.signal]; }
/// ```
pub fn exit_code_of(status: ExitStatus) -> i32 {
    // `status.code()` is `Some(code)` for a normal exit and `None` after a signal.
    if let Some(code) = status.code() {
        return code;
    }
    if let Some(signal) = status.signal() {
        return SIGNAL_EXIT_CODE_OFFSET + signal;
    }
    return EXEC_FAILED_EXIT_CODE;
}

/// What:
/// `pub fn run_program(program: &Path, arguments: &[OsString]) -> std::io::Result<i32>` returns `Ok(code)` when the
/// program ran to the end and `Err(error)` when it could not be started.
/// Why:
/// The caller reports a start failure with the right exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runProgram(program: string, args: string[]): number  // throws when the spawn fails
/// ```
pub fn run_program(program: &Path, arguments: &[OsString]) -> std::io::Result<i32> {
    // `Command::new(program).args(arguments).status()` starts the program with the standard streams inherited from this
    // process and waits for it; `?` leaves the function with the `Err` when the start failed.
    let status: ExitStatus = Command::new(program).args(arguments).status()?;
    return Ok(exit_code_of(status));
}

/// What:
/// `fn own_real_path() -> PathBuf` is the shadow's own resolved executable path, or an empty path when the system cannot say.
/// Why:
/// The PATH search skips every entry that resolves to it; an empty path matches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownRealPath(): string { return fs.realpathSync(process.execPath); }
/// ```
fn own_real_path() -> PathBuf {
    if let Ok(executable) = env::current_exe() {
        if let Ok(resolved) = fs::canonicalize(&executable) {
            return resolved;
        }
        return executable;
    }
    return PathBuf::new();
}

/// What:
/// `pub fn warn_about_workspace(start: &Path, sink: &mut dyn Write)` scans the workspace that contains `start` and
/// writes the report to `sink` when there is one.
/// Why:
/// The sink is a parameter so tests read the text; the shadow passes standard error. `&mut dyn Write` is a borrowed
/// writer of any type (a TS `Writable`). A write error is ignored on purpose: a closed standard error must not stop pnpm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function warnAboutWorkspace(start: string, sink: Writable): void
/// ```
pub fn warn_about_workspace(start: &Path, sink: &mut dyn Write) {
    // No `pnpm-workspace.yaml` above `start` means there is no workspace to check; pnpm runs without a word.
    let root: PathBuf = match find_workspace_root(start) {
        Some(found) => found,
        None => return,
    };
    let report = scan_workspace(&root);
    if let Some(text) = render_report(&root, &report) {
        // `let _ =` discards the write result on purpose.
        let _ = sink.write_all(text.as_bytes());
    }
}

/// What:
/// `pub fn run_shim(arguments: Vec<OsString>) -> i32` takes the arguments pnpm was started with (an owned list) and
/// returns the exit code.
/// Why:
/// `main.rs` calls it when the binary was started under the name `pnpm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runShim(args: Buffer[]): number
/// ```
pub fn run_shim(arguments: Vec<OsString>) -> i32 {
    // The warning comes first and on every run, `pnpm --version` included; it goes to standard error, so pnpm's
    // standard output stays exactly pnpm's.
    if let Ok(current) = env::current_dir() {
        warn_about_workspace(&current, &mut std::io::stderr());
    }

    let path_variable: String = env::var("PATH").unwrap_or_default();
    let own_path: PathBuf = own_real_path();
    let program: PathBuf = match find_in_path(SHADOWED_NAME, &path_variable, &own_path) {
        Some(found) => found,
        None => {
            eprintln!("pnpm-shadow: no real {SHADOWED_NAME} found on PATH");
            return NOT_FOUND_EXIT_CODE;
        }
    };
    let request: TopLevelRequest = top_level_request(&arguments);
    match run_program(&program, &arguments) {
        Ok(code) => {
            // Like hub after git's version: only after a successful top-level `--version` or `--help` of pnpm itself,
            // so the notice follows pnpm's own text and is the one place the shadow names itself. A closed standard
            // output is ignored on purpose: pnpm's output already went out.
            if code == 0 {
                if let Some(text) = notice_text(request, &program) {
                    let _ = std::io::stdout().write_all(text.as_bytes());
                }
            }
            return code;
        }
        Err(error) => {
            eprintln!("pnpm-shadow: could not start {}: {error}", program.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    }
}
