//! What:
//! Reading what the catalog sync needs from one crate manifest:
//! its package identity,
//! its named-catalog
//! selections,
//! and every dependency use site.
//! Why:
//! The parsed `DocumentMut` is kept inside `CrateInfo` so the edit step can change the same document
//! the read step classified,
//! and `to_string()` then reproduces every untouched byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // crate-info.ts
//! export type CrateInfo = { manifest: ManifestFile; crateDir: string; packageName: string; packageVersion: string; selections: Map<string, string>; sites: UseSite[] };
//! export function readCrateInfo(args: { manifest: ManifestFile; issues: SyncIssue[] }): CrateInfo
//! ```

/// Imports `toml_edit::{DocumentMut, Item, TableLike}`.
use toml_edit::{DocumentMut, Item, TableLike};

/// Imports `crate::error::{Error, SyncIssue}`.
use crate::error::{Error, SyncIssue};
/// Imports `crate::site::UseSite`.
use crate::site::UseSite;
/// Imports `crate::site_classify::{classify_site, dependency_tables, table_at}`.
use crate::site_classify::{classify_site, dependency_tables, table_at};

/// What:
/// `pub struct ManifestFile { ... }` is one crate manifest as the caller read it from disk.
/// Why:
/// `path` is the repository-relative,
/// forward-slash path (`package/ide/app/Cargo.toml`);
/// the library never
/// opens it,
/// it only derives the crate directory and names it in messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManifestFile = { readonly path: string; readonly source: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestFile {
    /// Repository-relative path with forward slashes.
    pub path: String,
    /// Exact file text.
    pub source: String,
}

/// What:
/// `pub struct CrateInfo { ... }` is one parsed manifest with its sites and selections.
/// Why:
/// `document` is mutable on purpose:
/// the sync edits it in place,
/// and untouched parts keep their bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CrateInfo = { manifest: ManifestFile; crateDir: string; packageName: string; packageVersion: string; selections: Map<string, string>; sites: UseSite[] };
/// ```
#[derive(Debug)]
pub struct CrateInfo {
    /// The manifest as read.
    pub manifest: ManifestFile,
    /// The parsed,
    /// editable document.
    pub document: DocumentMut,
    /// Directory of the manifest,
    /// repository-relative.
    pub crate_dir: String,
    /// `package.name`,
    /// or empty.
    pub package_name: String,
    /// `package.version`,
    /// or empty.
    pub package_version: String,
    /// Dependency name to selected named catalog,
    /// in source order.
    pub selections: Vec<(String, String)>,
    /// Every classified dependency use site.
    pub sites: Vec<UseSite>,
}

/// Methods of `CrateInfo`.
impl CrateInfo {
    /// The named catalog this crate selects for a dependency,
    /// if any.
    pub fn selection(&self, name: &str) -> Option<&str> {
        for (key, catalog_name) in &self.selections {
            if key == name {
                return Some(catalog_name);
            }
        }
        return None;
    }
}

/// Key path of the named-catalog selections inside a manifest.
const SELECTION_PATH: [&str; 4] = ["package", "metadata", "rust-catalog", "catalogs"];

/// Directory part of a forward-slash path (`a/b/Cargo.toml` gives `a/b`;
/// a bare file name gives `.`).
pub fn posix_dirname(path: &str) -> String {
    return match path.rfind('/') {
        Some(0) => "/".to_string(),
        Some(index) => path[..index].to_string(),
        None => ".".to_string(),
    };
}

/// Reads one string field of the `[package]` table;
/// empty when the table or the field is missing or not a string.
fn package_text(package: Option<&dyn TableLike>, key: &str) -> String {
    let found: Option<&str> = match package {
        Some(table) => table.get(key).and_then(Item::as_str),
        None => None,
    };
    return found.unwrap_or("").to_string();
}

/// Reads `[package.metadata.rust-catalog.catalogs]`,
/// the crate's named-catalog selections.
fn read_selections(manifest_path: &str, root: &dyn TableLike, issues: &mut Vec<SyncIssue>) -> Vec<(String, String)> {
    let mut path: Vec<String> = Vec::new();
    for part in SELECTION_PATH {
        path.push(part.to_string());
    }
    let mut selections: Vec<(String, String)> = Vec::new();
    let Some(table) = table_at(root, &path) else {
        return selections;
    };
    for (name, item) in table.iter() {
        match item.as_str() {
            Some(catalog_name) => selections.push((name.to_string(), catalog_name.to_string())),
            None => issues.push(SyncIssue {
                file: manifest_path.to_string(),
                location: format!("package.metadata.rust-catalog.catalogs.{name}"),
                problem: "is not a catalog name".to_string(),
                remedy: format!("write `{name} = \"<catalog name>\"` naming a `[catalogs.<name>]` table in rust-catalog.toml"),
            }),
        }
    }
    return selections;
}

/// Whether `workspace.dependencies` exists but is not a standard `[workspace.dependencies]` table.
fn workspace_dependencies_is_hand_written(root: &dyn TableLike) -> bool {
    let Some(workspace) = root.get("workspace").and_then(Item::as_table_like) else {
        return false;
    };
    let Some(dependencies) = workspace.get("dependencies") else {
        return false;
    };
    return match dependencies {
        Item::Table(table) => table.is_dotted() || (table.is_implicit() && !table.is_empty()),
        _ => true,
    };
}

/// What:
/// `pub fn read_crate_info(manifest: &ManifestFile, issues: &mut Vec<SyncIssue>) -> Result<CrateInfo, Error>`
/// parses one manifest and classifies its dependency entries.
/// Invalid TOML is a hard `Error::Toml`;
/// everything else becomes an issue so one run reports every problem.
/// Why:
/// A hand-written inline or dotted `workspace.dependencies` is reported,
/// because the sync only manages a
/// standard `[workspace.dependencies]` table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readCrateInfo(args: { manifest: ManifestFile; issues: SyncIssue[] }): CrateInfo // throws on invalid TOML
/// ```
pub fn read_crate_info(manifest: &ManifestFile, issues: &mut Vec<SyncIssue>) -> Result<CrateInfo, Error> {
    let parsed: Result<DocumentMut, toml_edit::TomlError> = manifest.source.parse::<DocumentMut>();
    let document: DocumentMut = match parsed {
        Ok(value) => value,
        Err(error) => {
            return Err(Error::Toml {
                path: manifest.path.clone(),
                message: error.to_string(),
            });
        }
    };
    let root: &dyn TableLike = document.as_table();
    let package: Option<&dyn TableLike> = root.get("package").and_then(Item::as_table_like);
    if workspace_dependencies_is_hand_written(root) {
        issues.push(SyncIssue {
            file: manifest.path.clone(),
            location: "workspace.dependencies".to_string(),
            problem: "is written as an inline table or dotted keys".to_string(),
            remedy: "delete it; file-enforcer generates a `[workspace.dependencies]` table from rust-catalog.toml".to_string(),
        });
    }
    let mut sites: Vec<UseSite> = Vec::new();
    for table_path in dependency_tables(root) {
        let Some(table) = table_at(root, &table_path) else {
            continue;
        };
        for (name, item) in table.iter() {
            if let Some(site) = classify_site(&manifest.path, &table_path, name, item, issues) {
                sites.push(site);
            }
        }
    }
    let selections: Vec<(String, String)> = read_selections(&manifest.path, root, issues);
    let package_name: String = package_text(package, "name");
    let package_version: String = package_text(package, "version");
    return Ok(CrateInfo {
        manifest: manifest.clone(),
        crate_dir: posix_dirname(&manifest.path),
        package_name,
        package_version,
        selections,
        sites,
        document,
    });
}
