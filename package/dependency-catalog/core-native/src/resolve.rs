//! What:
//! Resolving one crate:
//! which catalog entry each of its dependencies uses,
//! which edits its sites need,
//! and
//! what its generated `[workspace.dependencies]` table must hold.
//! Why:
//! Resolution only reads;
//! it records the edits as plain data so every problem across every crate is
//! collected before any document is changed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.ts (resolve half)
//! function resolveCrate(args: { info; defaultCatalog; namedCatalogs; internal; usage; issues }): { entries: Map<string, GeneratedEntry>; splices: Splice[] }
//! ```

/// Imports `std::collections::{HashMap, HashSet}`.
use std::collections::{HashMap, HashSet};

/// Imports `crate::crate_info::CrateInfo`.
use crate::crate_info::CrateInfo;
/// Imports `crate::default_features::generated_entries`.
use crate::default_features::generated_entries;
/// Imports `crate::error::SyncIssue`.
use crate::error::SyncIssue;
/// Imports `crate::posix_path::join_normalized`.
use crate::posix_path::join_normalized;
/// Imports `crate::quote::quote`.
use crate::quote::quote;
/// Imports `crate::site::{GeneratedEntry, SiteKind, UseSite}`.
use crate::site::{GeneratedEntry, SiteKind, UseSite};
/// Imports `crate::types::{CatalogDocument, Entry, EntryMap}`.
use crate::types::{CatalogDocument, Entry, EntryMap};

/// What:
/// `pub enum PendingEdit { ... }` is one manifest edit decided during resolution and applied later.
/// Why:
/// Keeping edits as data lets the sync apply nothing when any crate has a problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingEdit = { kind: 'inherit'; table: string[]; name: string } | { kind: 'path-version'; table: string[]; name: string; version: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingEdit {
    /// Turn a literal site into `workspace = true`.
    Inherit {
        /// Dependency table path.
        table: Vec<String>,
        /// Dependency key.
        name: String,
        /// Dotted location for the change report.
        location: String,
    },
    /// Write the exact target version on an internal path dependency.
    PathVersion {
        /// Dependency table path.
        table: Vec<String>,
        /// Dependency key.
        name: String,
        /// Target crate's `package.version`.
        version: String,
        /// Dotted location for the change report.
        location: String,
    },
}

/// Which catalog entries any crate uses,
/// to find unused ones.
#[derive(Debug, Default)]
pub struct Usage {
    /// Default-catalog names in use.
    pub default_names: HashSet<String>,
    /// `(catalog name, dependency name)` pairs in use.
    pub named_names: HashSet<(String, String)>,
}

/// The outcome of resolving one crate.
#[derive(Debug)]
pub struct Resolved {
    /// The crate's generated entries,
    /// in site order.
    pub entries: Vec<(String, GeneratedEntry)>,
    /// Manifest edits to apply.
    pub edits: Vec<PendingEdit>,
}

/// Inserts or replaces a pair,
/// keeping the first position of an existing name.
fn upsert(pairs: &mut Vec<(String, Entry)>, name: &str, entry: Entry) {
    for pair in pairs.iter_mut() {
        if pair.0 == name {
            pair.1 = entry;
            return;
        }
    }
    pairs.push((name.to_string(), entry));
}

/// Resolves a path site,
/// pushing an edit or an issue.
fn resolve_path_site(info: &CrateInfo, site: &UseSite, internal: &HashMap<String, &CrateInfo>, edits: &mut Vec<PendingEdit>, issues: &mut Vec<SyncIssue>) {
    let target_dir: String = join_normalized(&info.crate_dir, &site.path);
    let target: Option<&&CrateInfo> = internal.get(&target_dir);
    match target {
        Some(found) if !found.package_version.is_empty() => edits.push(PendingEdit::PathVersion {
            table: site.table.clone(),
            name: site.name.clone(),
            version: found.package_version.clone(),
            location: site.location(),
        }),
        _ => issues.push(SyncIssue {
            file: info.manifest.path.clone(),
            location: site.location(),
            problem: format!("path {} does not point at a first-party crate with a package.version", quote(&site.path)),
            remedy: "point `path` at a crate directory under package/ that declares `version`, or depend on a registry release instead".to_string(),
        }),
    }
}

/// What:
/// `pub fn resolve_crate(...) -> Resolved` resolves one crate against the post-adoption default catalog
/// and the named catalogs,
/// recording usage and issues.
/// Why:
/// A literal without an entry already failed adoption,
/// which reported why;
/// a second "missing entry" issue
/// would blame crates that are not at fault.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCrate(args: { info; defaultCatalog; namedCatalogs; internal; usage; issues }): { entries; splices }
/// ```
pub fn resolve_crate(
    info: &CrateInfo,
    default_catalog: &EntryMap,
    document: &CatalogDocument,
    internal: &HashMap<String, &CrateInfo>,
    usage: &mut Usage,
    issues: &mut Vec<SyncIssue>,
) -> Resolved {
    let mut resolved: Vec<(String, Entry)> = Vec::new();
    let mut edits: Vec<PendingEdit> = Vec::new();
    let mut catalog_names: HashSet<&str> = HashSet::new();
    for site in &info.sites {
        if site.kind == SiteKind::Path {
            resolve_path_site(info, site, internal, &mut edits, issues);
            continue;
        }
        catalog_names.insert(&site.name);
        let catalog_name: Option<&str> = info.selection(&site.name);
        let entry: Option<&Entry> = match catalog_name {
            None => default_catalog.get(&site.name),
            Some(chosen) => match document.named(chosen) {
                Some(entries) => entries.get(&site.name),
                None => None,
            },
        };
        if entry.is_none() && site.kind == SiteKind::Literal {
            continue;
        }
        let Some(found) = entry else {
            let problem: String = match catalog_name {
                None => format!("uses `workspace = true` but rust-catalog.toml [catalog] has no {} entry", site.name),
                Some(chosen) => format!("selects catalog \"{chosen}\", which has no {} entry", site.name),
            };
            issues.push(SyncIssue {
                file: info.manifest.path.clone(),
                location: site.location(),
                problem,
                remedy: format!("add {} to rust-catalog.toml, or write a version here and run the sync to adopt it", site.name),
            });
            continue;
        };
        match catalog_name {
            None => {
                usage.default_names.insert(site.name.clone());
            }
            Some(chosen) => {
                usage.named_names.insert((chosen.to_string(), site.name.clone()));
            }
        }
        upsert(&mut resolved, &site.name, found.clone());
        if site.kind == SiteKind::Literal {
            edits.push(PendingEdit::Inherit {
                table: site.table.clone(),
                name: site.name.clone(),
                location: site.location(),
            });
        }
    }
    for (name, catalog_name) in &info.selections {
        if !catalog_names.contains(name.as_str()) {
            issues.push(SyncIssue {
                file: info.manifest.path.clone(),
                location: format!("package.metadata.rust-catalog.catalogs.{name}"),
                problem: format!("selects catalog \"{catalog_name}\" for {name}, which this crate does not depend on"),
                remedy: "remove the selection".to_string(),
            });
        }
    }
    return Resolved {
        entries: generated_entries(info, &resolved, issues),
        edits,
    };
}
