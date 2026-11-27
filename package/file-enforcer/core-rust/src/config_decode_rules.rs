//! What:
//!  Decode one rule object by its `kind` tag:
//! this file holds `decode_rule`,
//! which reads `kind` and `id` and hands the rest of the object to the reader of that kind.
//!  The readers live beside it:
//! `config_decode_rules_built_in_kinds` holds the seven kinds this crate implements,
//! `config_decode_rules_kinds_from_other_crates` holds the three kinds another crate supplies a builder for,
//! `config_decode_rules_key_edits` holds the `edits` array that `toml-keys` and `json-keys` share,
//! and `config_decode_rules_path_and_mode_readers` holds the repo-relative path and octal mode members that several kinds read.
//! Why:
//!  Each kind has its own fields;
//! the per-kind readers keep the accepted keys next to the fields they read,
//! the same names the derived schema lists,
//! and the dispatch stays here so one place names every kind the engine knows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function decodeRule(reader: Reader, value: unknown, pointer: string): RuleConfig | undefined
//! ```

/// Imports `decode_concat`,
/// `decode_json_keys`,
/// `decode_mirror`,
/// `decode_mode_only`,
/// `decode_mise_path_list`,
/// `decode_package_licenses`,
/// `decode_toml_keys` from `crate::config_decode_rules_built_in_kinds`.
use crate::config_decode_rules_built_in_kinds::{
    decode_concat,
    decode_json_keys,
    decode_mirror,
    decode_mise_path_list,
    decode_mode_only,
    decode_package_licenses,
    decode_toml_keys,
};
/// Imports `decode_browserslist`,
/// `decode_jetbrains_harper`,
/// `decode_rust_catalog` from `crate::config_decode_rules_kinds_from_other_crates`.
use crate::config_decode_rules_kinds_from_other_crates::{
    decode_browserslist,
    decode_jetbrains_harper,
    decode_rust_catalog,
};
/// Imports `Reader` from `crate::config_reader`.
use crate::config_reader::Reader;
/// Imports `RuleConfig` from `crate::config_types`.
use crate::config_types::RuleConfig;
/// Imports `Value` from `serde_json`.
use serde_json::Value;

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
pub fn decode_rule(
    reader: &mut Reader,
    value: &Value,
    pointer: &str,
) -> Option<RuleConfig> {
    let object = reader.object(value, pointer)?;
    let kind = reader.text(object, "kind", pointer);
    let id = reader.text(object, "id", pointer);
    match kind.as_str() {
        "concat" => return Some(decode_concat(reader, object, pointer, id)),
        "mode-only" => return Some(decode_mode_only(reader, object, pointer, id)),
        "package-licenses" => return Some(decode_package_licenses(reader, object, pointer, id)),
        "mirror" => return Some(decode_mirror(reader, object, pointer, id)),
        "toml-keys" => return Some(decode_toml_keys(reader, object, pointer, id)),
        "json-keys" => return Some(decode_json_keys(reader, object, pointer, id)),
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
