//! What:
//! The transaction around the manifests that cargo's own `update --breaking` rewrites:
//! the text of every workspace manifest before the run,
//! the text after it,
//! and the restore that puts back only what this run wrote.
//! Why:
//! `cargo update --breaking` edits version requirements in `Cargo.toml` as well as the lockfile,
//! so a refusal or a dry run that restored only the lockfiles would leave the new requirements behind.
//! The restore follows the lockfile rule:
//! a manifest is replaced only while it still holds what this run's cargo wrote,
//! through a sibling file and an atomic rename,
//! and one that another process changed is left alone with a warning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // manifest-edit.ts
//! export type ManifestEdit = { path: string; before: string; produced?: string };
//! export function snapshotManifests(paths: string[]): ManifestEdit[]
//! export function recordManifestOutput(edits: ManifestEdit[]): void
//! export function restoreManifests(edits: ManifestEdit[]): string[]
//! ```

/// What:
/// `use std::cell::RefCell;` imports
/// `RefCell`
/// so this file can name them unqualified.
/// Why:
/// The list of manifests grows while the transaction holds a shared reference to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::fs;` imports
/// `fs`
/// so this file can name them unqualified.
/// Why:
/// `fs` reads and compares the manifest files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports
/// `io`
/// so this file can name them unqualified.
/// Why:
/// `io::Result` is the error type of reading and restoring a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io;
/// What:
/// `use std::path::PathBuf;` imports
/// `PathBuf`
/// so this file can name them unqualified.
/// Why:
/// A manifest's path is stored,
/// so it is the owned type (sibling:
/// `&Path`,
/// the borrowed one).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::PathBuf;

/// What:
/// `use crate::journal::{ReplaceOutcome, no_hook, replace_if_holds};` imports
/// `ReplaceOutcome`,
/// `replace_if_holds`
/// so this file can name them unqualified.
/// Why:
/// A manifest goes back with the same atomic compare-and-replace the lockfiles use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ReplaceOutcome, replaceIfHolds } from 'crate/journal';
/// ```
use crate::journal::{ReplaceOutcome, no_hook, replace_if_holds};

/// One manifest the run may rewrite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestEdit {
    /// Where the manifest is.
    pub path: PathBuf,
    /// Its text before the run.
    pub before: String,
    /// Its text right after this run's cargo call returned;
    /// `None` until then,
    /// and a manifest without it is never restored.
    pub produced: Option<String>,
}

/// What:
/// `pub fn snapshot_manifests(paths, edits) -> io::Result<()>` reads each manifest and adds it to `edits`.
/// Why:
/// The text must be taken before cargo runs;
/// a manifest that cannot be read stops the run before anything is mutated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function snapshotManifests(paths: string[], edits: ManifestEdit[]): void  // throws when a file cannot be read
/// ```
pub fn snapshot_manifests(paths: &[PathBuf], edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<()> {
    for path in paths {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
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
        let before: String = fs::read_to_string(path)?;
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        edits.borrow_mut().push(ManifestEdit { path: path.clone(), before, produced: None });
    }
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

/// What:
/// `pub fn record_manifest_output(edits)` reads each manifest again and stores the text as this run's output.
/// Why:
/// Called right after cargo returned,
/// success or failure,
/// so the restore can tell cargo's own edit from another writer's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recordManifestOutput(edits: ManifestEdit[]): void
/// ```
pub fn record_manifest_output(edits: &RefCell<Vec<ManifestEdit>>) {
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    for edit in edits.borrow_mut().iter_mut() {
        // A manifest that cannot be read now stays unrecorded and is therefore never restored.
        edit.produced = fs::read_to_string(&edit.path).ok();
    }
}

/// What:
/// `pub fn restore_manifests(edits) -> io::Result<Vec<String>>` puts each manifest back that still holds this run's output
/// and returns a warning for each one it left alone.
/// Why:
/// Unknown ownership never authorizes a restore:
/// a manifest with no recorded output,
/// or one that differs from it,
/// belongs to another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreManifests(edits: ManifestEdit[]): string[]  // throws when a write fails
/// ```
pub fn restore_manifests(edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<Vec<String>> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `warnings: Vec<String>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut warnings: Vec<String> = Vec::new();
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    for edit in edits.borrow().iter() {
        // Borrow `&edit.path`: lends the value without copying or giving it away.
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        // Typed binding: `live: String`.
        let live: String = fs::read_to_string(&edit.path).unwrap_or_default();
        if live == edit.before {
            continue;
        }
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        // Borrow `&edit.produced`: lends the value without copying or giving it away.
        match &edit.produced {
            // `Some`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&edit.path`: lends the value without copying or giving it away.
            Some(produced) if *produced == live => match replace_if_holds(&edit.path, produced, &edit.before, &no_hook)? {
                ReplaceOutcome::Replaced => {}
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      Text building and printing are written as one call instead of string concatenation.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                ReplaceOutcome::LeftAlone => warnings.push(format!(
                    "{} was changed by another process while it was being restored; it was left as that process wrote it",
                    edit.path.display()
                )),
                // Macro `format!`: builds or prints text (or a list) from a template.
                ReplaceOutcome::Preserved(kept) => warnings.push(format!(
                    "{} was changed twice by other processes while it was being restored; the file displaced is kept as {}",
                    edit.path.display(),
                    kept.display()
                )),
            },
            // Macro `format!`: builds or prints text (or a list) from a template.
            _ => warnings.push(format!(
                "{} differs from its text before the run, but this run did not leave it that way, so another process did; it was left as it is, not restored",
                edit.path.display()
            )),
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(warnings);
}
