//! What:
//! The install step, the counterpart of `pnpm install`: makes each listed crate root's `Cargo.lock` satisfy its manifests,
//! moving as little as the manifests force.
//! Why:
//! `mise run sync:files` renders catalog floors and path-crate versions into the manifests and never touches a lockfile, so a
//! crate whose lockfile held an older release is left with a lockfile its own manifest refuses; any plain cargo run then
//! rewrites it through cargo's own resolver, past the trust policy. pnpm repairs the same situation with `pnpm install`,
//! and follows these lockfile rules (https://pnpm.io/settings/build, https://pnpm.io/settings/store, read 2026-11-03):
//!
//! - Outside CI a stale lockfile is updated (`--no-frozen-lockfile`).
//! - In CI, or with `--frozen-lockfile`, a stale lockfile is a failure that names the roots and writes nothing.
//! - `--dry-run` reports what an install would update and writes nothing.
//! - A root that still satisfies its manifests keeps its lockfile byte for byte; in a stale root the dependencies whose
//!   requirements still hold keep their locked versions.
//!
//! Per crate root, offline: `cargo update --workspace --locked --offline` asks whether the lockfile already satisfies the
//! manifests (reads the index only, writes nothing); a refusal means stale; `cargo update --workspace --offline` re-locks the
//! workspace members and what their changed requirements force; `cargo metadata --locked --offline` confirms the result. A
//! non-frozen install is one transaction: every lockfile is snapshotted first, and any failure or refusal restores all of
//! them. A registry version that no lockfile held before goes through the release-age and trust checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install.ts
//! export async function installRustLockfiles({ root, frozenLockfile, dryRun, now, runCargo, registry }): Promise<InstallReport>
//! ```

use std::fmt;
use std::path::Path;

use monochromatic_dependency_catalog::Settings;

use crate::cargo_run::{CargoOutcome, CargoRunner};
use crate::checks::{CheckOutcome, check_release_age, check_trust};
use crate::crates::{LockSnapshot, read_locks};
use crate::journal::{begin, finish, process_exists, recover, restore_if_unchanged};
use crate::http::RegistryError;
use crate::lockfile::{LockedVersion, added_versions};
use crate::registry::Registry;
use crate::verify::{verify_added_since_base, verify_locked_entries};

/// Asks whether the lockfile satisfies the manifests: exits 0 without writing when it does.
pub const CHECK_ARGS: [&str; 4] = ["update", "--workspace", "--locked", "--offline"];

/// Re-locks the workspace members and whatever their changed requirements force, keeping every other locked release.
pub const UPDATE_ARGS: [&str; 3] = ["update", "--workspace", "--offline"];

/// The consistency check run after the update.
pub const VERIFY_ARGS: [&str; 5] = ["metadata", "--locked", "--offline", "--format-version", "1"];

/// What:
/// `pub fn arguments_for(args: &[&str], offline: bool) -> Vec<String>` is an argument list as owned text, without `--offline`
/// when the run may use the network.
/// Why:
/// A fresh CI runner has no cached registry index, so an offline check cannot read it; a developer machine has one, and the
/// offline form keeps the check from touching the network at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const argumentsFor = (args: string[], offline: boolean) => offline ? args : args.filter((a) => a !== '--offline');
/// ```
pub fn arguments_for(args: &[&str], offline: bool) -> Vec<String> {
    return args.iter().filter(|argument| { return offline || **argument != "--offline"; }).map(|argument| { return argument.to_string(); }).collect();
}

/// The values of the `CI` variable that mean this is not a continuous integration run.
const NOT_CI_VALUES: [&str; 3] = ["", "0", "false"];

/// Whether the environment is a continuous integration run, where `pnpm install` defaults `--frozen-lockfile` to true.
pub fn is_continuous_integration(ci_value: &str) -> bool {
    return !NOT_CI_VALUES.contains(&ci_value.to_lowercase().as_str());
}

/// The outcome of an install run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallReport {
    /// Every crate root examined.
    pub crate_dirs: Vec<String>,
    /// Roots whose lockfile did not satisfy its manifests (updated, or only reported under `--dry-run`).
    pub stale: Vec<String>,
    /// Registry versions the install newly locked (kept or restored).
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
    Stale {
        /// The repo-relative crate roots whose lockfile is stale.
        stale_dirs: Vec<String>,
    },
    /// Cargo could not bring a crate's lockfile in line.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup failed.
    Registry {
        /// What failed.
        message: String,
    },
    /// A file could not be read or written, or cargo could not be started.
    Io {
        /// What failed.
        message: String,
    },
}

