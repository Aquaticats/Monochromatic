//! What:
//! Reading a crates.io answer into the records the policy judges:
//! the sparse index's publish times and yanked flags,
//! the API's version history and owner list,
//! and the cache file that keeps a history on disk.
//! Why:
//! Published history never changes,
//! so a history that already lists every version being checked is read from
//! `.cache/cargo-shadow/crates-io/<name>.json` with no request,
//! and every text that is read is checked before a record is built from it.
//! Split out of `registry.rs`;
//! the public readers are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // registry_record_reading.ts: parseIndexPubtimes(text), parseIndexYanked(text), parseVersionHistory(text), parseOwnerIds(text), readCachedHistory(path)
//! ```

/// What:
/// `use crate::http::RegistryError;` imports the failure of a registry lookup.
/// Why:
/// A body that is not the JSON crates.io documents is reported as one,
/// with no HTTP status and so no retry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegistryError } from 'crate/http';
/// ```
use crate::http::RegistryError;
/// What:
/// `use crate::policy::VersionRecord;` imports the record of one published release that the trust rule reads.
/// Why:
/// The API's version history and the cache are both read into a list of these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { VersionRecord } from 'crate/policy';
/// ```
use crate::policy::VersionRecord;
/// What:
/// `use serde_json::Value;` imports the external crate's JSON value type (siblings:
/// `serde_json::Map`,
/// one object,
/// and `serde_json::Number`,
/// one number).
/// Why:
/// A body or an index line is parsed before any field is read from it,
/// and the parse answer is this type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JSON.parse(text): unknown
/// ```
use serde_json::Value;
/// What:
/// `use std::collections::{BTreeMap, BTreeSet};` imports the sorted map and the sorted set (siblings:
/// `HashMap`/`HashSet`,
/// unordered and faster).
/// Why:
/// Sorted iteration makes the reading order of the answers and the tests deterministic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // new Map() / new Set(), which keep insertion order
/// ```
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// A cached history is one text file,
/// and a missing or unreadable one is an empty history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::path::Path;` imports the borrowed path type from the standard library so this file can name it unqualified.
/// Why:
/// The cache file is only read here,
/// so its owned `PathBuf` is not named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;

/// Reads a string field,
/// empty when absent or not a string.
fn string_field(
    object: &Value,
    key: &str,
) -> String {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return object
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
}

/// Reads an id that crates.io writes as a number and the cache writes as text.
fn id_field(
    object: &Value,
    key: &str,
) -> String {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match object.get(key) {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        Some(Value::Number(number)) => number.to_string(),
        // `Some`: the present or the absent value of an `Option`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        Some(Value::String(text)) => text.clone(),
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        _ => String::new(),
    };
}

/// What:
/// `fn index_line_error(error: serde_json::Error) -> RegistryError` turns a line of the sparse index that is not valid JSON into a registry error with no HTTP status.
/// Why:
/// A damaged index line stops the policy check with a message that names the index,
/// and retrying would not help.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexLineError = (error: Error) => new RegistryError(`sparse index line: ${error.message}`);
/// ```
fn index_line_error(error: serde_json::Error) -> RegistryError {
    return RegistryError {
        message: format!("sparse index line: {error}"),
        status: None,
        retry_after_seconds: 0,
    };
}

/// The sparse index lines as JSON values;
/// a line that is not JSON is an error naming the crate.
fn index_lines(text: &str) -> Result<Vec<Value>, RegistryError> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `records: Vec<Value>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut records: Vec<Value> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // Typed binding: `record: Value`.
        let record: Value = serde_json::from_str(line)
            // `None`: the present or the absent value of an `Option`.
            // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
            // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
            // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = await expr;   // throws to the caller on failure
            // ```
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
            // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { ... } catch (error) { throw new MyError(String(error)); }
            // ```
            // Macro `format!`: builds or prints text (or a list) from a template.
            .map_err(index_line_error)?;
        records.push(record);
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(records);
}

