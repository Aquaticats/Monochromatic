//! What:
//! The fingerprint stamp that lets a cargo call skip the install check:
//! after the authoritative check says a workspace's
//! `Cargo.lock` satisfies its manifests,
//! the stamp records the size and modification time of the lockfile and of every
//! manifest the lockfile was resolved from.
//! Why:
//! The shadow sits on every cargo call,
//! and the authoritative check (`cargo update --workspace --locked --offline`) costs a
//! cargo start plus index reads;
//! measured on the largest lockfile in the repository it took 0.20 to 0.29 seconds.
//! If every
//! recorded file still has the same size and modification time,
//! nothing the check reads has changed,
//! so the answer is still
//! "satisfied" and no process is started.
//! This is the same test cargo's own fingerprints use for source files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stamp.ts
//! export function isFresh(stampFile: string): boolean
//! export function record(args: { stampFile: string; files: string[] }): void
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `fingerprint`,
/// `is_fresh`,
/// `record`.
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
/// `Path` is used by `stamp_file`,
/// `fingerprint`,
/// `is_fresh`,
/// `record`,
/// and others;
/// `PathBuf` is used by `stamp_file`,
/// `record`,
/// `manifests_from_metadata`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};
/// What:
/// `use std::time::UNIX_EPOCH;` imports `UNIX_EPOCH` from the standard library so this file can name it unqualified.
/// Why:
/// `UNIX_EPOCH` is used by `fingerprint`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UNIX_EPOCH } from 'std/time';
/// ```
use std::time::UNIX_EPOCH;

/// What:
/// `use serde_json::Value;` imports `Value` from the external crate `serde_json` so this file can name it unqualified.
/// Why:
/// `Value` is used by `is_fresh`,
/// `record`,
/// `manifests_from_metadata`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Value } from 'serde_json';
/// ```
use serde_json::Value;

/// Where stamps live,
/// relative to the repository root (gitignored with the rest of `.cache`).
pub const STAMP_DIR: &str = ".cache/cargo-shadow/stamps";

/// The stamp file of a crate root.
pub fn stamp_file(catalog_root: &Path, crate_dir: &str) -> PathBuf {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    return catalog_root.join(STAMP_DIR).join(format!("{}.json", crate_dir.replace('/', "__")));
}

/// A file's modification time in nanoseconds since 1970 and its length,
/// as text,
/// or `None` when it cannot be read.
fn fingerprint(path: &Path) -> Option<(String, String)> {
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    let metadata = fs::metadata(path).ok()?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let nanos: u128 = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return Some((nanos.to_string(), metadata.len().to_string()));
}

/// What:
/// `pub fn is_fresh(stamp: &Path, lock: &Path) -> bool` says whether the stamp lists the lockfile and every listed file still
/// has the recorded size and modification time.
/// Why:
/// Any missing stamp,
/// unreadable file,
/// or difference answers `false`,
/// which sends the caller to the authoritative check;
/// a
/// wrong `false` costs time,
/// a wrong `true` would let cargo rewrite the lockfile,
/// so every doubt is `false`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isFresh(stamp: string, lock: string): boolean
/// ```
pub fn is_fresh(stamp: &Path, lock: &Path) -> bool {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // Typed binding: `text: String`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let text: String = match fs::read_to_string(stamp) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => return false,
    };
    // `match`: picks the arm whose pattern fits the value.
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // Typed binding: `parsed: Value`.
    let parsed: Value = match serde_json::from_str(&text) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => return false,
    };
    // `match`: picks the arm whose pattern fits the value.
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // Borrow `&Vec`: lends the value without copying or giving it away.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `files: &Vec<Value>`.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let files: &Vec<Value> = match parsed.get("files").and_then(Value::as_array) {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        None => return false,
    };
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `lock_text: String`.
    let lock_text: String = lock.display().to_string();
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `lists_the_lock: bool`.
    let mut lists_the_lock: bool = false;
    for entry in files {
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        // Adapter `.and_then(`: transforms or selects items with a closure.
        // Typed binding: `path: &str`.
        let path: &str = entry.get("path").and_then(Value::as_str).unwrap_or("");
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Adapter `.and_then(`: transforms or selects items with a closure.
        // Typed binding: `nanos: &str`.
        let nanos: &str = entry.get("mtimeNanos").and_then(Value::as_str).unwrap_or("");
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Adapter `.and_then(`: transforms or selects items with a closure.
        // Typed binding: `length: &str`.
        let length: &str = entry.get("length").and_then(Value::as_str).unwrap_or("");
        if path == lock_text {
            lists_the_lock = true;
        }
        // `match`: picks the arm whose pattern fits the value.
        match fingerprint(Path::new(path)) {
            // `Some`: the present or the absent value of an `Option`.
            Some((current_nanos, current_length)) if current_nanos == nanos && current_length == length => {}
            _ => return false,
        }
    }
    return lists_the_lock;
}

