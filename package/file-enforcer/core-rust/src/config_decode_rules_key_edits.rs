//! What:
//!  Read the `edits` array that the `toml-keys` and `json-keys` kinds share:
//! one guarded key assignment per item.
//! Why:
//!  Both kinds make the same assignment,
//! so one reader keeps the accepted keys equal in both;
//!  split out of `config_decode_rules`,
//! which keeps the dispatcher.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // config_decode_rules_key_edits.ts
//! function decodeEdits(reader: Reader, object: Map<string, Value>, pointer: string): KeyEdit[]
//! ```

/// Imports `KeyEdit` from `crate::config_parts`.
use crate::config_parts::KeyEdit;
/// Imports `Reader` from `crate::config_reader`.
use crate::config_reader::Reader;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{
    Map,
    Value,
};

/// What:
///  Read the `edits` array shared by the `toml-keys` and `json-keys` kinds.
/// Why:
///  Both kinds make the same guarded key assignment,
/// so one reader keeps the accepted keys equal in both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeEdits(reader: Reader, object: Map<string, Value>, pointer: string): KeyEdit[]
/// ```
pub(crate) fn decode_edits(
    reader: &mut Reader,
    object: &Map<String, Value>,
    pointer: &str,
) -> Vec<KeyEdit> {
    let mut edits: Vec<KeyEdit> = Vec::new();
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
            edits.push(KeyEdit {
                guard,
                path,
                value,
            });
        }
    }
    return edits;
}
