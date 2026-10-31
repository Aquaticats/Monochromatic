//! What:
//! Helpers shared by the adoption planners:
//! entry equality and the named-catalog remedy text.
//! Why:
//! Both the default-catalog planner and the named-catalog check need them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // adopt-shared.ts
//! export function sameEntry(args: { left: RustCatalogEntry; right: RustCatalogEntry }): boolean
//! export function namedCatalogRemedy(args: { site: UseSite; requirement: string }): string
//! ```

/// Imports `crate::quote::quote`.
use crate::quote::quote;
/// Imports `crate::site::UseSite`.
use crate::site::UseSite;
/// Imports `crate::types::Entry`.
use crate::types::Entry;

/// What:
/// `pub fn same_entry(left: &Entry, right: &Entry) -> bool` says whether two entries are the same source:
/// same kind,
/// same version or git pin,
/// same rename.
/// Why:
/// Adoption uses it to decide that a literal already matches the catalog.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function sameEntry(args: { left: Entry; right: Entry }): boolean
/// ```
pub fn same_entry(left: &Entry, right: &Entry) -> bool {
    if left.package() != right.package() {
        return false;
    }
    return match (left, right) {
        (Entry::Git { git: left_git, rev: left_rev, .. }, Entry::Git { git: right_git, rev: right_rev, .. }) => {
            left_git == right_git && left_rev == right_rev
        }
        (Entry::Registry { version: left_version, .. }, Entry::Registry { version: right_version, .. }) => {
            left_version == right_version
        }
        _ => false,
    };
}

/// What:
/// `pub fn named_catalog_remedy(site: &UseSite, requirement: &str) -> String` builds the fix text for a
/// literal that needs a named catalog entry plus a selection in the crate.
/// Why:
/// The message names both edits so the contributor can make them without reading the decision record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function namedCatalogRemedy(args: { site: UseSite; requirement: string }): string
/// ```
pub fn named_catalog_remedy(site: &UseSite, requirement: &str) -> String {
    return format!(
        "add `{} = {}` to a named catalog in rust-catalog.toml (for example `[catalogs.pinned]`) and select it in {} with `[package.metadata.rust-catalog.catalogs]` `{} = \"pinned\"`",
        site.name,
        quote(requirement),
        site.manifest_path,
        site.name
    );
}
