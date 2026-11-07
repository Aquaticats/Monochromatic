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

/// Imports `Write`,
/// the trait that adds `write_all` to standard output.
use std::io::Write;

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

/// Imports `Child` (a started program that can be waited for),
/// `Command` (build and start a program) and `ExitStatus` (how a finished program ended).
use std::process::{Child, Command, ExitStatus};

/// Imports `cargo_home` from `crate::cargo_home`.
use crate::cargo_home::cargo_home;
/// Imports `EXEC_FAILED_EXIT_CODE`,
/// `NOT_FOUND_EXIT_CODE`,
/// `SHADOWED_NAME`,
/// `SIGNAL_EXIT_CODE_OFFSET` from `crate::constants`.
use crate::constants::{EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE, SHADOWED_NAME, SIGNAL_EXIT_CODE_OFFSET};
/// Imports `prune_stale_own_crates` from `crate::prune`.
use crate::prune::prune_stale_own_crates_reporting;
/// Imports the cleanup text and the variable that asks for the per-path list.
use crate::cleanup_report::{VERBOSE_ENV, cleanup_lines};
/// Imports the depth guard.
use crate::nesting::{NESTING_ENV, must_refuse, nesting_depth};
/// Imports the retrying start.
use crate::start_retry::{RetryReport, StartAttempt, retry_refused_start};
/// Imports the messages for a start that did not happen.
use crate::start_failure::{StartFailure, command_line, describe_nesting_refusal, describe_start_failure};
/// Imports what this module uses from `crate::shadow_notice`.
/// Imports the notice that follows the tool's version and help.
use crate::shadow_notice::{TopLevelRequest, help_section, top_level_request, version_line};
/// Imports the environment view and the toolchain roots (where `install` records version-keyed installs).
use crate::toolchain::scope::{EnvMap, toolchain_roots};
/// Imports `find_in_path` from `crate::real_binary`.
use crate::real_binary::find_in_path;
/// Imports the arrival marker rules and the check made on the open program just before its start.
use crate::shadow_identity::{ARRIVAL_EXIT_CODE, StartMode, Vetted, arrival_at_this_process, arrival_line, mark_start, vet};

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
    return match start_and_wait(program, arguments, 0) {
        ProgramRun::Finished { code, .. } => Ok(code),
        ProgramRun::NotStarted { error, .. } | ProgramRun::WaitFailed { error } => Err(error),
    };
}

/// What:
/// How a start-and-wait ended:
/// the program ran to the end (with its exit code,
/// and how often the system refused the start first),
/// it could not be started (with why,
/// and the retries made),
/// or it started but waiting for it failed.
/// Why:
/// The three need different messages,
/// and only the second means nothing of the command ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProgramRun = { kind: 'finished'; code: number; retry: RetryReport } | { kind: 'not-started'; error: Error; retry: RetryReport } | { kind: 'wait-failed'; error: Error };
/// ```
#[derive(Debug)]
pub enum ProgramRun {
    /// The program ran and ended.
    Finished {
        /// Its exit code (`128 + signal` after a signal).
        code: i32,
        /// What the retrying of the start did.
        retry: RetryReport,
    },
    /// The program could not be started.
    NotStarted {
        /// Why.
        error: std::io::Error,
        /// What the retrying of the start did before giving up.
        retry: RetryReport,
    },
    /// The program started but waiting for it failed.
    WaitFailed {
        /// Why.
        error: std::io::Error,
    },
}

/// One spawn of the vetted program with its arguments and nesting depth,
/// which the start retry repeats while the system refuses to create a process.
struct VettedSpawn<'start> {
    /// The program that was checked while open.
    vetted: &'start Vetted,
    /// The arguments the program is started with.
    arguments: &'start [OsString],
    /// How many shadows deep the child starts.
    child_depth: u32,
}

/// Lets the start retry try the spawn again after a refusal.
impl StartAttempt<Child> for VettedSpawn<'_> {
    /// Spawns the vetted program once.
    fn attempt(&mut self) -> std::io::Result<Child> {
        let mut command: Command = self.vetted.command();
        command.args(self.arguments).env(NESTING_ENV, self.child_depth.to_string());
        mark_start(&mut command, StartMode::Spawn);
        return command.spawn();
    }
}

