//! Helpers the journal tests share:
//! planting the journal of a killed run and finding journals.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs` for this file.
use std::fs;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

// Imports `crate::crates::LockSnapshot`.
/// Imports `crate::crates::LockSnapshot` for this file.
use crate::crates::LockSnapshot;
// Imports `crate::journal::{JOURNAL_DIR, begin}`.
/// Imports `crate::journal::{JOURNAL_DIR, begin}` for this file.
use crate::journal::{JOURNAL_DIR, begin};
// Imports `crate::manifest_edit::ManifestEdit`.
/// Imports `crate::manifest_edit::ManifestEdit` for this file.
use crate::manifest_edit::ManifestEdit;

/// Plants the journal of a run that was killed after it journaled its snapshots (and manifests):
/// the transaction is begun and then
/// dropped,
/// so nothing holds it any more and nothing finished it.
pub fn plant(root: &Path, snapshots: &[LockSnapshot], manifests: &[ManifestEdit]) -> PathBuf {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let before: Vec<PathBuf> = journal_dirs(root);
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    let transaction = begin(root, snapshots, !manifests.is_empty()).expect("begin");
    if !manifests.is_empty() {
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        transaction.add_manifests(manifests).expect("journals the manifests");
    }
    drop(transaction);
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `added: Vec<PathBuf>`.
    let mut added: Vec<PathBuf> = journal_dirs(root).into_iter().filter(|path| { return !before.contains(path); }).collect();
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(added.len(), 1, "expected exactly one new journal: {added:?}");
    return added.remove(0);
}

/// Every journal directory under the root (completed ones are gone;
/// claims and `.done` directories count too).
pub fn journal_dirs(root: &Path) -> Vec<PathBuf> {
    // Typed binding: `directory: PathBuf`.
    let directory: PathBuf = root.join(JOURNAL_DIR);
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
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
    // Only the initial absence of the journal directory is an empty listing; any other failure to list it, and any failure on an
    // entry, stops the test (certification group 3: a journal parent that became unreadable must not read as "no journals").
    let listing = match fs::read_dir(&directory) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => panic!("the journals of {} cannot be listed ({error}): an unreadable directory is not an empty one", directory.display()),
    };
    return listing
        .map(|entry| { return entry.unwrap_or_else(|error| { panic!("an entry of {} cannot be read ({error})", directory.display()); }).path(); })
        .collect();
}

/// The only journal directory under the root.
pub fn journal_dir(root: &Path) -> PathBuf {
    // `let mut`: a binding that may change.
    // Typed binding: `all: Vec<PathBuf>`.
    let mut all: Vec<PathBuf> = journal_dirs(root);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(all.len(), 1, "expected exactly one journal: {all:?}");
    return all.remove(0);
}