/// Publish times by version from sparse index text.
pub fn parse_index_pubtimes(text: &str) -> Result<BTreeMap<String, String>, RegistryError> {
    // `let mut`: a binding that may change.
    // Typed binding: `times: BTreeMap<String, String>`.
    // Constructor `BTreeMap::new()`: an empty owned collection or a boxed value.
    // What:     `BTreeSet<T>`/`BTreeMap<K, V>` are sorted sets/maps (siblings: `HashSet`/`HashMap`, unordered and faster).
    // Why:      Sorted iteration makes the output and the tests deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut times: BTreeMap<String, String> = BTreeMap::new();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    for record in index_lines(text)? {
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // Adapter `.and_then(`: transforms or selects items with a closure.
        if record.get("vers").and_then(Value::as_str).is_some() {
            // Borrow `&record`: lends the value without copying or giving it away.
            times.insert(
                string_field(&record, "vers"),
                string_field(&record, "pubtime"),
            );
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(times);
}

/// Yanked versions from sparse index text.
pub fn parse_index_yanked(text: &str) -> Result<BTreeSet<String>, RegistryError> {
    // `let mut`: a binding that may change.
    // Typed binding: `yanked: BTreeSet<String>`.
    // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
    let mut yanked: BTreeSet<String> = BTreeSet::new();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    for record in index_lines(text)? {
        // `Some`: the present or the absent value of an `Option`.
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // Adapter `.and_then(`: transforms or selects items with a closure.
        if record.get("yanked").and_then(Value::as_bool) == Some(true) {
            // Borrow `&record`: lends the value without copying or giving it away.
            yanked.insert(string_field(&record, "vers"));
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(yanked);
}

/// The failure for a body that is not the JSON the API documents.
fn body_error(
    what: &str,
    error: impl std::fmt::Display,
) -> RegistryError {
    // `None`: the present or the absent value of an `Option`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return RegistryError {
        message: format!("{what}: {error}"),
        status: None,
        retry_after_seconds: 0,
    };
}

/// What:
/// `pub fn parse_version_history(text: &str) -> Result<Vec<VersionRecord>, RegistryError>` reads the API's
/// `/crates/<name>/versions` body.
/// Why:
/// Only the fields the trust rule reads are kept:
/// number,
/// publish time,
/// publishing account,
/// Trusted Publishing repository
/// and declared repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseVersionHistory(text: string): VersionRecord[]
/// ```
pub fn parse_version_history(text: &str) -> Result<Vec<VersionRecord>, RegistryError> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `body: Value`.
    let body: Value = match serde_json::from_str(text) {
        Ok(found) => found,
        Err(error) => return Err(body_error("version list", error)),
    };
    // `let mut`: a binding that may change.
    // Typed binding: `records: Vec<VersionRecord>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut records: Vec<VersionRecord> = Vec::new();
    // Typed binding: `empty: Vec<Value>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let empty: Vec<Value> = Vec::new();
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Borrow `&empty`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.and_then(`: transforms or selects items with a closure.
    for version in body
        .get("versions")
        .and_then(Value::as_array)
        .unwrap_or(&empty)
    {
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Typed binding: `publisher: Value`.
        let publisher: Value = version.get("published_by").cloned().unwrap_or(Value::Null);
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Typed binding: `trustpub: Value`.
        let trustpub: Value = version.get("trustpub_data").cloned().unwrap_or(Value::Null);
        records.push(VersionRecord {
            num: string_field(version, "num"),
            created_at: string_field(version, "created_at"),
            // Borrow `&publisher`: lends the value without copying or giving it away.
            publisher_id: id_field(&publisher, "id"),
            // Borrow `&trustpub`: lends the value without copying or giving it away.
            trustpub_repository: string_field(&trustpub, "repository"),
            repository: string_field(version, "repository"),
        });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(records);
}

/// The account ids of listed individual owners (`kind: user`) from the API's owners body.
pub fn parse_owner_ids(text: &str) -> Result<BTreeSet<String>, RegistryError> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `body: Value`.
    let body: Value = match serde_json::from_str(text) {
        Ok(found) => found,
        Err(error) => return Err(body_error("owner list", error)),
    };
    // `let mut`: a binding that may change.
    // Typed binding: `ids: BTreeSet<String>`.
    // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
    let mut ids: BTreeSet<String> = BTreeSet::new();
    // Typed binding: `empty: Vec<Value>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let empty: Vec<Value> = Vec::new();
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Borrow `&empty`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.and_then(`: transforms or selects items with a closure.
    for user in body
        .get("users")
        .and_then(Value::as_array)
        .unwrap_or(&empty)
    {
        if string_field(user, "kind") == "user" {
            // Typed binding: `id: String`.
            let id: String = id_field(user, "id");
            if !id.is_empty() {
                ids.insert(id);
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(ids);
}

/// The cache file format:
/// the history as a JSON array with camelCase keys.
pub(crate) fn history_to_json(history: &[VersionRecord]) -> Value {
    // `let mut`: a binding that may change.
    // Typed binding: `entries: Vec<Value>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut entries: Vec<Value> = Vec::new();
    for record in history {
        // `let mut`: a binding that may change.
        // Typed binding: `entry: serde_json::Map<String, Value>`.
        let mut entry: serde_json::Map<String, Value> = serde_json::Map::new();
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("num".to_string(), Value::String(record.num.clone()));
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert(
            "createdAt".to_string(),
            Value::String(record.created_at.clone()),
        );
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert(
            "publisherId".to_string(),
            Value::String(record.publisher_id.clone()),
        );
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert(
            "trustpubRepository".to_string(),
            Value::String(record.trustpub_repository.clone()),
        );
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert(
            "repository".to_string(),
            Value::String(record.repository.clone()),
        );
        entries.push(Value::Object(entry));
    }
    return Value::Array(entries);
}

/// Reads a cached history;
/// a missing or unreadable cache file is an empty history (the API then fills it).
pub(crate) fn read_cached_history(path: &Path) -> Vec<VersionRecord> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: String`.
    let text: String = match fs::read_to_string(path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        Err(_) => return Vec::new(),
    };
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&text`: lends the value without copying or giving it away.
    // Typed binding: `parsed: Value`.
    let parsed: Value = match serde_json::from_str(&text) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        Err(_) => return Vec::new(),
    };
    // `let mut`: a binding that may change.
    // Typed binding: `records: Vec<VersionRecord>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut records: Vec<VersionRecord> = Vec::new();
    // Borrow `&Vec`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    for entry in parsed.as_array().unwrap_or(&Vec::new()) {
        records.push(VersionRecord {
            num: string_field(entry, "num"),
            created_at: string_field(entry, "createdAt"),
            publisher_id: string_field(entry, "publisherId"),
            trustpub_repository: string_field(entry, "trustpubRepository"),
            repository: string_field(entry, "repository"),
        });
    }
    return records;
}
