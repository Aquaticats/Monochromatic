//! What:
//! The command-line grammar of the binary when it is started under its own name,
//! `cargo-shadow` (or as the cargo
//! subcommand `cargo shadow`):
//! the whole-repository catalog commands.
//! Why:
//! The shadow mode passes cargo's arguments through untouched and has no grammar of its own;
//! the catalog commands that act on
//! every lockfile at once (an install over the repository,
//! an update over the repository) live
//! here,
//! with pnpm's flag names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cli.ts
//! program.name('cargo-shadow').command('install').option('--frozen-lockfile')...
//! ```

#![cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(
        avoidable_closure,
        reason = "clap's derive generates the missing-argument closures in its FromArgMatches implementation with the field type as their span, so no author-written closure exists to refactor; same cause as package/forbidden-strings/cli/doc/closure-lint-suppressions.md records"
    )
)]

/// What:
/// `use clap::{Parser, Subcommand};` imports `Parser`,
/// `Subcommand` from the external crate `clap` so this file can name them unqualified.
/// Why:
/// `Parser` must be in scope for its methods:
/// `try_parse_from` on the derived argument parser;
/// `Subcommand` is used by `Cli`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Parser, Subcommand } from 'clap';
/// ```
use clap::{Parser, Subcommand};

/// What:
/// `use crate::commands::run_command;` imports `run_command` from another module of this crate so this file can name it unqualified.
/// Why:
/// `run_command` is used by `run_cli`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_command } from 'crate/commands';
/// ```
use crate::commands::run_command;

/// What:
/// `#[derive(Parser)]` writes the argument parser from the attributes;
/// `#[command(subcommand)]` makes one required word
/// select the command.
/// Why:
/// A shadow with no commands of its own would still answer `--help` and `--version` by its own name;
/// these two do the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = new Command().name('cargo-shadow').version(version).description(about);
/// ```
#[derive(Parser)]
#[command(
    name = "cargo-shadow",
    version,
    about = "Shadows cargo. Started under the name `cargo` it makes the workspace's Cargo.lock satisfy its manifests under the rust-catalog.toml trust policy before a command that resolves dependencies, then runs the real cargo unchanged. Started under its own name it runs the repository-wide catalog commands: install, update.",
    long_about = None,
    arg_required_else_help = true
)]
pub struct Cli {
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The catalog commands.
#[derive(Subcommand)]
pub enum Command {
    /// pnpm install for Cargo.lock files:
    /// update the lockfiles that no longer satisfy their manifests,
    /// offline and moving
    /// only what the manifests force.
    /// Frozen (fails naming them) in CI or with --frozen-lockfile.
    Install {
        /// Fail naming the stale roots and write nothing (the default in CI).
        #[arg(long, conflicts_with = "no_frozen_lockfile")]
        frozen_lockfile: bool,
        /// Update stale lockfiles even in CI.
        #[arg(long)]
        no_frozen_lockfile: bool,
        /// Report which lockfiles are stale,
        /// then restore every Cargo.lock.
        #[arg(long)]
        dry_run: bool,
    },
    /// pnpm --recursive --no-save update for Cargo.lock files:
    /// move every lockfile to the newest releases that fit,
    /// under the
    /// release-age and trust policy.
    Update {
        /// Report what would change,
        /// then restore every Cargo.lock.
        #[arg(long)]
        dry_run: bool,
    },
    /// Keep the build folders lean:
    /// remove build trees,
    /// units,
    /// run records,
    /// toolchains and container images that nothing uses,
    /// each only after a proof it is not in use,
    /// or install the systemd user service that keeps this running.
    Lean {
        /// What to do.
        #[command(subcommand)]
        action: LeanAction,
    },
    /// Acknowledge a journal that a recovery refuses:
    /// you assert that every process or worker of the run that left it,
    /// and can modify the workspace,
    /// has stopped.
    /// It overrides only the uncertainty of the containment of that run.
    Acknowledge {
        /// The journal directory the refusal named.
        journal: std::path::PathBuf,
    },
}

/// The mechanisms of a lean run,
/// by the name `--only` takes.
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum MechanismName {
    /// Idle trees of the host build base.
    Trees,
    /// The older-compiler sweep and the size caps of profile directories.
    Units,
    /// Verification run records.
    Records,
    /// Dated nightly toolchains no checkout pins.
    Toolchains,
    /// Unused Podman images and volumes.
    Podman,
}

/// What `cargo-shadow lean` does.
#[derive(Subcommand)]
pub enum LeanAction {
    /// One pass over this machine from inside a checkout of the repository.
    /// Nothing is removed unless a lock and a process scan show it is not in use.
    Run {
        /// Report what would be removed and remove nothing.
        #[arg(long)]
        dry_run: bool,
        /// Run only these mechanisms (all by default).
        #[arg(long, value_delimiter = ',')]
        only: Vec<MechanismName>,
        /// The longest the whole run may take,
        /// in minutes.
        #[arg(long)]
        deadline_minutes: Option<u64>,
    },
    /// Install the systemd user service that keeps `lean run` running:
    /// each pass ends on its own deadline and the next starts a minute after it.
    /// It stops and disables `cargo-shadow-lean.timer`,
    /// the once-a-day timer the old install wrote,
    /// and removes its file.
    ServiceInstall {
        /// Write the unit files and leave the service off.
        #[arg(long)]
        no_enable: bool,
        /// Where the unit files go (default `~/.config/systemd/user`).
        #[arg(long)]
        unit_directory: Option<std::path::PathBuf>,
    },
    /// The old name of `service-install`:
    /// it now writes the continuous service,
    /// removes the once-a-day timer and enables the service,
    /// and it says so when it runs.
    #[command(hide = true)]
    TimerInstall {
        /// Write the unit files and leave the service off.
        #[arg(long)]
        no_enable: bool,
        /// Where the unit files go (default `~/.config/systemd/user`).
        #[arg(long)]
        unit_directory: Option<std::path::PathBuf>,
    },
    /// Sweep the run records under the verification directories a producing task writes,
    /// with the same age,
    /// size cap and proof of "not in use" as `lean run`.
    /// A producing task calls this before it writes a record,
    /// so its own directory is bounded without the task reimplementing the retention.
    Records {
        /// The `target/verification` directories to sweep;
        /// a relative path is taken from the working directory.
        #[arg(required = true)]
        directory: Vec<std::path::PathBuf>,
        /// Report what would be removed and remove nothing.
        #[arg(long)]
        dry_run: bool,
        /// The longest the sweep may take,
        /// in minutes.
        #[arg(long)]
        deadline_minutes: Option<u64>,
    },
}

/// What:
/// `pub fn run_cli(arguments: Vec<String>) -> i32` parses the arguments and returns the exit code.
/// Why:
/// clap prints its own help and error text;
/// the function only maps the outcome to the number the process exits with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runCli(args: string[]): number
/// ```
pub fn run_cli(arguments: Vec<String>) -> i32 {
    match Cli::try_parse_from(arguments) {
        Ok(parsed) => return run_command(parsed.command),
        Err(error) => {
            let code: i32 = error.exit_code();
            let _ = error.print();
            return code;
        }
    }
}
