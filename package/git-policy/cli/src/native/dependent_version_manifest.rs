//! What: Read the facts a dependent bump needs from one `package.json`: its name, its
//!       version, and the workspace names it depends on.
//! Why: The incumbent parses manifests with `JSON.parse` (`manifest-text.ts:130-161`). This
//!      module accepts exactly JSON: it parses with the crate's JSONC parser and first
//!      refuses the two things JSONC adds, comments and trailing commas. Keys follow
//!      `JSON.parse`: the last duplicate wins. A manifest is decoded as strict UTF-8 after
//!      one optional byte-order mark, as the incumbent's fatal `TextDecoder` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const facts = readManifestDependencyFacts({ path, text: new TextDecoder('utf-8', { fatal: true }).decode(bytes) });
//! ```

/// What: `use` brings names from sibling files and the JSONC crate into this file.
/// Why:  Failures are typed by cause; paths are shown as text in messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PolicyIncomplete, displayPath } from './dependent_version_content.ts';
/// ```
use super::dependent_version_content::{PolicyIncomplete, display_path};
/// The crate's JSONC parser and its value model.
use monochromatic_jsonc_edit::{JsoncEntry, JsoncValue, parse_jsonc};

/// What: The byte-order mark a UTF-8 decoder removes before the text. `char` is one Unicode
///       scalar value.
/// Why:  The incumbent's `TextDecoder` strips one; the planner keeps it in patched bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BYTE_ORDER_MARK = '﻿';
/// ```
const BYTE_ORDER_MARK: char = '\u{feff}';

/// What: Fields installers resolve, so every workspace package named there is an edge.
/// Why:  These edges carry a bump whether or not the dependent bundles the package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RUNTIME_DEPENDENCY_FIELDS = ['dependencies', 'peerDependencies', 'optionalDependencies'];
/// ```
const RUNTIME_DEPENDENCY_FIELDS: &[&str] =
    &["dependencies", "peerDependencies", "optionalDependencies"];

/// What: The dependency facts of one manifest. `Option<Vec<u16>>` is "UTF-16 code units, or
///       nothing": the version is kept as JavaScript holds it, so an escaped unpaired
///       surrogate survives comparison and quoting.
/// Why:  The plan compares versions between states, bumps release versions, and walks
///       names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManifestDependencyFacts = { name: string; version?: string; runtimeDependencyNames: string[]; devDependencyNames: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestFacts {
    /// The package name.
    pub name: String,
    /// The version, absent when the manifest declares none.
    pub version: Option<Vec<u16>>,
    /// Names under `dependencies`, `peerDependencies` and `optionalDependencies`, in that
    /// field order, each field's names in first-occurrence order.
    pub runtime_dependency_names: Vec<String>,
    /// Names under `devDependencies`.
    pub dev_dependency_names: Vec<String>,
}

/// What: Decode file bytes as strict UTF-8 and drop one leading byte-order mark.
///       `std::str::from_utf8` checks the bytes and borrows them as text.
/// Why:  A file that is not UTF-8 cannot be read as JSON or YAML text. The caller learns
///       how many bytes the mark took from the length difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
/// ```
pub fn strict_text<'bytes>(
    path: &[u8],
    bytes: &'bytes [u8],
) -> Result<&'bytes str, PolicyIncomplete> {
    // `match` on a `Result`: `Ok` holds the text, `Err` the decoding failure.
    let text: &str = match std::str::from_utf8(bytes) {
        Ok(decoded) => decoded,
        Err(_) => {
            return Err(PolicyIncomplete::NotUtf8 {
                path: path.to_vec(),
            });
        }
    };
    return Ok(text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text));
}

/// What: Find a comment or a trailing comma outside strings, the two JSONC extensions.
///       `Option<&str>` is "a description, or nothing".
/// Why:  `JSON.parse` rejects both; the JSONC parser accepts both. Everything else the
///       JSONC parser rejects, `JSON.parse` rejects too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncExtension(text: string): string | undefined;
/// ```
fn jsonc_extension(text: &str) -> Option<&'static str> {
    let mut in_string: bool = false;
    let mut escaped: bool = false;
    let mut comma_pending: bool = false;
    // `.bytes()` visits the UTF-8 bytes; every byte that matters here is ASCII.
    for byte in text.bytes() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'/' => return Some("comments"),
            b',' => comma_pending = true,
            b'}' | b']' if comma_pending => return Some("trailing commas"),
            b' ' | b'\t' | b'\r' | b'\n' => {}
            b'"' => {
                in_string = true;
                comma_pending = false;
            }
            _ => comma_pending = false,
        }
    }
    if comma_pending {
        return Some("trailing commas");
    }
    return None;
}

