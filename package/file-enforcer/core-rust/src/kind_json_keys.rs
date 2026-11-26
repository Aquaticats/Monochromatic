//! What:
//!  The `json-keys` rule kind:
//! set guarded members in every matching JSON manifest.
//! Why:
//!  A manifest's repository and homepage links are owned by the tool,
//! so the pre-transfer URL cannot come back;
//! every other member,
//! the indentation and the member order survive,
//! and a manifest that already conforms is not rewritten.
//!  One manifest the rule cannot read or parse is a finding for that manifest,
//! and the other manifests and rules of the run are still planned and reported.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function jsonKeysRule(manifests: string[], onlyDirs: string[] | undefined, edits: KeyEdit[]): Rule
//! ```

/// Imports `KeyEdit` from `crate::config_parts`.
use crate::config_parts::KeyEdit;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `token_text`,
/// `value_matches` from `crate::json_compare`.
use crate::json_compare::{
    token_text,
    value_matches,
};
/// Imports `parse_json_text` from `crate::json_ops`.
use crate::json_ops::parse_json_text;
/// Imports `JsonNode`,
/// `JsonText` from `crate::json_ops`.
use crate::json_ops::{
    JsonNode,
    JsonText,
};
/// Imports `parent_directory` from `crate::path_text`.
use crate::path_text::parent_directory;
/// Imports `EntryKind` from `crate::real_fs`.
use crate::real_fs::EntryKind;
/// Imports `Change`,
/// `Rule`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{
    Change,
    Rule,
    RulePlan,
};
/// Imports `Evaluated`,
/// `Vars`,
/// `evaluate` from `crate::value_eval`.
use crate::value_eval::{
    Evaluated,
    Vars,
    evaluate,
};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `expand_glob` from `crate::view_glob`.
use crate::view_glob::expand_glob;

/// What:
///  The largest manifest this rule reads, and parses several times per edit.
/// Why:
///  The largest of the 245 files `package/*/*/package.json` matches in this repository on 2026-11-26 is
///  4324 bytes (`find package -maxdepth 3 -name package.json -not -path '*/node_modules/*' -printf '%s %p\n' |
///  sort -rn | head -1`),
///  so 64 KiB leaves 15 times the room the largest one needs.
///  A larger file is refused by name rather than copied,
///  which is what keeps one runaway file from taking the run's memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MANIFEST_BYTES_MAX = 64 * 1024;
/// ```
const MANIFEST_BYTES_MAX: u64 = 64 * 1024;

/// What:
///  The UTF-8 byte-order mark, U+FEFF, which a JSON parser does not count as whitespace.
/// Why:
///  A manifest saved with one is still a manifest the rule owns;
///  the mark is kept outside the parsed text and put back before the write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BYTE_ORDER_MARK = '\uFEFF';
/// ```
const BYTE_ORDER_MARK: &str = "\u{feff}";

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
///  One manifest's text after every edit ran, or the original text beside the findings that stopped them.
/// Why:
///  A manifest with a finding must not be written at all,
/// so its text and its findings travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Transformed = { text: string; findings: Diagnostic[] };
/// ```
struct Transformed {
    /// The text to write.
    text: String,
    /// Findings scoped to this manifest;
    /// no change is planned beside one.
    findings: Vec<Diagnostic>,
}

/// What:
///  The words a message uses for a key path.
/// Why:
///  A finding names the member it is about,
/// and the document root has no name of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathText(segments: string[]): string
/// ```
fn path_text(segments: &[String]) -> String {
    if segments.is_empty() {
        return "the document root".to_string();
    }
    return segments.join(".");
}

/// What:
///  The finding for a value the rule cannot set members inside.
/// Why:
///  A `repository` written as a string,
/// `null` or an array it does not own is exactly how the pre-transfer owner survived the run;
/// the finding says what the manifest holds so it can be fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notAnObject(manifest: string, name: string, token: string, shape: string): Diagnostic
/// ```
fn not_an_object(
    manifest: &str,
    name: &str,
    token: &str,
    shape: &str,
) -> Diagnostic {
    let message = format!(
        "{manifest}: {name} holds {token}, which is {shape}; json-keys sets members inside {name}, so it must be an object"
    );
    return Diagnostic::new("manifest-shape", &message)
        .with_path(manifest)
        .with_hint(
            "write the value as an object, or remove it if this manifest should not be managed",
        );
}

