//! What:
//!  Build JSON objects without the `serde_json::json!` macro.
//! Why:
//!  `json!` expands to `Result::unwrap`, which the repository's `disallowed_methods` lint denies (the same cause as the
//!  schemars derive; see doc/troubleshooting/schemars-derive-trips-disallowed-methods.md), so objects are assembled
//!  from `Map::insert` calls instead.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function object(pairs: [string, unknown][]): Record<string, unknown>
//! ```

use serde_json::{Map, Value};

/// What:
///  Build a JSON object from key and value pairs, keeping their order.
///  `&[(&str, Value)]` is a borrowed list of pairs.
/// Why:
///  A plain function the lint accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function object(pairs: [string, unknown][]): Record<string, unknown>
/// ```
pub fn object(pairs: &[(&str, Value)]) -> Value {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), value.clone());
    }
    return Value::Object(map);
}