impl fmt::Display for InstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallError::Stale { stale_dirs } => {
                writeln!(
                    formatter,
                    "Frozen install failed: {} Cargo.lock file(s) are present but need an update to satisfy their manifests.",
                    stale_dirs.len()
                )?;
                for dir in stale_dirs {
                    writeln!(formatter, "- {dir}")?;
                }
                writeln!(formatter, "Nothing was written. Frozen is the default in CI, as in pnpm; `--locked` or `--frozen` on the cargo command asks for it too.")?;
                return write!(
                    formatter,
                    "To update them, run `cargo check` (or `cargo-shadow install --no-frozen-lockfile`) outside CI and commit the Cargo.lock changes."
                );
            }
            InstallError::Cargo { crate_dir, command, output } => {
                writeln!(formatter, "`{command}` failed in {crate_dir}; every lockfile was restored.")?;
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                writeln!(formatter, "The install runs cargo offline, so it can pick only releases the local registry index and cache already hold.")?;
                writeln!(formatter, "When cargo says an index entry or package is missing, fetch it once with network access (`cargo fetch` in the crate)")?;
                writeln!(formatter, "and run the install again, or run `cargo update`, which resolves online under the release-age and trust policy.")?;
                return write!(formatter, "When cargo reports conflicting requirements, fix the manifest or the rust-catalog.toml floor that it names.");
            }
            InstallError::Registry { message } => {
                return write!(formatter, "A registry lookup failed; every lockfile was restored: {message}");
            }
            InstallError::Io { message } => {
                return write!(formatter, "{message}");
            }
        }
    }
}

impl From<RegistryError> for InstallError {
    fn from(error: RegistryError) -> InstallError {
        return InstallError::Registry { message: error.message };
    }
}

/// What an install run needs.
pub struct InstallRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to examine, repo-relative.
    pub crate_dirs: &'a [String],
    /// Fail on a stale lockfile instead of updating it (pnpm's `--frozen-lockfile`, the CI default).
    pub frozen: bool,
    /// Keep every cargo call offline; false in CI, where no registry index is cached yet.
    pub offline: bool,
    /// Report what would be updated, then restore every lockfile.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Reads one crate's lockfile text at the base revision (empty when absent there); a frozen install under `trustLockfile`
    /// checks only the versions this reader's lockfiles did not hold.
    pub lock_base: Option<&'a dyn Fn(&str) -> String>,
}

/// Runs cargo in a crate root; a start failure is an `Io` error.
fn run_cargo(request: &InstallRequest, crate_dir: &str, args: &[&str]) -> Result<CargoOutcome, InstallError> {
    return request
        .runner
        .run(&request.root.join(crate_dir), &arguments_for(args, request.offline))
        .map_err(|message| { return InstallError::Io { message }; });
}

/// Whether a crate root's lockfile no longer satisfies its manifests, without writing anything.
pub fn is_stale(request: &InstallRequest, crate_dir: &str) -> Result<bool, InstallError> {
    return Ok(!run_cargo(request, crate_dir, &CHECK_ARGS)?.ok);
}

/// Runs one cargo command in a crate root and fails with the crate named when it exits non-zero.
fn run_or_fail(request: &InstallRequest, crate_dir: &str, args: &[&str]) -> Result<(), InstallError> {
    let outcome: CargoOutcome = run_cargo(request, crate_dir, args)?;
    if !outcome.ok {
        return Err(InstallError::Cargo {
            crate_dir: crate_dir.to_string(),
            command: format!("cargo {}", arguments_for(args, request.offline).join(" ")),
            output: outcome.output,
        });
    }
    return Ok(());
}

/// The lockfile texts of snapshots.
fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
    return snapshots.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
}

/// The crate roots whose lockfile is stale, found with read-only checks.
pub fn stale_roots(request: &InstallRequest) -> Result<Vec<String>, InstallError> {
    let mut stale: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        if is_stale(request, crate_dir)? {
            stale.push(crate_dir.clone());
        }
    }
    return Ok(stale);
}

/// The transaction body: update the stale roots, then check the new versions; restoring on refusal is the caller's.
fn update_stale_roots(request: &InstallRequest, before: &[LockSnapshot], produced: &mut Option<Vec<LockSnapshot>>) -> Result<InstallReport, InstallError> {
    let mut stale: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        if !is_stale(request, crate_dir)? {
            eprintln!("cargo-shadow: {crate_dir}: lockfile already satisfies its manifests");
            continue;
        }
        eprintln!("cargo-shadow: {crate_dir}: lockfile is stale, updating the workspace offline");
        run_or_fail(request, crate_dir, &UPDATE_ARGS)?;
        run_or_fail(request, crate_dir, &VERIFY_ARGS)?;
        stale.push(crate_dir.clone());
    }
    let after: Vec<LockSnapshot> =
        read_locks(request.root, request.crate_dirs).map_err(|error| { return InstallError::Io { message: error.to_string() }; })?;
    // What cargo produced, taken before the slow registry checks: a restore later replaces a lockfile only if it still holds this.
    *produced = Some(after.clone());
    let added: Vec<LockedVersion> = added_versions(&sources_of(before), &sources_of(&after))
        .map_err(|message| { return InstallError::Io { message: format!("a Cargo.lock is not valid TOML: {message}") }; })?;
    let age: CheckOutcome = if added.is_empty() {
        CheckOutcome::default()
    } else {
        check_release_age(&added, request.settings, request.registry, request.now)?
    };
    let trust: Vec<String> = if added.is_empty() { Vec::new() } else { check_trust(&added, request.settings, request.registry, request.now)? };
    let mut problems: Vec<String> = age.problems;
    problems.extend(trust);
    let keep: bool = problems.is_empty() && !request.dry_run;
    return Ok(InstallReport {
        crate_dirs: request.crate_dirs.to_vec(),
        stale,
        added,
        problems,
        warnings: age.warnings,
        written: keep,
    });
}

