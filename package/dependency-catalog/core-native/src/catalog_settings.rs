//! What:     Validation of the catalog's pnpm-named settings (`catalogPrune`, `minimumReleaseAge`, `trustPolicy` ...).
//! Why:      The update tooling and the sync both read these, so a wrong type is reported with the fix instead of
//!           silently falling back to a default.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parse-settings.ts
//! export function readSettings(args: { root: Record<string, unknown>; issues: Issue[] }): RustCatalogSettings
//! ```

use toml_edit::{Item, TableLike};

use crate::error::CatalogIssue;
use crate::quote::quote;
use crate::types::{Settings, TrustPolicy};

/// pnpm's documented default for `minimumReleaseAge`, in minutes.
const DEFAULT_MINIMUM_RELEASE_AGE: u64 = 1_440;

/// The largest whole number a JavaScript number holds exactly, the bound the TypeScript reader implicitly had.
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// Reads a list-of-strings setting; an empty list when absent or invalid.
fn read_string_list(key: &str, item: Option<&Item>, issues: &mut Vec<CatalogIssue>) -> Vec<String> {
    let Some(present) = item else {
        return Vec::new();
    };
    let mut strings: Vec<String> = Vec::new();
    let mut valid: bool = present.as_array().is_some();
    if let Some(array) = present.as_array() {
        for value in array.iter() {
            match value.as_str() {
                Some(text) => strings.push(text.to_string()),
                None => valid = false,
            }
        }
    }
    if !valid {
        issues.push(CatalogIssue {
            location: key.to_string(),
            problem: "is not a list of strings".to_string(),
            remedy: format!("write `{key} = [\"name\", \"name@1.2.3\"]`"),
        });
        return Vec::new();
    }
    return strings;
}

/// Reads a boolean setting; `fallback` when absent or invalid.
fn read_boolean(key: &str, item: Option<&Item>, fallback: bool, issues: &mut Vec<CatalogIssue>) -> bool {
    let Some(present) = item else {
        return fallback;
    };
    return match present.as_bool() {
        Some(value) => value,
        None => {
            issues.push(CatalogIssue {
                location: key.to_string(),
                problem: "is not a boolean".to_string(),
                remedy: format!("write `{key} = true` or `{key} = false`"),
            });
            fallback
        }
    };
}

/// What:     `fn read_minutes(...) -> Option<u64>` reads a non-negative whole number of minutes. TOML integers and
///           floats with no fraction pass (the TypeScript tool read both as a JS number); a quoted number does not.
/// Why:      `None` means "absent or invalid" so the caller can apply the default; invalid input is also reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readMinutes(args: { key: string; value: unknown; issues: Issue[] }): number | typeof NO_MINUTES
/// ```
fn read_minutes(key: &str, item: Option<&Item>, issues: &mut Vec<CatalogIssue>) -> Option<u64> {
    let present: &Item = item?;
    let mut minutes: Option<u64> = None;
    if let Some(value) = present.as_integer() {
        minutes = u64::try_from(value).ok();
    } else if let Some(value) = present.as_float()
        && value >= 0.0
        && value.fract() == 0.0
        && value <= MAX_SAFE_INTEGER
    {
        minutes = Some(value as u64);
    }
    if minutes.is_none() {
        issues.push(CatalogIssue {
            location: key.to_string(),
            problem: "is not a non-negative whole number of minutes".to_string(),
            remedy: format!("write `{key} = 1440`"),
        });
    }
    return minutes;
}

/// Reads the trust policy mode; `off` when absent (pnpm's default).
fn read_trust_policy(item: Option<&Item>, issues: &mut Vec<CatalogIssue>) -> TrustPolicy {
    let Some(present) = item else {
        return TrustPolicy::Off;
    };
    return match present.as_str() {
        Some("no-downgrade") => TrustPolicy::NoDowngrade,
        Some("off") => TrustPolicy::Off,
        _ => {
            let shown: String = match present.as_str() {
                Some(text) => quote(text),
                None => present.to_string().trim().to_string(),
            };
            issues.push(CatalogIssue {
                location: "trustPolicy".to_string(),
                problem: format!("{shown} is not a trust policy"),
                remedy: "write `trustPolicy = \"no-downgrade\"` or `trustPolicy = \"off\"`".to_string(),
            });
            TrustPolicy::Off
        }
    };
}

/// What:     `pub fn read_settings(root: &dyn TableLike, issues: &mut Vec<CatalogIssue>) -> Settings` reads every
///           setting with pnpm's names and defaults. `minimumReleaseAgeStrict` defaults to true only when
///           `minimumReleaseAge` is set explicitly, as pnpm documents.
/// Why:      Issues are pushed in the order the TypeScript reader reported them: the two minute settings first, then
///           the rest in declaration order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readSettings(args: { root: Record<string, unknown>; issues: Issue[] }): RustCatalogSettings
/// ```
pub fn read_settings(root: &dyn TableLike, issues: &mut Vec<CatalogIssue>) -> Settings {
    let explicit_age: Option<u64> = read_minutes("minimumReleaseAge", root.get("minimumReleaseAge"), issues);
    let ignore_after: Option<u64> = read_minutes("trustPolicyIgnoreAfter", root.get("trustPolicyIgnoreAfter"), issues);
    let catalog_prune: bool = read_boolean("catalogPrune", root.get("catalogPrune"), false, issues);
    let age_exclude: Vec<String> = read_string_list("minimumReleaseAgeExclude", root.get("minimumReleaseAgeExclude"), issues);
    let age_exclude_prune: bool = read_boolean(
        "minimumReleaseAgeExcludePrune",
        root.get("minimumReleaseAgeExcludePrune"),
        false,
        issues,
    );
    let ignore_missing: bool = read_boolean(
        "minimumReleaseAgeIgnoreMissingTime",
        root.get("minimumReleaseAgeIgnoreMissingTime"),
        true,
        issues,
    );
    let strict: bool = read_boolean(
        "minimumReleaseAgeStrict",
        root.get("minimumReleaseAgeStrict"),
        explicit_age.is_some(),
        issues,
    );
    let trust_policy: TrustPolicy = read_trust_policy(root.get("trustPolicy"), issues);
    let trust_exclude: Vec<String> = read_string_list("trustPolicyExclude", root.get("trustPolicyExclude"), issues);
    let trust_exclude_prune: bool = read_boolean(
        "trustPolicyExcludePrune",
        root.get("trustPolicyExcludePrune"),
        false,
        issues,
    );
    return Settings {
        catalog_prune,
        minimum_release_age: explicit_age.unwrap_or(DEFAULT_MINIMUM_RELEASE_AGE),
        minimum_release_age_exclude: age_exclude,
        minimum_release_age_exclude_prune: age_exclude_prune,
        minimum_release_age_ignore_missing_time: ignore_missing,
        minimum_release_age_strict: strict,
        trust_policy,
        trust_policy_exclude: trust_exclude,
        trust_policy_exclude_prune: trust_exclude_prune,
        trust_policy_ignore_after: ignore_after,
    };
}
