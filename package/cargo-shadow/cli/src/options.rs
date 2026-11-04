//! What:
//! Reads the few cargo options the shadow cares about from a command line without changing it:
//! where cargo will look
//! (`-C <dir>`,
//! `--manifest-path <file>`) and whether the user already asked for a frozen lockfile (`--locked`,
//! `--frozen`).
//! Why:
//! The install step runs for the Cargo workspace that holds the place cargo will work in,
//! and the user's own lock assertion
//! is this tool's `--frozen-lockfile`:
//! the shadow never adds an option to the user's command,
//! so the user's words are all
//! there is to read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // options.ts
//! export function manifestPath(args: string[]): string | undefined
//! export function changeDir(args: string[]): string | undefined
//! export function assertsLock(args: string[]): boolean
//! ```

// What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
// Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::ffi::OsString;` imports `OsString` from the standard library so this file can name it unqualified.
/// Why:
/// `OsString` is used by `manifest_path`,
/// `change_dir`,
/// `asserts_lock`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OsString } from 'std/ffi';
/// ```
use std::ffi::OsString;
// Imports `std::path::PathBuf`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::path::PathBuf;` imports `PathBuf` from the standard library so this file can name it unqualified.
/// Why:
/// `PathBuf` is used by `manifest_path`,
/// `change_dir`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PathBuf } from 'std/path';
/// ```
use std::path::PathBuf;

/// What:
/// `use crate::verb::{find_verb, words_before_double_dash};` imports `find_verb`,
/// `words_before_double_dash` from another module of this crate so this file can name them unqualified.
/// Why:
/// `find_verb` is used by `manifest_path`;
/// `words_before_double_dash` is used by `manifest_path`,
/// `change_dir`,
/// `asserts_lock`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_verb, words_before_double_dash } from 'crate/verb';
/// ```
use crate::verb::{find_verb, words_before_double_dash};

/// The value of an option written `--name value` or `--name=value`;
/// the last one wins,
/// as in cargo.
fn long_option_value(words: &[String], name: &str) -> Option<String> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
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
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let prefix: String = format!("{name}=");
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `found: Option<String>`.
    let mut found: Option<String> = None;
    // `let mut`: a binding that may change.
    // Typed binding: `index: usize`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut index: usize = 0;
    while index < words.len() {
        if words[index] == name && index + 1 < words.len() {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            found = Some(words[index + 1].clone());
            index += 2;
            continue;
        }
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        if let Some(value) = words[index].strip_prefix(&prefix) {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            found = Some(value.to_string());
        }
        index += 1;
    }
    return found;
}

/// The `--manifest-path` value,
/// when given;
/// for `cargo update` its short spelling `-m <path>` (shown by `cargo update --help`) counts too,
/// and the short spelling wins when both are given.
pub fn manifest_path(arguments: &[OsString]) -> Option<PathBuf> {
    // Typed binding: `words: Vec<String>`.
    let words: Vec<String> = words_before_double_dash(arguments);
    // Borrow `&words`: lends the value without copying or giving it away.
    // Typed binding: `long: Option<String>`.
    let long: Option<String> = long_option_value(&words, "--manifest-path");
    // Review finding (supported spellings): `-m` is read only for `cargo update`, where it means the manifest path; for
    // other commands it is not a cargo option the shadow knows.
    if find_verb(arguments).as_deref() == Some("update")
        // `Some`: the present or the absent value of an `Option`.
        // Borrow `&words`: lends the value without copying or giving it away.
        && let Some(short) = short_option_value(&words, "-m")
    {
        // `Some`: the present or the absent value of an `Option`.
        return Some(PathBuf::from(short));
    }
    // Adapter `.map(`: transforms or selects items with a closure.
    return long.map(PathBuf::from);
}

/// What:
/// `fn short_option_value(words, name) -> Option<String>` is the value after the last separate `-m <value>` style option.
/// Why:
/// The value-taking short options of `cargo update` are written with a space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortOptionValue = (words: string[], name: string) => words[words.lastIndexOf(name) + 1];
/// ```
fn short_option_value(words: &[String], name: &str) -> Option<String> {
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `found: Option<String>`.
    let mut found: Option<String> = None;
    // `let mut`: a binding that may change.
    // Typed binding: `index: usize`.
    let mut index: usize = 0;
    while index < words.len() {
        if words[index] == name && index + 1 < words.len() {
            // `Some`: the present or the absent value of an `Option`.
            // `.clone()`: an independent owned copy.
            found = Some(words[index + 1].clone());
            index += 2;
            continue;
        }
        // Review finding (2c 9): the attached form (`-m<path>` or `-m=<path>`) carries its value in the same word.
        if let Some(attached) = words[index].strip_prefix(name)
            && !attached.is_empty()
        {
            // `Some`: the present or the absent value of an `Option`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
            // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
            // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // value ?? fallback
            // ```
            found = Some(attached.strip_prefix('=').unwrap_or(attached).to_string());
        }
        index += 1;
    }
    return found;
}

/// The `-C <directory>` value,
/// when given (cargo's own option,
/// nightly only).
pub fn change_dir(arguments: &[OsString]) -> Option<PathBuf> {
    // Typed binding: `words: Vec<String>`.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let words: Vec<String> = words_before_double_dash(arguments);
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `found: Option<PathBuf>`.
    let mut found: Option<PathBuf> = None;
    // `let mut`: a binding that may change.
    // Typed binding: `index: usize`.
    let mut index: usize = 0;
    while index < words.len() {
        if words[index] == "-C" && index + 1 < words.len() {
            // `Some`: the present or the absent value of an `Option`.
            // Borrow `&words`: lends the value without copying or giving it away.
            found = Some(PathBuf::from(&words[index + 1]));
            index += 2;
            continue;
        }
        index += 1;
    }
    return found;
}

/// What:
/// `fn is_locked_or_frozen(word: &str) -> bool` tells whether one command-line word is cargo's `--locked` or `--frozen` option.
/// Why:
/// The install step is not given a second `--locked` when the user already wrote one of the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLockedOrFrozen = (word: string) => word === '--locked' || word === '--frozen';
/// ```
fn is_locked_or_frozen(word: &str) -> bool {
    return word == "--locked" || word == "--frozen";
}

/// Whether the command line asks cargo to keep the lockfile unchanged (`--locked`) or also stay offline (`--frozen`).
pub fn asserts_lock(arguments: &[OsString]) -> bool {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    return words_before_double_dash(arguments).iter().map(String::as_str).any(is_locked_or_frozen);
}
