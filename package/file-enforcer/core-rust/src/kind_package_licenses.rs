//! What:
//!  The `package-licenses` rule kind: copy the license texts a package needs into its `LICENSES` directory.
//! Why:
//!  Each package's manifest license expression decides which texts must ship in its artifact; texts the tool owns
//!  (the keys of `texts`) that are no longer required are removed, and any other file in the directory is left alone.
//!  A manifest with no license expression is not managed at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function packageLicensesRule(manifests: string[], directory: string, texts: Record<string, string>, requires: Requirement[]): Rule
//! ```

use crate::config_parts::LicenseRequirement;
use crate::diagnostic::Diagnostic;
use crate::real_fs::EntryKind;
use crate::rule::{Change, Rule, RulePlan};
use crate::view::View;
use crate::view_glob::expand_glob;
use std::collections::{BTreeMap, BTreeSet};

/// What:
///  A planned `package-licenses` rule.
/// Why:
///  Holds the manifest globs, the directory name, the owned texts and the requirement table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PackageLicensesRule implements Rule { ... }
/// ```
pub struct PackageLicensesRule {
    /// Rule id.
    pub id: String,
    /// Manifest globs.
    pub manifests: Vec<String>,
    /// Directory name beside each manifest.
    pub directory: String,
    /// License id to source path.
    pub texts: BTreeMap<String, String>,
    /// Which texts an expression needs.
    pub requires: Vec<LicenseRequirement>,
}

/// What:
///  Read the license expression out of one manifest.
///  `Option<String>` is "text or nothing" (TS: `string | undefined`).
/// Why:
///  `package.json` holds it under `license`, `Cargo.toml` under `[package] license`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function licenseOf(manifestPath: string, text: string): string | undefined
/// ```
fn license_of(manifest: &str, text: &str) -> Result<Option<String>, Diagnostic> {
    if manifest.ends_with("package.json") {
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(value) => value,
            Err(error) => {
                let message = format!("{manifest} is not valid JSON: {error}");
                return Err(Diagnostic::new("manifest-invalid", &message).with_path(manifest));
            }
        };
        return Ok(value.get("license").and_then(serde_json::Value::as_str).map(str::to_string));
    }
    let document: toml_edit::DocumentMut = match text.parse() {
        Ok(document) => document,
        Err(error) => {
            let message = format!("{manifest} is not valid TOML: {error}");
            return Err(Diagnostic::new("manifest-invalid", &message).with_path(manifest));
        }
    };
    let license = document.get("package").and_then(|package| return package.get("license")).and_then(|item| return item.as_str());
    return Ok(license.map(str::to_string));
}

impl Rule for PackageLicensesRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "package-licenses";
    }

    /// What:
    ///  Work out each package directory's required texts, copy them, and prune owned texts that are not required.
    /// Why:
    ///  A package with both a `package.json` and a `Cargo.toml` needs the union of what each asks for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let mut required: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for pattern in &self.manifests {
            for manifest in expand_glob(view, pattern, EntryKind::File)? {
                let package_dir = manifest.rsplit_once('/').map(|pair| return pair.0.to_string()).unwrap_or_default();
                let text = view.read_text(&manifest)?.unwrap_or_default();
                // A manifest without a license expression is not managed: its texts are neither added nor pruned.
                if let Some(expression) = license_of(&manifest, &text)? {
                    let wanted = required.entry(package_dir).or_default();
                    for requirement in &self.requires {
                        if expression.contains(&requirement.contains) {
                            for license in &requirement.texts {
                                wanted.insert(license.clone());
                            }
                        }
                    }
                }
            }
        }
        let mut plan = RulePlan::default();
        for (package_dir, wanted) in &required {
            for (license, source) in &self.texts {
                let target = format!("{package_dir}/{}/{license}.txt", self.directory);
                if wanted.contains(license) {
                    let Some(content) = view.read_bytes(source)? else {
                        let message = format!("the license text {source} does not exist");
                        return Err(Diagnostic::new("input-missing", &message).with_path(source));
                    };
                    plan.changes.push(Change::Write { path: target, content, mode: None });
                } else if view.mode_of(&target)?.is_some() {
                    plan.changes.push(Change::Remove { path: target });
                }
            }
        }
        return Ok(plan);
    }
}
