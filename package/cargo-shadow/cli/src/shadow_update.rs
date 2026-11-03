//! What:
//! The shadow's answer to `cargo update` and `cargo generate-lockfile`: the pnpm command each stands for, run with the
//! release-age and trust policy applied while versions are chosen.
//! Why:
//! The owner's rule (2026-11-03): follow pnpm semantics. `cargo update` is `pnpm update` and `cargo generate-lockfile` is
//! `pnpm install --lockfile-only`; `update_plan.rs` documents the flag mapping. Outside a catalog repository the command goes to
//! cargo untouched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-update.ts
//! export function runUpdateCommand(args: string[]): number | undefined  // undefined: run cargo itself
//! ```

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use crate::commands::{print_findings, print_install_report, print_update_report};
use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME};
use crate::crates::read_locks;
use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles};
use crate::lockfile::direct_dependency_names;
use crate::shadow_step::{Prepared, now_millis, prepare};
use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};
use crate::update_plan::{UpdatePlan, plan_update};

/// The lockfile text that stands in for a missing lockfile while a full update creates it.
const EMPTY_LOCK: &str = "version = 4\n";

/// What:
/// `pub fn run_update_command(arguments: &[OsString]) -> Option<i32>` runs the command and returns its exit code, or `None`
/// when it is not in a catalog repository (the caller then runs cargo itself).
/// Why:
/// Every outcome prints to standard error, as cargo's own `update` does, so a pipeline reading standard output sees nothing
/// new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runUpdateCommand(args: string[]): number | undefined
/// ```
pub fn run_update_command(arguments: &[OsString]) -> Option<i32> {
    let prepared: Prepared = match prepare(arguments) {
        Ok(Some(found)) => found,
        Ok(None) => return None,
        Err(message) => {
            eprintln!("{message}");
            return Some(INSTALL_FAILED_EXIT_CODE);
        }
    };
    let crate_dir: String = prepared.crate_dir.clone()?;
    let planned = match plan_update(arguments) {
        Ok(found) => found,
        Err(message) => {
            eprintln!("cargo-shadow: {message}");
            return Some(INSTALL_FAILED_EXIT_CODE);
        }
    };
    let crate_dirs: Vec<String> = vec![crate_dir.clone()];
    let root: &Path = &prepared.catalog_root;
    let lock_exists: bool = root.join(&crate_dir).join(LOCKFILE_NAME).is_file();
    let mut sink = std::io::stderr();
    match planned.plan {
        UpdatePlan::FrozenCheck | UpdatePlan::Install { .. } if lock_exists => {
            let (frozen, dry_run) = match planned.plan {
                UpdatePlan::Install { dry_run } => (prepared.frozen && !dry_run, dry_run),
                _ => (true, false),
            };
            let outcome = install_rust_lockfiles(&InstallRequest {
                root,
                crate_dirs: &crate_dirs,
                frozen,
                offline: prepared.offline,
                dry_run,
                now: now_millis(),
                settings: &prepared.document.settings,
                runner: &prepared.runner,
                registry: &prepared.registry,
            });
            return Some(report_install(outcome, dry_run, &mut sink));
        }
        UpdatePlan::Targeted { dry_run, cargo_args, names, precise } => {
            if precise && let Err(message) = check_direct(root, &crate_dir, &names) {
                eprintln!("cargo-shadow: {message}");
                return Some(INSTALL_FAILED_EXIT_CODE);
            }
            return Some(run_update(&prepared, &crate_dirs, dry_run, &UpdateKind::Targeted { cargo_args: &cargo_args }, &planned.forward, &mut sink));
        }
        UpdatePlan::Full { dry_run } => {
            return Some(run_full_update(&prepared, &crate_dirs, dry_run, &planned.forward, &mut sink));
        }
        UpdatePlan::Install { dry_run } => {
            // A missing lockfile: pnpm install --lockfile-only creates it, and here the full update does, under the policy.
            return Some(run_full_update(&prepared, &crate_dirs, dry_run, &planned.forward, &mut sink));
        }
        UpdatePlan::FrozenCheck => {
            eprintln!("cargo-shadow: {crate_dir} has no {LOCKFILE_NAME} to assert about");
            return Some(INSTALL_FAILED_EXIT_CODE);
        }
    }
}

/// pnpm's rule for `pnpm update <name>@<version>`: the name must be a direct dependency.
fn check_direct(root: &Path, crate_dir: &str, names: &[String]) -> Result<(), String> {
    let snapshots = read_locks(root, &[crate_dir.to_string()]).map_err(|error| { return error.to_string(); })?;
    let direct = direct_dependency_names(snapshots.first().map(|snapshot| { return snapshot.source.as_str(); }).unwrap_or(""))?;
    for name in names {
        if !direct.contains(name) {
            return Err(format!(
                "{name} is not a direct dependency of {crate_dir}, so --precise cannot pin it (pnpm: ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP, which names overrides for indirect dependencies; use [patch] or a direct requirement in Cargo.toml)"
            ));
        }
    }
    return Ok(());
}

/// Prints an install outcome and returns the exit code.
fn report_install(outcome: Result<InstallReport, crate::install::InstallError>, dry_run: bool, sink: &mut dyn Write) -> i32 {
    return match outcome {
        Ok(report) => {
            print_install_report(&report, dry_run, sink);
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

/// The full update for one crate root; a missing lockfile is stood in for while it is created and removed again on a refusal.
fn run_full_update(prepared: &Prepared, crate_dirs: &[String], dry_run: bool, forward: &[String], sink: &mut dyn Write) -> i32 {
    let lock_path = prepared.catalog_root.join(&crate_dirs[0]).join(LOCKFILE_NAME);
    let created_placeholder: bool = !lock_path.is_file() && std::fs::write(&lock_path, EMPTY_LOCK).is_ok();
    let code: i32 = run_update(prepared, crate_dirs, dry_run, &UpdateKind::Full, forward, sink);
    if created_placeholder && code != 0 {
        let _ = std::fs::remove_file(&lock_path);
    }
    return code;
}

/// Runs the update and prints its report.
fn run_update(prepared: &Prepared, crate_dirs: &[String], dry_run: bool, kind: &UpdateKind, forward: &[String], sink: &mut dyn Write) -> i32 {
    let catalog_path = prepared.catalog_root.join(CATALOG_MARKER);
    let outcome = update_rust_lockfiles(
        &UpdateRequest {
            root: &prepared.catalog_root,
            crate_dirs,
            dry_run,
            now: now_millis(),
            settings: &prepared.document.settings,
            catalog_path: &catalog_path,
            runner: &prepared.runner,
            registry: &prepared.registry,
            forward,
        },
        kind,
    );
    return match outcome {
        Ok(report) => {
            let finished: UpdateReport = report;
            print_update_report(&finished, dry_run, sink);
            if finished.problems.is_empty() {
                0
            } else {
                print_findings("Update refused; every Cargo.lock was restored", &finished.problems);
                INSTALL_FAILED_EXIT_CODE
            }
        }
        Err(error) => {
            eprintln!("{error}");
            INSTALL_FAILED_EXIT_CODE
        }
    };
}
