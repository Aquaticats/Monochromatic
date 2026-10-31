//! What:
//!  The building blocks rules are made of: value expressions, license requirements, TOML edits and blocks.
//! Why:
//!  They derive `JsonSchema` like the rule types, and keeping them apart holds each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ValueExpr = string | boolean | number | ValueExpr[] | { file: ValueExpr } | { var: string } | { concat: ValueExpr[] };
//! ```

// The `JsonSchema` derive expands to `Result::unwrap` calls inside its generated `json_schema` function, which
// the repository's `clippy.toml` disallows. Evidence and the failed configuration attempts:
// doc/troubleshooting/schemars-derive-trips-disallowed-methods.md.
#![expect(clippy::disallowed_methods, reason = "the schemars JsonSchema derive expands to Result::unwrap")]

use schemars::JsonSchema;

/// What:
///  A value the config can compute: a literal, a list, or a tagged object (`file`, `var`, `concat`).
/// Why:
///  Fixed tagged objects replace a string expression language, so every input path is visible without running anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ValueExpr = string | boolean | number | ValueExpr[] | { file: ValueExpr } | { var: string } | { concat: ValueExpr[] };
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(untagged, deny_unknown_fields, description = "A literal, a list, or a tagged object: file, var or concat.")]
pub enum ValueExpr {
    /// Literal text.
    Text(String),
    /// Literal boolean.
    Bool(bool),
    /// Literal integer.
    Integer(i64),
    /// List of values, used by array-valued Cargo keys.
    List(Vec<ValueExpr>),
    /// The text of a file, read through the virtual view so earlier rules' planned output is visible.
    File {
        /// Repo-relative path of the file to read.
        file: Box<ValueExpr>,
    },
    /// A variable a fan-out rule binds, such as `dir` (the crate directory) or `manifest`.
    Var {
        /// Variable name.
        var: String,
    },
    /// The text of every part joined end to end.
    Concat {
        /// Parts in order; each must evaluate to text.
        concat: Vec<ValueExpr>,
    },
}

/// What:
///  One license-text requirement: a manifest license expression containing `contains` needs these texts.
/// Why:
///  LGPL-3.0-or-later needs the GPL text as well, so one requirement can name several ids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LicenseRequirement = { contains: string; texts: string[] };
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(deny_unknown_fields, description = "License texts a manifest license expression needs.")]
pub struct LicenseRequirement {
    /// Substring of the manifest's license expression.
    pub contains: String,
    /// Text ids (keys of `texts`) the package then needs.
    pub texts: Vec<String>,
}

/// What:
///  One guarded key assignment inside a `Cargo.toml`.
/// Why:
///  The tool owns a key only where the crate already has its table or key, so crates that opt out stay untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TomlEdit = { guard: string[]; path: string[]; value: ValueExpr };
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(deny_unknown_fields, description = "A key assignment applied only where the guard key path exists.")]
pub struct TomlEdit {
    /// Key path that must exist before the edit applies; empty means always.
    pub guard: Vec<String>,
    /// Key path to set.
    pub path: Vec<String>,
    /// New value; text, boolean, integer, or a list of those.
    pub value: ValueExpr,
}

/// What:
///  A TOML block appended to a manifest when a key path is absent.
/// Why:
///  Whole tables such as `[lints.clippy]` are added once and then owned by the crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TomlBlock = { absentPath: string[]; text: ValueExpr };
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(deny_unknown_fields, rename_all = "camelCase", description = "A TOML block appended when a key path is absent.")]
pub struct TomlBlock {
    /// Key path whose absence triggers the append.
    pub absent_path: Vec<String>,
    /// Text appended after a blank line; usually a `{"file": ...}` value.
    pub text: ValueExpr,
}

/// What:
///  A second language server entry the Harper policy adds, scoped to named files.
/// Why:
///  AGENTS.md and CLAUDE.md need different linters than other Markdown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HarperScopedServer = { id: string; name: string; fileNames: string[]; languageId: string; disabledLinters: string[] };
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(deny_unknown_fields, rename_all = "camelCase", description = "A language server scoped to named files.")]
pub struct HarperScopedServer {
    /// Server id in the IDE settings.
    pub id: String,
    /// Display name.
    pub name: String,
    /// File names the server is scoped to.
    pub file_names: Vec<String>,
    /// Language id the server handles.
    pub language_id: String,
    /// Linters the server disables.
    pub disabled_linters: Vec<String>,
}
