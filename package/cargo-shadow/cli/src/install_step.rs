//! What:
//! The shadow's install step: before a cargo command that resolves dependencies, make the `Cargo.lock` of the Cargo workspace
//! that holds the working directory satisfy its manifests, under the catalog's trust policy.
//! Why:
//! This is pnpm's `verifyDepsBeforeRun: install` default ("Automatically runs install if `node_modules` is not up to
//! date", https://pnpm.io/settings/build, read 2026-11-03) for lockfiles: a stale lock is brought in line by the same code as
//! the install command (`install.rs`), in CI or under the user's own `--locked` or `--frozen` it fails naming the stale lock
//! instead (pnpm's `frozenLockfile`), and the user's cargo command is then run unchanged by the caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-step.ts
//! export function runInstallStep(request: StepRequest): StepOutcome
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

use crate::cargo_run::CargoRunner;
use crate::constants::{CATALOG_MARKER, LOCKFILE_NAME};
use crate::install::{InstallError, InstallReport, InstallRequest, VERIFY_ARGS, install_rust_lockfiles, owned_args};
use crate::lockfile::LockedVersion;
use crate::registry::Registry;
use crate::stamp::{is_fresh, manifests_from_metadata, record, stamp_file};
use crate::workspace::{find_cargo_root, find_catalog_root};

/// What the install step did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepOutcome {
    /// Nothing to do here, with the reason.
    Skipped(String),
    /// The recorded fingerprints still match: no cargo was started.
    FastPath,
    /// The authoritative check found the lockfile satisfied.
    Passed,
    /// The lockfile was stale and was updated.
    Updated {
        /// The repo-relative crate root.
        crate_dir: String,
        /// The registry versions newly locked.
        added: Vec<LockedVersion>,
    },
    /// The step refused or failed, with the message to show.
    Failed(String),
}

/// What the step needs.
pub struct StepRequest<'a> {
    /// Where cargo will work.
    pub start_dir: &'a Path,
    /// Fail on a stale lockfile instead of updating it.
    pub frozen: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
}

/// The crate root relative to the repository, with forward slashes; `None` when it is not inside it.
fn relative_crate_dir(catalog_root: &Path, cargo_root: &Path) -> Option<String> {
    let relative: &Path = cargo_root.strip_prefix(catalog_root).ok()?;
    let text: String = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        return None;
    }
    return Some(text);
}

/// Reads and validates `rust-catalog.toml`.
fn read_catalog(catalog_root: &Path) -> Result<CatalogDocument, String> {
    let path: PathBuf = catalog_root.join(CATALOG_MARKER);
    let text: String = fs::read_to_string(&path).map_err(|error| { return format!("{}: {error}", path.display()); })?;
    return parse_catalog(&text, CATALOG_MARKER).map_err(|error| { return error.to_string(); });
}

/// Records the stamp after a pass: one `cargo metadata` call lists the manifests the lockfile was resolved from.
fn write_stamp(request: &StepRequest, catalog_root: &Path, crate_dir: &str) {
    let cargo_root: PathBuf = catalog_root.join(crate_dir);
    let outcome = match request.runner.run(&cargo_root, &owned_args(&VERIFY_ARGS)) {
        Ok(found) => found,
        Err(_) => return,
    };
    if !outcome.ok {
        return;
    }
    let manifests: Vec<PathBuf> = manifests_from_metadata(&outcome.stdout);
    if manifests.is_empty() {
        return;
    }
    record(&stamp_file(catalog_root, crate_dir), &cargo_root.join(LOCKFILE_NAME), &manifests);
}

/// What:
/// `pub fn run_install_step(request: &StepRequest) -> StepOutcome` runs the step for the workspace of `request.start_dir`.
/// Why:
/// Outside a catalog repository, or in a workspace without a lockfile, the step does nothing. Otherwise a matching stamp
/// answers at once; without one, the install over this one crate root answers, and a pass or a kept update records a stamp.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runInstallStep(request): StepOutcome
/// ```
pub fn run_install_step(request: &StepRequest) -> StepOutcome {
    let catalog_root: PathBuf = match find_catalog_root(request.start_dir) {
        Some(found) => found,
        None => return StepOutcome::Skipped("no rust-catalog.toml above the working directory".to_string()),
    };
    let cargo_root: PathBuf = match find_cargo_root(request.start_dir) {
        Some(found) => found,
        None => return StepOutcome::Skipped("no Cargo.toml above the working directory".to_string()),
    };
    let crate_dir: String = match relative_crate_dir(&catalog_root, &cargo_root) {
        Some(found) => found,
        None => return StepOutcome::Skipped("the Cargo workspace is not below the catalog repository".to_string()),
    };
    let lock: PathBuf = cargo_root.join(LOCKFILE_NAME);
    if !lock.is_file() {
        return StepOutcome::Skipped(format!("{crate_dir} has no {LOCKFILE_NAME} yet"));
    }
    let stamp: PathBuf = stamp_file(&catalog_root, &crate_dir);
    if is_fresh(&stamp, &lock) {
        return StepOutcome::FastPath;
    }
    let document: CatalogDocument = match read_catalog(&catalog_root) {
        Ok(found) => found,
        Err(message) => return StepOutcome::Failed(message),
    };
    let crate_dirs: Vec<String> = vec![crate_dir.clone()];
    let report: Result<InstallReport, InstallError> = install_rust_lockfiles(&InstallRequest {
        root: &catalog_root,
        crate_dirs: &crate_dirs,
        frozen: request.frozen,
        dry_run: false,
        now: request.now,
        settings: &document.settings,
        runner: request.runner,
        registry: request.registry,
    });
    let finished: InstallReport = match report {
        Ok(found) => found,
        Err(error) => return StepOutcome::Failed(error.to_string()),
    };
    if !finished.problems.is_empty() {
        let mut lines: Vec<String> = vec![format!(
            "Install refused; every Cargo.lock was restored: {} finding(s).",
            finished.problems.len()
        )];
        for problem in &finished.problems {
            lines.push(format!("- {problem}"));
        }
        lines.push("Settings live in rust-catalog.toml; see doc/decision/rust-dependency-catalog.md.".to_string());
        return StepOutcome::Failed(lines.join("\n"));
    }
    write_stamp(request, &catalog_root, &crate_dir);
    if finished.stale.is_empty() {
        return StepOutcome::Passed;
    }
    return StepOutcome::Updated { crate_dir, added: finished.added };
}
