//! What:
//!  One text value inside one object of a JSONC document,
//!  read without deciding what the value means.
//! Why:
//!  The root-owned handler configuration is one JSONC document holding objects
//!  that other parts of the route own,
//!  and each part reads its own object out of it
//!  (`package/git-policy/cli/src/native/start_probe/probe_configuration.rs`
//!  reads `startProbe`,
//!  the clone-mount sweep reads `cloneMounts`).
//!  A reader that answered only text would hide a value of the wrong type,
//!  so a member that is there and is not text is its own answer,
//!  and a document that does not parse is another.
//!  Nothing here resolves a path or checks a name:
//!  what the text must be is the caller's rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const value = textOfMember(document, 'cloneMounts', 'parent'); // { kind: 'not-jsonc' | 'absent' | 'not-text' | 'text' }
//! ```

use monochromatic_jsonc_edit::units_to_string;

/// What:
///  What one nested member's value is.
/// Why:
///  A caller needs to tell a document it cannot read from an object another
///  part of the route has not written yet,
///  and from a value of the wrong type,
///  because each sends a person somewhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MemberReading = { kind: 'document-not-jsonc' } | { kind: 'absent' } | { kind: 'not-text' } | { kind: 'text'; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberReading {
    /// The document is not one JSONC value,
    /// so no member can be read from it.
    DocumentNotJsonc,
    /// The object or the member is not there.
    Absent,
    /// The member is there and its value is not text,
    /// or its text is not UTF-8.
    NotText,
    /// The member's text.
    Text(String),
}

impl MemberReading {
    /// The text,
    /// or nothing for every answer but one.
    pub fn text(&self) -> Option<&str> {
        return match self {
            MemberReading::Text(text) => Some(text.as_str()),
            MemberReading::DocumentNotJsonc | MemberReading::Absent | MemberReading::NotText => {
                None
            }
        };
    }
}

/// What:
///  Read the text at `object`.`key` in one JSONC document.
/// Why:
///  Every reader of the handler's configuration asks the same two questions,
///  and one function keeps the answer's shape the same for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = textOfMember(document, 'cloneMounts', 'checkout');
/// ```
pub fn text_of_member(
    document: &str,
    object: &str,
    key: &str,
) -> MemberReading {
    let root: monochromatic_jsonc_edit::JsoncValue =
        match monochromatic_jsonc_edit::parse_jsonc(document) {
            Ok(value) => value,
            Err(_error) => return MemberReading::DocumentNotJsonc,
        };
    let Some(entries) = root.entries() else {
        return MemberReading::Absent;
    };
    let object_units: Vec<u16> = object.encode_utf16().collect();
    let Some(object_entry) = entries
        .iter()
        .find(|entry| return entry.key.units == object_units)
    else {
        return MemberReading::Absent;
    };
    let Some(object_entries) = object_entry.value.entries() else {
        return MemberReading::Absent;
    };
    let key_units: Vec<u16> = key.encode_utf16().collect();
    let Some(entry) = object_entries
        .iter()
        .find(|entry| return entry.key.units == key_units)
    else {
        return MemberReading::Absent;
    };
    let Some(units) = entry.value.text_units() else {
        return MemberReading::NotText;
    };
    return match units_to_string(units) {
        Ok(text) => MemberReading::Text(text),
        Err(_error) => MemberReading::NotText,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "jsonc_member_text_tests.rs"]
mod tests;
