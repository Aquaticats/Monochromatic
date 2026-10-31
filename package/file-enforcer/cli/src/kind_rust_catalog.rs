//! What:
//!  The `rust-catalog` rule kind:
//! render `rust-catalog.toml` and every Cargo workspace root's `[workspace.dependencies]` table in one pass.
//! Why:
//!  The dependency-catalog library is a pure function from file contents to expected contents.
//!  This rule reads the catalog and every manifest through the virtual view,
//! so the manifests it sees already carry what the `toml-keys` rules planned,
//! and plans the catalog and all manifests together:
//! one failure writes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function rustCatalogRule(catalog: string, manifests: string[]): Rule
//! ```

/// Imports `Error`,
/// `ManifestFile`,
/// `SyncInput`,
/// `sync` from `monochromatic_dependency_catalog`.
use monochromatic_dependency_catalog::{Error, ManifestFile, SyncInput, sync};
/// Imports `RuleConfig` from `monochromatic_file_enforcer::config_types`.
use monochromatic_file_enforcer::config_types::RuleConfig;
/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `EntryKind` from `monochromatic_file_enforcer::real_fs`.
use monochromatic_file_enforcer::real_fs::EntryKind;
/// Imports `Change`,
/// `Rule`,
/// `RuleBuilder`,
/// `RulePlan` from `monochromatic_file_enforcer::rule`.
use monochromatic_file_enforcer::rule::{Change, Rule, RuleBuilder, RulePlan};
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;
/// Imports `expand_glob` from `monochromatic_file_enforcer::view_glob`.
use monochromatic_file_enforcer::view_glob::expand_glob;

/// What:
///  Builds a `RustCatalogRule` from a decoded `rust-catalog` config entry.
/// Why:
///  Registered under the kind `rust-catalog`,
/// replacing the placeholder that warns `kind-not-wired`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RustCatalogBuilder implements RuleBuilder {}
/// ```
pub struct RustCatalogBuilder;

/// The `impl RuleBuilder for RustCatalogBuilder` block.
impl RuleBuilder for RustCatalogBuilder {
    /// Build the rule,
    /// or explain that the entry is another kind.
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        match config {
            RuleConfig::RustCatalog { id, catalog, manifests } => {
                return Ok(Box::new(RustCatalogRule { id: id.clone(), catalog: catalog.clone(), manifests: manifests.clone() }));
            }
            other => {
                let message = format!("the rust-catalog builder was given a rule of kind {}", other.kind());
                return Err(Diagnostic::new("kind-unknown", &message).with_rule(other.id()));
            }
        }
    }
}

/// What:
///  A planned `rust-catalog` rule.
/// Why:
///  Holds the catalog path and the manifest globs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RustCatalogRule implements Rule { id: string; catalog: string; manifests: string[] }
/// ```
pub struct RustCatalogRule {
    /// Rule id.
    pub id: String,
    /// Repo-relative catalog path.
    pub catalog: String,
    /// Manifest globs.
    pub manifests: Vec<String>,
}

/// What:
///  Turn a library error into the diagnostic the run reports.
/// Why:
///  The library's messages already name the file and the fix;
/// the code says which stage failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function diagnose(error: CatalogError, catalog: string, rule: string): Diagnostic
/// ```
fn diagnose(error: &Error, catalog: &str, rule: &str) -> Diagnostic {
    let message = error.to_string();
    match error {
        Error::Toml { path, .. } => return Diagnostic::new("manifest-invalid", &message).with_path(path).with_rule(rule),
        Error::Catalog { source_path, .. } => {
            return Diagnostic::new("catalog-invalid", &message).with_path(source_path).with_rule(rule);
        }
        Error::Sync { .. } => return Diagnostic::new("catalog-sync-failed", &message).with_path(catalog).with_rule(rule),
    }
}

/// The `impl RustCatalogRule` block.
impl RustCatalogRule {
    /// What:
    ///  Read the catalog through the view.
    /// Why:
    ///  A missing catalog is a failure,
    /// not an empty catalog,
    /// because every manifest table is derived from it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readCatalog(view: View): string
    /// ```
    fn read_catalog(&self, view: &mut View) -> Result<String, Diagnostic> {
        match view.read_text(&self.catalog)? {
            Some(text) => return Ok(text),
            None => {
                let message = format!("the dependency catalog {} does not exist", self.catalog);
                return Err(Diagnostic::new("catalog-missing", &message).with_path(&self.catalog).with_rule(&self.id));
            }
        }
    }

    /// What:
    ///  Read every manifest the globs match,
    /// sorted by path.
    /// Why:
    ///  The library takes file contents;
    /// the view supplies the planned content of manifests earlier rules rewrote.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readManifests(view: View): ManifestFile[]
    /// ```
    fn read_manifests(&self, view: &mut View) -> Result<Vec<ManifestFile>, Diagnostic> {
        let mut paths: Vec<String> = Vec::new();
        for pattern in &self.manifests {
            paths.extend(expand_glob(view, pattern, EntryKind::File)?);
        }
        paths.sort();
        paths.dedup();
        let mut files: Vec<ManifestFile> = Vec::new();
        for path in paths {
            if let Some(source) = view.read_text(&path)? {
                files.push(ManifestFile { path, source });
            }
        }
        return Ok(files);
    }
}

/// The `impl Rule for RustCatalogRule` block.
impl Rule for RustCatalogRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "rust-catalog";
    }

    /// Transforms manifests other rules (the `toml-keys` kind) also transform,
    /// so it shares their output files.
    fn composes(&self) -> bool {
        return true;
    }

    /// What:
    ///  Sync the catalog and every manifest,
    /// and plan a write for each file whose expected text differs.
    /// Why:
    ///  One library call sees all manifests together,
    /// because adoption of literal versions into the catalog depends on every crate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let catalog_source = self.read_catalog(view)?;
        let manifests = self.read_manifests(view)?;
        let input = SyncInput { catalog_path: &self.catalog, catalog_source: &catalog_source, manifests: &manifests };
        let output = match sync(&input) {
            Ok(output) => output,
            Err(error) => return Err(diagnose(&error, &self.catalog, &self.id)),
        };
        let mut plan = RulePlan::default();
        for file in output.drift() {
            plan.changes.push(Change::Write { path: file.path.clone(), content: file.expected.clone().into_bytes(), mode: None });
        }
        for warning in &output.warnings {
            plan.warnings.push(Diagnostic::new("catalog-warning", warning).with_rule(&self.id));
        }
        return Ok(plan);
    }
}

/// Tests for the rule.
#[cfg(test)]
#[path = "kind_rust_catalog_tests.rs"]
mod tests;
