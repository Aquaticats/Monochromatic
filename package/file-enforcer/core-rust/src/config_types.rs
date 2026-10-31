//! What:
//!  The typed shape of `file-enforcer.config.jsonc` and of the per-user config.
//! Why:
//!  One set of types is both what the hand-written decoder builds and what `schemars` turns into
//!  `file-enforcer.schema.json`, so the schema and the decoder describe the same fields.
//!  The types derive `JsonSchema` only, never serde `Deserialize`: the decoder in `config_decode.rs` reads them by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Config = { $schema?: string; rules: RuleConfig[] };
//! ```

// The `JsonSchema` derive expands to `Result::unwrap` calls inside its generated `json_schema` function, which
// the repository's `clippy.toml` disallows. Evidence and the failed configuration attempts:
// doc/troubleshooting/schemars-derive-trips-disallowed-methods.md.
#![expect(clippy::disallowed_methods, reason = "the schemars JsonSchema derive expands to Result::unwrap")]

use crate::config_parts::{HarperScopedServer, LicenseRequirement, TomlBlock, TomlEdit, ValueExpr};
use schemars::JsonSchema;
use std::collections::BTreeMap;

/// What:
///  The whole config file: an optional `$schema` pointer and the ordered list of rules.
/// Why:
///  Rules run in file order, so a later rule can read what an earlier rule planned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Config = { $schema?: string; rules: RuleConfig[] };
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(deny_unknown_fields, description = "The file-enforcer config: an ordered list of rules.")]
pub struct Config {
    /// Path or URL of the JSON Schema; editors read it, the tool ignores it.
    #[schemars(rename = "$schema")]
    pub schema: Option<String>,
    /// Rules in execution order.
    pub rules: Vec<RuleConfig>,
}

/// What:
///  One rule: a kind tag plus that kind's fields.
/// Why:
///  Fixed Rust rule kinds with structured values keep the whole config checkable by a JSON Schema.
///  Struct variants (not newtype variants) keep each kind's fields beside the `kind` tag in the schema.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RuleConfig = { kind: 'concat'; id: string; ... } | { kind: 'mirror'; id: string; ... } | ...;
/// ```
#[derive(Clone, Debug, JsonSchema, PartialEq)]
#[schemars(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields,
    description = "One rule: a kind tag plus that kind's fields."
)]
pub enum RuleConfig {
    /// Write a file whose content is the parts joined end to end.
    Concat {
        /// Unique rule id.
        id: String,
        /// Repo-relative output path.
        output: String,
        /// Parts in order.
        parts: Vec<ValueExpr>,
        /// Octal mode such as `0600`; absent keeps the existing mode.
        #[schemars(regex(pattern = r"^[0-7]{3,4}$"))]
        mode: Option<String>,
    },
    /// Keep a file's mode when the file exists; never creates it.
    ModeOnly {
        /// Unique rule id.
        id: String,
        /// Repo-relative path.
        path: String,
        /// Octal mode such as `0600`.
        #[schemars(regex(pattern = r"^[0-7]{3,4}$"))]
        mode: String,
    },
    /// Copy license texts into every matching package's `LICENSES` directory and prune the owned ones.
    PackageLicenses {
        /// Unique rule id.
        id: String,
        /// Manifest globs (`package.json` or `Cargo.toml`) whose license field drives the texts.
        manifests: Vec<String>,
        /// Directory name beside each manifest.
        directory: String,
        /// License id to the repo-relative path of its text; only these ids are ever pruned.
        texts: BTreeMap<String, String>,
        /// Which texts a license expression needs.
        requires: Vec<LicenseRequirement>,
    },
    /// Make a directory an exact copy of another; the target is owned outright.
    Mirror {
        /// Unique rule id.
        id: String,
        /// Repo-relative source directory.
        from: String,
        /// Repo-relative target directory.
        to: String,
    },
    /// Set guarded keys and append absent blocks in matching `Cargo.toml` files.
    TomlKeys {
        /// Unique rule id.
        id: String,
        /// Manifest globs.
        manifests: Vec<String>,
        /// Crate directories the rule is limited to; absent means every matching manifest.
        only_dirs: Option<Vec<String>>,
        /// Guarded assignments.
        edits: Vec<TomlEdit>,
        /// Blocks appended when absent.
        blocks: Vec<TomlBlock>,
    },
    /// Write a Mise config listing the existing non-empty directories that match the globs.
    MisePathList {
        /// Unique rule id.
        id: String,
        /// Repo-relative output path.
        output: String,
        /// Directory globs, for example `package/*/*/node_modules/.bin`.
        directories: Vec<String>,
        /// Text written before the `[env]` table.
        header: Option<ValueExpr>,
    },
    /// Render the Rust dependency catalog into every manifest; provided by the dependency-catalog library.
    RustCatalog {
        /// Unique rule id.
        id: String,
        /// Repo-relative catalog path.
        catalog: String,
        /// Manifest globs.
        manifests: Vec<String>,
    },
    /// Resolve `.browserslistrc` into a JSON array of targets; provided by the in-repo browserslist crate.
    Browserslist {
        /// Unique rule id.
        id: String,
        /// Repo-relative query file.
        source: String,
        /// Repo-relative output path.
        output: String,
        /// Config section to resolve, such as `production`.
        environment: Option<String>,
    },
    /// Apply the Harper language-server policy to the IDE's settings; per-user config only.
    JetbrainsHarper {
        /// Unique rule id.
        id: String,
        /// IDE product prefixes, newest installed version wins.
        product_prefixes: Vec<String>,
        /// Linters disabled on the base server.
        disabled_linters: Vec<String>,
        /// Patterns added to the base server's exclude list.
        exclude_patterns: Vec<String>,
        /// Extra servers scoped to named files.
        scoped_servers: Vec<HarperScopedServer>,
    },
}

