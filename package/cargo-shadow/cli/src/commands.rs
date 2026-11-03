//! What:
//! Runs the repository-wide catalog commands for the CLI: install, update, verify.
//! Why:
//! They share one setup (repository root, real cargo under the pinned toolchain, crates.io registry) and one way of printing
//! a report and turning findings into an exit code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commands.ts
//! export async function main(args: string[]): Promise<void>
//! ```

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

use crate::cargo_run::RealCargo;
use crate::cli::Command;
use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE};
use crate::crates::locked_crate_dirs;
use crate::http::UreqFetcher;
use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles, is_continuous_integration};
use crate::real_binary::find_real_cargo;
use crate::registry::CratesIo;
use crate::toolchain::cargo_toolchain;
use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};
use crate::verify::{git_base_lock_reader, verify_rust_lockfiles};
use crate::workspace::find_catalog_root;

/// Prints findings under a heading, with the pointer to the settings.
pub fn print_findings(heading: &str, problems: &[String]) {
    eprintln!("{heading}: {} finding(s).", problems.len());
    for problem in problems {
        eprintln!("- {problem}");
    }
    eprintln!("Settings live in rust-catalog.toml; see doc/decision/rust-dependency-catalog.md.");
}

/// Prints an install report.
pub fn print_install_report(report: &InstallReport, dry_run: bool, sink: &mut dyn Write) {
    let _ = writeln!(sink, 
        "{} lockfiles, {} did not satisfy their manifests{}",
        report.crate_dirs.len(),
        report.stale.len(),
        if report.stale.is_empty() { "." } else { ":" }
    );
    for crate_dir in &report.stale {
        let _ = writeln!(sink, "  {crate_dir}");
    }
    if !report.added.is_empty() {
        let _ = writeln!(sink, "{} versions no lockfile held before:", report.added.len());
    }
    for added in &report.added {
        let _ = writeln!(sink, "  {} {}", added.name, added.version);
    }
    for warning in &report.warnings {
        let _ = writeln!(sink, "Warning: {warning}");
    }
    if dry_run {
        let _ = writeln!(sink, "Dry run: every Cargo.lock was restored.");
    } else if report.written {
        let _ = writeln!(sink, "Every Cargo.lock satisfies its manifests. Review the diff.");
    }
}

/// Prints an update report.
pub fn print_update_report(report: &UpdateReport, dry_run: bool, sink: &mut dyn Write) {
    let _ = writeln!(sink, "{} lockfiles, {} newly locked versions:", report.crate_dirs.len(), report.added.len());
    for added in &report.added {
        let _ = writeln!(sink, "  {} {}", added.name, added.version);
    }
    for crate_dir in &report.fell_back {
        let _ = writeln!(sink, "Resolved {crate_dir} without the release-age cutoff (minimumReleaseAgeStrict = false).");
    }
    for warning in &report.warnings {
        let _ = writeln!(sink, "Warning: {warning}");
    }
    for selector in &report.pruned {
        let _ = writeln!(sink, "Pruned {selector} from rust-catalog.toml.");
    }
    if dry_run {
        let _ = writeln!(sink, "Dry run: every Cargo.lock was restored.");
    } else if report.written {
        let _ = writeln!(sink, "Kept the new lockfiles. Next: `mise run catalog:tighten`, then `mise run sync:files`, then `cargo-shadow install`.");
    }
}

/// Milliseconds since 1970 now.
fn now_millis() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| { return elapsed.as_millis() as i64; }).unwrap_or(0);
}

