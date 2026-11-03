//! What:
//! The release-age rule and the trust rule of `rust-catalog.toml`,
//! as pure functions over one crate version and the
//! crate's published history.
//! Why:
//! pnpm applies `minimumReleaseAge` and `trustPolicy: no-downgrade` while it resolves;
//! cargo has neither,
//! so the catalog
//! tools apply them to every registry version a lockfile newly holds.
//! Trust levels are heuristics from crates.io data:
//! 3 is a Trusted Publishing release,
//! 2 is a publish by an account that published an earlier release or is a listed
//! individual owner,
//! 1 is any other account.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // policy.ts
//! export function releaseAgeProblems(args): string[]
//! export function trustProblems(args): string[]
//! ```

/// What:
/// `use std::collections::BTreeSet;` imports `BTreeSet` from the standard library so this file can name it unqualified.
/// Why:
/// `BTreeSet` is used by `trust_level`,
/// `trust_problems`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeSet } from 'std/collections';
/// ```
use std::collections::BTreeSet;

/// What:
/// `use monochromatic_dependency_catalog::{Settings, TrustPolicy};` imports `Settings`,
/// `TrustPolicy` from the external crate `monochromatic_dependency_catalog` so this file can name them unqualified.
/// Why:
/// `Settings` is used by `release_age_problems`,
/// `trust_problems`;
/// `TrustPolicy` is used by `trust_problems`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings, TrustPolicy } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::{Settings, TrustPolicy};

/// What:
/// `use crate::selectors::is_excluded;` imports `is_excluded` from another module of this crate so this file can name it unqualified.
/// Why:
/// `is_excluded` is used by `release_age_problems`,
/// `trust_problems`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_excluded } from 'crate/selectors';
/// ```
use crate::selectors::is_excluded;
/// What:
/// `use crate::time_text::{MS_PER_MINUTE, format_iso_millis, parse_rfc3339_millis};` imports `MS_PER_MINUTE`,
/// `format_iso_millis`,
/// `parse_rfc3339_millis` from another module of this crate so this file can name them unqualified.
/// Why:
/// `MS_PER_MINUTE` is used by `release_age_problems`,
/// `trust_problems`;
/// `format_iso_millis` is used by `release_age_problems`;
/// `parse_rfc3339_millis` is used by `release_age_problems`,
/// `created_millis`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { MS_PER_MINUTE, format_iso_millis, parse_rfc3339_millis } from 'crate/time_text';
/// ```
use crate::time_text::{MS_PER_MINUTE, format_iso_millis, parse_rfc3339_millis};

/// One published release of a crate,
/// as crates.io lists it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionRecord {
    /// The version number.
    pub num: String,
    /// When it was published,
    /// RFC 3339.
    pub created_at: String,
    /// The publishing account's id;
    /// empty for a Trusted Publishing release.
    pub publisher_id: String,
    /// The repository of the Trusted Publishing workflow;
    /// empty for a token publish.
    pub trustpub_repository: String,
    /// The declared `package.repository`;
    /// empty when undeclared.
    pub repository: String,
}

/// The level of a Trusted Publishing release.
const TRUSTED_PUBLISHING_LEVEL: u8 = 3;

/// The level of a publish by a known publisher.
const KNOWN_PUBLISHER_LEVEL: u8 = 2;

/// The level of a publish by any other account.
const FIRST_TIME_PUBLISHER_LEVEL: u8 = 1;

/// What:
/// `pub fn release_age_problems(...) -> Vec<String>` returns the findings for one version's publish time.
/// Why:
/// A version passes when an exclude selector covers it,
/// or when its publish time plus `minimumReleaseAge` minutes is not in
/// the future.
/// A missing publish time passes only with `minimumReleaseAgeIgnoreMissingTime`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function releaseAgeProblems({ name, version, pubtime, settings, now }): string[]
/// ```
pub fn release_age_problems(name: &str, version: &str, pubtime: &str, settings: &Settings, now: i64) -> Vec<String> {
    if is_excluded(&settings.minimum_release_age_exclude, name, version) {
        return Vec::new();
    }
    let published: i64 = match parse_rfc3339_millis(pubtime) {
        Some(found) => found,
        None => {
            if settings.minimum_release_age_ignore_missing_time {
                return Vec::new();
            }
            return vec![format!(
                "{name}@{version} has no publish time in the crates.io index, and minimumReleaseAgeIgnoreMissingTime is false"
            )];
        }
    };
    let mature: i64 = published + (settings.minimum_release_age as i64) * MS_PER_MINUTE;
    if mature <= now {
        return Vec::new();
    }
    return vec![format!(
        "{name}@{version} was published {pubtime} and passes minimumReleaseAge ({} minutes) at {}; wait until then or add \"{name}\" to minimumReleaseAgeExclude in rust-catalog.toml",
        settings.minimum_release_age,
        format_iso_millis(mature)
    )];
}

/// Whether a version number is a prerelease (a `-` before any `+build` part).
fn is_prerelease(version: &str) -> bool {
    let without_build: &str = version.split('+').next().unwrap_or("");
    return without_build.contains('-');
}

/// Publish time in milliseconds;
/// an unreadable time sorts first.
fn created_millis(record: &VersionRecord) -> i64 {
    return parse_rfc3339_millis(&record.created_at).unwrap_or(0);
}

