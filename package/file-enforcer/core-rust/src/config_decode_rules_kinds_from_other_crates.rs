//! What:
//!  Decode the three rule kinds another crate supplies a builder for:
//! `rust-catalog`,
//! `browserslist` and
//! `jetbrains-harper`.
//! Why:
//!  This crate checks the fields these kinds accept but never implements them,
//! so their decoders sit apart from the built-in ones;
//!  split out of `config_decode_rules`,
//! which keeps the dispatcher.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // config_decode_rules_kinds_from_other_crates.ts
//! function decodeRustCatalog(...): RuleConfig
//! function decodeBrowserslist(...): RuleConfig
//! function decodeJetbrainsHarper(...): RuleConfig
//! ```

/// Imports `repo_path` from `crate::config_decode_rules_path_and_mode_readers`.
use crate::config_decode_rules_path_and_mode_readers::repo_path;
/// Imports `HarperScopedServer` from `crate::config_parts`.
use crate::config_parts::HarperScopedServer;
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

/// Decodes the fields of a `rust-catalog` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_rust_catalog(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(object, pointer, &["kind", "id", "catalog", "manifests"]);
    let catalog = repo_path(reader, object, "catalog", pointer);
    let manifests = reader.text_list(object, "manifests", pointer);
    return RuleConfig::RustCatalog {
        id,
        catalog,
        manifests,
    };
}

/// Decodes the fields of a `browserslist` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_browserslist(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    reader.reject_unknown(
        object,
        pointer,
        &["kind", "id", "source", "output", "lock", "environment"],
    );
    let source = repo_path(reader, object, "source", pointer);
    let output = repo_path(reader, object, "output", pointer);
    let lock = repo_path(reader, object, "lock", pointer);
    let environment = reader.optional_text(object, "environment", pointer);
    return RuleConfig::Browserslist {
        id,
        source,
        output,
        lock,
        environment,
    };
}

/// Decodes the fields of a `jetbrains-harper` rule into a `RuleConfig`,
/// recording every problem in the reader.
pub(crate) fn decode_jetbrains_harper(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
    id: String,
) -> RuleConfig {
    let allowed = [
        "kind",
        "id",
        "productPrefixes",
        "disabledLinters",
        "excludePatterns",
        "scopedServers",
    ];
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
    return RuleConfig::JetbrainsHarper {
        id,
        product_prefixes,
        disabled_linters,
        exclude_patterns,
        scoped_servers,
    };
}