/// What:
/// `pub fn run_command(command: Command) -> i32` runs one catalog command from the repository that holds the working
/// directory and returns the exit code: 0 on success, 1 on a failure or a policy refusal.
/// Why:
/// Errors and findings go to standard error, reports to standard output, so a pipeline can read the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function runCommand(command: Command): Promise<number>
/// ```
pub fn run_command(command: Command) -> i32 {
    let current: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(error) => {
            eprintln!("cargo-shadow: no working directory: {error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    let root: PathBuf = match find_catalog_root(&current) {
        Some(found) => found,
        None => {
            eprintln!("cargo-shadow: no {CATALOG_MARKER} above {}", current.display());
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    let text: String = match std::fs::read_to_string(root.join(CATALOG_MARKER)) {
        Ok(found) => found,
        Err(error) => {
            eprintln!("cargo-shadow: {CATALOG_MARKER}: {error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    let document: CatalogDocument = match parse_catalog(&text, CATALOG_MARKER) {
        Ok(found) => found,
        Err(error) => {
            eprintln!("{error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    let registry: CratesIo = CratesIo::with_defaults(&root, Box::new(UreqFetcher::new()), Box::new(UreqFetcher::new()));
    if let Command::Verify { base } = &command {
        let reader = match git_base_lock_reader(&root, base) {
            Ok(found) => found,
            Err(message) => {
                eprintln!("{message}");
                return INSTALL_FAILED_EXIT_CODE;
            }
        };
        return match verify_rust_lockfiles(&root, now_millis(), &document.settings, &registry, &reader) {
            Ok(problems) if problems.is_empty() => {
                println!("Every registry version locked since {base} passes the trust policy.");
                0
            }
            Ok(problems) => {
                print_findings("Trust verification failed", &problems);
                INSTALL_FAILED_EXIT_CODE
            }
            Err(message) => {
                eprintln!("{message}");
                INSTALL_FAILED_EXIT_CODE
            }
        };
    }
    let program: PathBuf = match find_real_cargo() {
        Some(found) => found,
        None => {
            eprintln!("cargo-shadow: no real cargo found on PATH");
            return crate::constants::NOT_FOUND_EXIT_CODE;
        }
    };
    let explicit: String = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_default();
    let toolchain: String = match cargo_toolchain(&root, &explicit) {
        Ok((chosen, warning)) => {
            if let Some(notice) = warning {
                eprintln!("cargo-shadow: warning: {notice}");
            }
            chosen
        }
        Err(message) => {
            eprintln!("cargo-shadow: toolchain.lock.json is not readable: {message}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    let runner: RealCargo = RealCargo { program, toolchain };
    let crate_dirs: Vec<String> = match locked_crate_dirs(&root) {
        Ok(found) => found,
        Err(error) => {
            eprintln!("cargo-shadow: {error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    match command {
        Command::Install { frozen_lockfile, no_frozen_lockfile, dry_run } => {
            let frozen: bool = if frozen_lockfile {
                true
            } else if no_frozen_lockfile {
                false
            } else {
                is_continuous_integration(&std::env::var("CI").unwrap_or_default())
            };
            let outcome = install_rust_lockfiles(&InstallRequest {
                root: &root,
                crate_dirs: &crate_dirs,
                frozen,
                offline: !is_continuous_integration(&std::env::var("CI").unwrap_or_default()),
                dry_run,
                now: now_millis(),
                settings: &document.settings,
                runner: &runner,
                registry: &registry,
            });
            return match outcome {
                Ok(report) => {
                    print_install_report(&report, dry_run, &mut std::io::stdout());
                    if report.problems.is_empty() {
                        0
                    } else {
                        print_findings("Install refused; every Cargo.lock was restored", &report.problems);
                        INSTALL_FAILED_EXIT_CODE
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    INSTALL_FAILED_EXIT_CODE
                }
            };
        }
        Command::Update { dry_run } => {
            let outcome = update_rust_lockfiles(
                &UpdateRequest {
                    root: &root,
                    crate_dirs: &crate_dirs,
                    dry_run,
                    now: now_millis(),
                    settings: &document.settings,
                    catalog_path: &root.join(CATALOG_MARKER),
                    runner: &runner,
                    registry: &registry,
                    forward: &[],
                },
                &UpdateKind::Full,
            );
            return match outcome {
                Ok(report) => {
                    print_update_report(&report, dry_run, &mut std::io::stdout());
                    if report.problems.is_empty() {
                        0
                    } else {
                        print_findings("Update refused; every Cargo.lock was restored", &report.problems);
                        INSTALL_FAILED_EXIT_CODE
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    INSTALL_FAILED_EXIT_CODE
                }
            };
        }
        Command::Verify { .. } => return 0,
    }
}
