//! What:
//! Finds the repository that uses the Rust dependency catalog and the Cargo workspace that holds a directory.
//! Why:
//! The install step runs for exactly one Cargo workspace,
//! the one cargo itself would use for the current directory,
//! and only
//! inside a repository that has a `rust-catalog.toml`;
//! anywhere else the shadow is silent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // workspace.ts
//! export function findCatalogRoot(start: string): string | undefined
//! export function findCargoRoot(start: string): string | undefined
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `find_cargo_root`.
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
/// `Path` is used by `find_catalog_root`,
/// `find_cargo_root`;
/// `PathBuf` is used by `find_catalog_root`,
/// `find_cargo_root`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use toml_edit::DocumentMut;` imports `DocumentMut` from the external crate `toml_edit` so this file can name it unqualified.
/// Why:
/// `DocumentMut` is used by `declares_workspace`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DocumentMut } from 'toml_edit';
/// ```
use toml_edit::DocumentMut;

/// What:
/// `use crate::constants::{CATALOG_MARKER, MANIFEST_NAME};` imports `CATALOG_MARKER`,
/// `MANIFEST_NAME` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CATALOG_MARKER` is used by `find_catalog_root`;
/// `MANIFEST_NAME` is used by `find_cargo_root`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER, MANIFEST_NAME } from 'crate/constants';
/// ```
use crate::constants::{CATALOG_MARKER, MANIFEST_NAME};

/// The nearest ancestor (the directory itself included) that holds `rust-catalog.toml`.
pub fn find_catalog_root(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        if directory.join(CATALOG_MARKER).is_file() {
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            return Some(directory.to_path_buf());
        }
    }
    // `None`: the present or the absent value of an `Option`.
    return None;
}

/// Whether manifest text declares a `[workspace]` table.
fn declares_workspace(text: &str) -> bool {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match text.parse::<DocumentMut>() {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Ok(document) => document.contains_key("workspace"),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => false,
    };
}

/// What:
/// `pub fn find_cargo_root(start: &Path) -> Option<PathBuf>` walks up from `start` to the nearest manifest that declares
/// `[workspace]`;
/// with none,
/// it is the nearest manifest of any kind.
/// Why:
/// Cargo's own rule:
/// a package belongs to the closest ancestor workspace.
/// In this repository every crate is its own
/// workspace root,
/// so the nearest manifest is nearly always the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findCargoRoot(start: string): string | undefined
/// ```
pub fn find_cargo_root(start: &Path) -> Option<PathBuf> {
    // `None`: the present or the absent value of an `Option`.
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let mut nearest: Option<PathBuf> = None;
    for directory in start.ancestors() {
        // Typed binding: `manifest: PathBuf`.
        let manifest: PathBuf = directory.join(MANIFEST_NAME);
        // `match`: picks the arm whose pattern fits the value.
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // Typed binding: `text: String`.
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let text: String = match fs::read_to_string(&manifest) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(_) => continue,
        };
        if nearest.is_none() {
            // `Some`: the present or the absent value of an `Option`.
            nearest = Some(directory.to_path_buf());
        }
        // Borrow `&text`: lends the value without copying or giving it away.
        if declares_workspace(&text) {
            // `Some`: the present or the absent value of an `Option`.
            return Some(directory.to_path_buf());
        }
    }
    return nearest;
}
