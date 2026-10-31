//! What:
//! The pure sync of crate manifests with `rust-catalog.toml`:
//! file contents in,
//! expected file contents,
//! drift and warnings out.
//! Why:
//! This is the library's whole public job.
//! It reads nothing and writes nothing:
//! the Rust file-enforcer
//! engine passes the catalog text and every manifest text,
//! then writes (write mode) or diffs (check mode)
//! the returned contents.
//! Every problem is collected first,
//! so a failed sync changes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.ts
//! export function syncRustCatalog(args: { catalogPath: string; catalogSource: string; manifests: ManifestFile[] }): SyncResult
//! ```

/// Imports `std::collections::HashMap`.
use std::collections::HashMap;

/// Imports `toml_edit::DocumentMut`.
use toml_edit::DocumentMut;

/// Imports `crate::adopt::{AdoptionPlan, plan_adoption}`.
use crate::adopt::{AdoptionPlan, plan_adoption};
/// Imports `crate::catalog::parse_catalog`.
use crate::catalog::parse_catalog;
/// Imports `crate::change::Change`.
use crate::change::Change;
/// Imports `crate::crate_info::{CrateInfo, ManifestFile, read_crate_info}`.
use crate::crate_info::{CrateInfo, ManifestFile, read_crate_info};
/// Imports `crate::edit_catalog::edit_catalog`.
use crate::edit_catalog::edit_catalog;
/// Imports `crate::edit_manifest::{TableChange, inherit_site, item_mut, set_path_version, sync_generated_table}`.
use crate::edit_manifest::{TableChange, inherit_site, item_mut, set_path_version, sync_generated_table};
/// Imports `crate::error::{Error, SyncIssue}`.
use crate::error::{Error, SyncIssue};
/// Imports `crate::resolve::{PendingEdit, Resolved, Usage, resolve_crate}`.
use crate::resolve::{PendingEdit, Resolved, Usage, resolve_crate};
/// Imports `crate::types::CatalogDocument`.
use crate::types::CatalogDocument;

/// What:
/// `pub struct SyncInput<'a> { ... }` is everything a sync reads.
/// `'a` is a lifetime:
/// it says the struct
/// only borrows its text and slice from the caller and must not outlive them.
/// Why:
/// Borrowing avoids copying every manifest;
/// the caller keeps ownership of its buffers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SyncInput = { readonly catalogPath: string; readonly catalogSource: string; readonly manifests: readonly ManifestFile[] };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SyncInput<'a> {
    /// Catalog path for messages and warnings.
    pub catalog_path: &'a str,
    /// Exact text of `rust-catalog.toml`.
    pub catalog_source: &'a str,
    /// Every first-party crate manifest.
    pub manifests: &'a [ManifestFile],
}

/// What:
/// `pub struct FileResult { ... }` is the expected state of one file.
/// Why:
/// `changes` is empty exactly when `expected == actual`,
/// so check mode can name why a file drifts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileResult = { path: string; actual: string; expected: string; changes: Change[] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    /// Path as passed in.
    pub path: String,
    /// Text as passed in.
    pub actual: String,
    /// Text the sync wants the file to hold.
    pub expected: String,
    /// Edits that turn `actual` into `expected`.
    pub changes: Vec<Change>,
}

/// Methods of `FileResult`.
impl FileResult {
    /// Whether the file differs from what the sync wants.
    pub fn drifts(&self) -> bool {
        return self.actual != self.expected;
    }
}

/// The result of one sync pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOutput {
    /// `rust-catalog.toml` after adoption,
    /// floor raises and pruning.
    pub catalog: FileResult,
    /// Every manifest,
    /// in input order.
    pub manifests: Vec<FileResult>,
    /// Crate directories whose generated `[workspace.dependencies]` table changed.
    pub affected_crates: Vec<String>,
    /// Non-fatal notes,
    /// such as entries no crate uses while `catalogPrune` is off.
    pub warnings: Vec<String>,
}

/// Methods of `SyncOutput`.
impl SyncOutput {
    /// Every file (catalog first,
    /// then manifests) whose expected text differs from its actual text.
    pub fn drift(&self) -> Vec<&FileResult> {
        let mut drifting: Vec<&FileResult> = Vec::new();
        if self.catalog.drifts() {
            drifting.push(&self.catalog);
        }
        for manifest in &self.manifests {
            if manifest.drifts() {
                drifting.push(manifest);
            }
        }
        return drifting;
    }

    /// The expected text of one manifest by path.
    pub fn expected_manifest(&self, path: &str) -> Option<&str> {
        for manifest in &self.manifests {
            if manifest.path == path {
                return Some(&manifest.expected);
            }
        }
        return None;
    }
}

/// Dotted form of a prune path,
/// such as `catalogs.pinned.salsa`.
fn dotted(path: &[String]) -> String {
    return path.join(".");
}

/// Entry paths no crate uses:
/// default-catalog names first,
/// then each named catalog's names.
fn unused_paths(document: &CatalogDocument, default_catalog: &crate::types::EntryMap, usage: &Usage) -> Vec<Vec<String>> {
    let mut unused: Vec<Vec<String>> = Vec::new();
    for (name, _) in default_catalog.iter() {
        if !usage.default_names.contains(name) {
            unused.push(vec!["catalog".to_string(), name.clone()]);
        }
    }
    for (catalog_name, entries) in &document.catalogs {
        for (name, _) in entries.iter() {
            if !usage.named_names.contains(&(catalog_name.clone(), name.clone())) {
                unused.push(vec!["catalogs".to_string(), catalog_name.clone(), name.clone()]);
            }
        }
    }
    return unused;
}

