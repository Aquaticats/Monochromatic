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

/// What:
/// `use std::collections::BTreeSet;` imports `BTreeSet` from the standard library so this file can name it unqualified.
/// Why:
/// `BTreeSet` is used by `prune_excludes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeSet } from 'std/collections';
/// ```
use std::collections::BTreeSet;

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `Settings` is used by `prune_excludes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;
/// What:
/// `use toml_edit::{Array, DocumentMut, Item, Value};` imports `Array`,
/// `DocumentMut`,
/// `Item`,
/// `Value` from the external crate `toml_edit` so this file can name them unqualified.
/// Why:
/// `Array` is used by `prune_excludes`;
/// `DocumentMut` is used by `prune_excludes`;
/// `Item` is used by `prune_excludes`;
/// `Value` is used by `prune_excludes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Array, DocumentMut, Item, Value } from 'toml_edit';
/// ```
use toml_edit::{Array, DocumentMut, Item, Value};

/// What:
/// `use crate::lockfile::LockedVersion;` imports `LockedVersion` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockedVersion` is used by `prune_excludes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;
/// What:
/// `use crate::selectors::prune_selectors;` imports `prune_selectors` from another module of this crate so this file can name it unqualified.
/// Why:
/// `prune_selectors` is used by `prune_excludes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { prune_selectors } from 'crate/selectors';
/// ```
use crate::selectors::prune_selectors;
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `prune_excludes`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;

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
    let keys: BTreeSet<String> = locked.iter().map(LockedVersion::key).collect();
    let lists: [(&str, bool, &Vec<String>); 2] = [
        ("minimumReleaseAgeExclude", settings.minimum_release_age_exclude_prune, &settings.minimum_release_age_exclude),
        ("trustPolicyExclude", settings.trust_policy_exclude_prune, &settings.trust_policy_exclude),
    ];
    let mut document: DocumentMut = source.parse::<DocumentMut>().map_err(text_of)?;
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
