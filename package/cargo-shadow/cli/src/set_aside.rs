//! What:
//! Replaces a newly locked version that the release-age or trust policy refuses with the newest older release in the same
//! caret class that passes, the way pnpm's picker skips a version the policy refuses and takes the next best.
//! Why:
//! `cargo update` has no release-age cutoff and no trust policy, so cargo picks the newest release; this step moves the
//! refused crates down with `cargo update --package <name>@<version> --precise <candidate>`, in rounds, because a move can
//! lock new transitive versions that need checking in turn. A refusal with no workable candidate stays a finding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // set-aside.ts
//! export async function setAsideUntilClean(args): Promise<string[]>  // warnings
//! ```

use std::collections::BTreeSet;
use std::path::Path;

use monochromatic_dependency_catalog::Settings;

use crate::cargo_run::CargoRunner;
use crate::checks::{TrustRefusal, group_by_name, trust_refusals};
use crate::crates::{LockSnapshot, read_locks};
use crate::http::RegistryError;
use crate::lockfile::{LockedVersion, added_versions, registry_versions};
use crate::policy::{VersionRecord, release_age_problems, trust_problems};
use crate::registry::Registry;
use crate::time_text::parse_rfc3339_millis;
use crate::version_order::{caret_compatible, compare_texts, is_prerelease_text};

/// How many rounds of moving refused crates are tried.
pub const SET_ASIDE_ROUNDS: usize = 5;

/// How many candidates are tried per refused version and crate.
pub const PRECISE_ATTEMPTS: usize = 10;

/// What a set-aside round needs.
pub struct SetAsideRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots in play.
    pub crate_dirs: &'a [String],
    /// Lockfile texts before the whole run.
    pub before: &'a [String],
    /// The registry.
    pub registry: &'a dyn Registry,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The current time in milliseconds.
    pub now: i64,
    /// The newest allowed publish time in milliseconds; zero for no cutoff.
    pub cutoff_ms: i64,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// Options forwarded to every cargo call (`--offline`, `--config`, ...).
    pub forward: &'a [String],
}

/// What:
/// `fn age_refusals(added, request) -> Result<Vec<TrustRefusal>, RegistryError>` lists the new versions that the release-age rule refuses, as refusals. `Result<T, E>` is Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing); `?` on one returns early with the failure.
/// Why:
/// Under a strict `minimumReleaseAge` a too-young version must be set aside like a version the trust rule refuses; without a cutoff in `cargo update` the age rule cannot steer the resolution, so it has to be a refusal here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ageRefusals(added: LockedVersion[], request: SetAsideRequest): TrustRefusal[]  // throws RegistryError
/// ```
fn age_refusals(added: &[LockedVersion], request: &SetAsideRequest) -> Result<Vec<TrustRefusal>, RegistryError> {
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    if request.settings.minimum_release_age == 0 || !request.settings.minimum_release_age_strict {
        return Ok(refusals);
    }
    for (name, versions) in group_by_name(added) {
        let times = request.registry.pubtimes(&name)?;
        for version in versions {
            let pubtime: &str = times.get(&version).map(|text| { return text.as_str(); }).unwrap_or("");
            let problems: Vec<String> = release_age_problems(&name, &version, pubtime, request.settings, request.now);
            if !problems.is_empty() {
                refusals.push(TrustRefusal { name: name.clone(), version, problems });
            }
        }
    }
    return Ok(refusals);
}

/// What:
/// `fn is_candidate(record, refused, prerelease_allowed, cutoff_ms, yanked, rejected) -> bool` says whether one published release may replace the refused version. `&` marks a borrowed read-only argument; `BTreeSet<String>` is an ordered set of owned text (sibling: `HashSet`, unordered).
/// Why:
/// A replacement must be older, in the same caret class, not yanked, not already rejected, published before the cutoff, and a prerelease only when the refused version is one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCandidate(args): boolean
/// ```
fn is_candidate(record: &VersionRecord, refused: &TrustRefusal, prerelease_allowed: bool, cutoff_ms: i64, yanked: &BTreeSet<String>, rejected: &BTreeSet<String>) -> bool {
    if yanked.contains(&record.num) || rejected.contains(&format!("{}@{}", refused.name, record.num)) {
        return false;
    }
    if cutoff_ms != 0 && parse_rfc3339_millis(&record.created_at).unwrap_or(0) > cutoff_ms {
        return false;
    }
    if !prerelease_allowed && is_prerelease_text(&record.num) {
        return false;
    }
    return compare_texts(&record.num, &refused.version) == std::cmp::Ordering::Less && caret_compatible(&refused.version, &record.num);
}

