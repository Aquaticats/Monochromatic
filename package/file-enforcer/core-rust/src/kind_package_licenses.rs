//! What:
//!  The `package-licenses` rule kind:
//! copy the license texts a package needs into its `LICENSES` directory.
//! Why:
//!  Each package's manifest license expression decides which texts must ship in its artifact;
//! texts the tool owns
//!  (the keys of `texts` and of `packageTexts`) that are no longer required are removed,
//! and any other file in the directory is left alone.
//!  A manifest with no license expression is not managed at all.
//!  A requirement names a whole SPDX id of the expression,
//! never a substring of it,
//! so an `AGPL-3.0-or-later` manifest never pulls the `GPL-3.0-or-later` text.
//!  A text under `texts` comes from one repository-wide source and serves every package.
//!  A text under `packageTexts` is the copy each package carries itself:
//! the rule reads it and never writes one,
//! because an MIT text names the author of the crate a package forked,
//! so it only removes that copy once the expression stops naming the id.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function packageLicensesRule(manifests: string[], directory: string, texts: Record<string, string>, packageTexts: Record<string, string>, requires: Requirement[]): Rule
//! ```

/// Imports `LicenseRequirement` from `crate::config_parts`.
use crate::config_parts::LicenseRequirement;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `parent_directory` from `crate::path_text`.
use crate::path_text::parent_directory;
/// Imports `EntryKind` from `crate::real_fs`.
use crate::real_fs::EntryKind;
/// Imports `Change`,
/// `Rule`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{Change, Rule, RulePlan};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `expand_glob` from `crate::view_glob`.
use crate::view_glob::expand_glob;
/// Imports `BTreeMap`,
/// `BTreeSet` from `std::collections`.
use std::collections::{BTreeMap, BTreeSet};

/// What:
///  A planned `package-licenses` rule.
/// Why:
///  Holds the manifest globs,
/// the directory name,
/// the owned texts and the requirement table.
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
    /// License id to the repository-wide source path of its text.
    pub texts: BTreeMap<String, String>,
    /// License id to the path of the text inside each package directory;
    /// each package's own copy is that text.
    pub package_texts: BTreeMap<String, String>,
    /// Which texts an expression needs.
    pub requires: Vec<LicenseRequirement>,
}

/// What:
///  Read the license expression out of one manifest.
///  `Option<String>` is "text or nothing" (TS:
/// `string | undefined`).
/// Why:
///  `package.json` holds it under `license`,
/// `Cargo.toml` under `[package] license`.
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
    let Some(package) = document.get("package") else { return Ok(None) };
    let Some(item) = package.get("license") else { return Ok(None) };
    return Ok(item.as_str().map(str::to_string));
}

/// What:
///  Whether a character separates two SPDX ids.
/// Why:
///  `split` takes this instead of a closure,
/// because a closure that captures nothing is a named function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSpdxSeparator(character: string): boolean
/// ```
fn is_spdx_separator(character: char) -> bool {
    return character.is_whitespace() || matches!(character, '(' | ')');
}

/// What:
///  Whether a split token is a license id rather than an operator or an empty piece.
/// Why:
///  The operators `AND`,
/// `OR` and `WITH` join ids and are not ids themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSpdxOperand(token: string): boolean
/// ```
fn is_spdx_operand(token: &&str) -> bool {
    return !token.is_empty() && !matches!(*token, "AND" | "OR" | "WITH");
}

/// What:
///  The SPDX license ids a manifest license expression names,
/// each read whole.
/// Why:
///  A requirement names an id,
/// so a substring match would let `AGPL-3.0-or-later` satisfy a `GPL-3.0-or-later` requirement.
///  Operators and parentheses split the ids and are not ids themselves;
/// one linear pass keeps this total for an expression of any length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spdxOperands(expression: string): Set<string>
/// ```
fn spdx_operands(expression: &str) -> BTreeSet<&str> {
    return expression.split(is_spdx_separator).filter(is_spdx_operand).collect();
}

/// What:
///   How the `package-licenses` kind plugs into the engine:
/// its id,
/// its kind name and its plan.
/// Why:
///   The plan reads each manifest's license expression,
/// copies the required texts,
/// and removes only the texts this rule owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PackageLicensesRule implements Rule { id(); kind(); plan(view) }
/// ```
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
    ///  Work out each package directory's required texts,
    /// copy those with a repository-wide source,
    /// check those each package carries itself,
    /// and prune owned texts that are not required.
    /// Why:
    ///  A package with both a `package.json` and a `Cargo.toml` needs the union of what each asks for.
    ///  A text a package carries itself is never written here:
    /// the package's own copy is the text,
    /// so there is nothing to copy over it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let mut required: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for pattern in &self.manifests {
            for manifest in expand_glob(view, pattern, EntryKind::File)? {
                let package_dir = parent_directory(&manifest).to_string();
                let text = view.read_text(&manifest)?.unwrap_or_default();
                // A manifest without a license expression is not managed: its texts are neither added nor pruned.
                if let Some(expression) = license_of(&manifest, &text)? {
                    let operands = spdx_operands(&expression);
                    let wanted = required.entry(package_dir).or_default();
                    for requirement in &self.requires {
                        if operands.contains(requirement.operand.as_str()) {
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
            for (license, relative) in &self.package_texts {
                let target = format!("{package_dir}/{relative}");
                if wanted.contains(license) {
                    own_text(view, &target)?;
                } else if view.mode_of(&target)?.is_some() {
                    plan.changes.push(Change::Remove { path: target });
                }
            }
        }
        return Ok(plan);
    }
}

/// What:
///  Read the text one package carries itself,
///  and stop the run when the package carries no usable one.
/// Why:
///  The rule writes this file only when it removes it,
/// because the text names the author of the crate the package forked,
/// and the rule knows no such author.
/// So a package that needs the text and carries none stops the run,
/// and so does a text that states nothing,
/// because either way the package would publish without a usable license text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownText(view: View, target: string): void
/// ```
fn own_text(view: &mut View, target: &str) -> Result<(), Diagnostic> {
    let Some(content) = view.read_bytes(target)? else {
        let message = format!(
            "{target} does not exist, and a package text is the copy its package carries: the copyright line in it names the author of the crate the package forked. Copy that author's license text from the upstream package to {target}."
        );
        return Err(Diagnostic::new("license-text-missing", &message).with_path(target));
    };
    if is_blank(&content) {
        let message = format!(
            "{target} holds no text, so the package would ship a license file that grants nothing. Write the author's license text there."
        );
        return Err(Diagnostic::new("license-text-blank", &message).with_path(target));
    }
    return Ok(());
}

/// What:
///  Whether a text holds nothing but whitespace.
/// Why:
///  An empty license file says nothing about the terms it is meant to carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isBlank(content: Uint8Array): boolean
/// ```
fn is_blank(content: &[u8]) -> bool {
    return content.iter().all(u8::is_ascii_whitespace);
}
