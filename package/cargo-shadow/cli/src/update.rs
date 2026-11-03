//! What:
//! The catalog update, the counterpart of `pnpm update`: moves lockfiles to newer releases under the release-age and trust
//! policy in `rust-catalog.toml`, as one transaction.
//! Why:
//! pnpm's `update` is its own policy-aware command (https://pnpm.io/cli/update, read 2026-11-03): it moves dependencies
//! within their ranges and applies `minimumReleaseAge` and `trustPolicy` while it chooses. Cargo's resolver applies neither,
//! so the shadow answers `cargo update` with this procedure instead of passing it to cargo's resolver.
//!
//! Two kinds. The full kind re-resolves each crate root with
//! `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>`, which picks the newest releases
//! published before the cutoff (only `generate-lockfile` accepts `--publish-time`, re-probed 2026-10-29), then moves crates
//! the catalog exempts from the release age to their newest release. The targeted kind runs the user's `cargo update` command
//! (named packages, `--precise`, `--breaking`) as given. Both then collect the registry versions no lockfile held before,
//! set aside what the policy refuses, check what is left, and keep the new lockfiles only when nothing failed and it is not a
//! dry run; otherwise every lockfile is restored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update.ts
//! export async function updateRustLockfiles({ root, dryRun, now, runCargo, registry }): Promise<UpdateReport>
//! ```

use std::fmt;
use std::fs;
use std::path::Path;

use monochromatic_dependency_catalog::Settings;

use crate::cargo_run::{CargoOutcome, CargoRunner};
use crate::checks::{CheckOutcome, check_release_age, check_trust};
use crate::crates::{LockSnapshot, read_locks};
use crate::install::policy_findings;
use crate::journal::{begin, finish, process_exists, recover, restore_if_unchanged};
use crate::lockfile::{LockedVersion, added_versions, parse_registry_versions, registry_versions};
use crate::prune::prune_excludes;
use crate::registry::Registry;
use crate::selectors::{matches_pattern, parse_selector};
use crate::set_aside::{SetAsideRequest, set_aside_until_clean};
use crate::tighten::{Syncer, raise_floors, raised_names};
use crate::time_text::publish_time_cutoff;

/// Which procedure the update runs.
pub enum UpdateKind<'a> {
    /// Re-resolve every listed crate root under the release-age cutoff (`pnpm update` with no names).
    Full,
    /// Run the user's `cargo update` arguments as given (`pnpm update <name>[@<version>]`).
    Targeted {
        /// The complete cargo arguments, starting with `update`.
        cargo_args: &'a [String],
    },
}

/// What an update needs.
pub struct UpdateRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to update, repo-relative.
    pub crate_dirs: &'a [String],
    /// Report without keeping anything.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The catalog file, edited to prune stale excludes after a kept full update.
    pub catalog_path: &'a Path,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Cargo options forwarded to every cargo call (`--offline`, `--config`, `-Z`, `--color`, `-q`, `-v`).
    pub forward: &'a [String],
    /// Renders the catalog into the manifests after `--breaking` raised floors.
    pub syncer: &'a dyn Syncer,
    /// Prune exclude selectors that no locked version needs. Only an update that covers every lockfile of the repository may
    /// say that nothing needs a selector; an update of some workspaces must leave the lists alone.
    pub prune_catalog: bool,
}

/// The outcome of an update.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateReport {
    /// The crate roots updated.
    pub crate_dirs: Vec<String>,
    /// Registry versions newly locked (kept or restored).
    pub added: Vec<LockedVersion>,
    /// Findings that refused the update.
    pub problems: Vec<String>,
    /// Findings reported without refusing, including set-asides.
    pub warnings: Vec<String>,
    /// Roots resolved without the release-age cutoff (`minimumReleaseAgeStrict = false`).
    pub fell_back: Vec<String>,
    /// Whether the new lockfiles were kept.
    pub written: bool,
    /// Exclude selectors pruned from the catalog.
    pub pruned: Vec<String>,
}

/// Why an update did not complete; every lockfile was restored first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateError {
    /// Cargo could not resolve a crate.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup, file operation or parse failed.
    Other {
        /// What failed.
        message: String,
    },
}

