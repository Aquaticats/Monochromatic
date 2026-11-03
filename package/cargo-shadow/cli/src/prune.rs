//! What:
//! Removes exclude selectors from `rust-catalog.toml` that no locked version needs,
//! when the matching prune setting is on.
//! Why:
//! `minimumReleaseAgeExcludePrune` and `trustPolicyExcludePrune` (pnpm-style) keep the exemption lists from collecting
//! entries for versions that are gone.
//! The edit keeps the file's comments and layout.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prune.ts
//! export function pruneExcludes({ source, settings, locked }): { source: string; removed: string[] }
//! ```

use std::collections::BTreeSet;

use monochromatic_dependency_catalog::Settings;
use toml_edit::{Array, DocumentMut, Item, Value};

use crate::lockfile::LockedVersion;
use crate::selectors::prune_selectors;

/// The result of pruning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pruned {
    /// The catalog text after pruning.
    pub source: String,
    /// `key: selector` for each removed selector.
    pub removed: Vec<String>,
}

/// Prunes the two exclude lists whose flags are on;
/// the text is returned unchanged when nothing is stale.
pub fn prune_excludes(source: &str, settings: &Settings, locked: &[LockedVersion]) -> Result<Pruned, String> {
    let keys: BTreeSet<String> = locked.iter().map(|version| { return version.key(); }).collect();
    let lists: [(&str, bool, &Vec<String>); 2] = [
        ("minimumReleaseAgeExclude", settings.minimum_release_age_exclude_prune, &settings.minimum_release_age_exclude),
        ("trustPolicyExclude", settings.trust_policy_exclude_prune, &settings.trust_policy_exclude),
    ];
    let mut document: DocumentMut = source.parse::<DocumentMut>().map_err(|error| { return error.to_string(); })?;
    let mut removed: Vec<String> = Vec::new();
    for (key, enabled, before) in lists {
        if !enabled {
            continue;
        }
        let after: Vec<String> = prune_selectors(before, &keys);
        if &after == before {
            continue;
        }
        for selector in before {
            if !after.contains(selector) {
                removed.push(format!("{key}: {selector}"));
            }
        }
        let mut array: Array = Array::new();
        for selector in &after {
            array.push(Value::from(selector.as_str()));
        }
        document[key] = Item::Value(Value::Array(array));
    }
    if removed.is_empty() {
        return Ok(Pruned { source: source.to_string(), removed });
    }
    return Ok(Pruned { source: document.to_string(), removed });
}