/// What:
///  The finding for a file the parser cannot read as JSONC.
/// Why:
///  The message says what this parser does accept,
/// so a manifest carrying a comment or a trailing comma is not read as a broken file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notJsonc(manifest: string, error: string): Diagnostic
/// ```
fn not_jsonc(
    manifest: &str,
    error: &str,
) -> Diagnostic {
    let message = format!(
        "{manifest} is not JSONC: {error}. The parser accepts // and /* */ comments, trailing commas, and one UTF-8 byte-order mark at the start of the file"
    );
    return Diagnostic::new("manifest-invalid", &message).with_path(manifest);
}

/// What:
///  The value an added member needs to have an object to sit in.
/// Why:
///  Splitting the shape check from the edit keeps the two refusals in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function memberParent(document: JsonText, manifest: string, parents: string[]): JsonNode | undefined
/// ```
fn member_parent<'a>(
    document: &'a JsonText,
    manifest: &str,
    parents: &[String],
) -> Result<Option<JsonNode<'a>>, Diagnostic> {
    let Some(parent) = document.node(parents) else {
        // A parent the manifest does not have is left alone: the rule manages what is there.
        return Ok(None);
    };
    if !parent.is_object() {
        let name = path_text(parents);
        return Err(not_an_object(
            manifest,
            &name,
            document.token(&parent),
            parent.shape(),
        ));
    }
    return Ok(Some(parent));
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
fn apply_edit(
    text: &str,
    manifest: &str,
    edit: &KeyEdit,
    wanted: &Evaluated,
) -> Result<String, Diagnostic> {
    let document = match parse_json_text(text) {
        Ok(document) => document,
        Err(error) => {
            let message = error.to_string();
            return Err(not_jsonc(manifest, &message));
        }
    };
    if !edit.guard.is_empty() && document.node(&edit.guard).is_none() {
        return Ok(text.to_string());
    }
    let Some((last, parents)) = edit.path.split_last() else {
        return Ok(text.to_string());
    };
    let token = token_text(wanted);
    if let Some(existing) = document.node(&edit.path) {
        if value_matches(&existing.comparable(), wanted) {
            return Ok(text.to_string());
        }
        return Ok(document.replaced(&existing, &token));
    }
    let Some(parent) = member_parent(&document, manifest, parents)? else {
        return Ok(text.to_string());
    };
    match document.with_member(&parent, last, &token) {
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
/// class JsonKeysRule { transform(view, manifest, text): Transformed }
/// ```
impl JsonKeysRule {
    /// What:
    ///  Apply every edit to one manifest's text;
    /// returns the new text and any finding that stopped the edits.
    /// Why:
    ///  Each manifest binds `dir` and `manifest`,
    /// so a homepage link derives from where the file sits,
    /// and one unmanageable member does not take the other manifests down with it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// transform(view: View, manifest: string, text: string): Transformed
    /// ```
    fn transform(
        &self,
        view: &mut View,
        manifest: &str,
        text: &str,
    ) -> Result<Transformed, Diagnostic> {
        let mut vars = Vars::new();
        vars.insert("dir".to_string(), parent_directory(manifest).to_string());
        vars.insert("manifest".to_string(), manifest.to_string());
        let (mark, body) = match text.strip_prefix(BYTE_ORDER_MARK) {
            Some(rest) => (BYTE_ORDER_MARK, rest),
            None => ("", text),
        };
        let mut current = body.to_string();
        let mut findings: Vec<Diagnostic> = Vec::new();
        for edit in &self.edits {
            let wanted = evaluate(view, &edit.value, &vars)?;
            match apply_edit(&current, manifest, edit, &wanted) {
                Ok(edited) => current = edited,
                Err(finding) => {
                    findings.push(finding);
                    break;
                }
            }
        }
        if !findings.is_empty() {
            // A manifest with a finding keeps its own text, so no edit of the run lands half-applied in it.
            return Ok(Transformed {
                text: text.to_string(),
                findings,
            });
        }
        return Ok(Transformed {
            text: format!("{mark}{current}"),
            findings,
        });
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
    /// so a clean tree plans nothing,
    /// and a manifest the rule cannot read or parse becomes a finding of its own instead of stopping the rule.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(
        &self,
        view: &mut View,
    ) -> Result<RulePlan, Diagnostic> {
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
            let text = match view.read_text_at_most(&manifest, MANIFEST_BYTES_MAX) {
                Ok(Some(text)) => text,
                // A manifest a glob matched but that is gone by now plans nothing.
                Ok(None) => continue,
                Err(error) => {
                    plan.errors.push(error);
                    continue;
                }
            };
            let transformed = self.transform(view, &manifest, &text)?;
            plan.errors.extend(transformed.findings);
            if transformed.text != text {
                plan.changes.push(Change::Write {
                    path: manifest,
                    content: transformed.text.into_bytes(),
                    mode: None,
                });
            }
        }
        return Ok(plan);
    }
}
