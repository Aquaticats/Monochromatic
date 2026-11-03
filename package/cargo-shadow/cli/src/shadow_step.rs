//! What:
//! The glue between a cargo command line and the steps the shadow runs around it: where cargo will work, the install step
//! before it, and the catalog warning after it.
//! Why:
//! `shim.rs` decides whether to run cargo; this module decides what happens around the run, so each can be tested alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-step.ts
//! export function startDir(args: string[], cwd: string): string
//! export function beforeCargo(args: string[]): string | undefined  // a failure message
//! ```

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cargo_run::RealCargo;
use crate::catalog_warning::warn_about_manifest;
use crate::constants::MANIFEST_NAME;
use crate::http::UreqFetcher;
use crate::install::is_continuous_integration;
use crate::install_step::{StepOutcome, StepRequest, run_install_step};
use crate::options::{asserts_lock, change_dir, manifest_path};
use crate::real_binary::find_real_cargo;
use crate::registry::CratesIo;
use crate::toolchain::cargo_toolchain;
use crate::workspace::find_catalog_root;

/// What:
/// `pub fn start_dir(arguments: &[OsString], current: &Path) -> PathBuf` is where cargo will look for its manifest: the
/// directory of `--manifest-path` when given, else the working directory moved by `-C`.
/// Why:
/// The install step runs for the workspace cargo will use, and `cargo check --manifest-path other/Cargo.toml` uses another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startDir(args: string[], cwd: string): string
/// ```
pub fn start_dir(arguments: &[OsString], current: &Path) -> PathBuf {
    let base: PathBuf = match change_dir(arguments) {
        Some(directory) => current.join(directory),
        None => current.to_path_buf(),
    };
    if let Some(manifest) = manifest_path(arguments) {
        let full: PathBuf = base.join(manifest);
        if let Some(parent) = full.parent() {
            return parent.to_path_buf();
        }
    }
    return base;
}

/// Milliseconds since 1970 now.
fn now_millis() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| { return elapsed.as_millis() as i64; }).unwrap_or(0);
}

/// What:
/// `pub fn before_cargo(arguments: &[OsString]) -> Option<String>` runs the install step for the command line and returns the
/// message to show when the command must not run.
/// Why:
/// `None` means run cargo. The real cargo for the step is the first `cargo` on PATH that is not this binary, run under the
/// toolchain the repository pins, so every lockfile the step writes has one format whatever toolchain the user's command
/// selects. In CI, or when the user's own command carries `--locked` or `--frozen`, a stale lock is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beforeCargo(args: string[]): string | undefined
/// ```
pub fn before_cargo(arguments: &[OsString]) -> Option<String> {
    let current: PathBuf = std::env::current_dir().ok()?;
    let start: PathBuf = start_dir(arguments, &current);
    let catalog_root: PathBuf = find_catalog_root(&start)?;
    let program: PathBuf = match find_real_cargo() {
        Some(found) => found,
        None => return Some("cargo-shadow: no real cargo found on PATH for the install step".to_string()),
    };
    let explicit: String = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_default();
    let (toolchain, warning) = match cargo_toolchain(&catalog_root, &explicit) {
        Ok(chosen) => chosen,
        Err(message) => return Some(format!("cargo-shadow: toolchain.lock.json is not readable: {message}")),
    };
    if let Some(text) = warning {
        eprintln!("cargo-shadow: warning: {text}");
    }
    let runner: RealCargo = RealCargo { program, toolchain };
    let registry: CratesIo = CratesIo::with_defaults(&catalog_root, Box::new(UreqFetcher::new()));
    let frozen: bool = is_continuous_integration(&std::env::var("CI").unwrap_or_default()) || asserts_lock(arguments);
    let outcome: StepOutcome =
        run_install_step(&StepRequest { start_dir: &start, frozen, now: now_millis(), runner: &runner, registry: &registry });
    match outcome {
        StepOutcome::Failed(message) => return Some(message),
        StepOutcome::Updated { crate_dir, added } => {
            eprintln!(
                "cargo-shadow: {crate_dir}/Cargo.lock did not satisfy its manifests and was updated offline ({} newly locked registry versions passed the policy)",
                added.len()
            );
            return None;
        }
        StepOutcome::Skipped(_) | StepOutcome::FastPath | StepOutcome::Passed => return None,
    }
}

/// Writes the catalog warning for the manifest cargo worked on: the one named by `--manifest-path`, else the nearest
/// `Cargo.toml` above the start directory.
pub fn warn_after_cargo(arguments: &[OsString], sink: &mut dyn Write) {
    let current: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(_) => return,
    };
    let start: PathBuf = start_dir(arguments, &current);
    if find_catalog_root(&start).is_none() {
        return;
    }
    for directory in start.ancestors() {
        let manifest: PathBuf = directory.join(MANIFEST_NAME);
        if manifest.is_file() {
            warn_about_manifest(&manifest, sink);
            return;
        }
    }
}