/// What: Parse text as one JSON value of any kind.
/// Why:  The JSONC parser accepts only an object or array at the root, while `JSON.parse`
///       accepts any value; wrapping the text in `[` and `]` and requiring exactly one
///       element accepts exactly the JSON values. A non-object root then fails as a shape
///       problem, as in the incumbent, instead of as a syntax error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value: unknown = JSON.parse(text);
/// ```
fn parse_json(path: &[u8], text: &str) -> Result<JsoncValue, PolicyIncomplete> {
    let syntax = |detail: String| {
        return PolicyIncomplete::ManifestSyntax {
            path: path.to_vec(),
            detail,
        };
    };
    // `if let Some(found) = ...` runs only when an extension was found.
    if let Some(extension) = jsonc_extension(text) {
        return Err(syntax(format!("JSON allows no {extension}")));
    }
    let wrapped: JsoncValue = match parse_jsonc(&format!("[{text}]")) {
        Ok(value) => value,
        Err(error) => return Err(syntax(error.message)),
    };
    // A slice pattern with one binding matches a list of exactly one element.
    if let Some([single]) = wrapped.elements() {
        return Ok(single.clone());
    }
    return Err(syntax(String::from(
        "the text is not exactly one JSON value",
    )));
}

/// What: The value of the last member with a key, as `JSON.parse` keeps it.
///       `.iter().rev().find(...)` searches from the end.
/// Why:  A duplicated key's last value wins in JavaScript.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = parsed[key];
/// ```
fn member<'value>(entries: &'value [JsoncEntry], key: &str) -> Option<&'value JsoncValue> {
    return entries
        .iter()
        .rev()
        .find(|entry: &&JsoncEntry| return entry.key.units.iter().copied().eq(key.encode_utf16()))
        .map(|entry: &JsoncEntry| return &entry.value);
}

/// What: The keys of one dependency field, unique, in first-occurrence order; nothing when
///       the field is absent or not an object.
/// Why:  A key holding an unpaired surrogate cannot be a workspace name (the planner
///       refuses such names), so dropping it cannot change any edge.
/// Gotcha: `Object.keys` lists integer-like keys first; this keeps source order. No result
///         of the plan depends on the order of names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return isJsonObject(manifest[field]) ? Object.keys(manifest[field]) : [];
/// ```
fn dependency_names(entries: &[JsoncEntry], field: &str, names: &mut Vec<String>) {
    let start: usize = names.len();
    let fields: &[JsoncEntry] = member(entries, field)
        .and_then(JsoncValue::entries)
        .unwrap_or(&[]);
    for entry in fields {
        // `String::from_utf16` fails on an unpaired surrogate; `if let Ok` keeps the rest.
        if let Ok(name) = String::from_utf16(&entry.key.units)
            && !names[start..].contains(&name)
        {
            names.push(name);
        }
    }
}

/// What: A shape problem with the incumbent's message for a manifest.
/// Why:  Shape problems compare message for message with the incumbent's
///       `ManifestShapeError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new ManifestShapeError(`${path} ${problem}`);
/// ```
pub fn shape(path: &[u8], problem: &str) -> PolicyIncomplete {
    return PolicyIncomplete::ManifestShape {
        path: path.to_vec(),
        problem: format!("{} {problem}", display_path(path)),
    };
}

/// What: Parse the dependency facts of one manifest's text.
/// Why:  A manifest must be a JSON object with a string `name` and, when present, a string
///       `version`; dependency fields that are not objects contribute nothing.
/// Gotcha: A name holding an escaped unpaired surrogate is refused here; the incumbent
///         accepts it, but no npm package can carry it and messages could not show it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readManifestDependencyFacts({ path, text }): ManifestDependencyFacts;
/// ```
pub fn read_manifest_facts(path: &[u8], text: &str) -> Result<ManifestFacts, PolicyIncomplete> {
    let value: JsoncValue = parse_json(path, text)?;
    let Some(entries) = value.entries() else {
        return Err(shape(path, "is not a JSON object"));
    };
    let Some(name_units) = member(entries, "name").and_then(JsoncValue::text_units) else {
        return Err(shape(path, "has no string \"name\""));
    };
    let Ok(name) = String::from_utf16(name_units) else {
        return Err(shape(
            path,
            "has a \"name\" holding an unpaired UTF-16 surrogate",
        ));
    };
    let version: Option<Vec<u16>> = match member(entries, "version") {
        None => None,
        Some(found) => match found.text_units() {
            Some(units) => Some(units.to_vec()),
            None => return Err(shape(path, "has a non-string \"version\"")),
        },
    };
    let mut runtime_dependency_names: Vec<String> = Vec::new();
    for field in RUNTIME_DEPENDENCY_FIELDS {
        dependency_names(entries, field, &mut runtime_dependency_names);
    }
    let mut dev_dependency_names: Vec<String> = Vec::new();
    dependency_names(entries, "devDependencies", &mut dev_dependency_names);
    return Ok(ManifestFacts {
        name,
        version,
        runtime_dependency_names,
        dev_dependency_names,
    });
}

/// Manifest decoding, strict JSON and fact extraction.
#[cfg(test)]
#[path = "dependent_version_manifest_tests.rs"]
mod tests;
