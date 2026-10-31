//! What:     Parsing and validation of `rust-catalog.toml` text into a `CatalogDocument`.
//! Why:      Unlike the pnpm workspace reader, which warns and skips, this parser fails closed: one invalid entry
//!           returns an `Error::Catalog` naming every problem at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parse.ts
//! export function parseRustCatalog(args: { source: string; sourcePath: string }): RustCatalogDocument // throws RustCatalogError
//! ```

use toml_edit::{DocumentMut, Item, TableLike};

use crate::catalog_entry::parse_catalog_table;
use crate::catalog_settings::read_settings;
use crate::error::{CatalogIssue, Error};
use crate::types::{CatalogDocument, EntryMap, Settings};

/// Top-level keys the catalog file may declare, in the order the error message lists them.
const TOP_LEVEL_KEYS: [&str; 12] = [
    "catalog",
    "catalogPrune",
    "catalogs",
    "minimumReleaseAge",
    "minimumReleaseAgeExclude",
    "minimumReleaseAgeExcludePrune",
    "minimumReleaseAgeIgnoreMissingTime",
    "minimumReleaseAgeStrict",
    "trustPolicy",
    "trustPolicyExclude",
    "trustPolicyExcludePrune",
    "trustPolicyIgnoreAfter",
];

/// Validates the `[catalogs.<name>]` tables into named catalogs in source order.
fn parse_named_catalogs(item: Option<&Item>, issues: &mut Vec<CatalogIssue>) -> Vec<(String, EntryMap)> {
    let mut catalogs: Vec<(String, EntryMap)> = Vec::new();
    let Some(present) = item else {
        return catalogs;
    };
    let Some(table) = present.as_table_like() else {
        issues.push(CatalogIssue {
            location: "catalogs".to_string(),
            problem: "is not a table".to_string(),
            remedy: "declare named catalogs as `[catalogs.<name>]`".to_string(),
        });
        return catalogs;
    };
    for (catalog_name, catalog_item) in table.iter() {
        if catalog_name == "default" {
            issues.push(CatalogIssue {
                location: "catalogs.default".to_string(),
                problem: "repeats the default catalog".to_string(),
                remedy: "put default entries under `[catalog]`".to_string(),
            });
            continue;
        }
        let location: String = format!("catalogs.{catalog_name}");
        let entries: EntryMap = parse_catalog_table(&location, Some(catalog_item), true, issues);
        catalogs.push((catalog_name.to_string(), entries));
    }
    return catalogs;
}

/// What:     `pub fn parse_catalog(source: &str, source_path: &str) -> Result<CatalogDocument, Error>` parses and
///           validates catalog text. `source_path` only appears in messages.
/// Why:      The sync needs the validated document; callers that only want to check the file call it directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRustCatalog(args: { source: string; sourcePath: string }): RustCatalogDocument // throws RustCatalogError | TomlEditError
/// ```
pub fn parse_catalog(source: &str, source_path: &str) -> Result<CatalogDocument, Error> {
    let parsed: Result<DocumentMut, toml_edit::TomlError> = source.parse::<DocumentMut>();
    let document: DocumentMut = match parsed {
        Ok(value) => value,
        Err(error) => {
            return Err(Error::Toml {
                path: source_path.to_string(),
                message: error.to_string(),
            });
        }
    };
    let root: &dyn TableLike = document.as_table();
    let mut issues: Vec<CatalogIssue> = Vec::new();
    for (key, _) in root.iter() {
        if !TOP_LEVEL_KEYS.contains(&key) {
            issues.push(CatalogIssue {
                location: key.to_string(),
                problem: "is not a catalog table or a supported setting".to_string(),
                remedy: format!("remove it; supported keys are {}", TOP_LEVEL_KEYS.join(", ")),
            });
        }
    }
    let catalog: EntryMap = parse_catalog_table("catalog", root.get("catalog"), false, &mut issues);
    let catalogs: Vec<(String, EntryMap)> = parse_named_catalogs(root.get("catalogs"), &mut issues);
    let settings: Settings = read_settings(root, &mut issues);
    if !issues.is_empty() {
        return Err(Error::Catalog {
            source_path: source_path.to_string(),
            issues,
        });
    }
    return Ok(CatalogDocument {
        catalog,
        catalogs,
        settings,
    });
}
