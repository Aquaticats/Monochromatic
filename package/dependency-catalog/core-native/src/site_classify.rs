//! What:     Finding and classifying every dependency use site in one crate manifest.
//! Why:      The sync must know, for each dependency entry, whether it already uses the catalog, points at an
//!           internal crate, or holds a literal to adopt; anything else is reported with the fix.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scan.ts
//! export function dependencyTables(manifest: Record<string, unknown>): string[][]
//! export function classifySite(args: { manifestPath: string; table: string[]; name: string; value: unknown; issues: SyncIssue[] }): UseSite[]
//! ```

use toml_edit::{Item, TableLike};

use crate::error::SyncIssue;
use crate::site::{SiteKind, UseSite};
use crate::types::Entry;

/// Dependency table names Cargo reads, at top level and under `target.<cfg>`.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Keys a `workspace = true` use site may carry beside it (Cargo's own rule).
const INHERITED_SITE_KEYS: [&str; 4] = ["default-features", "features", "optional", "workspace"];

/// Keys a literal or path use site may carry; the catalog moves or keeps each.
const LITERAL_SITE_KEYS: [&str; 8] = ["default-features", "features", "git", "optional", "package", "path", "rev", "version"];

/// What:     `pub fn dependency_tables(root: &dyn TableLike) -> Vec<Vec<String>>` lists every dependency table path
///           present in a manifest, top-level first, then each `target.<cfg>` in source order.
/// Why:      Cargo's reading order is the order diagnostics and edits follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function dependencyTables(manifest: Record<string, unknown>): string[][]
/// ```
pub fn dependency_tables(root: &dyn TableLike) -> Vec<Vec<String>> {
    let mut paths: Vec<Vec<String>> = Vec::new();
    for table in DEPENDENCY_TABLES {
        if root.get(table).and_then(Item::as_table_like).is_some() {
            paths.push(vec![table.to_string()]);
        }
    }
    let Some(target) = root.get("target").and_then(Item::as_table_like) else {
        return paths;
    };
    for (cfg, body_item) in target.iter() {
        let Some(body) = body_item.as_table_like() else {
            continue;
        };
        for table in DEPENDENCY_TABLES {
            if body.get(table).and_then(Item::as_table_like).is_some() {
                paths.push(vec!["target".to_string(), cfg.to_string(), table.to_string()]);
            }
        }
    }
    return paths;
}

/// Resolves a key path to a table-like item, or `None` when any step is missing or not a table.
pub fn table_at<'a>(root: &'a dyn TableLike, path: &[String]) -> Option<&'a dyn TableLike> {
    let mut current: &'a dyn TableLike = root;
    for segment in path {
        current = current.get(segment)?.as_table_like()?;
    }
    return Some(current);
}

/// Placeholder literal for sites that are not literals.
fn no_literal() -> Entry {
    return Entry::Registry {
        version: String::new(),
        package: None,
    };
}

/// Pushes one classification problem.
fn report(issues: &mut Vec<SyncIssue>, file: &str, location: &str, problem: String, remedy: &str) {
    issues.push(SyncIssue {
        file: file.to_string(),
        location: location.to_string(),
        problem,
        remedy: remedy.to_string(),
    });
}

/// What:     `pub fn classify_site(...) -> Option<UseSite>` classifies one dependency entry, recording problems as
///           issues. `None` means the entry cannot be handled (the reason is already in `issues`).
/// Why:      The decision order matches the TypeScript classifier: string, `workspace = true`, unsupported keys,
///           `path`, `git`, `version`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function classifySite(args: { manifestPath: string; table: string[]; name: string; value: unknown; issues: SyncIssue[] }): UseSite[]
/// ```
pub fn classify_site(manifest_path: &str, table: &[String], name: &str, item: &Item, issues: &mut Vec<SyncIssue>) -> Option<UseSite> {
    let mut location_parts: Vec<&str> = table.iter().map(String::as_str).collect();
    location_parts.push(name);
    let location: String = location_parts.join(".");
    let value: Option<&dyn TableLike> = item.as_table_like();
    let disables: bool = match value {
        Some(entries) => entries.get("default-features").and_then(Item::as_bool) == Some(false),
        None => false,
    };
    let base = UseSite {
        manifest_path: manifest_path.to_string(),
        table: table.to_vec(),
        name: name.to_string(),
        kind: SiteKind::Literal,
        literal: no_literal(),
        path: String::new(),
        disables_default_features: disables,
    };
    if let Some(version) = item.as_str() {
        return Some(UseSite {
            literal: Entry::Registry {
                version: version.to_string(),
                package: None,
            },
            ..base
        });
    }
    let Some(entries) = value else {
        report(
            issues,
            manifest_path,
            &location,
            "is neither a version string nor a table".to_string(),
            "write `name = \"1.2\"` or `name = { workspace = true }`",
        );
        return None;
    };
    let keys: Vec<&str> = entries.iter().map(|pair| return pair.0).collect();
    if entries.get("workspace").and_then(Item::as_bool) == Some(true) {
        let extra: Vec<String> = keys
            .iter()
            .filter(|key: &&&str| return !INHERITED_SITE_KEYS.contains(key))
            .map(|key: &&str| return format!("`{key}`"))
            .collect();
        if !extra.is_empty() {
            report(
                issues,
                manifest_path,
                &location,
                format!("declares {} beside `workspace = true`", extra.join(", ")),
                "move version, git, rev, and package into rust-catalog.toml; keep only features, optional, and default-features here",
            );
            return None;
        }
        return Some(UseSite {
            kind: SiteKind::Catalog,
            ..base
        });
    }
    let unsupported: Vec<&str> = keys.iter().copied().filter(|key: &&str| return !LITERAL_SITE_KEYS.contains(key)).collect();
    if !unsupported.is_empty() {
        report(
            issues,
            manifest_path,
            &location,
            format!("declares {}, which the Rust catalog does not support", unsupported.join(", ")),
            "pin git dependencies with `rev`; registries other than crates.io and artifact dependencies need a decision record before the catalog can carry them",
        );
        return None;
    }
    let rename: Option<String> = entries.get("package").and_then(Item::as_str).map(str::to_string);
    if let Some(path) = entries.get("path").and_then(Item::as_str) {
        return Some(UseSite {
            kind: SiteKind::Path,
            path: path.to_string(),
            ..base
        });
    }
    if let Some(git) = entries.get("git").and_then(Item::as_str) {
        let Some(rev) = entries.get("rev").and_then(Item::as_str) else {
            report(
                issues,
                manifest_path,
                &location,
                "git dependency has no `rev`".to_string(),
                "pin it with `rev = \"<full commit hash>\"`",
            );
            return None;
        };
        return Some(UseSite {
            literal: Entry::Git {
                git: git.to_string(),
                rev: rev.to_string(),
                package: rename,
            },
            ..base
        });
    }
    if let Some(version) = entries.get("version").and_then(Item::as_str) {
        return Some(UseSite {
            literal: Entry::Registry {
                version: version.to_string(),
                package: rename,
            },
            ..base
        });
    }
    report(
        issues,
        manifest_path,
        &location,
        "has no version, git, path, or `workspace = true`".to_string(),
        "add a version, or `workspace = true` with a catalog entry",
    );
    return None;
}
