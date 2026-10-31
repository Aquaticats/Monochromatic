//! What:
//!  The `mise-path-list` rule kind:
//! a Mise config that lists the existing non-empty directories matching the globs.
//! Why:
//!  `.config/mise/conf.d/10-bin.local.toml` carries the `package/*/*/node_modules/.bin` directories pnpm created;
//!  only directories that exist and hold something are listed,
//! sorted so the file does not depend on listing order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function misePathListRule(output: string, directories: string[], header?: ValueExpr): Rule
//! ```

/// Imports `ValueExpr` from `crate::config_parts`.
use crate::config_parts::ValueExpr;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `EntryKind` from `crate::real_fs`.
use crate::real_fs::EntryKind;
/// Imports `Change`,
/// `Rule`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{Change, Rule, RulePlan};
/// Imports `Vars`,
/// `evaluate_text` from `crate::value_eval`.
use crate::value_eval::{Vars, evaluate_text};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `expand_glob` from `crate::view_glob`.
use crate::view_glob::expand_glob;

/// What:
///  A planned `mise-path-list` rule.
/// Why:
///  Holds the output path,
/// the directory globs and the optional header.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MisePathListRule implements Rule { id: string; output: string; directories: string[]; header?: ValueExpr }
/// ```
pub struct MisePathListRule {
    /// Rule id.
    pub id: String,
    /// Output path.
    pub output: String,
    /// Directory globs.
    pub directories: Vec<String>,
    /// Text placed before `[env]`.
    pub header: Option<ValueExpr>,
}

/// What:
///  Quote a path as a TOML basic string.
/// Why:
///  Paths with a quote or a backslash must not break the generated file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tomlString(text: string): string { return JSON.stringify(text); }
/// ```
fn toml_string(text: &str) -> String {
    let mut quoted = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    return quoted;
}

/// The `impl Rule for MisePathListRule` block.
impl Rule for MisePathListRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "mise-path-list";
    }

    /// What:
    ///  Expand the globs,
    /// keep non-empty directories,
    /// and render the TOML.
    /// Why:
    ///  Empty `.bin` directories add nothing to PATH and would churn the file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let mut found: Vec<String> = Vec::new();
        for pattern in &self.directories {
            for directory in expand_glob(view, pattern, EntryKind::Directory)? {
                if !view.list(&directory)?.is_empty() {
                    found.push(directory);
                }
            }
        }
        found.sort();
        found.dedup();
        let vars = Vars::new();
        let mut text = String::new();
        if let Some(header) = &self.header {
            text.push_str(&evaluate_text(view, header, &vars)?);
        }
        text.push_str("[env]\n_.path = [\n");
        for directory in &found {
            text.push_str(&format!("  {},\n", toml_string(directory)));
        }
        text.push_str("]\n");
        let change = Change::Write { path: self.output.clone(), content: text.into_bytes(), mode: None };
        return Ok(RulePlan { changes: vec![change], warnings: Vec::new() });
    }
}