/// What:
/// `pub fn set_aside_candidates(...) -> Result<Vec<String>, RegistryError>` lists the releases that may replace a refused
/// version, newest first.
/// Why:
/// A candidate must be older, in the same caret class, not yanked, not already rejected, published before the cutoff, and
/// pass the trust policy against the full history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function setAsideCandidates({ refusal, registry, settings, now, cutoffMs, rejected }): Promise<string[]>
/// ```
pub fn set_aside_candidates(refused: &TrustRefusal, request: &SetAsideRequest, rejected: &BTreeSet<String>) -> Result<Vec<String>, RegistryError> {
    let history: Vec<VersionRecord> = request.registry.history(&refused.name, std::slice::from_ref(&refused.version))?;
    let yanked: BTreeSet<String> = request.registry.yanked(&refused.name)?;
    let prerelease_allowed: bool = is_prerelease_text(&refused.version);
    let mut candidates: Vec<String> = Vec::new();
    for record in &history {
        if !is_candidate(record, refused, prerelease_allowed, request.cutoff_ms, &yanked, rejected) {
            continue;
        }
        let trust: Vec<String> = trust_problems(&refused.name, &record.num, &history, request.settings, request.now, &BTreeSet::new());
        if trust.is_empty() {
            candidates.push(record.num.clone());
        }
    }
    candidates.sort_by(|left, right| { return compare_texts(right, left); });
    return Ok(candidates);
}

/// What:
/// `fn crates_locking(snapshots, key) -> Vec<String>` lists the crate roots whose lockfile holds `name@version`. `Vec<String>` is a growable list of owned text (siblings: `&[String]`, a borrowed view; `[String; N]`, fixed size).
/// Why:
/// Only those roots need the refused version moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cratesLocking(snapshots: LockSnapshot[], key: string): string[]
/// ```
fn crates_locking(snapshots: &[LockSnapshot], key: &str) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    for snapshot in snapshots {
        if let Ok(versions) = registry_versions(std::slice::from_ref(&snapshot.source))
            && versions.iter().any(|locked| { return locked.key() == key; })
        {
            dirs.push(snapshot.crate_dir.clone());
        }
    }
    return dirs;
}

/// What:
/// `fn move_crate(request, crate_dir, refused, candidates) -> Result<String, String>` tries the candidates in order in one crate root with `cargo update --package name@version --precise candidate` and returns the first that cargo accepts, or the empty text.
/// Why:
/// Cargo refuses a candidate the dependency graph cannot hold; the next one is tried, at most ten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function moveCrate(args): Promise<string>
/// ```
fn move_crate(request: &SetAsideRequest, crate_dir: &str, refused: &TrustRefusal, candidates: &[String]) -> Result<String, String> {
    for candidate in candidates.iter().take(PRECISE_ATTEMPTS) {
        let mut args: Vec<String> = vec![
            "update".to_string(),
            "--package".to_string(),
            format!("{}@{}", refused.name, refused.version),
            "--precise".to_string(),
            candidate.clone(),
        ];
        args.extend(request.forward.iter().cloned());
        let outcome = request.runner.run(&request.root.join(crate_dir), &args)?;
        if outcome.ok {
            return Ok(candidate.clone());
        }
        eprintln!("cargo-shadow: {crate_dir}: cargo refused {} {candidate} ({})", refused.name, outcome.output.trim());
    }
    return Ok(String::new());
}

/// What:
/// `pub fn set_aside_until_clean(request: &SetAsideRequest) -> Result<Vec<String>, String>` repeats rounds of finding refused
/// new versions and moving them down, and returns one warning per move.
/// Why:
/// The rounds stop when nothing is refused, when nothing could be moved, or after the round limit. What stays refused is
/// reported by the caller's final check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function setAsideUntilClean(args): Promise<string[]>
/// ```
pub fn set_aside_until_clean(request: &SetAsideRequest) -> Result<Vec<String>, String> {
    let mut rejected: BTreeSet<String> = BTreeSet::new();
    let mut warnings: Vec<String> = Vec::new();
    for _round in 0..SET_ASIDE_ROUNDS {
        let current: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return error.to_string(); })?;
        let sources: Vec<String> = current.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
        let added: Vec<LockedVersion> = added_versions(request.before, &sources)?;
        let mut pending: Vec<TrustRefusal> = age_refusals(&added, request).map_err(|error| { return error.message; })?;
        for refusal in trust_refusals(&added, request.settings, request.registry, request.now).map_err(|error| { return error.message; })? {
            if !pending.iter().any(|known| { return known.name == refusal.name && known.version == refusal.version; }) {
                pending.push(refusal);
            }
        }
        pending.retain(|refusal| { return !rejected.contains(&format!("{}@{}", refusal.name, refusal.version)); });
        if pending.is_empty() {
            break;
        }
        for refusal in &pending {
            rejected.insert(format!("{}@{}", refusal.name, refusal.version));
        }
        let mut moved_any: bool = false;
        for refusal in &pending {
            let key: String = format!("{}@{}", refusal.name, refusal.version);
            let candidates: Vec<String> = set_aside_candidates(refusal, request, &rejected).map_err(|error| { return error.message; })?;
            if candidates.is_empty() {
                eprintln!("cargo-shadow: no older release can replace {key}; its finding stands");
                continue;
            }
            let snapshots: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return error.to_string(); })?;
            for crate_dir in crates_locking(&snapshots, &key) {
                let instead: String = move_crate(request, &crate_dir, refusal, &candidates)?;
                if instead.is_empty() {
                    eprintln!("cargo-shadow: {crate_dir}: no candidate for {key} resolved; its finding stands");
                    continue;
                }
                moved_any = true;
                warnings.push(format!("Set aside {key} (release-age or trust policy) and locked {instead} in {crate_dir}."));
            }
        }
        if !moved_any {
            break;
        }
    }
    return Ok(warnings);
}