/// Writes the stamp for the lockfile and the manifests;
/// a failure to write only costs the next call a check.
pub fn record(stamp: &Path, lock: &Path, manifests: &[PathBuf]) {
    // `let mut`: a binding that may change.
    // Typed binding: `entries: Vec<Value>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut entries: Vec<Value> = Vec::new();
    // Borrow `&Path`: lends the value without copying or giving it away.
    // `let mut`: a binding that may change.
    // Typed binding: `all: Vec<&Path>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut all: Vec<&Path> = vec![lock];
    for manifest in manifests {
        all.push(manifest);
    }
    for path in all {
        // `match`: picks the arm whose pattern fits the value.
        let (nanos, length) = match fingerprint(path) {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // `None`: the present or the absent value of an `Option`.
            None => return,
        };
        // `let mut`: a binding that may change.
        // Typed binding: `entry: serde_json::Map<String, Value>`.
        let mut entry: serde_json::Map<String, Value> = serde_json::Map::new();
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("path".to_string(), Value::String(path.display().to_string()));
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("mtimeNanos".to_string(), Value::String(nanos));
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        entry.insert("length".to_string(), Value::String(length));
        entries.push(Value::Object(entry));
    }
    // `let mut`: a binding that may change.
    // Typed binding: `body: serde_json::Map<String, Value>`.
    let mut body: serde_json::Map<String, Value> = serde_json::Map::new();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    body.insert("files".to_string(), Value::Array(entries));
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    if let Some(parent) = stamp.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    // Macro `format!`: builds or prints text (or a list) from a template.
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = fs::write(stamp, format!("{}\n", Value::Object(body)));
}

/// The manifests a `cargo metadata` document says the lockfile was resolved from:
/// every package without a registry or git
/// source (workspace members and path dependencies) plus the workspace root manifest.
pub fn manifests_from_metadata(metadata_json: &str) -> Vec<PathBuf> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `parsed: Value`.
    let parsed: Value = match serde_json::from_str(metadata_json) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        Err(_) => return Vec::new(),
    };
    // `let mut`: a binding that may change.
    // Typed binding: `manifests: Vec<PathBuf>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut manifests: Vec<PathBuf> = Vec::new();
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Adapter `.and_then(`: transforms or selects items with a closure.
    if let Some(root) = parsed.get("workspace_root").and_then(Value::as_str) {
        manifests.push(Path::new(root).join("Cargo.toml"));
    }
    // Typed binding: `empty: Vec<Value>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let empty: Vec<Value> = Vec::new();
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Borrow `&empty`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.and_then(`: transforms or selects items with a closure.
    for package in parsed.get("packages").and_then(Value::as_array).unwrap_or(&empty) {
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `has_source: bool`.
        let has_source: bool = !package.get("source").map(Value::is_null).unwrap_or(true);
        if has_source {
            continue;
        }
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // Closure `|value|`: an anonymous function, like a TS arrow function.
        // Adapter `.and_then(`: transforms or selects items with a closure.
        if let Some(path) = package.get("manifest_path").and_then(Value::as_str) {
            manifests.push(PathBuf::from(path));
        }
    }
    manifests.sort();
    manifests.dedup();
    return manifests;
}