impl fmt::Display for UpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Cargo { crate_dir, command, output } => {
                writeln!(formatter, "`{command}` failed in {crate_dir}; every lockfile was restored.")?;
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                writeln!(formatter, "Fix the cause cargo names. When it is that no release old enough satisfies a requirement,")?;
                writeln!(formatter, "either wait, add the crate to minimumReleaseAgeExclude in rust-catalog.toml,")?;
                writeln!(formatter, "or set minimumReleaseAgeStrict = false to fall back to a younger release.")?;
                return write!(formatter, "When cargo says `-Z` needs the nightly channel, set RUSTUP_TOOLCHAIN to the nightly toolchain.lock.json pins.");
            }
            UpdateError::Other { message } => return write!(formatter, "{message}"),
        }
    }
}

/// What:
/// `fn run_or_fail(request, crate_dir, args) -> Result<CargoOutcome, UpdateError>` runs one cargo command in a crate root and turns a non-zero exit into an error naming the crate. `Result<T, E>` is Rust's success-or-failure value (sibling: `Option<T>`, a value or nothing); `?` on one returns early with the failure.
/// Why:
/// Every cargo step of an update fails the same way, so the message is in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runOrFail(args): Promise<CargoOutcome>  // throws
/// ```
fn run_or_fail(request: &UpdateRequest, crate_dir: &str, args: &[String]) -> Result<CargoOutcome, UpdateError> {
    let outcome: CargoOutcome = request
        .runner
        .run(&request.root.join(crate_dir), args)
        .map_err(|message| { return UpdateError::Other { message }; })?;
    if !outcome.ok {
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", args.join(" ")), output: outcome.output });
    }
    return Ok(outcome);
}

/// What:
/// `pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String>` lists `name@version` for
/// each locked registry crate that a name-only selector exempts from the release age.
/// Why:
/// Those crates move to their newest release with `cargo update`; a selector naming exact versions only exempts those
/// versions from the checks and does not move the crate, because the cutoff already chose its version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function excludedUpdateSpecs({ lock, selectors }): string[]
/// ```
pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String> {
    let mut patterns: Vec<String> = Vec::new();
    for selector in selectors {
        let parsed = parse_selector(selector);
        if parsed.versions.is_empty() {
            patterns.push(parsed.pattern);
        }
    }
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let mut specs: Vec<String> = Vec::new();
    for locked in parse_registry_versions(lock)? {
        if patterns.iter().any(|pattern| { return matches_pattern(pattern, &locked.name); }) {
            specs.push(locked.key());
        }
    }
    specs.sort();
    specs.dedup();
    return Ok(specs);
}

/// What:
/// `fn resolve_crate(request, crate_dir, cutoff) -> Result<bool, UpdateError>` re-resolves one crate root under the cutoff and returns whether it had to fall back to ignoring the cutoff.
/// Why:
/// With `minimumReleaseAgeStrict` false a crate that cannot resolve under the cutoff resolves without it and is reported; strict fails instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCrate(args): Promise<boolean>  // throws
/// ```
fn resolve_crate(request: &UpdateRequest, crate_dir: &str, cutoff: &str) -> Result<bool, UpdateError> {
    let mut constrained: Vec<String> = Vec::new();
    if cutoff.is_empty() {
        constrained.push("generate-lockfile".to_string());
    } else {
        constrained.extend(["-Zunstable-options".to_string(), "generate-lockfile".to_string(), "--publish-time".to_string(), cutoff.to_string()]);
    }
    constrained.extend(request.forward.iter().cloned());
    let first: CargoOutcome = request.runner.run(&request.root.join(crate_dir), &constrained).map_err(|message| { return UpdateError::Other { message }; })?;
    if first.ok {
        return Ok(false);
    }
    if request.settings.minimum_release_age_strict || cutoff.is_empty() {
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", constrained.join(" ")), output: first.output });
    }
    eprintln!("cargo-shadow: {crate_dir}: no resolution satisfies the release-age cutoff {cutoff}; minimumReleaseAgeStrict is false, so resolving without it");
    let mut fallback: Vec<String> = vec!["generate-lockfile".to_string()];
    fallback.extend(request.forward.iter().cloned());
    run_or_fail(request, crate_dir, &fallback)?;
    return Ok(true);
}

