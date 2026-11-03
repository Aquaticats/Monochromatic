//! What:
//! Release-age and trust checks over a set of newly locked versions, one registry lookup per crate.
//! Why:
//! Both the install step and the update gate every version that no lockfile held before through these two checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checks.ts
//! export async function checkReleaseAge({ versions, settings, registry, now }): Promise<{ problems: string[]; warnings: string[] }>
//! export async function trustRefusals({ versions, settings, registry, now }): Promise<TrustRefusal[]>
//! ```

use std::collections::BTreeSet;

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::http::RegistryError;
use crate::lockfile::LockedVersion;
use crate::policy::{release_age_problems, trust_problems};
use crate::registry::Registry;

/// The findings of a check pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckOutcome {
    /// Findings that refuse the run.
    pub problems: Vec<String>,
    /// Findings reported without refusing (release age with `minimumReleaseAgeStrict = false`).
    pub warnings: Vec<String>,
}

/// One version the trust policy refuses, with every finding against it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustRefusal {
    /// The crate.
    pub name: String,
    /// The refused version.
    pub version: String,
    /// The findings.
    pub problems: Vec<String>,
}

/// Versions grouped by crate name, in first-seen order, so each crate costs one lookup.
pub fn group_by_name(versions: &[LockedVersion]) -> Vec<(String, Vec<String>)> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for locked in versions {
        match groups.iter_mut().find(|group| { return group.0 == locked.name; }) {
            Some(group) => group.1.push(locked.version.clone()),
            None => groups.push((locked.name.clone(), vec![locked.version.clone()])),
        }
    }
    return groups;
}

/// What:
/// `pub fn check_release_age(...) -> Result<CheckOutcome, RegistryError>` checks each version's publish time from the sparse
/// index against `minimumReleaseAge`.
/// Why:
/// Findings are problems when `minimumReleaseAgeStrict` is true, warnings otherwise; age zero checks nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function checkReleaseAge({ versions, settings, registry, now }): Promise<CheckOutcome>
/// ```
pub fn check_release_age(
    versions: &[LockedVersion],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<CheckOutcome, RegistryError> {
    if settings.minimum_release_age == 0 {
        return Ok(CheckOutcome::default());
    }
    let mut findings: Vec<String> = Vec::new();
    for (name, name_versions) in group_by_name(versions) {
        let times = registry.pubtimes(&name)?;
        for version in name_versions {
            let pubtime: &str = times.get(&version).map(|text| { return text.as_str(); }).unwrap_or("");
            findings.extend(release_age_problems(&name, &version, pubtime, settings, now));
        }
    }
    if settings.minimum_release_age_strict {
        return Ok(CheckOutcome { problems: findings, warnings: Vec::new() });
    }
    return Ok(CheckOutcome { problems: Vec::new(), warnings: findings });
}

/// The refusals of several versions of one crate under one owner list.
fn crate_trust_refusals(
    name: &str,
    versions: &[String],
    history: &[crate::policy::VersionRecord],
    settings: &Settings,
    now: i64,
    owner_ids: &BTreeSet<String>,
) -> Vec<TrustRefusal> {
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    for version in versions {
        let problems: Vec<String> = trust_problems(name, version, history, settings, now, owner_ids);
        if !problems.is_empty() {
            refusals.push(TrustRefusal { name: name.to_string(), version: version.clone(), problems });
        }
    }
    return refusals;
}

/// What:
/// `pub fn trust_refusals(...) -> Result<Vec<TrustRefusal>, RegistryError>` checks the trust policy: one history lookup per
/// crate, plus an owners lookup only for a crate that would otherwise be refused.
/// Why:
/// A listed owner's first release ranks as a known publisher, so a refusal is rechecked against the current owner list
/// before it stands; most crates pass without that request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function trustRefusals({ versions, settings, registry, now }): Promise<TrustRefusal[]>
/// ```
pub fn trust_refusals(
    versions: &[LockedVersion],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<TrustRefusal>, RegistryError> {
    if settings.trust_policy == TrustPolicy::Off {
        return Ok(Vec::new());
    }
    let groups: Vec<(String, Vec<String>)> = group_by_name(versions);
    eprintln!("cargo-shadow: checking the trust policy for {} versions of {} crates", versions.len(), groups.len());
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    for (name, name_versions) in groups {
        let history = registry.history(&name, &name_versions)?;
        let without_owners = crate_trust_refusals(&name, &name_versions, &history, settings, now, &BTreeSet::new());
        if without_owners.is_empty() {
            continue;
        }
        let owner_ids: BTreeSet<String> = registry.owners(&name)?;
        refusals.extend(crate_trust_refusals(&name, &name_versions, &history, settings, now, &owner_ids));
    }
    return Ok(refusals);
}

/// The findings of `trust_refusals`, flattened.
pub fn check_trust(
    versions: &[LockedVersion],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<String>, RegistryError> {
    let mut messages: Vec<String> = Vec::new();
    for refusal in trust_refusals(versions, settings, registry, now)? {
        messages.extend(refusal.problems);
    }
    return Ok(messages);
}