/// What:
/// `pub fn trust_level(record, earlier, owner_ids) -> u8` ranks one release:
/// 3 Trusted Publishing,
/// 2 known publisher,
/// 1 first-time publisher.
/// Why:
/// A release by an account that published an earlier release,
/// or that is a listed individual owner now,
/// is a known publisher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trustLevel({ record, earlier, ownerIds }): number
/// ```
pub fn trust_level(record: &VersionRecord, earlier: &[VersionRecord], owner_ids: &BTreeSet<String>) -> u8 {
    if !record.trustpub_repository.is_empty() {
        return TRUSTED_PUBLISHING_LEVEL;
    }
    if !record.publisher_id.is_empty() && owner_ids.contains(&record.publisher_id) {
        return KNOWN_PUBLISHER_LEVEL;
    }
    for previous in earlier {
        if !previous.publisher_id.is_empty() && previous.publisher_id == record.publisher_id {
            return KNOWN_PUBLISHER_LEVEL;
        }
    }
    return FIRST_TIME_PUBLISHER_LEVEL;
}

/// What:
/// `fn trusted_repository_of(entry: &VersionRecord) -> Option<String>` returns the repository that published an earlier release through Trusted Publishing,
/// or `None` when it was not published that way.
/// Why:
/// The repository-change rule compares a new release with the repositories that published the earlier ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trustedRepositoryOf = (entry: VersionRecord): string | undefined => entry.trustpubRepository || undefined;
/// ```
fn trusted_repository_of(entry: &VersionRecord) -> Option<String> {
    if entry.trustpub_repository.is_empty() {
        return None;
    }
    return Some(entry.trustpub_repository.clone());
}

/// What:
/// `fn version_record_of(history, version) -> Option<&VersionRecord>` finds the record of one version number in a crate's history.
/// Why:
/// The trust check starts from the record of the locked version,
/// and a version missing from the history is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const versionRecordOf = (history: VersionRecord[], version: string) => history.find((e) => e.num === version);
/// ```
fn version_record_of<'a>(history: &'a [VersionRecord], version: &str) -> Option<&'a VersionRecord> {
    for entry in history {
        if entry.num == version {
            return Some(entry);
        }
    }
    return None;
}

/// What:
/// `pub fn trust_problems(name, version, history, settings, now, owner_ids) -> Vec<String>` returns the findings against one
/// version under `trustPolicy = "no-downgrade"`.
/// Why:
/// A version fails when an earlier stable release ranked higher,
/// when its Trusted Publishing repository differs from earlier
/// trusted releases,
/// or when its declared repository differs from the release just before it.
/// `trustPolicyIgnoreAfter`
/// skips releases older than that many minutes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trustProblems({ name, version, history, settings, now, ownerIds }): string[]
/// ```
pub fn trust_problems(
    name: &str,
    version: &str,
    history: &[VersionRecord],
    settings: &Settings,
    now: i64,
    owner_ids: &BTreeSet<String>,
) -> Vec<String> {
    if settings.trust_policy == TrustPolicy::Off || is_excluded(&settings.trust_policy_exclude, name, version) {
        return Vec::new();
    }
    let record: &VersionRecord = match version_record_of(history, version) {
        Some(found) => found,
        None => {
            return vec![format!(
                "{name}@{version} is not in the crates.io version list, so its trust cannot be checked; add it to trustPolicyExclude if it is expected"
            )];
        }
    };
    if let Some(ignore_after) = settings.trust_policy_ignore_after
        && created_millis(record) + (ignore_after as i64) * MS_PER_MINUTE <= now
    {
        return Vec::new();
    }
    let record_millis: i64 = created_millis(record);
    let mut earlier: Vec<VersionRecord> = Vec::new();
    for entry in history {
        if created_millis(entry) < record_millis {
            earlier.push(entry.clone());
        }
    }
    earlier.sort_by_key(created_millis);
    let mut problems: Vec<String> = Vec::new();
    let level: u8 = trust_level(record, &earlier, owner_ids);
    let mut best: u8 = 0;
    for (index, entry) in earlier.iter().enumerate() {
        if !is_prerelease(version) && is_prerelease(&entry.num) {
            continue;
        }
        best = best.max(trust_level(entry, &earlier[..index], owner_ids));
    }
    if best > level {
        problems.push(format!(
            "{name}@{version} has trust level {level}, below an earlier release's {best} (3 trusted publishing, 2 known publisher, 1 first-time publisher)"
        ));
    }
    let trusted_repositories: BTreeSet<String> = earlier
        .iter()
        .filter_map(trusted_repository_of)
        .collect();
    if !record.trustpub_repository.is_empty()
        && !trusted_repositories.is_empty()
        && !trusted_repositories.contains(&record.trustpub_repository)
    {
        problems.push(format!(
            "{name}@{version} was trusted-published from {}, not {}",
            record.trustpub_repository,
            trusted_repositories.iter().cloned().collect::<Vec<String>>().join(", ")
        ));
    }
    if let Some(previous) = earlier.last()
        && !previous.repository.is_empty()
        && !record.repository.is_empty()
        && previous.repository != record.repository
    {
        problems.push(format!(
            "{name}@{version} declares repository {:?}, but {} declared {:?}",
            record.repository, previous.num, previous.repository
        ));
    }
    return problems
        .into_iter()
        .map(|problem| {
            return format!(
                "{problem}; review the release, then add \"{name}@{version}\" to trustPolicyExclude in rust-catalog.toml if it is legitimate"
            );
        })
        .collect();
}
