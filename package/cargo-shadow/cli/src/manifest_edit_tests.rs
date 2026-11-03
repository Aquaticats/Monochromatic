//! The unit tests of the manifest transaction around cargo's breaking update.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::cell::RefCell;
// Imports `std::fs`.
use std::fs;

// Imports `crate::manifest_edit::{ManifestEdit, record...`.
use crate::manifest_edit::{ManifestEdit, record_manifest_output, restore_manifests, snapshot_manifests};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

fn edits_for(path: &std::path::Path) -> RefCell<Vec<ManifestEdit>> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
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
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
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
    snapshot_manifests(&[path.to_path_buf()], &edits).expect("snapshots");
    return edits;
}

#[test]
fn a_manifest_that_still_holds_the_runs_output_is_put_back() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(&path);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "cargo's edit").expect("cargo edits it");
    // Borrow `&edits`: lends the value without copying or giving it away.
    record_manifest_output(&edits);

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restore_manifests(&edits).expect("restores");

    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert!(warnings.is_empty());
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&path).expect("read"), "before");
}

#[test]
fn a_manifest_another_process_changed_after_the_run_is_left_alone_with_a_warning() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(&path);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "cargo's edit").expect("cargo edits it");
    // Borrow `&edits`: lends the value without copying or giving it away.
    record_manifest_output(&edits);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "somebody else's edit").expect("another process");

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restore_manifests(&edits).expect("restores");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(warnings.len(), 1);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&path).expect("read"), "somebody else's edit");
}

#[test]
fn a_manifest_whose_output_was_never_recorded_is_never_restored() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(&path);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "unknown author").expect("someone edits it");

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restore_manifests(&edits).expect("restores");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(warnings.len(), 1);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&path).expect("read"), "unknown author");
}

#[test]
fn an_unchanged_manifest_needs_nothing_and_a_missing_one_stops_the_snapshot() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(&path);
    // Borrow `&edits`: lends the value without copying or giving it away.
    record_manifest_output(&edits);

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(restore_manifests(&edits).expect("restores").is_empty());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    assert!(snapshot_manifests(&[root.path().join("missing.toml")], &RefCell::new(Vec::new())).is_err());
}