/// What:
/// `fn resolve_all(request, cutoff) -> Result<Vec<String>, UpdateError>` resolves every listed crate root and then moves the exempt crates to their newest release.
/// Why:
/// The exempt update reads the lockfile that the same root's resolution just wrote, so the two steps stay together per root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveAll(args): Promise<string[]>
/// ```
fn resolve_all(request: &UpdateRequest, cutoff: &str) -> Result<Vec<String>, UpdateError> {
    let mut fell_back: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        eprintln!("cargo-shadow: resolving {crate_dir}{}", if cutoff.is_empty() { String::new() } else { format!(" with releases published before {cutoff}") });
        if resolve_crate(request, crate_dir, cutoff)? {
            fell_back.push(crate_dir.clone());
        }
        let snapshots: Vec<LockSnapshot> = read_locks(request.root, std::slice::from_ref(crate_dir)).map_err(|error| { return UpdateError::Other { message: error.to_string() }; })?;
        let source: &str = snapshots.first().map(|snapshot| { return snapshot.source.as_str(); }).unwrap_or("");
        let specs: Vec<String> = excluded_update_specs(source, &request.settings.minimum_release_age_exclude).map_err(|message| { return UpdateError::Other { message }; })?;
        if !specs.is_empty() {
            let mut args: Vec<String> = vec!["update".to_string()];
            args.extend(specs);
            args.extend(request.forward.iter().cloned());
            run_or_fail(request, crate_dir, &args)?;
        }
    }
    return Ok(fell_back);
}

/// What:
/// `fn sources_of(snapshots) -> Vec<String>` copies the lockfile texts out of snapshots; `.clone()` makes an independent copy of owned text.
/// Why:
/// The lockfile diff takes plain texts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourcesOf = (snapshots) => snapshots.map((s) => s.source)
/// ```
fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
    return snapshots.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
}

/// What:
/// `fn other(message) -> UpdateError` wraps a message in the catch-all update error.
/// Why:
/// `map_err(other)` turns a plain text failure into the error type of the function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const other = (message: string) => new UpdateError(message)
/// ```
fn other(message: String) -> UpdateError {
    return UpdateError::Other { message };
}

