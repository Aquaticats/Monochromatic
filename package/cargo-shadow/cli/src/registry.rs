//! What:
//! The crates.io lookups the policy needs:
//! the sparse index for publish times and yanked flags,
//! the web API for the full
//! version history and the owner list,
//! with the history cached on disk.
//! Why:
//! A published version never changes,
//! so a history that already lists every version being checked is read from
//! `.cache/cargo-shadow/crates-io/<name>.json` with no request.
//! The API allows about one request per second,
//! so requests
//! are spaced and an HTTP 429 is retried after its `Retry-After`,
//! or a doubling backoff,
//! at most five times.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // registry.ts
//! export type Registry = { pubtimes(name): Map<string, string>; yanked(name): Set<string>; history({ name, versions }): VersionRecord[]; owners(name): Set<string> };
//! export function createRegistry({ root, fetchText }): Registry
//! ```

/// What:
/// `use std::cell::Cell;` imports `Cell` from the standard library so this file can name it unqualified.
/// Why:
/// `Cell` is used by `CratesIo`,
/// `new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Cell } from 'std/cell';
/// ```
use std::cell::Cell;
/// What:
/// `use std::collections::{BTreeMap, BTreeSet};` imports `BTreeMap`,
/// `BTreeSet` from the standard library so this file can name them unqualified.
/// Why:
/// `BTreeMap` is used by `pubtimes`,
/// `parse_index_pubtimes`;
/// `BTreeSet` is used by `yanked`,
/// `owners`,
/// `prefetch`,
/// `parse_index_yanked`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeMap, BTreeSet } from 'std/collections';
/// ```
use std::collections::{BTreeMap, BTreeSet};
/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `read_cached_history`,
/// `history`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `read_cached_history`,
/// `with_defaults`,
/// `new`;
/// `PathBuf` is used by `CratesIo`,
/// `history`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};
/// What:
/// `use std::thread::sleep;` imports `sleep` from the standard library so this file can name it unqualified.
/// Why:
/// `sleep` is used by `spaced_api_get`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { sleep } from 'std/thread';
/// ```
use std::thread::sleep;
// Imports `std::time::{Duration, Instant}`.
// What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
// Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: T[] / Set / Map / number
// ```
/// What:
/// `use std::time::{Duration, Instant};` imports `Duration`,
/// `Instant` from the standard library so this file can name them unqualified.
/// Why:
/// `Duration` is used by `API_MIN_INTERVAL`,
/// `RATE_LIMIT_BACKOFF`,
/// `CratesIo`,
/// `new`,
/// and others;
/// `Instant` is used by `CratesIo`,
/// `spaced_api_get`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Duration, Instant } from 'std/time';
/// ```
use std::time::{Duration, Instant};

/// What:
/// `use serde_json::Value;` imports `Value` from the external crate `serde_json` so this file can name it unqualified.
/// Why:
/// `Value` is used by `string_field`,
/// `id_field`,
/// `index_lines`,
/// `parse_version_history`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Value } from 'serde_json';
/// ```
use serde_json::Value;

/// What:
/// `use crate::dump::fill_histories_from_dump;` imports `fill_histories_from_dump` from another module of this crate so this file can name it unqualified.
/// Why:
/// `fill_histories_from_dump` is used by `prefetch`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fill_histories_from_dump } from 'crate/dump';
/// ```
use crate::dump::fill_histories_from_dump;
/// What:
/// `use crate::http::{DumpSource, RegistryError, TextFetcher};` imports `DumpSource`,
/// `RegistryError`,
/// `TextFetcher` from another module of this crate so this file can name them unqualified.
/// Why:
/// `DumpSource` is used by `CratesIo`,
/// `with_defaults`,
/// `with_dump`,
/// `prefetch`;
/// `RegistryError` is used by `pubtimes`,
/// `yanked`,
/// `history`,
/// `owners`,
/// and others;
/// `TextFetcher` is used by `CratesIo`,
/// `with_defaults`,
/// `new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DumpSource, RegistryError, TextFetcher } from 'crate/http';
/// ```
use crate::http::{DumpSource, RegistryError, TextFetcher};
/// What:
/// `use crate::policy::VersionRecord;` imports `VersionRecord` from another module of this crate so this file can name it unqualified.
/// Why:
/// `VersionRecord` is used by `history`,
/// `parse_version_history`,
/// `history_to_json`,
/// `read_cached_history`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { VersionRecord } from 'crate/policy';
/// ```
use crate::policy::VersionRecord;

/// The sparse index host.
const SPARSE_INDEX_URL: &str = "https://index.crates.io";

/// The web API root.
const API_URL: &str = "https://crates.io/api/v1";

