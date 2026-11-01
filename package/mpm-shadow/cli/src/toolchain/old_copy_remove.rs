//! What:
//! The two removals the cleanup of old copies ends with:
//! an old copy in the shared cargo bin directory (and its cargo record),
//! and one older version directory of a managed copy.
//! Why:
//! Both are the last step after the links moved,
//! and both treat "already gone" as done:
//! two `install` runs (one per scope) can reach the same copy,
//! and the one that lost the race has nothing left to fail about.
//! The crate name that reaches `cargo uninstall` comes from `.crates.toml`,
//! a file this program did not write,
//! so it is read again at removal time,
//! must still be one this repository publishes,
//! and follows `--` so cargo never reads it as an option.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old-copy-remove.ts
//! export function removeCopy(ctx: Context, plan: OldCopyPlan): string  // throws
//! export function removeVersion(plan: LoserPlan, directory: string, version: string): void  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;

/// Imports what this module uses from `super::cargo_record`.
use super::cargo_record::{RecordedInstall, install_naming_bin, read_recorded_installs};
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `cargo_home_dir` from `super::layout`.
use super::layout::cargo_home_dir;
/// Imports `LoserPlan` from `super::managed_losers`.
use super::managed_losers::LoserPlan;
/// Imports `OldCopyPlan` from `super::old_copies`.
use super::old_copies::OldCopyPlan;
/// Imports `remove_downloads` from `super::published_archive`.
use super::published_archive::remove_downloads;

/// Whether nothing exists at `path` any more (not even a dangling link).
fn is_gone(path: &Path) -> bool {
    return match fs::symlink_metadata(path) {
        Ok(_) => false,
        Err(error) => error.kind() == ErrorKind::NotFound,
    };
}

/// What:
/// Asks cargo to uninstall the one bin of the crate that `.crates.toml` names for it,
/// and says what happened to the record.
/// Why:
/// `cargo uninstall` updates both of cargo's records and removes only the bin named,
/// so a crate with other bins stays recorded for them.
/// A failure after which the copy and its record are both gone means another run finished first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uninstallBin(ctx: Context, plan: OldCopyPlan, cargoHome: string, install: RecordedInstall): string  // throws
/// ```
fn uninstall_bin(ctx: &Context, plan: &OldCopyPlan, cargo_home: &Path, install: &RecordedInstall) -> Result<String, String> {
    let home_text: String = cargo_home.to_string_lossy().to_string();
    let request = ctx.request("cargo", &["uninstall", "--root", &home_text, "--bin", &plan.bin, "--", &install.crate_name]);
    let outcome = ctx.run(&request);
    if outcome.succeeded() {
        return Ok(format!("`cargo uninstall` removed it from cargo's record of {} {}", install.crate_name, install.version));
    }
    if is_gone(&plan.path) && install_naming_bin(&read_recorded_installs(cargo_home), &plan.bin).is_none() {
        return Ok("another run had already removed it and its record".to_string());
    }
    return Err(format!(
        "`{}` failed, so the copy stays (its links already moved); fix what cargo reports and run `mpm-shadow install` again: {}",
        request.line(),
        outcome.failure_text()
    ));
}

/// What:
/// Removes the old copy:
/// `cargo uninstall` for the bin when cargo's record names it,
/// then the file itself when it is still there.
/// Returns what happened to the record,
/// as text for the report.
/// Why:
/// It decides from `.crates.toml` alone (measured 2026-11-01),
/// so a bin that file does not name is removed directly.
/// A record that now attributes the bin to a crate this repository does not publish stops the removal:
/// the planning skipped such a bin,
/// so the file changed since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeCopy(ctx: Context, plan: OldCopyPlan): string  // throws
/// ```
pub fn remove_copy(ctx: &Context, plan: &OldCopyPlan) -> Result<String, String> {
    let cargo_home: PathBuf = cargo_home_dir(ctx.env, &ctx.scope.home);
    let installs: Vec<RecordedInstall> = read_recorded_installs(&cargo_home);
    let mut note: String = "cargo has no record of it".to_string();
    let mut cargo_ran: bool = false;
    if let Some(install) = install_naming_bin(&installs, &plan.bin) {
        if !install.crate_name.starts_with(OWN_CRATE_PREFIX) {
            return Err(format!(
                "cargo's record now attributes {} to {}, a crate this repository does not publish, so the copy stays",
                plan.bin, install.crate_name
            ));
        }
        note = uninstall_bin(ctx, plan, &cargo_home, install)?;
        cargo_ran = true;
    }
    match fs::remove_file(&plan.path) {
        Ok(()) => {}
        // Cargo removes the file itself, so a file that is gone after `cargo uninstall` is the normal case.
        Err(error) if error.kind() == ErrorKind::NotFound && cargo_ran => {}
        Err(error) if error.kind() == ErrorKind::NotFound => note = "another run had already removed the file".to_string(),
        Err(error) => return Err(format!("cannot remove the file: {error}")),
    }
    return Ok(note);
}

/// What:
/// Removes one older version's directory
/// (and,
/// for a release archive,
/// its downloads).
/// Why:
/// A directory that is already gone is done:
/// another run removed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeVersion(plan: LoserPlan, directory: string, version: string): void  // throws
/// ```
pub fn remove_version(plan: &LoserPlan, directory: &Path, version: &str) -> Result<(), String> {
    match fs::remove_dir_all(directory) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot remove {}: {error}", directory.display())),
    }
    if plan.archive
        && let Err(error) = remove_downloads(&plan.root, &plan.tool_id, version)
    {
        return Err(error.message);
    }
    return Ok(());
}