/// What:
/// `fn run_update(request, kind, before, produced) -> Result<UpdateReport, UpdateError>` is the transaction body: resolve or run the user's update, set refused versions aside, check what is left, prune and raise catalog entries when kept. `&mut Option<...>` is a borrowed slot the function may fill.
/// Why:
/// The caller restores on an error or a refusal; `produced` records what cargo wrote before the slow registry checks, so the restore can tell a concurrent writer from this run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runUpdate(args): Promise<UpdateReport>
/// ```
fn run_update(request: &UpdateRequest, kind: &UpdateKind, before: &[LockSnapshot], produced: &mut Option<Vec<LockSnapshot>>) -> Result<UpdateReport, UpdateError> {
    let settings: &Settings = request.settings;
    let cutoff: String = if settings.minimum_release_age == 0 { String::new() } else { publish_time_cutoff(request.now, settings.minimum_release_age) };
    let cutoff_ms: i64 = if settings.minimum_release_age == 0 { 0 } else { request.now - (settings.minimum_release_age as i64) * crate::time_text::MS_PER_MINUTE };
    let mut fell_back: Vec<String> = Vec::new();
    let mut set_aside_allowed: bool = true;
    match kind {
        UpdateKind::Full => fell_back = resolve_all(request, &cutoff)?,
        UpdateKind::Targeted { cargo_args } => {
            let mut args: Vec<String> = cargo_args.to_vec();
            args.extend(request.forward.iter().cloned());
            // An explicit `--precise` names the version the user wants: the policy decides whether it stands, and nothing is
            // set aside in its place.
            set_aside_allowed = !cargo_args.iter().any(|argument| { return argument == "--precise" || argument.starts_with("--precise="); });
            for crate_dir in request.crate_dirs {
                run_or_fail(request, crate_dir, &args)?;
            }
        }
    }
    let before_sources: Vec<String> = sources_of(before);
    let mut warnings: Vec<String> = Vec::new();
    if set_aside_allowed {
        warnings = set_aside_until_clean(&SetAsideRequest {
            root: request.root,
            crate_dirs: request.crate_dirs,
            before: &before_sources,
            registry: request.registry,
            settings,
            now: request.now,
            cutoff_ms,
            runner: request.runner,
            forward: request.forward,
        })
        .map_err(other)?;
    }
    let after_snapshots: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return other(error.to_string()); })?;
    // What cargo produced, taken before the slow registry checks: a restore later replaces a lockfile only if it still holds this.
    *produced = Some(after_snapshots.clone());
    let after: Vec<String> = sources_of(&after_snapshots);
    let added: Vec<LockedVersion> = added_versions(&before_sources, &after).map_err(other)?;
    let age: CheckOutcome = check_release_age(&added, settings, request.registry, request.now).map_err(|error| { return other(error.message); })?;
    let trust: Vec<String> = check_trust(&added, settings, request.registry, request.now).map_err(|error| { return other(error.message); })?;
    let mut problems: Vec<String> = age.problems;
    problems.extend(trust);
    warnings.extend(age.warnings);
    let keep: bool = problems.is_empty() && !request.dry_run;
    let mut pruned: Vec<String> = Vec::new();
    if keep && request.prune_catalog && matches!(kind, UpdateKind::Full) {
        let source: String = fs::read_to_string(request.catalog_path).map_err(|error| { return other(error.to_string()); })?;
        let locked: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        let result = prune_excludes(&source, settings, &locked).map_err(other)?;
        if !result.removed.is_empty() {
            fs::write(request.catalog_path, &result.source).map_err(|error| { return other(error.to_string()); })?;
        }
        pruned = result.removed;
    }
    // `--breaking` is `pnpm update --latest`: the new range goes into the catalog entry and the sync renders it (tighten.rs).
    if keep && let UpdateKind::Targeted { cargo_args } = kind && cargo_args.iter().any(|argument| { return argument == "--breaking"; }) {
        let before_versions: Vec<LockedVersion> = registry_versions(&before_sources).map_err(other)?;
        let after_versions: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        let source: String = fs::read_to_string(request.catalog_path).map_err(|error| { return other(error.to_string()); })?;
        let (raised_source, changes) = raise_floors(&source, &after_versions, &raised_names(&before_versions, &after_versions)).map_err(other)?;
        if !changes.is_empty() {
            fs::write(request.catalog_path, &raised_source).map_err(|error| { return other(error.to_string()); })?;
            warnings.extend(changes.iter().map(|change| { return format!("Raised the catalog floor {change}"); }));
            match request.syncer.sync(request.root) {
                Ok(done) => warnings.push(done),
                Err(message) => warnings.push(message),
            }
        }
    }
    return Ok(UpdateReport { crate_dirs: request.crate_dirs.to_vec(), added, problems, warnings, fell_back, written: keep, pruned });
}

/// What:
/// `pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError>` runs the
/// update over the listed crate roots.
/// Why:
/// It first finishes any run that was killed (`journal.rs`), snapshots every lockfile, and puts the snapshots back on a failure,
/// a refusal or a dry run, but only into lockfiles that still hold what this run produced, so a refused version never stays in
/// some crates and not others and a change written by another process is never discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function updateRustLockfiles(args): Promise<UpdateReport>
/// ```
pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError> {
    let recovered: Vec<String> = recover(request.root, &process_exists, &|old, live| {
        return policy_findings(request.settings, request.registry, request.now, old, live);
    });
    for message in &recovered {
        eprintln!("cargo-shadow: warning: {message}");
    }
    let before: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return other(error.to_string()); })?;
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    let journaled: bool = begin(request.root, &before).is_ok();
    let mut produced: Option<Vec<LockSnapshot>> = None;
    let outcome: Result<UpdateReport, UpdateError> = run_update(request, kind, &before, &mut produced);
    let keep: bool = matches!(&outcome, Ok(report) if report.written);
    let mut restore_warnings: Vec<String> = Vec::new();
    if !keep {
        eprintln!("cargo-shadow: restoring every lockfile that this run changed");
        let state: Vec<LockSnapshot> = match produced {
            Some(found) => found,
            None => read_locks(request.root, request.crate_dirs).unwrap_or_default(),
        };
        restore_warnings = restore_if_unchanged(request.root, &before, &state).map_err(|error| { return other(error.to_string()); })?;
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
