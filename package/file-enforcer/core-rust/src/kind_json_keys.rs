//! What:
//!  The `json-keys` rule kind:
//! set guarded members in every matching JSON manifest.
//! Why:
//!  A manifest's repository and homepage links are owned by the tool,
//! so the pre-transfer URL cannot come back;
//! every other member,
//! the indentation and the member order survive,
//! and a manifest that already conforms is not rewritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function jsonKeysRule(manifests: string[], onlyDirs: string[] | undefined, edits: KeyEdit[]): Rule
//! ```

/// Imports `KeyEdit` from `crate::config_parts`.
use crate::config_parts::KeyEdit;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `parse_json_text`,
/// `token_text`,
/// `value_matches` from `crate::json_ops`.
use crate::json_ops::{parse_json_text, token_text, value_matches};
/// Imports `parent_directory` from `crate::path_text`.
use crate::path_text::parent_directory;
/// Imports `EntryKind` from `crate::real_fs`.
use crate::real_fs::EntryKind;
/// Imports `Change`,
/// `Rule`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{Change, Rule, RulePlan};
/// Imports `Evaluated`,
/// `Vars`,
/// `evaluate` from `crate::value_eval`.
use crate::value_eval::{Evaluated, Vars, evaluate};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `expand_glob` from `crate::view_glob`.
use crate::view_glob::expand_glob;

/// What:
///  A planned `json-keys` rule.
/// Why:
///  Holds the manifest globs,
/// the optional directory filter and the guarded edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsonKeysRule implements Rule { ... }
/// ```
pub struct JsonKeysRule {
    /// Rule id.
    pub id: String,
    /// Manifest globs.
    pub manifests: Vec<String>,
    /// Manifest directories the rule is limited to.
    pub only_dirs: Option<Vec<String>>,
    /// Guarded assignments.
    pub edits: Vec<KeyEdit>,
}

/// What:
///  Apply one edit to one manifest's text.
/// Why:
///  Each edit is spliced against a fresh parse,
/// because an earlier edit moved the offsets a later one needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyEdit(text: string, manifest: string, edit: KeyEdit, wanted: Evaluated): string
/// ```
fn apply_edit(text: &str, manifest: &str, edit: &KeyEdit, wanted: &Evaluated) -> Result<String, Diagnostic> {
    let document = match parse_json_text(text) {
        Ok(document) => document,
        Err(error) => {
            let message = format!("{manifest} is not valid JSON: {error}");
            return Err(Diagnostic::new("manifest-invalid", &message).with_path(manifest));
        }
    };
    if !edit.guard.is_empty() && document.node(&edit.guard).is_none() {
        return Ok(text.to_string());
    }
    let Some((last, parents)) = edit.path.split_last() else { return Ok(text.to_string()) };
    let token = token_text(wanted);
    if let Some(existing) = document.node(&edit.path) {
        if value_matches(&existing.comparable, wanted) {
            return Ok(text.to_string());
        }
        return Ok(document.replaced(existing, &token));
    }
    let Some(parent) = document.node(parents) else { return Ok(text.to_string()) };
    match document.with_member(parent, last, &token) {
        Some(edited) => return Ok(edited),
        None => return Ok(text.to_string()),
    }
}

/// What:
///   The text transformation for one manifest.
/// Why:
///   Splitting it from `plan` keeps the per-file edits apart from the glob expansion and the writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsonKeysRule { transform(view, manifest, text): string }
/// ```
impl JsonKeysRule {
    /// What:
    ///  Apply every edit to one manifest's text;
    /// returns the new text.
    /// Why:
    ///  Each manifest binds `dir` and `manifest`,
    /// so a homepage link derives from where the file sits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// transform(view: View, manifest: string, text: string): string
    /// ```
    fn transform(&self, view: &mut View, manifest: &str, text: &str) -> Result<String, Diagnostic> {
        let mut vars = Vars::new();
        vars.insert("dir".to_string(), parent_directory(manifest).to_string());
        vars.insert("manifest".to_string(), manifest.to_string());
        let mut current = text.to_string();
        for edit in &self.edits {
            let wanted = evaluate(view, &edit.value, &vars)?;
            current = apply_edit(&current, manifest, edit, &wanted)?;
        }
        return Ok(current);
    }
}

/// What:
///   How the `json-keys` kind plugs into the engine:
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
/// class JsonKeysRule implements Rule { id(); kind(); composes(): true; plan(view) }
/// ```
impl Rule for JsonKeysRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "json-keys";
    }

    /// Transforms a file in place,
    /// so it composes with other transforming rules.
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
