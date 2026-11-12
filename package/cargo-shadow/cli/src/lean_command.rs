//! What:
//! The command-line glue of `cargo-shadow lean`:
//! it turns the parsed arguments into a run,
//! a sweep of one package's run records,
//! or a service installation.
//! Why:
//! `lean_run.rs`,
//! `lean_run_steps.rs` and `lean_service_units.rs` take plain values,
//! so tests call them without a command line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runLeanAction(action: LeanAction): number
//! ```

/// Files and paths.
use std::{fs, path::{Path, PathBuf}, time::Duration};

/// Imports the names this file uses from `crate::cli`.
use crate::cli::{LeanAction, MechanismName};
/// Imports the names this file uses from `crate::lean_checkouts`.
use crate::lean_checkouts::main_checkout;
/// Imports the names this file uses from `crate::lean_guard`.
use crate::lean_guard::GuardEnvironment;
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::Deadline;
/// Imports the names this file uses from `crate::lean_policy`.
use crate::lean_policy::RUN_DEADLINE_SECONDS;
/// Imports the names this file uses from `crate::lean_run`.
use crate::lean_run::{Mechanism, RunOptions, run_lean};
/// Imports the names this file uses from `crate::lean_run_report`.
use crate::lean_run_report::summary_line;
/// Imports the names this file uses from `crate::lean_run_steps`.
use crate::lean_run_steps::run_records;
/// Imports the names this file uses from `crate::lean_service_units`.
use crate::lean_service_units::{SERVICE_NAME, TIMER_NAME, install_service};

/// What:
/// `fn absent_notes(directories: &[PathBuf]) -> Vec<String>` names each directory that is not a directory on disk.
/// Why:
/// A producing task calls the sweep before its first record exists,
/// so a directory that is not there yet holds nothing and is not a failure,
/// while a path that names something else is at least said in the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absentNotes = (directories: string[]): string[] => directories.filter((d) => !isDirectory(d)).map((d) => `${d} is not there yet, so it holds no record`);
/// ```
pub(crate) fn absent_notes(directories: &[PathBuf]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for directory in directories {
        if !directory.is_dir() {
            lines.push(format!("run records: {} is not a directory, so it holds no record", directory.display()));
        }
    }
    return lines;
}

/// What:
/// `pub(crate) fn directories_from(start: &Path, given: &[PathBuf]) -> Vec<PathBuf>` resolves each named directory to one absolute path.
/// Why:
/// The in-use scan compares a root with the working directory,
/// program,
/// open files and mapped files of every process,
/// and the kernel reports those resolved,
/// so a path relative to a task's own folder,
/// a path through a symbolic link and a path with a `..` step all have to name the same root the scan will compare.
/// A directory that is not there yet keeps its joined name,
/// because nothing can be using it and the sweep reports it as absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const directoriesFrom = (start: string, given: string[]): string[] => given.map((one) => { try { return realpathSync(isAbsolute(one) ? one : join(start, one)); } catch { return isAbsolute(one) ? one : join(start, one); } });
/// ```
pub(crate) fn directories_from(start: &Path, given: &[PathBuf]) -> Vec<PathBuf> {
    let mut resolved: Vec<PathBuf> = Vec::new();
    for path in given {
        let joined: PathBuf = if path.is_absolute() { path.clone() } else { start.join(path) };
        resolved.push(fs::canonicalize(&joined).unwrap_or(joined));
    }
    return resolved;
}

/// What:
/// `fn mechanism_of(name: MechanismName) -> Mechanism` maps the command-line name to the run's mechanism.
/// Why:
/// The grammar and the run keep separate types,
/// and the test of the run's order reads every name the command line takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mechanismOf = (name: MechanismName): Mechanism
/// ```
pub(crate) fn mechanism_of(name: MechanismName) -> Mechanism {
    return match name {
        MechanismName::Trees => Mechanism::Trees,
        MechanismName::Units => Mechanism::Units,
        MechanismName::Records => Mechanism::Records,
        MechanismName::Toolchains => Mechanism::Toolchains,
        MechanismName::Podman => Mechanism::Podman,
    };
}

/// What:
/// `fn install_service_action(start: &Path, invoked_as: &str, no_enable: bool, unit_directory: Option<&Path>) -> i32`
/// installs the service from the main checkout of `start` and returns the exit code.
/// Why:
/// `service-install` and the old name `timer-install` do the same thing now,
/// and every message names the command the user actually ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installServiceAction(start: string, invokedAs: string, noEnable: boolean, unitDirectory?: string): number
/// ```
fn install_service_action(start: &Path, invoked_as: &str, no_enable: bool, unit_directory: Option<&Path>) -> i32 {
    let Some(repository) = main_checkout(start) else {
        eprintln!("cargo-shadow: lean {invoked_as}: {} is not inside a Git checkout; start it from the repository", start.display());
        return 1;
    };
    let home: PathBuf = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let units: PathBuf = match unit_directory {
        Some(chosen) => chosen.to_path_buf(),
        None => home.join(".config").join("systemd").join("user"),
    };
    let binary: PathBuf = repository.join(".cache").join("toolchain-bin").join("cargo-shadow");
    return match install_service(&repository, &binary, &units, !no_enable) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            0
        }
        Err(reason) => {
            eprintln!("cargo-shadow: lean {invoked_as}: {reason}");
            1
        }
    };
}

/// What:
/// `pub fn run_lean_action(action: &LeanAction) -> i32` runs one `lean` action and returns the exit code.
/// Why:
/// Errors go to standard error and reports to standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runLeanAction(action: LeanAction): number
/// ```
pub fn run_lean_action(action: &LeanAction) -> i32 {
    let start: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(error) => {
            eprintln!("cargo-shadow: lean: cannot read the working directory: {error}");
            return 1;
        }
    };
    match action {
        LeanAction::Run { dry_run, only, deadline_minutes } => {
            let seconds: u64 = match deadline_minutes {
                Some(minutes) => minutes * 60,
                None => RUN_DEADLINE_SECONDS,
            };
            let mut mechanisms: Vec<Mechanism> = Vec::new();
            for name in only {
                mechanisms.push(mechanism_of(*name));
            }
            let options: RunOptions = RunOptions { dry_run: *dry_run, only: mechanisms, deadline: Duration::from_secs(seconds), start };
            return run_lean(&options);
        }
        LeanAction::Records { directory, dry_run, deadline_minutes } => {
            let seconds: u64 = match deadline_minutes {
                Some(minutes) => minutes * 60,
                None => RUN_DEADLINE_SECONDS,
            };
            let given: Vec<PathBuf> = directories_from(&start, directory);
            for line in absent_notes(&given) {
                summary_line(&line);
            }
            let deadline: Deadline = Deadline::after(Duration::from_secs(seconds));
            let environment: GuardEnvironment = GuardEnvironment::real();
            for line in run_records(&given, *dry_run, &deadline, &environment) {
                summary_line(&line);
            }
            return 0;
        }
        LeanAction::ServiceInstall { no_enable, unit_directory } => {
            return install_service_action(&start, "service-install", *no_enable, unit_directory.as_deref());
        }
        LeanAction::TimerInstall { no_enable, unit_directory } => {
            println!("cargo-shadow: `lean timer-install` is now `lean service-install`: it writes {SERVICE_NAME}, removes the once-a-day {TIMER_NAME}, and enables the service, which runs one pass after another");
            return install_service_action(&start, "timer-install", *no_enable, unit_directory.as_deref());
        }
    }
}
