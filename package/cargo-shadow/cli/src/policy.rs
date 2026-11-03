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

/// What:
/// `fn created_millis(record) -> Option<i64>` reads a release's publish time in milliseconds,
/// and is `None` when the time is missing or not RFC 3339.
/// Why:
/// An unknown time must never establish eligibility for the `trustPolicyIgnoreAfter` age exemption,
/// so the caller has to see the unknown case instead of a zero that reads as "very old".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const createdMillis = (record: VersionRecord): number | undefined => parseRfc3339Millis(record.createdAt);
/// ```
fn created_millis(record: &VersionRecord) -> Option<i64> {
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    return parse_rfc3339_millis(&record.created_at);
}

/// What:
/// `fn ordering_millis(record) -> i64` reads a release's publish time for sorting records that are known to have a readable time.
/// Why:
/// Review finding (2c 7):
/// an unreadable time cannot be ordered,
/// so such a record is never part of the earlier history
/// (`earlier_releases` leaves it out).
/// It must not give a publisher a reputation it did not earn;
/// the zero here is never reached
/// for a record in that list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const orderingMillis = (record: VersionRecord): number => parseRfc3339Millis(record.createdAt) ?? 0;
/// ```
fn ordering_millis(record: &VersionRecord) -> i64 {
    return created_millis(record).unwrap_or(0);
}

/// What:
/// `fn earlier_releases(history, record_millis) -> Vec<VersionRecord>` lists the releases published before a given time,
/// oldest first,
/// leaving out any release whose publish time is unreadable.
/// Why:
/// Both the trust decision and the question "can the owner list change this decision" need the same set.
/// A release that cannot be placed in time cannot be called earlier,
/// so it establishes no publisher history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const earlierReleases = (history: VersionRecord[], recordMillis: number) =>
///   history.filter((entry) => (createdMillis(entry) ?? Infinity) < recordMillis).sort(byMillis);
/// ```
fn earlier_releases(history: &[VersionRecord], record_millis: i64) -> Vec<VersionRecord> {
    let mut earlier: Vec<VersionRecord> = Vec::new();
    for entry in history {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(published) = created_millis(entry)
            && published < record_millis
        {
            earlier.push(entry.clone());
        }
    }
    earlier.sort_by_key(ordering_millis);
    return earlier;
}

/// What:
/// `pub fn owners_can_change_verdict(...) -> bool` says whether the crate's owner list could turn a clean ownerless
/// trust evaluation into a refusal for any of `versions`.
/// Why:
/// Owners only raise levels.
/// A refusal needs an earlier release ranked above the checked one,
/// so the owner list can matter
/// only for a checked release that ranks as a first-time publisher without owners and has earlier releases.
/// The registry call for owners is rate limited,
/// so it is asked only when it can change the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ownersCanChangeVerdict(name, versions, history, settings, now): boolean
/// ```
pub fn owners_can_change_verdict(name: &str, versions: &[String], history: &[VersionRecord], settings: &Settings, now: i64) -> bool {
    if settings.trust_policy == TrustPolicy::Off {
        return false;
    }
    for version in versions {
        // Borrow `&settings.trust_policy_exclude`: lends the value without copying or giving it away.
        if is_excluded(&settings.trust_policy_exclude, name, version) {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `record: &VersionRecord`.
        let record: &VersionRecord = match version_record_of(history, version) {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // `None`: the present or the absent value of an `Option`.
            None => continue,
        };
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `record_millis: i64`.
        let record_millis: i64 = match created_millis(record) {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // `None`: the present or the absent value of an `Option`.
            None => continue,
        };
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        if let Some(ignore_after) = settings.trust_policy_ignore_after
            // What:     `value as i64` converts a number to another numeric type (siblings: `i32`, `u32`, `u64`, `usize`, `f64`); it truncates silently if the value does not fit.
            // Why:      Time arithmetic and sizes come in different integer widths and must be made equal.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // Number(value)  // JS has one number type, so no cast is needed
            // ```
            && record_millis + (ignore_after as i64) * MS_PER_MINUTE <= now
        {
            continue;
        }
        // Typed binding: `earlier: Vec<VersionRecord>`.
        let earlier: Vec<VersionRecord> = earlier_releases(history, record_millis);
        if earlier.is_empty() {
            continue;
        }
        // Borrow `&earlier`: lends the value without copying or giving it away.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        if trust_level(record, &earlier, &BTreeSet::new()) == FIRST_TIME_PUBLISHER_LEVEL {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn publisher_text(record) -> String` says who published a release,
/// as far as the registry reports it:
/// the trusted-publishing repository,
/// else the account id,
/// else that it is unknown.
/// Why:
/// A trust downgrade is a change of publisher,
/// and the finding should name both ends of the change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const publisherText = (record: VersionRecord): string => record.trustpubRepository ? `trusted publishing from ${record.trustpubRepository}` : record.publisherId ? `account ${record.publisherId}` : 'an unknown publisher';
/// ```
fn publisher_text(record: &VersionRecord) -> String {
    if !record.trustpub_repository.is_empty() {
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        return format!("trusted publishing from {}", record.trustpub_repository);
    }
    if !record.publisher_id.is_empty() {
        // Macro `format!`: builds or prints text (or a list) from a template.
        return format!("account {}", record.publisher_id);
    }
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return "an unknown publisher".to_string();
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
    // An unknown publish time cannot be ordered against the earlier releases and never grants the age exemption.
    let record_millis: i64 = match created_millis(record) {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        None => {
            // Macro `vec!`: builds or prints text (or a list) from a template.
            return vec![format!(
                "{name}@{version} has no readable publish time in the crates.io version list, so its trust cannot be ordered against earlier releases; add it to trustPolicyExclude if it is expected"
            )];
        }
    };
    if let Some(ignore_after) = settings.trust_policy_ignore_after
        // Cast `as i64`: converts between numeric types.
        && record_millis + (ignore_after as i64) * MS_PER_MINUTE <= now
    {
        return Vec::new();
    }
    // Typed binding: `earlier: Vec<VersionRecord>`.
    let earlier: Vec<VersionRecord> = earlier_releases(history, record_millis);
    let mut problems: Vec<String> = Vec::new();
    let level: u8 = trust_level(record, &earlier, owner_ids);
    let mut best: u8 = 0;
    // The most recent earlier release that reached the best level, named in the finding so the publisher change is visible.
    let mut best_entry: Option<&VersionRecord> = None;
    for (index, entry) in earlier.iter().enumerate() {
        if !is_prerelease(version) && is_prerelease(&entry.num) {
            continue;
        }
        // Borrow `&earlier`: lends the value without copying or giving it away.
        // Typed binding: `entry_level: u8`.
        let entry_level: u8 = trust_level(entry, &earlier[..index], owner_ids);
        if entry_level >= best {
            best = entry_level;
            // `Some`: the present or the absent value of an `Option`.
            best_entry = Some(entry);
        }
    }
    if best > level {
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `earlier_text: String`.
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let earlier_text: String = match best_entry {
            // `Some`: the present or the absent value of an `Option`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Some(entry) => format!("{} was published by {}", entry.num, publisher_text(entry)),
            // `None`: the present or the absent value of an `Option`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            None => "the earlier release's publisher is not known".to_string(),
        };
        problems.push(format!(
            "{name}@{version} has trust level {level}, below an earlier release's {best} (3 trusted publishing, 2 known publisher, 1 first-time publisher); it was published by {}, where {earlier_text}",
            publisher_text(record)
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