/// The least time between two API requests:
/// crates.io allows one per second,
/// plus a margin.
pub const API_MIN_INTERVAL: Duration = Duration::from_millis(1_100);

/// How many times a rate-limited request is tried.
/// Implementer's pick,
/// open to veto,
/// made under the owner's delegation of 2026-11-03 ("Pick options as if I had picked them and invite
/// veto when I say I'm back.";
/// doc/planning/cargo-shadow-update.md),
/// which may not reach a later round;
/// not a measurement.
/// The number of 429 answers crates.io gives in a row was not observed.
const RATE_LIMIT_ATTEMPTS: u32 = 5;

/// The longest pause for an HTTP 429 that cargo-shadow sits through.
/// Implementer's pick,
/// open to veto,
/// made under the owner's delegation of 2026-11-03 ("Pick options as if I had picked them and invite
/// veto when I say I'm back.";
/// doc/planning/cargo-shadow-update.md),
/// which may not reach a later round;
/// not a measurement.
/// No `Retry-After` value was observed in this repository's logs or runs.
/// Two minutes assumes crates.io asks for seconds;
/// a longer request ends the call,
/// so the wait never grows past this.
const MAX_PAUSE: Duration = Duration::from_secs(120);

/// The first backoff when a 429 carries no `Retry-After`;
/// it doubles each attempt.
/// Implementer's pick,
/// open to veto,
/// made under the owner's delegation of 2026-11-03 ("Pick options as if I had picked them and invite
/// veto when I say I'm back.";
/// doc/planning/cargo-shadow-update.md),
/// which may not reach a later round;
/// not a measurement.
/// Used only when a 429 carries no `Retry-After`;
/// the values crates.io sends were not observed.
const RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(30);

/// The HTTP status for "too many requests".
const TOO_MANY_REQUESTS: u16 = 429;

/// Where histories are cached,
/// relative to the repository root.
pub const HISTORY_CACHE_DIR: &str = ".cache/cargo-shadow/crates-io";

