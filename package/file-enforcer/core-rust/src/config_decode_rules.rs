//! What:
//!  Decode one rule object by its `kind` tag.
//! Why:
//!  Each kind has its own fields;
//! the per-kind readers keep the accepted keys next to the fields they read,
//! the same
//!  names the derived schema lists.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function decodeRule(value: unknown, pointer: string): RuleConfig | undefined
//! ```

/// Imports `is_octal_mode`,
/// `is_repo_relative` from `crate::config_decode`.
use crate::config_decode::{is_octal_mode, is_repo_relative};
/// Imports `HarperScopedServer`,
/// `LicenseRequirement`,
/// `TomlBlock`,
/// `TomlEdit` from `crate::config_parts`.
use crate::config_parts::{HarperScopedServer, LicenseRequirement, TomlBlock, TomlEdit};
/// Imports `Reader` from `crate::config_reader`.
use crate::config_reader::Reader;
/// Imports `RuleConfig` from `crate::config_types`.
use crate::config_types::RuleConfig;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{Map, Value};
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;

/// What:
///  Decode one rule;
/// `None` when it is not an object or its kind is unknown.
///  `Option<RuleConfig>` is "a rule or nothing" (TS:
/// `RuleConfig | undefined`).
/// Why:
///  The caller keeps decoding the other rules after one fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeRule(reader: Reader, value: unknown, pointer: string): RuleConfig | undefined
/// ```
pub fn decode_rule(reader: &mut Reader, value: &Value, pointer: &str) -> Option<RuleConfig> {
    let object = reader.object(value, pointer)?;
    let kind = reader.text(object, "kind", pointer);
    let id = reader.text(object, "id", pointer);
    match kind.as_str() {
        "concat" => return Some(decode_concat(reader, object, pointer, id)),
        "mode-only" => return Some(decode_mode_only(reader, object, pointer, id)),
        "package-licenses" => return Some(decode_package_licenses(reader, object, pointer, id)),
        "mirror" => return Some(decode_mirror(reader, object, pointer, id)),
        "toml-keys" => return Some(decode_toml_keys(reader, object, pointer, id)),
        "mise-path-list" => return Some(decode_mise_path_list(reader, object, pointer, id)),
        "rust-catalog" => return Some(decode_rust_catalog(reader, object, pointer, id)),
        "browserslist" => return Some(decode_browserslist(reader, object, pointer, id)),
        "jetbrains-harper" => return Some(decode_jetbrains_harper(reader, object, pointer, id)),
        _ => {
            let message = format!("unknown kind {kind:?}");
            reader.error(&format!("{pointer}/kind"), &message);
            return None;
        }
    }
}

/// What:
///  Read a repo-relative path member and check it stays inside the root.
/// Why:
///  An absolute path or a `..` segment would let a rule write outside the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repoPath(object, key, pointer): string
/// ```
fn repo_path(reader: &mut Reader, object: &Map<String, Value>, key: &str, pointer: &str) -> String {
    let path = reader.text(object, key, pointer);
    if !path.is_empty() && !is_repo_relative(&path) {
        let message = format!("{path:?} must be a relative path without empty, . or .. segments");
        reader.error(&format!("{pointer}/{key}"), &message);
    }
    return path;
}

/// What:
///  Read a required mode member and check it is octal.
/// Why:
///  A mode like `rw` would fail at publish time otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeText(object, key, pointer): string
/// ```
fn mode_text(reader: &mut Reader, object: &Map<String, Value>, key: &str, pointer: &str) -> String {
    let mode = reader.text(object, key, pointer);
    if !mode.is_empty() && !is_octal_mode(&mode) {
        reader.error(&format!("{pointer}/{key}"), &format!("{mode:?} must be three or four octal digits such as 0600"));
    }
    return mode;
}

/// Decodes the fields of a `concat` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_concat(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "output", "parts", "mode"]);
    let output = repo_path(reader, object, "output", pointer);
    let items = reader.array(object, "parts", pointer);
    let mut parts = Vec::new();
    for (index, item) in items.iter().enumerate() {
        parts.push(reader.value_expr(item, &format!("{pointer}/parts/{index}")));
    }
    let mode = reader.optional_text(object, "mode", pointer);
    if let Some(text) = &mode
        && !is_octal_mode(text)
    {
        reader.error(&format!("{pointer}/mode"), &format!("{text:?} must be three or four octal digits"));
    }
    return RuleConfig::Concat { id, output, parts, mode };
}

/// Decodes the fields of a `mode-only` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_mode_only(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "path", "mode"]);
    let path = repo_path(reader, object, "path", pointer);
    let mode = mode_text(reader, object, "mode", pointer);
    return RuleConfig::ModeOnly { id, path, mode };
}

/// Decodes the fields of a `package-licenses` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_package_licenses(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "manifests", "directory", "texts", "requires"]);
    let manifests = reader.text_list(object, "manifests", pointer);
    let directory = reader.text(object, "directory", pointer);
    let mut texts: BTreeMap<String, String> = BTreeMap::new();
    match object.get("texts") {
        Some(Value::Object(map)) => {
            for (license, source) in map {
                match source.as_str() {
                    Some(path) => {
                        texts.insert(license.clone(), path.to_string());
                    }
                    None => reader.error(&format!("{pointer}/texts/{license}"), "must be a string"),
                }
            }
        }
        Some(_) => reader.error(&format!("{pointer}/texts"), "must be an object"),
        None => reader.error(pointer, "is missing the required key \"texts\""),
    }
    let mut requires: Vec<LicenseRequirement> = Vec::new();
    let items = reader.array(object, "requires", pointer);
    for (index, item) in items.iter().enumerate() {
        let item_pointer = format!("{pointer}/requires/{index}");
        if let Some(entry) = reader.object(item, &item_pointer) {
            reader.reject_unknown(entry, &item_pointer, &["contains", "texts"]);
            let contains = reader.text(entry, "contains", &item_pointer);
            let needed = reader.text_list(entry, "texts", &item_pointer);
            for license in &needed {
                if !texts.contains_key(license) {
                    reader.error(&format!("{item_pointer}/texts"), &format!("{license:?} is not a key of texts"));
                }
            }
            requires.push(LicenseRequirement { contains, texts: needed });
        }
    }
    return RuleConfig::PackageLicenses { id, manifests, directory, texts, requires };
}

