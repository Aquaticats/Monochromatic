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

use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

use crate::cargo_run::RealCargo;
use crate::catalog_warning::warn_about_manifest;
use crate::constants::{CATALOG_MARKER, MANIFEST_NAME};
use crate::git_base::{base_from_environment, git_base_lock_reader};
use crate::http::UreqFetcher;
use crate::install::is_continuous_integration;
use crate::install_step::{StepOutcome, StepRequest, run_install_step};
use crate::options::{asserts_lock, change_dir, manifest_path};
use crate::real_binary::find_real_cargo;
use crate::registry::CratesIo;
use crate::toolchain::cargo_toolchain;
use crate::workspace::{find_cargo_root, find_catalog_root};

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
pub fn now_millis() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| { return elapsed.as_millis() as i64; }).unwrap_or(0);
}

/// What the steps around a cargo command need, found once per run.
pub struct Prepared {
    /// Where cargo will work.
    pub start: PathBuf,
    /// The repository root.
    pub catalog_root: PathBuf,
    /// The repo-relative crate root of the workspace, when it is below the repository.
    pub crate_dir: Option<String>,
    /// The validated catalog.
    pub document: CatalogDocument,
    /// The real cargo under the pinned toolchain.
    pub runner: RealCargo,
    /// crates.io.
    pub registry: CratesIo,
    /// Whether a stale lockfile is a failure: in CI, or the user's own command carries `--locked` or `--frozen`.
    pub frozen: bool,
    /// The base-revision lockfile reader, built when a frozen install under `trustLockfile` will need it.
    pub lock_base: Option<Box<dyn Fn(&str) -> String>>,
    /// Whether the install step keeps cargo offline: always, except in CI, where no registry index is cached yet.
    pub offline: bool,
}

/// What:
/// `pub fn prepare(arguments: &[OsString]) -> Result<Option<Prepared>, String>` finds the repository, the workspace, the
/// catalog, the real cargo and the toolchain for a command line; `Ok(None)` means the command is not in a catalog repository.
/// Why:
/// The real cargo for the steps is the first `cargo` on PATH that is not this binary, run under the toolchain the repository
/// pins, so every lockfile the shadow writes has one format whatever toolchain the user's own command selects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function prepare(args: string[]): Prepared | undefined  // throws
/// ```
pub fn prepare(arguments: &[OsString]) -> Result<Option<Prepared>, String> {
    let current: PathBuf = std::env::current_dir().map_err(|error| { return format!("cargo-shadow: no working directory: {error}"); })?;
    let start: PathBuf = start_dir(arguments, &current);
    let catalog_root: PathBuf = match find_catalog_root(&start) {
        Some(found) => found,
        None => return Ok(None),
    };
    let program: PathBuf = find_real_cargo().ok_or_else(|| { return "cargo-shadow: no real cargo found on PATH for the install step".to_string(); })?;
    let explicit: String = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_default();
    let (toolchain, warning) = cargo_toolchain(&catalog_root, &explicit).map_err(|message| { return format!("cargo-shadow: toolchain.lock.json is not readable: {message}"); })?;
    if let Some(text) = warning {
        eprintln!("cargo-shadow: warning: {text}");
    }
    let catalog_text: String = std::fs::read_to_string(catalog_root.join(CATALOG_MARKER))
        .map_err(|error| { return format!("cargo-shadow: {CATALOG_MARKER}: {error}"); })?;
    let document: CatalogDocument = parse_catalog(&catalog_text, CATALOG_MARKER).map_err(|error| { return error.to_string(); })?;
    let in_ci: bool = is_continuous_integration(&std::env::var("CI").unwrap_or_default());
    let crate_dir: Option<String> = find_cargo_root(&start).and_then(|cargo_root| { return cargo_root.strip_prefix(&catalog_root).ok().map(|relative| { return relative.to_string_lossy().replace('\\', "/"); }); });
    let frozen: bool = in_ci || asserts_lock(arguments);
    let lock_base: Option<Box<dyn Fn(&str) -> String>> = lock_base_reader(&catalog_root, &document, frozen)?;
    return Ok(Some(Prepared {
        start,
        lock_base,
        runner: RealCargo { program, toolchain },
        registry: CratesIo::with_defaults(&catalog_root, Box::new(UreqFetcher::new()), Box::new(UreqFetcher::new())),
        catalog_root,
        crate_dir,
        document,
        frozen,
        offline: !in_ci,
    }));
}

/// What:
/// `pub fn lock_base_reader(catalog_root: &Path, document: &CatalogDocument, needed: bool) -> Result<Option<Box<dyn Fn(&str) -> String>>, String>`
/// builds the base-revision lockfile reader a frozen install needs under `trustLockfile`, and nothing otherwise.
/// Why:
/// The reader runs `git`, so it is built only when a frozen check will use it. The base comes from the environment
/// (`git_base.rs`), `HEAD` by default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lockBaseReader(root: string, settings: Settings, needed: boolean): ReadBaseLock | undefined
/// ```
pub fn lock_base_reader(catalog_root: &Path, document: &CatalogDocument, needed: bool) -> Result<Option<Box<dyn Fn(&str) -> String>>, String> {
    if !needed || !document.settings.trust_lockfile {
        return Ok(None);
    }
    return git_base_lock_reader(catalog_root, &base_from_environment()).map(Some);
}

/// What:
/// `pub fn before_cargo(arguments: &[OsString]) -> Option<String>` runs the install step for the command line and returns the
/// message to show when the command must not run.
/// Why:
/// `None` means run cargo. In CI, or when the user's own command carries `--locked` or `--frozen`, a stale lock is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beforeCargo(args: string[]): string | undefined
/// ```
pub fn before_cargo(arguments: &[OsString]) -> Option<String> {
    let prepared: Prepared = match prepare(arguments) {
        Ok(Some(found)) => found,
        Ok(None) => return None,
        Err(message) => return Some(message),
    };
    let outcome: StepOutcome = run_install_step(&StepRequest {
        start_dir: &prepared.start,
        frozen: prepared.frozen,
        offline: prepared.offline,
        now: now_millis(),
        runner: &prepared.runner,
        registry: &prepared.registry,
        lock_base: prepared.lock_base.as_deref(),
    });
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

/// What:
/// `pub fn with_locked(arguments: &[OsString]) -> Vec<OsString>` is the user's command line with `--locked` added before any
/// `--`, unless the user already passed `--locked` or `--frozen`.
/// Why:
/// This is the one place the shadow adds an option, and only after a failed or refused install step: pnpm runs the command
/// anyway when its install step fails, because `pnpm run` never rewrites the lockfile; cargo does rewrite a stale lock, so the
/// faithful form of "run it anyway" is to run it with `--locked`, which makes cargo stop with its own lock error where it would
/// have changed the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function withLocked(args: string[]): string[]
/// ```
pub fn with_locked(arguments: &[OsString]) -> Vec<OsString> {
    if asserts_lock(arguments) {
        return arguments.to_vec();
    }
    let mut result: Vec<OsString> = Vec::new();
    let mut added: bool = false;
    for argument in arguments {
        if !added && argument == "--" {
            result.push(OsString::from("--locked"));
            added = true;
        }
        result.push(argument.clone());
    }
    if !added {
        result.push(OsString::from("--locked"));
    }
    return result;
}
