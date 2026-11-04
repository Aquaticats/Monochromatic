//! The unit tests of the manifest transaction around cargo's breaking update.

use std::cell::RefCell;
use std::fs;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::PathBuf;

// Imports `crate::journal::KeepAs`.
use crate::journal::KeepAs;
// Imports `crate::manifest_edit::{ManifestEdit, restor...`.
use crate::manifest_edit::{ManifestEdit, restore_manifests, run_recording_manifests, snapshot_manifests};
use crate::test_scratch::scratch;

const ROOT_A: &str = "package/a";
const ROOT_B: &str = "package/b";

fn keep() -> KeepAs {
    return KeepAs::next_to_target(".displaced");
}

fn edits_for(crate_dir: &str, path: &std::path::Path) -> RefCell<Vec<ManifestEdit>> {
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
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    snapshot_manifests(crate_dir, &[path.to_path_buf()], &edits).expect("snapshots");
    return edits;
}

#[test]
fn a_manifest_the_call_rewrote_is_recorded_and_put_back() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(ROOT_A, &path);

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    run_recording_manifests(ROOT_A, &edits, || { fs::write(&path, "cargo's edit").expect("cargo edits it"); }).expect("records");
    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(&edits);

    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(warnings.len(), 1, "the put-back names the file it kept: {warnings:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(warnings[0].contains("is kept as"), "{warnings:?}");
    assert_eq!(fs::read_to_string(&path).expect("read"), "before");
}

#[test]
fn a_manifest_another_process_changed_after_the_call_is_left_alone_with_a_warning() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(ROOT_A, &path);
    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    run_recording_manifests(ROOT_A, &edits, || { fs::write(&path, "cargo's edit").expect("cargo edits it"); }).expect("records");
    fs::write(&path, "somebody else's edit").expect("another process");

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(&edits);

    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read_to_string(&path).expect("read"), "somebody else's edit");
}

/// Review finding (2b 1,
/// 2c 4):
/// an editor changes a member the call never touched,
/// during the call of another root;
/// that change must
/// not be recorded as the call's output and must survive the restore.
#[test]
fn an_edit_to_a_manifest_the_call_did_not_write_is_not_recorded_and_survives() {
    let root = scratch();
    let first = root.path().join("a.toml");
    let second = root.path().join("b.toml");
    // Borrow `&first`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&first, "a before").expect("manifest");
    // Borrow `&second`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&second, "b before").expect("manifest");
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
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
    // Borrow `&[`: lends the value without copying or giving it away.
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    snapshot_manifests(ROOT_A, std::slice::from_ref(&first), &edits).expect("snapshots");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.clone()`: an independent owned copy.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    snapshot_manifests(ROOT_B, std::slice::from_ref(&second), &edits).expect("snapshots");

    // Borrow `&edits`: lends the value without copying or giving it away.
    run_recording_manifests(ROOT_A, &edits, || {
        // Borrow `&first`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(&first, "a edited by cargo").expect("cargo edits its own root");
        // Borrow `&second`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(&second, "b edited by a person meanwhile").expect("an editor");
    })
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    .expect("records");
    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(&edits);

    // Borrow `&first`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&first).expect("read"), "a before");
    // Borrow `&second`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&second).expect("read"), "b edited by a person meanwhile");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(warnings.len(), 2, "one for the put-back of a, one for the untouched edit of b: {warnings:?}");
}

/// A change that lands after the call returned is outside its window and is not the call's output.
#[test]
fn a_change_published_after_the_call_returned_is_not_taken_for_its_output() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(ROOT_A, &path);
    // `.clone()`: an independent owned copy.
    // Typed binding: `late: PathBuf`.
    let late: PathBuf = path.clone();

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    run_recording_manifests(ROOT_A, &edits, || {}).expect("records");
    // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
    // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    std::thread::sleep(std::time::Duration::from_millis(50));
    // Borrow `&late`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&late, "published after the call").expect("a late writer");
    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(&edits);

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&path).expect("read"), "published after the call");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

#[test]
fn a_manifest_whose_output_was_never_recorded_is_never_restored() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(ROOT_A, &path);
    fs::write(&path, "unknown author").expect("someone edits it");

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let warnings = restored(&edits);

    assert_eq!(warnings.len(), 1);
    assert_eq!(fs::read_to_string(&path).expect("read"), "unknown author");
}

#[test]
fn an_unchanged_manifest_needs_nothing_and_a_missing_or_unreadable_one_stops_the_snapshot() {
    let root = scratch();
    let path = root.path().join("Cargo.toml");
    fs::write(&path, "before").expect("manifest");
    // Borrow `&path`: lends the value without copying or giving it away.
    let edits = edits_for(ROOT_A, &path);

    // Borrow `&edits`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(restored(&edits).is_empty());
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    assert!(snapshot_manifests(ROOT_A, &[root.path().join("missing.toml")], &RefCell::new(Vec::new())).is_err());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join("binary.toml"), [0xff, 0xfe, 0x00]).expect("not utf-8");
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    assert!(snapshot_manifests(ROOT_A, &[root.path().join("binary.toml")], &RefCell::new(Vec::new())).is_err(), "invalid UTF-8 is an error, not an empty manifest");
}

/// Restores the manifests and requires that nothing failed.
fn restored(edits: &RefCell<Vec<ManifestEdit>>) -> Vec<String> {
    // Borrow `&keep`: lends the value without copying or giving it away.
    let (warnings, failure) = restore_manifests(edits, &keep());
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(failure.is_none(), "{failure:?}");
    return warnings;
}