/// Decodes the fields of a `mirror` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_mirror(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "from", "to"]);
    let from = repo_path(reader, object, "from", pointer);
    let to = repo_path(reader, object, "to", pointer);
    return RuleConfig::Mirror { id, from, to };
}

/// Decodes the fields of a `toml-keys` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_toml_keys(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "manifests", "onlyDirs", "edits", "blocks"]);
    let manifests = reader.text_list(object, "manifests", pointer);
    let only_dirs = reader.optional_text_list(object, "onlyDirs", pointer);
    let mut edits: Vec<TomlEdit> = Vec::new();
    let edit_items = reader.array(object, "edits", pointer);
    for (index, item) in edit_items.iter().enumerate() {
        let item_pointer = format!("{pointer}/edits/{index}");
        if let Some(entry) = reader.object(item, &item_pointer) {
            reader.reject_unknown(entry, &item_pointer, &["guard", "path", "value"]);
            let guard = reader.text_list(entry, "guard", &item_pointer);
            let path = reader.text_list(entry, "path", &item_pointer);
            let value = match entry.get("value") {
                Some(raw) => reader.value_expr(raw, &format!("{item_pointer}/value")),
                None => {
                    reader.error(&item_pointer, "is missing the required key \"value\"");
                    crate::config_parts::ValueExpr::Text(String::new())
                }
            };
            edits.push(TomlEdit { guard, path, value });
        }
    }
    let mut blocks: Vec<TomlBlock> = Vec::new();
    let block_items = reader.array(object, "blocks", pointer);
    for (index, item) in block_items.iter().enumerate() {
        let item_pointer = format!("{pointer}/blocks/{index}");
        if let Some(entry) = reader.object(item, &item_pointer) {
            reader.reject_unknown(entry, &item_pointer, &["absentPath", "text"]);
            let absent_path = reader.text_list(entry, "absentPath", &item_pointer);
            let text = match entry.get("text") {
                Some(raw) => reader.value_expr(raw, &format!("{item_pointer}/text")),
                None => {
                    reader.error(&item_pointer, "is missing the required key \"text\"");
                    crate::config_parts::ValueExpr::Text(String::new())
                }
            };
            blocks.push(TomlBlock { absent_path, text });
        }
    }
    return RuleConfig::TomlKeys { id, manifests, only_dirs, edits, blocks };
}

/// Decodes the fields of a `mise-path-list` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_mise_path_list(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "output", "directories", "header"]);
    let output = repo_path(reader, object, "output", pointer);
    let directories = reader.text_list(object, "directories", pointer);
    let header = match object.get("header") {
        None | Some(Value::Null) => None,
        Some(raw) => Some(reader.value_expr(raw, &format!("{pointer}/header"))),
    };
    return RuleConfig::MisePathList { id, output, directories, header };
}

/// Decodes the fields of a `rust-catalog` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_rust_catalog(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "catalog", "manifests"]);
    let catalog = repo_path(reader, object, "catalog", pointer);
    let manifests = reader.text_list(object, "manifests", pointer);
    return RuleConfig::RustCatalog { id, catalog, manifests };
}

/// Decodes the fields of a `browserslist` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_browserslist(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "source", "output", "lock", "environment"]);
    let source = repo_path(reader, object, "source", pointer);
    let output = repo_path(reader, object, "output", pointer);
    let lock = repo_path(reader, object, "lock", pointer);
    let environment = reader.optional_text(object, "environment", pointer);
    return RuleConfig::Browserslist { id, source, output, lock, environment };
}

/// Decodes the fields of a `jetbrains-harper` rule into a `RuleConfig`,
/// recording every problem in the reader.
fn decode_jetbrains_harper(reader: &mut Reader, object: &Map<String, Value>, pointer: &str, id: String) -> RuleConfig {
    let allowed = ["kind", "id", "productPrefixes", "disabledLinters", "excludePatterns", "scopedServers"];
    reader.reject_unknown(object, pointer, &allowed);
    let product_prefixes = reader.text_list(object, "productPrefixes", pointer);
    let disabled_linters = reader.text_list(object, "disabledLinters", pointer);
    let exclude_patterns = reader.text_list(object, "excludePatterns", pointer);
    let mut scoped_servers: Vec<HarperScopedServer> = Vec::new();
    let items = reader.array(object, "scopedServers", pointer);
    for (index, item) in items.iter().enumerate() {
        let item_pointer = format!("{pointer}/scopedServers/{index}");
        if let Some(entry) = reader.object(item, &item_pointer) {
            let keys = ["id", "name", "fileNames", "languageId", "disabledLinters"];
            reader.reject_unknown(entry, &item_pointer, &keys);
            scoped_servers.push(HarperScopedServer {
                id: reader.text(entry, "id", &item_pointer),
                name: reader.text(entry, "name", &item_pointer),
                file_names: reader.text_list(entry, "fileNames", &item_pointer),
                language_id: reader.text(entry, "languageId", &item_pointer),
                disabled_linters: reader.text_list(entry, "disabledLinters", &item_pointer),
            });
        }
    }
    return RuleConfig::JetbrainsHarper { id, product_prefixes, disabled_linters, exclude_patterns, scoped_servers };
}
