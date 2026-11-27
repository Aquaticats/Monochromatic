//! What:
//!  Decode the seven rule kinds this crate implements itself:
//! `concat`,
//! `mode-only`,
//! `package-licenses`,
//! `mirror`,
//! `toml-keys`,
//! `json-keys` and
//! `mise-path-list`.
//! Why:
//!  The accepted keys of a kind belong next to the fields that read them,
//! the same names the derived schema lists;
//!  split out of `config_decode_rules`,
//! which keeps the dispatcher.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // config_decode_rules_built_in_kinds.ts
//! function decodeConcat(...): RuleConfig
//! function decodeModeOnly(...): RuleConfig
//! function decodePackageLicenses(...): RuleConfig
//! function decodeMirror(...): RuleConfig
//! function decodeTomlKeys(...): RuleConfig
//! function decodeJsonKeys(...): RuleConfig
//! function decodeMisePathList(...): RuleConfig
//! ```

/// Imports `is_octal_mode` from `crate::config_decode`.
use crate::config_decode::is_octal_mode;
/// Imports `decode_edits` from `crate::config_decode_rules_key_edits`.
use crate::config_decode_rules_key_edits::decode_edits;
/// Imports `mode_text`,
/// `repo_path` from `crate::config_decode_rules_path_and_mode_readers`.
use crate::config_decode_rules_path_and_mode_readers::{
    mode_text,
    repo_path,
};
/// Imports `pointer_segment` from `crate::config_locations`.
use crate::config_locations::pointer_segment;
/// Imports `LicenseRequirement`,
/// `TomlBlock` from `crate::config_parts`.
use crate::config_parts::{
    LicenseRequirement,
    TomlBlock,
};
/// Imports `Reader` from `crate::config_reader`.
use crate::config_reader::Reader;
/// Imports `RuleConfig` from `crate::config_types`.
use crate::config_types::RuleConfig;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{
    Map,
    Value,
};
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;

/// Decodes the fields of a `concat` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_concat(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
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
        reader.error(
            &format!("{pointer}/mode"),
            &format!("{text:?} must be three or four octal digits"),
        );
    }
    return RuleConfig::Concat {
        id,
        output,
        parts,
        mode,
    };
}

/// Decodes the fields of a `mode-only` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_mode_only(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "path", "mode"]);
    let path = repo_path(reader, object, "path", pointer);
    let mode = mode_text(reader, object, "mode", pointer);
    return RuleConfig::ModeOnly {
        id,
        path,
        mode,
    };
}

/// Decodes the fields of a `package-licenses` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_package_licenses(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(
        object,
        pointer,
        &["kind", "id", "manifests", "directory", "texts", "requires"],
    );
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
                    None => reader.error(
                        &format!("{pointer}/texts/{}", pointer_segment(license)),
                        "must be a string",
                    ),
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
                    reader.error(
                        &format!("{item_pointer}/texts"),
                        &format!("{license:?} is not a key of texts"),
                    );
                }
            }
            requires.push(LicenseRequirement {
                contains,
                texts: needed,
            });
        }
    }
    return RuleConfig::PackageLicenses {
        id,
        manifests,
        directory,
        texts,
        requires,
    };
}

/// Decodes the fields of a `mirror` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_mirror(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "from", "to"]);
    let from = repo_path(reader, object, "from", pointer);
    let to = repo_path(reader, object, "to", pointer);
    return RuleConfig::Mirror {
        id,
        from,
        to,
    };
}

/// Decodes the fields of a `toml-keys` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_toml_keys(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(
        object,
        pointer,
        &["kind", "id", "manifests", "onlyDirs", "edits", "blocks"],
    );
    let manifests = reader.text_list(object, "manifests", pointer);
    let only_dirs = reader.optional_text_list(object, "onlyDirs", pointer);
    let edits = decode_edits(reader, object, pointer);
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
            blocks.push(TomlBlock {
                absent_path,
                text,
            });
        }
    }
    return RuleConfig::TomlKeys {
        id,
        manifests,
        only_dirs,
        edits,
        blocks,
    };
}

/// Decodes the fields of a `json-keys` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_json_keys(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(
        object,
        pointer,
        &["kind", "id", "manifests", "onlyDirs", "edits"],
    );
    let manifests = reader.text_list(object, "manifests", pointer);
    let only_dirs = reader.optional_text_list(object, "onlyDirs", pointer);
    let edits = decode_edits(reader, object, pointer);
    return RuleConfig::JsonKeys {
        id,
        manifests,
        only_dirs,
        edits,
    };
}

/// Decodes the fields of a `mise-path-list` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_mise_path_list(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(
        object,
        pointer,
        &["kind", "id", "output", "directories", "header"],
    );
    let output = repo_path(reader, object, "output", pointer);
    let directories = reader.text_list(object, "directories", pointer);
    let header = match object.get("header") {
        None | Some(Value::Null) => None,
        Some(raw) => Some(reader.value_expr(raw, &format!("{pointer}/header"))),
    };
    return RuleConfig::MisePathList {
        id,
        output,
        directories,
        header,
    };
}