/// Applies one crate's pending edits and generated table;
/// returns the new text and the changes made.
fn apply_crate(info: &mut CrateInfo, resolved: &Resolved) -> (String, Vec<Change>) {
    let mut changes: Vec<Change> = Vec::new();
    for edit in &resolved.edits {
        match edit {
            PendingEdit::Inherit { table, name, location } => {
                if let Some(item) = item_mut(info.document.as_table_mut(), table, name) {
                    inherit_site(item);
                    changes.push(Change::SiteInherited {
                        location: location.clone(),
                    });
                }
            }
            PendingEdit::PathVersion { table, name, version, location } => {
                if let Some(item) = item_mut(info.document.as_table_mut(), table, name)
                    && set_path_version(item, version)
                {
                    changes.push(Change::PathVersionWritten {
                        location: location.clone(),
                        version: format!("={version}"),
                    });
                }
            }
        }
    }
    match sync_generated_table(&mut info.document, &resolved.entries) {
        TableChange::Unchanged => {}
        TableChange::Written => changes.push(Change::GeneratedTableWritten),
        TableChange::Removed => changes.push(Change::GeneratedTableRemoved),
    }
    return (info.document.to_string(), changes);
}

/// What:
/// `pub fn sync(input: &SyncInput) -> Result<SyncOutput, Error>` brings every manifest and the catalog in
/// line:
/// adopts literal versions,
/// resolves each crate's generated table and internal path versions,
/// and
/// finds unused entries.
/// Why:
/// On `Err` nothing is partially applied:
/// the caller has no new text to write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncRustCatalog(args: { catalogPath; catalogSource; manifests }): SyncResult // throws RustCatalogError | RustCatalogSyncError
/// ```
pub fn sync(input: &SyncInput) -> Result<SyncOutput, Error> {
    let document: CatalogDocument = parse_catalog(input.catalog_source, input.catalog_path)?;
    let mut issues: Vec<SyncIssue> = Vec::new();
    let mut crates: Vec<CrateInfo> = Vec::new();
    for manifest in input.manifests {
        crates.push(read_crate_info(manifest, &mut issues)?);
    }
    let plan: AdoptionPlan = plan_adoption(&document, &crates, &mut issues);
    let mut usage: Usage = Usage::default();
    let mut resolutions: Vec<Resolved> = Vec::new();
    {
        let mut internal: HashMap<String, &CrateInfo> = HashMap::new();
        for info in &crates {
            internal.insert(info.crate_dir.clone(), info);
        }
        for info in &crates {
            resolutions.push(resolve_crate(info, &plan.default_catalog, &document, &internal, &mut usage, &mut issues));
        }
    }
    let unused: Vec<Vec<String>> = unused_paths(&document, &plan.default_catalog, &usage);
    if !issues.is_empty() {
        return Err(Error::Sync { issues });
    }
    let mut manifests: Vec<FileResult> = Vec::new();
    let mut affected_crates: Vec<String> = Vec::new();
    for (info, resolved) in crates.iter_mut().zip(resolutions.iter()) {
        let (expected, changes) = apply_crate(info, resolved);
        if changes.contains(&Change::GeneratedTableWritten) || changes.contains(&Change::GeneratedTableRemoved) {
            affected_crates.push(info.crate_dir.clone());
        }
        manifests.push(FileResult {
            path: info.manifest.path.clone(),
            actual: info.manifest.source.clone(),
            expected,
            changes,
        });
    }
    let prune: bool = document.settings.catalog_prune;
    let prunes: Vec<Vec<String>> = if prune { unused.clone() } else { Vec::new() };
    let mut catalog_document: DocumentMut = input
        .catalog_source
        .parse::<DocumentMut>()
        .map_err(|error: toml_edit::TomlError| return Error::Toml {
            path: input.catalog_path.to_string(),
            message: error.to_string(),
        })?;
    edit_catalog(&mut catalog_document, input.catalog_source, &plan, &prunes);
    let mut catalog_changes: Vec<Change> = Vec::new();
    for (name, _) in &plan.additions {
        catalog_changes.push(Change::CatalogEntryAdded { name: name.clone() });
    }
    for (name, requirement) in &plan.floor_raises {
        catalog_changes.push(Change::CatalogFloorRaised {
            name: name.clone(),
            requirement: requirement.clone(),
        });
    }
    for path in &prunes {
        catalog_changes.push(Change::CatalogEntryPruned { path: dotted(path) });
    }
    let mut warnings: Vec<String> = Vec::new();
    if !prune {
        for path in &unused {
            warnings.push(format!(
                "{}: {} is used by no crate; set catalogPrune = true to remove unused entries on sync",
                input.catalog_path,
                dotted(path)
            ));
        }
    }
    return Ok(SyncOutput {
        catalog: FileResult {
            path: input.catalog_path.to_string(),
            actual: input.catalog_source.to_string(),
            expected: catalog_document.to_string(),
            changes: catalog_changes,
        },
        manifests,
        affected_crates,
        warnings,
    });
}