/// What:
/// Starts `program` with the standard streams inherited and `NESTING_ENV` set to `child_depth`,
/// starting again with growing pauses (within a bound) when the system refuses for lack of processes,
/// then waits for it.
/// Why:
/// A host at its process limit usually recovers within moments;
/// the child needs its depth so a chain of shadows can be stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startAndWait(program: string, args: Buffer[], childDepth: number): ProgramRun
/// ```
pub fn start_and_wait(program: &Path, arguments: &[OsString], child_depth: u32) -> ProgramRun {
    // The program is checked while open and started through that file, and the child carries the arrival marker (`shadow_identity.rs`).
    let vetted: Vetted = match vet(program) {
        Ok(checked) => checked,
        Err(reason) => return ProgramRun::NotStarted { error: std::io::Error::other(reason), retry: RetryReport::default() },
    };
    let mut attempt: VettedSpawn<'_> = VettedSpawn { vetted: &vetted, arguments, child_depth };
    let (started, retry) = retry_refused_start(&mut attempt);
    let mut child = match started {
        Ok(child) => child,
        Err(error) => return ProgramRun::NotStarted { error, retry },
    };
    if retry.refusals > 0 {
        eprintln!(
            "mpm-shadow: starting {} was refused {} times for lack of processes; it started after {} ms of waiting",
            program.display(),
            retry.refusals,
            retry.paused.as_millis()
        );
    }
    return match child.wait() {
        Ok(status) => ProgramRun::Finished { code: exit_code_of(status), retry },
        Err(error) => ProgramRun::WaitFailed { error },
    };
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
        let roots: Vec<PathBuf> = toolchain_roots(&EnvMap::from_process());
        let report = prune_stale_own_crates_reporting(&directory, &roots);
        let verbose: bool = match env::var(VERBOSE_ENV) {
            Ok(value) => !value.is_empty(),
            Err(_) => false,
        };
        for line in cleanup_lines(&report, &directory, verbose) {
            eprintln!("{line}");
        }
    }
}

/// What:
/// `fn print_shadow_notice(request: TopLevelRequest, real_mpm: &Path)` declares a private function that writes
///           the shadow's own line (after `--version`) or section (after `--help`) to standard output.
/// Why:
/// A shadow that prints only mpm's text cannot be told apart from mpm.
/// A closed standard output is ignored:
/// the real mpm's output already went out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function printShadowNotice(request: 'version' | 'help' | 'other', realMpm: string): void
/// ```
fn print_shadow_notice(request: TopLevelRequest, real_mpm: &Path) {
    let text: String = match request {
        TopLevelRequest::Version => version_line(real_mpm),
        TopLevelRequest::Help => help_section(real_mpm),
        TopLevelRequest::Other => return,
    };
    // `let _ =` discards the write result on purpose.
    let _ = std::io::stdout().write_all(text.as_bytes());
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
    // The arrival stop comes before anything else: a shadow that a shadow started as its real mpm does nothing and says so in one line
    // (`shadow_identity.rs`), whatever the nesting count below would have said.
    if let Some(arrival) = arrival_at_this_process() {
        eprintln!("{}", arrival_line(SHADOWED_NAME, &arrival));
        return ARRIVAL_EXIT_CODE;
    }
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

    let depth: u32 = nesting_depth(env::var(NESTING_ENV).ok().as_deref());
    if must_refuse(depth) {
        eprintln!("{}", describe_nesting_refusal(&arguments, &program, depth));
        return EXEC_FAILED_EXIT_CODE;
    }

    let request: TopLevelRequest = top_level_request(&arguments);
    match start_and_wait(&program, &arguments, depth + 1) {
        ProgramRun::Finished { code, .. } => {
            // Like hub after git's version, but only after a successful plain `--version` or `--help` of mpm itself.
            if code == 0 {
                print_shadow_notice(request, &program);
            }
            prune_after_run();
            return code;
        }
        ProgramRun::NotStarted { error, retry } => {
            let failure = StartFailure { outer: &arguments, program: &program, error: &error, retry, depth };
            eprintln!("{}", describe_start_failure(&failure));
            return EXEC_FAILED_EXIT_CODE;
        }
        ProgramRun::WaitFailed { error } => {
            eprintln!(
                "mpm-shadow: started {} for `{}` but waiting for it failed ({error}); whether the command finished is unknown, and the post-run cleanup did not run",
                program.display(),
                command_line(&arguments)
            );
            return EXEC_FAILED_EXIT_CODE;
        }
    }
}
