//! What:
//! The shadow mode:
//! when the binary is started as `mpm`,
//! it finds the real `mpm` on PATH,
//! runs it as a child
//!           with the arguments untouched,
//! waits for it,
//! deletes the superseded cached copies of this repository's own
//!           crates,
//! and exits with the child's own status.
//! Why:
//! The prune must run after mpm finishes,
//! so the process cannot replace itself with mpm;
//! it has to wait.
//!           The prune runs after every run,
//! not only after `install`,
//! `restore` or `upgrade`:
//! the verb is hard to
//!           find (options may come before it),
//! a zero exit status does not prove anything was provisioned (`mpm
//!           upgrade --all` exits 0 when some managers fail),
//! and the prune is idempotent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim.ts
//! export function runShim(args: string[]): number {
//!   const real = findInPath('mpm', process.env.PATH ?? '', ownPath);
//!   const result = spawnSync(real, args, { stdio: 'inherit' });
//!   pruneStaleOwnCrates(cargoHome());
//!   return result.status ?? 128 + signalNumber(result.signal);
//! }
//! ```

/// Imports the process-environment module (PATH,
/// CARGO_HOME,
/// HOME).
use std::env;

/// Imports the owned operating-system text type:
/// arguments may be invalid UTF-8,
/// and the shim passes them byte for byte.
use std::ffi::OsString;

/// Imports the file-system module (resolves the shadow's own path).
use std::fs;

/// What:
/// `use std::os::unix::process::ExitStatusExt;` imports a trait that adds `.signal()` to a process's exit
///           status.
/// Why:
/// A child killed by a signal has no exit code,
/// and the shim reports it the way a shell does.
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

/// Imports `cargo_home` from `crate::cargo_home`.
use crate::cargo_home::cargo_home;
/// Imports `EXEC_FAILED_EXIT_CODE`,
/// `NOT_FOUND_EXIT_CODE`,
/// `SHADOWED_NAME`,
/// `SIGNAL_EXIT_CODE_OFFSET` from `crate::constants`.
use crate::constants::{EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE, SHADOWED_NAME, SIGNAL_EXIT_CODE_OFFSET};
/// Imports `prune_stale_own_crates` from `crate::prune`.
use crate::prune::prune_stale_own_crates;
/// Imports `find_in_path` from `crate::real_binary`.
use crate::real_binary::find_in_path;

/// What:
/// `pub fn exit_code_of(status: ExitStatus) -> i32` declares a public function turning a finished child's
///           status into the number the shim exits with.
/// Why:
/// A normal exit passes its code through;
/// death by signal becomes `128 + signal`,
/// the shell convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function exitCodeOf(result: SpawnSyncReturns): number { return result.status ?? 128 + os.constants.signals[result.signal]; }
/// ```
pub fn exit_code_of(status: ExitStatus) -> i32 {
    // `status.code()` returns `Some(code)` for a normal exit and `None` after a signal; `if let Some(code)` unwraps it.
    if let Some(code) = status.code() {
        return code;
    }
    if let Some(signal) = status.signal() {
        return SIGNAL_EXIT_CODE_OFFSET + signal;
    }
    return EXEC_FAILED_EXIT_CODE;
}

/// What:
/// `pub fn run_program(program: &Path, arguments: &[OsString]) -> std::io::Result<i32>` declares a public
///           function returning a `Result`:
/// `Ok(code)` when the program ran to the end,
/// `Err(error)` when it could not
///           be started.
/// Why:
/// The caller prunes only when the program really ran,
/// and reports a start failure with the right exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runProgram(program: string, args: string[]): number  // throws when the spawn fails
/// ```
pub fn run_program(program: &Path, arguments: &[OsString]) -> std::io::Result<i32> {
    // What:     `Command::new(program).args(arguments).status()` builds the command, starts it with the standard
    //           streams inherited from this process, and waits for it. `?` leaves the function with the `Err` when the
    //           start failed, and otherwise unwraps the `Ok`.
    // Why:      Inheriting the streams keeps mpm's output, prompts and colors exactly as they would be without the shim.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result = spawnSync(program, arguments, { stdio: 'inherit' });  // throws on spawn failure
    // ```
    let status: ExitStatus = Command::new(program).args(arguments).status()?;

    // What:     `Ok(exit_code_of(status))` wraps the number in the success case of `Result`. No trailing `;` and no
    //           `return`, so a tail expression would be the return value; this file writes `return` explicitly.
    // Why:      Hand the child's status to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return exitCodeOf(result);
    // ```
    return Ok(exit_code_of(status));
}

/// What:
/// `fn own_real_path() -> PathBuf` declares a private function returning an owned path.
/// Why:
/// The PATH search skips every entry that resolves to this executable.
/// Falls back to an empty path,
/// which
///           matches nothing,
/// if the system cannot say.
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
/// `fn prune_after_run()` declares a private function with no result.
/// Why:
/// It deletes the superseded copies and says on standard error which ones,
/// so a person running mpm sees it;
///           standard output stays exactly mpm's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pruneAfterRun(): void { ... }
/// ```
fn prune_after_run() {
    // `env::var("CARGO_HOME").ok()` turns the `Result` into an `Option`: `Some(text)` when set, `None` when unset.
    let home: Option<PathBuf> = cargo_home(env::var("CARGO_HOME").ok(), env::var("HOME").ok());
    if let Some(directory) = home {
        for path in prune_stale_own_crates(&directory) {
            eprintln!("mpm-shadow: deleted stale {}", path.display());
        }
    }
}

/// What:
/// `pub fn run_shim(arguments: Vec<OsString>) -> i32` declares a public function taking the arguments mpm was
///           started with (an owned list) and returning the exit code.
/// Why:
/// `main.rs` calls it when the binary was started under the name `mpm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runShim(args: Buffer[]): number
/// ```
pub fn run_shim(arguments: Vec<OsString>) -> i32 {
    let path_variable: String = env::var("PATH").unwrap_or_default();
    let own_path: PathBuf = own_real_path();

    // What:     `match find_in_path(...) { Some(program) => ..., None => ... }` picks the branch for found or not
    //           found. `&` lends the borrowed arguments.
    // Why:      With no real mpm on PATH there is nothing to run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const program = findInPath(...); if (program === undefined) { console.error(...); return 127; }
    // ```
    let program: PathBuf = match find_in_path(SHADOWED_NAME, &path_variable, &own_path) {
        Some(found) => found,
        None => {
            eprintln!("mpm-shadow: no real {SHADOWED_NAME} found on PATH");
            return NOT_FOUND_EXIT_CODE;
        }
    };

    match run_program(&program, &arguments) {
        Ok(code) => {
            prune_after_run();
            return code;
        }
        Err(error) => {
            eprintln!("mpm-shadow: could not start {}: {error}", program.display());
            return EXEC_FAILED_EXIT_CODE;
        }
    }
}