impl RuleConfig {
    /// What:
    ///  Borrow the rule's id, whatever its kind.
    ///  `&str` is a borrowed read-only text view (sibling: owned `String`).
    /// Why:
    ///  Diagnostics, duplicate checks and `explain` all name rules by id.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function ruleId(rule: RuleConfig): string { return rule.id; }
    /// ```
    pub fn id(&self) -> &str {
        match self {
            RuleConfig::Concat { id, .. } => return id,
            RuleConfig::ModeOnly { id, .. } => return id,
            RuleConfig::PackageLicenses { id, .. } => return id,
            RuleConfig::Mirror { id, .. } => return id,
            RuleConfig::TomlKeys { id, .. } => return id,
            RuleConfig::MisePathList { id, .. } => return id,
            RuleConfig::RustCatalog { id, .. } => return id,
            RuleConfig::Browserslist { id, .. } => return id,
            RuleConfig::JetbrainsHarper { id, .. } => return id,
        }
    }

    /// What:
    ///  The kind tag as written in the config.
    /// Why:
    ///  Diagnostics and the registry look builders up by kind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function ruleKind(rule: RuleConfig): string { return rule.kind; }
    /// ```
    pub fn kind(&self) -> &'static str {
        match self {
            RuleConfig::Concat { .. } => return "concat",
            RuleConfig::ModeOnly { .. } => return "mode-only",
            RuleConfig::PackageLicenses { .. } => return "package-licenses",
            RuleConfig::Mirror { .. } => return "mirror",
            RuleConfig::TomlKeys { .. } => return "toml-keys",
            RuleConfig::MisePathList { .. } => return "mise-path-list",
            RuleConfig::RustCatalog { .. } => return "rust-catalog",
            RuleConfig::Browserslist { .. } => return "browserslist",
            RuleConfig::JetbrainsHarper { .. } => return "jetbrains-harper",
        }
    }
}