/// What:
/// `pub trait Registry { ... }` is what the checks ask of crates.io.
/// Why:
/// Tests give the checks a fake with canned histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Registry { pubtimes(name): Map<string, string>; yanked(name): Set<string>; history(args): VersionRecord[]; owners(name): Set<string> }
/// ```
pub trait Registry {
    /// Publish time (RFC 3339) of each version of a crate,
    /// from the sparse index;
    /// empty when the index has none.
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError>;
    /// The yanked versions of a crate.
    fn yanked(&self, name: &str) -> Result<BTreeSet<String>, RegistryError>;
    /// Every release of a crate;
    /// read from the cache when it lists all of `versions`.
    fn history(&self, name: &str, versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError>;
    /// The account ids of the crate's listed individual owners.
    fn owners(&self, name: &str) -> Result<BTreeSet<String>, RegistryError>;
    /// Warms the history cache for these crates before they are asked for one by one;
    /// a registry with nothing to warm does nothing.
    fn prefetch(&self, _names: &BTreeSet<String>) -> Result<(), RegistryError> {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        return Ok(());
    }
}

/// The database dump,
/// a gzip stream of a tar archive.
pub const DATABASE_DUMP_URL: &str = "https://static.crates.io/db-dump.tar.gz";

/// How many uncached crates make the dump cheaper than the API (one request per 1.1 seconds each).
pub const DUMP_BREAKEVEN_CRATES: usize = 100;

/// The sparse index path of a crate:
/// `1/a`,
/// `2/ab`,
/// `3/a/abc`,
/// `ab/cd/abcdef`.
pub fn sparse_index_path(name: &str) -> String {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let lower: String = name.to_lowercase();
    if lower.len() <= 2 {
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        return format!("{}/{lower}", lower.len());
    }
    if lower.len() == 3 {
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // Macro `format!`: builds or prints text (or a list) from a template.
        return format!("3/{}/{lower}", &lower[..1]);
    }
    // Borrow `&lower`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!("{}/{}/{lower}", &lower[..2], &lower[2..4]);
}

/// Reads a string field,
/// empty when absent or not a string.
fn string_field(object: &Value, key: &str) -> String {
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
    return object.get(key).and_then(Value::as_str).unwrap_or("").to_string();
}

/// Reads an id that crates.io writes as a number and the cache writes as text.
fn id_field(object: &Value, key: &str) -> String {
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
    return RegistryError { message: format!("sparse index line: {error}"), status: None, retry_after_seconds: 0 };
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
            times.insert(string_field(&record, "vers"), string_field(&record, "pubtime"));
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
fn body_error(what: &str, error: impl std::fmt::Display) -> RegistryError {
    // `None`: the present or the absent value of an `Option`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return RegistryError { message: format!("{what}: {error}"), status: None, retry_after_seconds: 0 };
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
    for version in body.get("versions").and_then(Value::as_array).unwrap_or(&empty) {
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
    for user in body.get("users").and_then(Value::as_array).unwrap_or(&empty) {
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
fn history_to_json(history: &[VersionRecord]) -> Value {
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
        entry.insert("createdAt".to_string(), Value::String(record.created_at.clone()));
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("publisherId".to_string(), Value::String(record.publisher_id.clone()));
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("trustpubRepository".to_string(), Value::String(record.trustpub_repository.clone()));
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("repository".to_string(), Value::String(record.repository.clone()));
        entries.push(Value::Object(entry));
    }
    return Value::Array(entries);
}

/// Reads a cached history;
/// a missing or unreadable cache file is an empty history (the API then fills it).
fn read_cached_history(path: &Path) -> Vec<VersionRecord> {
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

/// crates.io through a fetcher,
/// with spaced API requests and the on-disk history cache.
pub struct CratesIo {
    /// What:
    /// Field `cache_dir: PathBuf` is the directory of cached version histories (`PathBuf` is an owned path;
    /// sibling:
    /// `&Path`,
    /// borrowed).
    /// Why:
    /// A published version never changes,
    /// so a history that lists every version being checked is read from disk instead of the network.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cacheDir: string
    /// ```
    cache_dir: PathBuf,
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    /// What:
    /// Field `fetcher: Box<dyn TextFetcher>` is the HTTP source for the index and the API (`Box<dyn Trait>`:
    /// any implementation behind one owning pointer).
    /// Why:
    /// Tests substitute a fetcher that answers from memory or fails on purpose.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fetcher: TextFetcher
    /// ```
    fetcher: Box<dyn TextFetcher>,
    // `dyn` trait object `dyn DumpSource`: any type that implements the trait.
    /// What:
    /// Field `dump` is the optional source of the database dump (`Option`:
    /// it may be absent).
    /// Why:
    /// A registry built without it never reads the dump and uses the API for every crate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dump?: DumpSource
    /// ```
    dump: Option<Box<dyn DumpSource>>,
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    /// What:
    /// Field `dump_breakeven: usize` is how many uncached crates make the dump cheaper than the API (`usize`,
    /// a count).
    /// Why:
    /// The dump costs a download of gigabytes,
    /// the API one request per crate at 1.1 seconds,
    /// so below this count the API wins.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dumpBreakeven: number
    /// ```
    dump_breakeven: usize,
    /// What:
    /// Field `min_interval: Duration` is the least time between two API requests.
    /// Why:
    /// crates.io allows about one per second;
    /// sending faster earns HTTP 429.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// minIntervalMs: number
    /// ```
    min_interval: Duration,
    /// What:
    /// Field `backoff: Duration` is the first wait after an HTTP 429 that carries no `Retry-After`;
    /// it doubles each attempt.
    /// Why:
    /// The registry retries instead of failing the whole install on a busy minute.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// backoffMs: number
    /// ```
    backoff: Duration,
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    /// What:
    /// Field `last_request` remembers when the last API request started (`Cell` lets a method that only borrows `&self` still replace the value).
    /// Why:
    /// The spacing must hold across calls made through a shared reference to the registry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastRequestAt?: number
    /// ```
    last_request: Cell<Option<Instant>>,
}

/// What:
/// `impl CratesIo { ... }` attaches the constructors and the spaced API call to the registry struct.
/// Why:
/// The constructors choose real or test timing,
/// and one method owns the rate-limit logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CratesIo { constructor(...) {} spacedApiGet(url) {} }
/// ```
impl CratesIo {
    /// Builds the registry for a repository root with the real spacing and backoff.
    pub fn with_defaults(root: &Path, fetcher: Box<dyn TextFetcher>, dump: Box<dyn DumpSource>) -> CratesIo {
        // `let mut`: a binding that may change.
        // Typed binding: `registry: CratesIo`.
        let mut registry: CratesIo = CratesIo::new(root, fetcher, API_MIN_INTERVAL, RATE_LIMIT_BACKOFF);
        // `Some`: the present or the absent value of an `Option`.
        registry.dump = Some(dump);
        return registry;
    }

    /// Sets how many uncached crates make the dump worth reading;
    /// tests lower it.
    pub fn with_dump(mut self, dump: Box<dyn DumpSource>, breakeven: usize) -> CratesIo {
        // `Some`: the present or the absent value of an `Option`.
        self.dump = Some(dump);
        self.dump_breakeven = breakeven;
        return self;
    }

    /// Builds the registry for a repository root;
    /// `min_interval` and `backoff` are parameters so tests do not wait.
    pub fn new(root: &Path, fetcher: Box<dyn TextFetcher>, min_interval: Duration, backoff: Duration) -> CratesIo {
        return CratesIo {
            cache_dir: root.join(HISTORY_CACHE_DIR),
            fetcher,
            // `None`: the present or the absent value of an `Option`.
            dump: None,
            dump_breakeven: DUMP_BREAKEVEN_CRATES,
            min_interval,
            backoff,
            // `None`: the present or the absent value of an `Option`.
            // Constructor `Cell::new(`: an empty owned collection or a boxed value.
            last_request: Cell::new(None),
        };
    }

    /// One API GET:
    /// waits out the spacing,
    /// then retries an HTTP 429 up to the attempt limit.
    fn spaced_api_get(&self, url: &str) -> Result<String, RegistryError> {
        // `let mut`: a binding that may change.
        // Typed binding: `attempt: u32`.
        let mut attempt: u32 = 1;
        loop {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
            // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (value !== undefined) { const inner = value; ... }
            // ```
            if let Some(last) = self.last_request.get() {
                // Typed binding: `elapsed: Duration`.
                let elapsed: Duration = last.elapsed();
                if elapsed < self.min_interval {
                    sleep(self.min_interval - elapsed);
                }
            }
            // `Some`: the present or the absent value of an `Option`.
            self.last_request.set(Some(Instant::now()));
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: fetching {url}");
            // `match`: picks the arm whose pattern fits the value.
            match self.fetcher.get(url) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(body) => return Ok(body),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(error) => {
                    // `Some`: the present or the absent value of an `Option`.
                    // Closure `||`: an anonymous function, like a TS arrow function.
                    if error.status != Some(TOO_MANY_REQUESTS) || attempt >= RATE_LIMIT_ATTEMPTS {
                        // `Err(...)`: builds the success or the failure variant of a `Result`.
                        return Err(error);
                    }
                    // Typed binding: `pause: Duration`.
                    let asked: Duration = if error.retry_after_seconds > 0 { Duration::from_secs(error.retry_after_seconds) } else { self.backoff * 2u32.pow(attempt - 1) };
                    // Owner rule of 2026-11-03, "nothing should ever wait forever or indefinitely": a pause longer than the bound ends
                    // the call with an error that names the blocker; every file is kept.
                    if error.retry_after_seconds > 0 && asked > MAX_PAUSE {
                        // `Err(...)`: builds the success or the failure variant of a `Result`.
                        return Err(RegistryError {
                            // Macro `format!`: builds or prints text (or a list) from a template.
                            message: format!(
                                "{url} answered HTTP 429 and asks for a pause of {asked:?}, longer than the {MAX_PAUSE:?} cargo-shadow waits; every file is kept; wait that long, then run the command again"
                            ),
                            status: error.status,
                            retry_after_seconds: error.retry_after_seconds,
                        });
                    }
                    // Typed binding: `pause: Duration`.
                    let pause: Duration = asked.min(MAX_PAUSE);
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!("cargo-shadow: {url} answered HTTP 429 (attempt {attempt} of {RATE_LIMIT_ATTEMPTS}); retrying in {pause:?}");
                    sleep(pause);
                    attempt += 1;
                }
            }
        }
    }
}

/// What:
/// `fn version_number_of(record: &VersionRecord) -> &str` lends the version number of one cached release.
/// Why:
/// The cache is trusted only when it already lists every version the lockfile names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const versionNumberOf = (record: VersionRecord) => record.num;
/// ```
fn version_number_of(record: &VersionRecord) -> &str {
    return record.num.as_str();
}

/// What:
/// `impl Registry for CratesIo` makes crates.io usable as the `Registry` the checks ask for.
/// Why:
/// The release-age and trust checks only know the trait,
/// so tests use a fake with canned histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CratesIo implements Registry { ... }
/// ```
impl Registry for CratesIo {
    /// What:
    /// `fn prefetch(&self, names)` fills the history cache for the crates in `names` from the database dump when enough of them are uncached.
    /// Why:
    /// A frozen install of every lockfile asks about a thousand crates;
    /// one streaming pass replaces a thousand rate-limited requests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prefetch(names: Set<string>): void  // throws RegistryError
    /// ```
    fn prefetch(&self, names: &BTreeSet<String>) -> Result<(), RegistryError> {
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&dyn`: lends the value without copying or giving it away.
        // Typed binding: `dump: &dyn DumpSource`.
        // `dyn` trait object `dyn DumpSource`: any type that implements the trait.
        let dump: &dyn DumpSource = match &self.dump {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found.as_ref(),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            None => return Ok(()),
        };
        // Closure `|name|`: an anonymous function, like a TS arrow function.
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.filter(`: transforms or selects items with a closure.
        // Typed binding: `uncached: BTreeSet<String>`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let mut uncached: BTreeSet<String> = BTreeSet::new();
        for name in names {
            if !self.cache_dir.join(format!("{name}.json")).exists() {
                uncached.insert(name.clone());
            }
        }
        if uncached.len() < self.dump_breakeven {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: {} crates lack a cached history (below {}); using the crates.io API for them", uncached.len(), self.dump_breakeven);
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(());
        }
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: {} crates lack a cached history; reading {DATABASE_DUMP_URL}", uncached.len());
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&uncached`: lends the value without copying or giving it away.
        let result = fill_histories_from_dump(dump.open(DATABASE_DUMP_URL)?, &uncached, &self.cache_dir)
            // `None`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            .map_err(|error| { return RegistryError { message: error.message, status: None, retry_after_seconds: 0 }; })?;
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: dump: {} found, {} written, {} already cached, {} left to the API (possible Trusted Publishing releases)",
            result.found,
            result.written,
            result.kept,
            result.left_to_api.len()
        );
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }

    /// What:
    /// `fn pubtimes(&self, name)` returns the publish time of each version of a crate from the sparse index.
    /// Why:
    /// The release-age rule compares those times with the cutoff.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pubtimes(name: string): Map<string, string>  // throws RegistryError
    /// ```
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.fetcher.get`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return parse_index_pubtimes(&self.fetcher.get(&format!("{SPARSE_INDEX_URL}/{}", sparse_index_path(name)))?);
    }

    /// What:
    /// `fn yanked(&self, name)` returns the yanked versions of a crate from the sparse index.
    /// Why:
    /// A set-aside candidate that was yanked must not be locked in place of a refused version.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// yanked(name: string): Set<string>  // throws RegistryError
    /// ```
    fn yanked(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.fetcher.get`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return parse_index_yanked(&self.fetcher.get(&format!("{SPARSE_INDEX_URL}/{}", sparse_index_path(name)))?);
    }

    /// What:
    /// `fn owners(&self, name)` returns the account ids of the crate's listed individual owners from the API.
    /// Why:
    /// A first release by a listed owner counts as a known publisher,
    /// so the trust rule asks for owners only for a release it would otherwise refuse.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// owners(name: string): Set<string>  // throws RegistryError
    /// ```
    fn owners(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.spaced_api_get`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return parse_owner_ids(&self.spaced_api_get(&format!("{API_URL}/crates/{name}/owners"))?);
    }

    /// What:
    /// `fn history(&self, name, versions)` returns every release of a crate,
    /// from the cache when it lists all of `versions`,
    /// otherwise from the API (and writes the cache).
    /// Why:
    /// The trust rule compares a version with all earlier releases,
    /// so it needs the whole list,
    /// and published history never changes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// history(name: string, versions: string[]): VersionRecord[]  // throws RegistryError
    /// ```
    fn history(&self, name: &str, versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        // Typed binding: `path: PathBuf`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let path: PathBuf = self.cache_dir.join(format!("{name}.json"));
        // Borrow `&path`: lends the value without copying or giving it away.
        // Typed binding: `cached: Vec<VersionRecord>`.
        let cached: Vec<VersionRecord> = read_cached_history(&path);
        // Closure `|record|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // `.collect()`: runs the iterator and gathers the items into the bound collection type.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `known: BTreeSet<&str>`.
        let known: BTreeSet<&str> = cached.iter().map(version_number_of).collect();
        // Closure `|version|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        if !cached.is_empty() && versions.iter().all(|version| { return known.contains(version.as_str()); }) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(cached);
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.spaced_api_get`: lends the value without copying or giving it away.
        // Typed binding: `fresh: Vec<VersionRecord>`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let fresh: Vec<VersionRecord> = parse_version_history(&self.spaced_api_get(&format!("{API_URL}/crates/{name}/versions"))?)?;
        // Borrow `&self.cache_dir`: lends the value without copying or giving it away.
        if fs::create_dir_all(&self.cache_dir).is_ok() {
            // Borrow `&path`: lends the value without copying or giving it away.
            // Macro `format!`: builds or prints text (or a list) from a template.
            // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
            // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // void expr;   // or try { expr } catch {}
            // ```
            let _ = fs::write(&path, format!("{}\n", history_to_json(&fresh)));
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(fresh);
    }
}
