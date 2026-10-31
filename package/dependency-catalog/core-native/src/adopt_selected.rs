//! What:     Checks literals whose crate selects a named catalog for that dependency.
//! Why:      A selected literal is not adopted into the default catalog; it must be replaceable by the named entry,
//!           or the sync would change what the crate resolves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // adopt-selected.ts
//! export function checkSelectedLiterals(args: { document: RustCatalogDocument; sites: UseSite[]; selections: Selections; issues: SyncIssue[] }): void
//! ```

use crate::adopt_shared::same_entry;
use crate::error::SyncIssue;
use crate::requirement::split_requirement;
use crate::site::UseSite;
use crate::types::{CatalogDocument, Entry, EntryMap};
use crate::version::format_version;

/// Whether the named entry can replace the literal.
fn compatible(entry: &Entry, literal: &Entry) -> bool {
    if let (Entry::Registry { version: named, .. }, Entry::Registry { version: wanted, .. }) = (entry, literal) {
        let Ok(split) = split_requirement(named) else {
            return false;
        };
        let floor: String = format_version(&split.version);
        return crate::requirement::satisfies(&floor, wanted).unwrap_or(false);
    }
    return same_entry(entry, literal);
}

/// What:     `pub fn check_selected_literals(...)` reports each selected literal the named entry cannot replace.
///           `selected` pairs a site with the named catalog its crate chose.
/// Why:      Missing entries and incompatible literals are issues, not silent rewrites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkSelectedLiterals(args: { document; sites; selections; issues }): void
/// ```
pub fn check_selected_literals(document: &CatalogDocument, selected: &[(&UseSite, &str)], issues: &mut Vec<SyncIssue>) {
    for (site, catalog_name) in selected {
        let named: Option<&EntryMap> = document.named(catalog_name);
        let entry: Option<&Entry> = match named {
            Some(entries) => entries.get(&site.name),
            None => None,
        };
        let Some(found) = entry else {
            issues.push(SyncIssue {
                file: site.manifest_path.clone(),
                location: site.location(),
                problem: format!("selects catalog \"{catalog_name}\", which has no {} entry", site.name),
                remedy: format!(
                    "add `{}` under `[catalogs.{catalog_name}]` in rust-catalog.toml, or remove the selection",
                    site.name
                ),
            });
            continue;
        };
        if !compatible(found, &site.literal) {
            issues.push(SyncIssue {
                file: site.manifest_path.clone(),
                location: site.location(),
                problem: format!("literal does not match the selected entry in [catalogs.{catalog_name}]"),
                remedy: format!(
                    "change the entry in [catalogs.{catalog_name}] or replace the literal with `workspace = true`"
                ),
            });
        }
    }
}
