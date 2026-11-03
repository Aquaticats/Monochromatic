//! The tests of the owner's rule of 2026-11-03,
//! "nothing should ever wait forever or indefinitely",
//! for the waits that are not
//! journal locks:
//! a cargo call,
//! the file-enforcer sync and a registry pause.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fs;
// Imports `std::os::unix::fs::PermissionsExt`.
use std::os::unix::fs::PermissionsExt;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::{Path, PathBuf};
// Imports `std::time::{Duration, Instant}`.
// What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
// Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: T[] / Set / Map / number
// ```
use std::time::{Duration, Instant};

// Imports `crate::cargo_run::{CargoRunner, RealCargo}`.
use crate::cargo_run::{CargoRunner, RealCargo};
// Imports `crate::http::{RegistryError, TextFetcher}`.
use crate::http::{RegistryError, TextFetcher};
// Imports `crate::registry::{CratesIo, Registry}`.
use crate::registry::{CratesIo, Registry};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

/// Writes an executable shell script.
fn script(directory: &Path, name: &str, body: &str) -> PathBuf {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let path: PathBuf = directory.join(name);
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    return path;
}

/// A cargo that outlives its bound is stopped,
/// and the error names the call and the command to run again.
#[test]
fn a_cargo_that_outlives_its_bound_is_stopped_and_named() {
    let directory = scratch();
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let cargo = RealCargo { program: script(directory.path(), "cargo", "sleep 5"), toolchain: String::new(), bound: Duration::from_millis(300) };
    let started = Instant::now();

    // Borrow `&[`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let result = cargo.run(directory.path(), &["update".to_string(), "--workspace".to_string()]);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    // Typed binding: `message: String`.
    let message: String = result.expect_err("a stopped cargo has no outcome");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("cargo update --workspace") && message.contains("did not finish"), "{message}");
}

/// A fetcher that always answers HTTP 429 with a long `Retry-After`.
struct Throttled;

impl TextFetcher for Throttled {
    /// What:
    /// `fn get(&self, url) -> Result<String, RegistryError>` always answers "too many requests,
    /// retry in a day".
    /// Why:
    /// The registry must not sleep for what the server asks when that is longer than its own bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get(url: string): string  // always throws a 429 with Retry-After: 86400
    /// ```
    fn get(&self, url: &str) -> Result<String, RegistryError> {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(RegistryError { message: format!("{url} answered HTTP 429"), status: Some(429), retry_after_seconds: 86_400 });
    }
}

/// A server that asks for a pause longer than the registry's own bound ends the call with an error that says so;
/// it does not sleep.
#[test]
fn a_retry_after_longer_than_the_bound_ends_the_call_instead_of_sleeping() {
    let root = scratch();
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let registry = CratesIo::new(root.path(), Box::new(Throttled), Duration::ZERO, Duration::from_millis(1));
    let started = Instant::now();

    let result = registry.owners("serde");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
    // Typed binding: `message: String`.
    let message: String = result.expect_err("no answer").message;
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("429") && message.contains("run the command again"), "{message}");
}
