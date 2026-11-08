//! What:
//! The command-line glue of `cargo-shadow lean`:
//! it turns the parsed arguments into a run or a timer installation.
//! Why:
//! `lean_run.rs` and `lean_timer_units.rs` take plain values,
//! so tests call them without a command line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runLeanAction(action: LeanAction): number
//! ```

/// Paths and time.
use std::{path::PathBuf, time::Duration};

use crate::cli::{LeanAction, MechanismName};
use crate::lean_checkouts::main_checkout;
use crate::lean_policy::RUN_DEADLINE_SECONDS;
use crate::lean_run::{Mechanism, RunOptions, run_lean};
use crate::lean_timer_units::install_timer;

/// What:
/// `fn mechanism_of(name: MechanismName) -> Mechanism` maps the command-line name to the run's mechanism.
/// Why:
/// The grammar and the run keep separate types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mechanismOf = (name: MechanismName): Mechanism
/// ```
fn mechanism_of(name: MechanismName) -> Mechanism {
    return match name {
        MechanismName::Trees => Mechanism::Trees,
        MechanismName::Units => Mechanism::Units,
        MechanismName::Records => Mechanism::Records,
        MechanismName::Toolchains => Mechanism::Toolchains,
        MechanismName::Podman => Mechanism::Podman,
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
            let seconds: u64 = deadline_minutes.map_or(RUN_DEADLINE_SECONDS, |minutes| { return minutes * 60; });
            let options: RunOptions = RunOptions { dry_run: *dry_run, only: only.iter().map(|name| { return mechanism_of(*name); }).collect(), deadline: Duration::from_secs(seconds), start };
            return run_lean(&options);
        }
        LeanAction::TimerInstall { no_enable, unit_directory } => {
            let Some(repository) = main_checkout(&start) else {
                eprintln!("cargo-shadow: lean timer-install: {} is not inside a Git checkout; start it from the repository", start.display());
                return 1;
            };
            let home: PathBuf = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
            let units: PathBuf = unit_directory.clone().unwrap_or_else(|| { return home.join(".config").join("systemd").join("user"); });
            let binary: PathBuf = repository.join(".cache").join("toolchain-bin").join("cargo-shadow");
            return match install_timer(&repository, &binary, &units, !no_enable) {
                Ok(lines) => {
                    for line in lines {
                        println!("{line}");
                    }
                    0
                }
                Err(reason) => {
                    eprintln!("cargo-shadow: lean timer-install: {reason}");
                    1
                }
            };
        }
    }
}