/// The policy findings for the registry versions `after` holds that `before` did not; used to judge a killed run's leftovers.
pub fn policy_findings(settings: &Settings, registry: &dyn Registry, now: i64, before: &[LockSnapshot], after: &[LockSnapshot]) -> Vec<String> {
    let added: Vec<LockedVersion> = match added_versions(&sources_of(before), &sources_of(after)) {
        Ok(found) => found,
        Err(message) => return vec![format!("a Cargo.lock is not valid TOML: {message}")],
    };
    if added.is_empty() {
        return Vec::new();
    }
    let mut findings: Vec<String> = Vec::new();
    match check_release_age(&added, settings, registry, now) {
        Ok(outcome) => findings.extend(outcome.problems),
        Err(error) => findings.push(error.message),
    }
    match check_trust(&added, settings, registry, now) {
        Ok(problems) => findings.extend(problems),
        Err(error) => findings.push(error.message),
    }
    return findings;
}

/// What:
/// `pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError>` runs the install over the
/// listed crate roots.
/// Why:
/// A frozen install only checks and fails naming the stale roots. A non-frozen install first finishes any run that was killed
/// (`journal.rs`), then snapshots every lockfile, updates the stale ones, checks the newly locked versions, and puts the
/// snapshots back on a failure, a refusal or a dry run, but only into lockfiles that still hold what this run produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function installRustLockfiles(args): Promise<InstallReport>  // throws StaleLockfilesError, CargoInstallError
/// ```
pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError> {
    if request.frozen {
        let stale: Vec<String> = stale_roots(request)?;
        if !stale.is_empty() {
            eprintln!("cargo-shadow: frozen install: {} stale lockfiles, nothing written", stale.len());
            return Err(InstallError::Stale { stale_dirs: stale });
        }
        let locks: Vec<LockSnapshot> =
            read_locks(request.root, request.crate_dirs).map_err(|error| { return InstallError::Io { message: error.to_string() }; })?;
        // `trustLockfile = true` (the owner's decision of 2026-10-29): committed lockfiles are trusted, so only the versions no
        // lockfile held at the base revision are checked. With it false, pnpm's default: every lockfile entry is re-validated
        // against the release-age and trust policy, and a lockfile that passed before is remembered by its digest (verify.rs).
        let problems: Vec<String> = if request.settings.trust_lockfile {
            let read_base = request.lock_base.ok_or_else(|| {
                return InstallError::Io {
                    message: "trustLockfile is true in rust-catalog.toml, but no base-revision lockfile reader was given, so nothing can be trusted".to_string(),
                };
            })?;
            verify_added_since_base(&locks, read_base, request.settings, request.registry, request.now)
                .map_err(|message| { return InstallError::Registry { message }; })?
        } else {
            verify_locked_entries(request.root, &locks, request.settings, request.registry, request.now)
                .map_err(|message| { return InstallError::Registry { message }; })?
        };
        return Ok(InstallReport { crate_dirs: request.crate_dirs.to_vec(), problems, ..InstallReport::default() });
    }
    let recovered: Vec<String> = recover(request.root, &process_exists, &|old, live| { return policy_findings(request.settings, request.registry, request.now, old, live); });
    for message in &recovered {
        eprintln!("cargo-shadow: warning: {message}");
    }
    let before: Vec<LockSnapshot> =
        read_locks(request.root, request.crate_dirs).map_err(|error| { return InstallError::Io { message: error.to_string() }; })?;
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    let journaled: bool = begin(request.root, &before).is_ok();
    let mut produced: Option<Vec<LockSnapshot>> = None;
    let outcome: Result<InstallReport, InstallError> = update_stale_roots(request, &before, &mut produced);
    let keep: bool = matches!(&outcome, Ok(report) if report.written);
    let mut restore_warnings: Vec<String> = Vec::new();
    if !keep {
        eprintln!("cargo-shadow: restoring every lockfile that this run changed");
        let state: Vec<LockSnapshot> = match produced {
            Some(found) => found,
            None => read_locks(request.root, request.crate_dirs).unwrap_or_default(),
        };
        restore_warnings = restore_if_unchanged(request.root, &before, &state).map_err(|error| { return InstallError::Io { message: error.to_string() }; })?;
        for warning in &restore_warnings {
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    if journaled {
        finish(request.root);
    }
    return outcome.map(|mut report| {
        report.warnings.extend(recovered);
        report.warnings.extend(restore_warnings);
        return report;
    });
}
