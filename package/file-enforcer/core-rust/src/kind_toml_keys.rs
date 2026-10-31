//! What:
//!  The `toml-keys` rule kind:
//! set guarded keys and append absent blocks in every matching `Cargo.toml`.
//! Why:
//!  Edition,
//! license,
//! lints,
//! repository,
//! homepage and profiles are owned by the tool inside hand-written manifests;
//!  comments,
//! order and unowned keys survive,
//! and a manifest that already conforms is not rewritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function tomlKeysRule(manifests: string[], onlyDirs: string[] | undefined, edits: TomlEdit[], blocks: TomlBlock[]): Rule
//! ```

/// Imports `TomlBlock`,
/// `TomlEdit` from `crate::config_parts`.
use crate::config_parts::{TomlBlock, TomlEdit};
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
/// Imports `append_block`,
/// `lookup`,
/// `set_key` from `crate::toml_ops`.
use crate::toml_ops::{append_block, lookup, set_key};
/// Imports `Vars`,
/// `evaluate`,
/// `evaluate_text` from `crate::value_eval`.
use crate::value_eval::{Vars, evaluate, evaluate_text};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `expand_glob` from `crate::view_glob`.
use crate::view_glob::expand_glob;
/// Imports `DocumentMut` from `toml_edit`.
use toml_edit::DocumentMut;

/// What:
///  A planned `toml-keys` rule.
/// Why:
///  Holds the manifest globs,
/// the optional directory filter,
/// the edits and the blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TomlKeysRule implements Rule { ... }
/// ```
pub struct TomlKeysRule {
    /// Rule id.
    pub id: String,
    /// Manifest globs.
    pub manifests: Vec<String>,
    /// Crate directories the rule is limited to.
    pub only_dirs: Option<Vec<String>>,
    /// Guarded assignments.
    pub edits: Vec<TomlEdit>,
    /// Blocks appended when absent.
    pub blocks: Vec<TomlBlock>,
}

/// What:
///  Parse a manifest,
/// naming the file when the TOML is malformed.
/// Why:
///  A broken manifest fails the whole run with its path instead of being skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseManifest(path: string, text: string): TomlDocument
/// ```
fn parse_manifest(path: &str, text: &str) -> Result<DocumentMut, Diagnostic> {
    match text.parse::<DocumentMut>() {
        Ok(document) => return Ok(document),
        Err(error) => {
            let message = format!("{path} is not valid TOML: {error}");
            return Err(Diagnostic::new("manifest-invalid", &message).with_path(path));
        }
    }
}

/// What:
///   The text transformation for one manifest.
/// Why:
///   Splitting it from `plan` keeps the per-file edit apart from the glob expansion and the writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TomlKeysRule { transform(view, manifest, text): string }
/// ```
impl TomlKeysRule {
    /// What:
    ///  Apply every edit and block to one manifest's text;
    /// returns the new text.
    /// Why:
    ///  Edits run first on the parsed document,
    /// then blocks are appended one at a time,
    /// re-parsing after each so a
    ///  later block sees the earlier one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// transform(view: View, manifest: string, text: string): string
    /// ```
    fn transform(&self, view: &mut View, manifest: &str, text: &str) -> Result<String, Diagnostic> {
        let directory = parent_directory(manifest);
        let mut vars = Vars::new();
        vars.insert("dir".to_string(), directory.to_string());
        vars.insert("manifest".to_string(), manifest.to_string());
        let mut document = parse_manifest(manifest, text)?;
        for edit in &self.edits {
            let wanted = evaluate(view, &edit.value, &vars)?;
            set_key(&mut document, &edit.guard, &edit.path, &wanted);
        }
        let mut current = document.to_string();
        for block in &self.blocks {
            let reparsed = parse_manifest(manifest, &current)?;
            if lookup(&reparsed, &block.absent_path).is_none() {
                let addition = evaluate_text(view, &block.text, &vars)?;
                current = append_block(&current, &addition);
            }
        }
        return Ok(current);
    }
}

/// What:
///   How the `toml-keys` kind plugs into the engine:
/// its id,
/// its kind name,
/// `composes` and its plan.
/// Why:
///   The kind composes:
/// it reads each manifest through the view and writes the changed text,
///   so other rules that edit the same manifest can run before or after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TomlKeysRule implements Rule { id(); kind(); composes(): true; plan(view) }
/// ```
impl Rule for TomlKeysRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "toml-keys";
    }

    /// Transforms a file in place,
    /// so it composes with other transforming rules (the catalog rule).
    fn composes(&self) -> bool {
        return true;
    }

    /// What:
    ///  Transform each matching manifest and plan a write for the ones that changed.
    /// Why:
    ///  Manifests that already conform produce no change,
    /// so a clean tree plans nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let mut plan = RulePlan::default();
        let mut manifests: Vec<String> = Vec::new();
        for pattern in &self.manifests {
            manifests.extend(expand_glob(view, pattern, EntryKind::File)?);
        }
        manifests.sort();
        manifests.dedup();
        for manifest in manifests {
            let directory = parent_directory(&manifest).to_string();
            if let Some(only) = &self.only_dirs
                && !only.contains(&directory)
            {
                continue;
            }
            let text = view.read_text(&manifest)?.unwrap_or_default();
            let transformed = self.transform(view, &manifest, &text)?;
            if transformed != text {
                plan.changes.push(Change::Write { path: manifest, content: transformed.into_bytes(), mode: None });
            }
        }
        return Ok(plan);
    }
}
