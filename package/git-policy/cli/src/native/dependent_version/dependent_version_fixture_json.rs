//! What: Read differential fixtures and write canonical results, for the tests that
//!       compare the native planner with the incumbent.
//! Why: Both implementations read the same case files and write one canonical JSON text
//!      per case: keys in a fixed order, no spaces, strings quoted as `JSON.stringify`
//!      quotes them, bytes as lower-case hexadecimal. Equal texts are equal results.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const output = JSON.stringify(result); // compared byte for byte with the native output
//! ```

/// Quoting that matches the incumbent's.
use crate::dependent_version_release::json_quote_units;
/// The crate's JSONC parser reads fixtures.
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue};

/// The member of a fixture object; a missing member is a broken fixture.
pub fn field<'value>(value: &'value JsoncValue, key: &str) -> &'value JsoncValue {
    return optional_field(value, key).unwrap_or_else(|| panic!("fixture lacks {key:?}"));
}

/// The member of a fixture object, if present.
pub fn optional_field<'value>(value: &'value JsoncValue, key: &str) -> Option<&'value JsoncValue> {
    return value.entries().and_then(|entries| {
        return entries
            .iter()
            .rev()
            .find(|entry| return entry.key.units.iter().copied().eq(key.encode_utf16()))
            .map(|entry| return &entry.value);
    });
}

/// The elements of a fixture array.
pub fn elements(value: &JsoncValue) -> &[JsoncValue] {
    return value
        .elements()
        .unwrap_or_else(|| panic!("fixture value is not an array: {value:?}"));
}

/// The code units of a fixture string.
pub fn units(value: &JsoncValue) -> Vec<u16> {
    return value
        .text_units()
        .unwrap_or_else(|| panic!("fixture value is not a string: {value:?}"))
        .to_vec();
}

/// A fixture string as Rust text; fixtures carry no unpaired surrogates where text is read.
pub fn text(value: &JsoncValue) -> String {
    return String::from_utf16(&units(value)).unwrap_or_else(|error| panic!("{error}"));
}

/// Fixture strings as Rust text.
pub fn texts(value: &JsoncValue) -> Vec<String> {
    return elements(value).iter().map(text).collect();
}

/// Whether a fixture value is `null`.
pub fn is_null(value: &JsoncValue) -> bool {
    return matches!(value.kind, JsoncKind::Null);
}

/// Whether a fixture value is `true`.
pub fn is_true(value: &JsoncValue) -> bool {
    return matches!(value.kind, JsoncKind::Boolean { value: true });
}

/// The bytes of a lower-case hexadecimal fixture string.
pub fn hex(value: &JsoncValue) -> Vec<u8> {
    let digits: Vec<u16> = units(value);
    return digits
        .chunks(2)
        .map(|pair| {
            let high: u32 = char::from_u32(u32::from(pair[0]))
                .and_then(|c| return c.to_digit(16))
                .unwrap_or_else(|| panic!("bad hex"));
            let low: u32 = char::from_u32(u32::from(pair[1]))
                .and_then(|c| return c.to_digit(16))
                .unwrap_or_else(|| panic!("bad hex"));
            return u8::try_from(high * 16 + low).unwrap_or_else(|error| panic!("{error}"));
        })
        .collect();
}

/// Text quoted as `JSON.stringify` quotes it.
pub fn quote(value: &str) -> String {
    return json_quote_units(&value.encode_utf16().collect::<Vec<u16>>());
}

/// Bytes as a quoted lower-case hexadecimal string.
pub fn quote_hex(bytes: &[u8]) -> String {
    let digits: String = bytes
        .iter()
        .map(|byte| return format!("{byte:02x}"))
        .collect();
    return format!("\"{digits}\"");
}

/// A canonical JSON array of already canonical items.
pub fn array(items: &[String]) -> String {
    return format!("[{}]", items.join(","));
}

/// A canonical JSON object of keys and already canonical values, in the given order.
pub fn object(members: &[(&str, String)]) -> String {
    let rendered: Vec<String> = members
        .iter()
        .map(|(key, value)| return format!("{}:{value}", quote(key)))
        .collect();
    return format!("{{{}}}", rendered.join(","));
}

/// The canonical text of a parsed fixture value, as compact `JSON.stringify` writes it.
pub fn canonical(value: &JsoncValue) -> String {
    match &value.kind {
        JsoncKind::Null => return String::from("null"),
        JsoncKind::Boolean { value: flag } => return flag.to_string(),
        JsoncKind::Number { raw, .. } => return raw.clone(),
        JsoncKind::Text { units, .. } => return json_quote_units(units),
        JsoncKind::Array { elements } => {
            return array(&elements.iter().map(canonical).collect::<Vec<String>>());
        }
        JsoncKind::Record { entries } => {
            let rendered: Vec<String> = entries
                .iter()
                .map(|entry| {
                    return format!(
                        "{}:{}",
                        json_quote_units(&entry.key.units),
                        canonical(&entry.value)
                    );
                })
                .collect();
            return format!("{{{}}}", rendered.join(","));
        }
    }
}
