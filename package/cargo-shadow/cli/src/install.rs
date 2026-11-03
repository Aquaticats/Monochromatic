//! What:
//! The install step, the counterpart of `pnpm install`: makes each listed crate root's `Cargo.lock` satisfy its manifests.
//! Why:
//! (stub for the red test run)

use std::path::Path;

use monochromatic_dependency_catalog::Settings;

use crate::cargo_run::CargoRunner;
use crate::lockfile::LockedVersion;
use crate::registry::Registry;

/// The outcome of an install run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallReport {
    /// Every crate root examined.
    pub crate_dirs: Vec<String>,
    /// Roots whose lockfile did not satisfy its manifests.
    pub stale: Vec<String>,
    /// Registry versions the install newly locked.
    pub added: Vec<LockedVersion>,
    /// Findings that refused the install.
    pub problems: Vec<String>,
    /// Findings reported without refusing.
    pub warnings: Vec<String>,
    /// Whether updated lockfiles were kept.
    pub written: bool,
}

/// Why an install did not complete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallError {
    /// A frozen install found stale lockfiles.
    Stale { stale_dirs: Vec<String> },
    /// Cargo could not bring a crate's lockfile in line.
    Cargo { crate_dir: String, command: String, output: String },
    /// A registry lookup failed.
    Registry { message: String },
    /// A file could not be read or written.
    Io { message: String },
}

/// What an install run needs.
pub struct InstallRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to examine, repo-relative.
    pub crate_dirs: &'a [String],
    /// Fail on a stale lockfile instead of updating it.
    pub frozen: bool,
    /// Report without keeping anything.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
}

/// Stub.
pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError> {
    return Ok(InstallReport { crate_dirs: request.crate_dirs.to_vec(), ..InstallReport::default() });
}
