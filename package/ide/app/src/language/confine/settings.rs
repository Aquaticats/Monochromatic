//! The settings a confined server gets:
//!  overrides merged into the server's JSON settings table.

/// `Value` is any JSON value;
/// `json!` builds one from literal syntax.
use serde_json::{Value, json};

/// What:
///  Set one nested member of a JSON object,
///  creating objects on the way.
///  `&mut Value`
///       lends the value for modification;
///  `&[&str]` is a borrowed list of member names.
/// Why:
///  Overrides are merged into a settings table the user of the seam already filled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setPath(target: Record<string, unknown>, path: string[], leaf: unknown): void
/// ```
fn set_path(target: &mut Value, path: &[&str], leaf: Value) {
    // `split_first` returns the first name and the rest, or nothing for an empty path.
    let Some((first, rest)) = path.split_first() else {
        *target = leaf;
        return;
    };
    if !target.is_object() {
        *target = json!({});
    }
    // `as_object_mut` borrows the object's member table; `entry(...).or_insert` finds or creates a member.
    if let Some(members) = target.as_object_mut() {
        let child = members.entry((*first).to_string()).or_insert(Value::Null);
        set_path(child, rest, leaf);
    }
}

/// What:
///  The settings a server gets inside the sandbox.
///  `Option<Value>` is "settings,
///  or none".
/// Why:
///  Without network,
///  automatic type acquisition can only fail;
///  switching it off keeps the
///      TypeScript servers from starting `npm` at all (measured in the confinement plan).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function confinedSettings(server: string, settings?: unknown): unknown
/// ```
pub fn confined_settings(server: &str, settings: Option<&Value>) -> Option<Value> {
    // `cloned()` copies the borrowed settings into an owned value, when there are any.
    let mut result = settings.cloned();
    if server == "typescript-native" {
        // `get_or_insert` fills an absent value with an empty object first.
        let table = result.get_or_insert(json!({}));
        let off = ["tsserver", "automaticTypeAcquisition", "enabled"];
        set_path(table, &["typescript", off[0], off[1], off[2]], json!(false));
        set_path(table, &["js/ts", off[0], off[1], off[2]], json!(false));
    }
    if server == "typescript-language-server" {
        let table = result.get_or_insert(json!({}));
        set_path(table, &["disableAutomaticTypingAcquisition"], json!(true));
    }
    return result;
}
