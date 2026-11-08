//! The tests of the third review of the replace protocol and the journal (3b):
//! data is never unlinked while it can still be
//! open elsewhere,
//! a failed root is never defaulted to a restoration,
//! and a disappearance is left alone.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::cell::{Cell, RefCell};
// Imports `std::fs::{self, File, OpenOptions}`.
use std::fs::{self, File, OpenOptions};
// Imports `std::io::Write`.
use std::io::Write;
// Imports `std::os::unix::fs::MetadataExt`.
use std::os::unix::fs::MetadataExt;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::{Path, PathBuf};

// Imports `crate::crates::LockSnapshot`.
use crate::crates::LockSnapshot;
// Imports `crate::journal::{JOURNAL_DIR, KeepAs, Repla...`.
use crate::journal::{JOURNAL_DIR, KeepAs, ReplaceOutcome, begin, recover, replace_if_holds, replace_if_holds_with_hooks};
// Imports `crate::journal_support::plant`.
use crate::journal_support::plant;
// Imports `crate::journal_replace::{Seams, replace_wit...`.
use crate::journal_replace::{Seams, replace_with_seams};
// Imports `crate::manifest_edit::ManifestEdit`.
use crate::manifest_edit::ManifestEdit;
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/j/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the given lockfile.
fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

/// Every file name in a directory.
fn names_in(directory: &Path) -> Vec<String> {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return fs::read_dir(directory).expect("list").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
}

/// Whether some file in the directory holds exactly this text.
fn some_file_holds(directory: &Path, text: &str) -> bool {
    // Closure `|name|`: an anonymous function, like a TS arrow function.
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    return names_in(directory).iter().any(|name| { return fs::read_to_string(directory.join(name)).map(|found| { return found == text; }).unwrap_or(false); });
}

/// Whether the process may ignore directory permissions,
/// which makes the permission-based failures unreliable.
fn ignores_permissions() -> bool {
    // Closure `|metadata|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    return fs::metadata("/proc/self").map(|metadata| { return metadata.uid() == 0; }).unwrap_or(true);
}

/// Review finding (3b 1):
/// after the exchange was undone the replacement inode has been live,
/// and another writer can hold it open;
/// it must be kept,
/// not unlinked,
/// so a write through that descriptor still has a name.
#[test]
fn a_replacement_that_another_writer_holds_open_is_kept_when_the_exchange_is_undone() {
    let root = repository("expected");
    let directory = root.path().join(CRATE_DIR);
    let path = directory.join("Cargo.lock");
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let first_writer = |target: &Path| { fs::write(target, "first writer").expect("the first writer"); };
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
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
    let held: RefCell<Option<File>> = RefCell::new(None);
    // `Some`: the present or the absent value of an `Option`.
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let open_replacement = |target: &Path| { *held.borrow_mut() = Some(OpenOptions::new().write(true).open(target).expect("open the live replacement")); };

    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_if_holds_with_hooks(&path, "expected", "new text", &first_writer, &open_replacement).expect("answers");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    held.borrow_mut().as_mut().expect("held").write_all(b"late edit").expect("the late write");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(fs::read_to_string(&path).expect("live"), "first writer", "{outcome:?}");
    // Borrow `&directory`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(some_file_holds(&directory, "late edit"), "the write through the open descriptor has no name left: {:?}", names_in(&directory));
}

/// Review finding (3b 2 B):
/// when the expected file was displaced but could not be linked to its kept name,
/// the snapshot is live and
/// the outcome says so,
/// instead of reading like an untouched file.
#[test]
fn a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live() {
    let root = repository("expected");
    let directory = root.path().join(CRATE_DIR);
    let path = directory.join("Cargo.lock");
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    let no_link = |_existing: &Path, _new: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected link failure")); };
    // Closure `|_path: &Path|`: an anonymous function, like a TS arrow function.
    let none = |_path: &Path| {};
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    let sync_file = |target: &Path| -> std::io::Result<()> { return File::open(target)?.sync_all(); };
    let sync_dir = |target: &Path| -> std::io::Result<()> { return crate::journal::sync_directory(target); };
    let swap = |first: &Path, second: &Path| -> std::io::Result<()> { return crate::journal_replace::exchange_paths(first, second); };
    let seams = Seams { between: &none, after_exchange: &none, swap: &swap, link: &no_link, sync_file: &sync_file, sync_dir: &sync_dir };


    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_with_seams(&path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &seams).expect("answers");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&path).expect("live"), "new text");
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    match outcome {
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Macro `assert_eq!`: builds or prints text (or a list) from a template.
        ReplaceOutcome::Displaced(kept) => assert_eq!(fs::read_to_string(kept).expect("the displaced file"), "expected"),
        // Macro `panic!`: builds or prints text (or a list) from a template.
        other => panic!("expected Displaced, got {other:?}"),
    }
}

/// Review finding (3b 10):
/// a file deleted after the comparison stays deleted.
#[test]
fn a_file_deleted_after_the_comparison_stays_deleted() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // Closure `|target: &Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let delete = |target: &Path| { fs::remove_file(target).expect("the deletion"); };

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let outcome = replace_if_holds(&path, "expected", "new text", &delete).expect("answers");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(outcome, ReplaceOutcome::LeftAlone);
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!path.exists(), "the deletion was undone");
}

/// Review finding (3b 2 A):
/// a lockfile whose restoration failed leaves its root's manifest alone;
/// the pair is never split.
#[test]
fn a_failed_lockfile_restore_leaves_the_roots_manifest_alone() {
    if ignores_permissions() {
        return;
    }
    use std::os::unix::fs::PermissionsExt;
    let root = repository("unchecked output of the killed run");
    // Typed binding: `manifest_dir: PathBuf`.
    let manifest_dir: PathBuf = root.path().join("manifests");
    // Borrow `&manifest_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&manifest_dir).expect("manifest dir");
    // Typed binding: `manifest: PathBuf`.
    let manifest: PathBuf = manifest_dir.join("Cargo.toml");
    // Borrow `&manifest`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&manifest, "manifest rewritten by the killed run").expect("manifest");
    // `None`: the present or the absent value of an `Option`.
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let edit = ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "manifest before".to_string(), produced: None };
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[edit]);
    let crate_dir = root.path().join(CRATE_DIR);
    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o555)).expect("make the lockfile directory read-only");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&crate_dir, fs::Permissions::from_mode(0o755)).expect("writable again");
    // Borrow `&manifest`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&manifest).expect("manifest"), "manifest rewritten by the killed run", "{messages:?}");
}

/// Review finding (3b 9):
/// a temporary name that already exists is never truncated;
/// the next name is used.
#[test]
fn a_temporary_name_that_exists_is_never_truncated() {
    let root = scratch();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(".x.shadow-same.tmp"), "keep me").expect("existing temporary");
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    let mut names = ["same", "fresh"].into_iter().map(str::to_string);

    // Borrow `&mut`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let (created, _device, _inode) = crate::journal_replace::create_exclusive(root.path(), "x", "new", 0o644, &mut || { return names.next().expect("a name"); }).expect("creates");

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(".x.shadow-same.tmp")).expect("existing"), "keep me");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(created).expect("created"), "new");
}

/// A name generator that never yields a free name ends in an error that names the directory,
/// not in a loop that never ends.
#[test]
fn a_temporary_name_search_ends_with_an_error() {
    let root = scratch();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(".x.shadow-same.tmp"), "keep me").expect("existing temporary");

    // Borrow `&mut`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let result = crate::journal_replace::create_exclusive(root.path(), "x", "new", 0o644, &mut || { return "same".to_string(); });

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `message: String`.
    let message: String = result.expect_err("no free name").to_string();
    // Borrow `&root.path`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains(&root.path().display().to_string()), "{message}");
}

/// Review finding (3b 4):
/// the lock the owner holds is the proof that it lives;
/// a record that names a dead process,
/// a renamed
/// directory or an unreadable `/proc` do not change that.
#[test]
fn a_journal_whose_owner_still_holds_its_lock_is_not_recovered() {
    let root = repository("unchecked output of the running owner");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let _owner = begin(root.path(), &[snapshot("original")], false).expect("begin");
    let journal = crate::journal_support::journal_dir(root.path());
    // Borrow `&journal`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::rename(&journal, root.path().join(JOURNAL_DIR).join("4194305")).expect("a name that says the process is dead");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the running owner", "{messages:?}");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 1);
}

/// Review finding (3b 7):
/// a ready journal whose manifest index is missing is kept,
/// not read as "no manifests".
#[test]
fn a_missing_manifest_index_keeps_the_journal_and_its_snapshots() {
    let root = repository("unchecked output of the killed run");
    let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
    // Borrow `&manifest`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&manifest, "edited by cargo").expect("manifest");
    // `None`: the present or the absent value of an `Option`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let edit = ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest, before: "manifest before".to_string(), produced: None };
    // Borrow `&[`: lends the value without copying or giving it away.
    let journal = plant(root.path(), &[snapshot("original")], &[edit]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(journal.join("manifests.txt")).expect("lose the manifest index");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal.join("m0.toml").exists(), "the only copy of the manifest was deleted with the journal");
}

/// A journal whose preparation never finished (its owner was killed before cargo ran) changed nothing;
/// with every lockfile equal to
/// its snapshot the copies are redundant,
/// so recovery removes it and does not touch the lockfile.
#[test]
fn a_journal_that_never_became_ready_is_removed_without_touching_the_lockfile() {
    let root = repository("original");
    let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin, manifests promised but never journaled");
    drop(transaction);

    recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "original");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty(), "an unready journal with redundant copies is removed");
}
/// A journal whose removal was interrupted after the completion marker is finished by the next recovery.
#[test]
fn a_journal_marked_done_is_removed_by_the_next_recovery() {
    let root = repository("good output");
    // Borrow `&[`: lends the value without copying or giving it away.
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let done = journal.with_file_name(format!("{}.done", journal.file_name().expect("name").to_string_lossy()));
    // Borrow `&journal`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::rename(&journal, &done).expect("mark it done");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "good output");
}

/// Finishing a transaction removes its journal,
/// and a second recovery has nothing to do.
#[test]
fn finish_removes_the_journal() {
    let root = repository("x");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let transaction = begin(root.path(), &[snapshot("x")], false).expect("begin");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 1);

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    transaction.finish().expect("finish");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty());
}

/// A journal of a killed run whose snapshots match the live files needs nothing and is removed;
/// the files are not touched.
#[test]
fn a_killed_run_that_changed_nothing_leaves_the_lockfile_alone() {
    let root = repository("original");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["never asked".to_string()]; });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages.is_empty(), "{messages:?}");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "original");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty());
}

/// Review finding (4b2 1):
/// one workspace lock covers every transaction,
/// so a second one cannot start while the first lives;
/// it fails at once,
/// names the holder and leaves the first journal as it was.
#[test]
fn a_second_begin_while_the_first_transaction_lives_fails_and_names_the_holder() {
    let root = repository("lock");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let _first = begin(root.path(), &[snapshot("first original")], false).expect("first begin");

    // Borrow `&[`: lends the value without copying or giving it away.
    let second = begin(root.path(), &[snapshot("second original")], false);

    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `message: String`.
    let message: String = second.err().map(|error| { return error.to_string(); }).unwrap_or_default();
    // Borrow `&std`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains(&std::process::id().to_string()) && message.contains("another cargo-shadow run"), "{message}");
    let journals = crate::journal_support::journal_dirs(root.path());
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(journals.len(), 1, "{journals:?}");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(journals[0].join("0.lock")).expect("snapshot"), "first original");
}

/// Review finding (4b2 1):
/// recovery of a dead run's journal defers while a live transaction holds the workspace.
#[test]
fn recovery_defers_while_a_live_transaction_holds_the_workspace() {
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original of the killed run")], &[]);
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let _owner = begin(root.path(), &[snapshot("original of the live run")], false).expect("the live transaction");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // The live transaction is still held when the recovery must answer (certification group 1: a busy lock returns at once).
    let path: PathBuf = root.path().to_path_buf();
    let messages = crate::round_sixteen_tests::answers_within("the recovery", std::time::Duration::from_secs(3), move || { return recover(&path, &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; }); });

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the killed run", "{messages:?}");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(crate::journal_support::journal_dirs(root.path()).len(), 2, "the killed run's journal is kept for a later run");
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages.iter().any(|message| { return message.contains("another cargo-shadow run"); }), "{messages:?}");
}

/// Review finding (4b2 1):
/// while a recovery replaces files it holds the workspace,
/// so a transaction cannot start in the gap.
#[test]
fn a_transaction_cannot_begin_while_a_recovery_replaces_files() {
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    // `None`: the present or the absent value of an `Option`.
    // Typed binding: `refused: std::cell::RefCell<Option<bool>>`.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let refused: std::cell::RefCell<Option<bool>> = std::cell::RefCell::new(None);
    // `Some`: the present or the absent value of an `Option`.
    // Closure `|_path: &Path|`: an anonymous function, like a TS arrow function.
    // Borrow `&[`: lends the value without copying or giving it away.
    let try_begin = |_path: &Path| { *refused.borrow_mut() = Some(begin(root.path(), &[snapshot("x")], false).is_err()); };

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // Borrow `&try_begin`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    crate::journal::recover_with(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; }, &try_begin);

    // `Some`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(*refused.borrow(), Some(true), "a transaction started in the middle of a recovery");
}

/// Review finding (4b2 2):
/// a child of a killed run that is still alive keeps its journal from being recovered,
/// even though the
/// wrapper's lock is gone.
#[test]
fn a_journal_whose_recorded_process_group_is_still_alive_is_left_alone() {
    use std::os::unix::process::CommandExt;
    let root = repository("unchecked output of the killed run");
    // Borrow `&[`: lends the value without copying or giving it away.
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // `let mut`: a binding that may change.
    let mut orphan = std::process::Command::new("sleep").arg("30").process_group(0).spawn().expect("a surviving child");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(journal.join("groups"), format!("pgroup {}\n", orphan.id())).expect("record the group");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = orphan.kill();
    // `let _ =`: discards the result on purpose.
    let _ = orphan.wait();

    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the killed run", "{messages:?}");
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages.iter().any(|message| { return message.contains("still running"); }), "{messages:?}");
}

/// Review finding (4b2 3):
/// the outputs are made durable before the completion marker exists.
/// Round 6 (5b2 2):
/// a group record that cannot be read or understood is uncertainty,
/// never "no children".
#[test]
fn a_group_record_that_cannot_be_read_defers_the_recovery() {
    let root = repository("unchecked output of the killed run");
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    fs::remove_file(journal.join("groups")).expect("remove the record");
    fs::create_dir(journal.join("groups")).expect("a directory where the record was: reading it fails");

    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the killed run", "{messages:?}");
    assert!(messages.iter().any(|message| { return message.contains("record"); }), "{messages:?}");
    // The refusal keeps the journal itself, with its snapshot (certification, third pass): a recovery that reported the record and
    // then discarded the journal would pass the two assertions above.
    assert_eq!(crate::journal_support::journal_dirs(root.path()), vec![journal.clone()], "the journal with the unreadable record was removed or replaced: {messages:?}");
    assert!(journal.join("0.lock").exists() && journal.join("groups").is_dir(), "the journal's snapshot or its unreadable record is gone: {messages:?}");
}

/// Round 6 (5b2 2):
/// a malformed group record is uncertainty too.
#[test]
fn a_malformed_group_record_defers_the_recovery() {
    let root = repository("unchecked output of the killed run");
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    fs::write(journal.join("groups"), "pgroup 4242\nnot a number\n").expect("write a damaged record");

    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the killed run", "{messages:?}");
}

/// Round 6 (5b2 1):
/// a surviving child of one killed run stops the recovery of every other journal of the workspace,
/// because that child still writes the same files.
#[test]
fn a_survivor_of_one_journal_stops_the_recovery_of_another() {
    use std::os::unix::process::CommandExt;
    let root = repository("unchecked output of the killed run");
    let survivor_journal = plant(root.path(), &[snapshot("first")], &[]);
    let _other = plant(root.path(), &[snapshot("second")], &[]);
    let mut orphan = std::process::Command::new("sleep").arg("30").process_group(0).spawn().expect("a surviving child");
    fs::write(survivor_journal.join("groups"), format!("pgroup {}\n", orphan.id())).expect("record the group");

    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });
    let _ = orphan.kill();
    let _ = orphan.wait();

    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "unchecked output of the killed run", "{messages:?}");
}

/// Round 6 (5b2 1):
/// a new transaction does not start while a child of a killed run may still be writing.
#[test]
fn a_new_transaction_does_not_start_beside_a_survivor() {
    use std::os::unix::process::CommandExt;
    let root = repository("unchecked output of the killed run");
    let journal = plant(root.path(), &[snapshot("first")], &[]);
    let mut orphan = std::process::Command::new("sleep").arg("30").process_group(0).spawn().expect("a surviving child");
    fs::write(journal.join("groups"), format!("pgroup {}\n", orphan.id())).expect("record the group");

    let started = begin(root.path(), &[snapshot("second")], false);
    let _ = orphan.kill();
    let _ = orphan.wait();

    assert!(started.is_err(), "a transaction started beside a surviving child");
}

#[test]
fn finish_makes_the_outputs_durable_before_the_done_marker() {
    let root = repository("final output");
    let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
    // Borrow `&manifest`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&manifest, "final manifest").expect("manifest");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin");
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.clone()`: an independent owned copy.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    transaction.add_manifests(&[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "before".to_string(), produced: None }]).expect("manifests");
    // Typed binding: `seen: std::cell::RefCell<Vec<(PathBuf, bool...`.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let seen: std::cell::RefCell<Vec<(PathBuf, bool)>> = std::cell::RefCell::new(Vec::new());
    let journals = root.path().join(JOURNAL_DIR);
    // Closure `|path: &Path|`: an anonymous function, like a TS arrow function.
    let record = |path: &Path| {
        // Closure `|listing|`: an anonymous function, like a TS arrow function.
        // Borrow `&journals`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `done_exists: bool`.
        let done_exists: bool = fs::read_dir(&journals).map(|listing| { return listing.flatten().any(|entry| { return entry.file_name().to_string_lossy().ends_with(".done"); }); }).unwrap_or(false);
        seen.borrow_mut().push((path.to_path_buf(), done_exists));
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    };

    // Borrow `&record`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    transaction.finish_with(&record).expect("finish");

    let calls = seen.borrow();
    // Closure `|(path, _)|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(calls.iter().any(|(path, _)| { return path.ends_with("Cargo.lock"); }), "{calls:?}");
    // Closure `|(path, _)|`: an anonymous function, like a TS arrow function.
    // Borrow `&manifest`: lends the value without copying or giving it away.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(calls.iter().any(|(path, _)| { return path == &manifest; }), "{calls:?}");
    // Closure `|(_, done)|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(calls.iter().all(|(_, done)| { return !done; }), "an output was synced after the marker: {calls:?}");
}

/// Review finding (4b2 6):
/// a journal that has an index but no ready record is kept when a lockfile differs from its snapshot,
/// because that could be a damaged ready journal.
#[test]
fn an_indexed_journal_without_a_ready_record_is_kept_when_a_lockfile_differs() {
    let root = repository("changed by something");
    // Borrow `&[`: lends the value without copying or giving it away.
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(journal.join("ready")).expect("lose the ready record");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(journal.join("0.lock").exists(), "the only copy of the lockfile was deleted");
}

/// The same journal is removed when every lockfile equals its snapshot,
/// because the copies then lose nothing.
#[test]
fn an_indexed_journal_without_a_ready_record_is_removed_when_every_lockfile_equals_its_snapshot() {
    let root = repository("original");
    // Borrow `&[`: lends the value without copying or giving it away.
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(journal.join("ready")).expect("lose the ready record");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return vec!["never asked".to_string()]; });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(crate::journal_support::journal_dirs(root.path()).is_empty());
}

/// Review finding (4b2 6):
/// a ready transaction's payload is frozen;
/// adding manifests twice is refused and changes nothing.
#[test]
fn adding_manifests_twice_is_refused() {
    let root = repository("lock");
    let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin");
    // `None`: the present or the absent value of an `Option`.
    // `.clone()`: an independent owned copy.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let edit = || { return ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "first".to_string(), produced: None }; };
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    transaction.add_manifests(&[edit()]).expect("first call");

    // Borrow `&[`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    let again = transaction.add_manifests(&[ManifestEdit { before: "second".to_string(), ..edit() }]);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(again.is_err());
    let journal = crate::journal_support::journal_dir(root.path());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(journal.join("m0.toml")).expect("snapshot"), "first");
}

/// Standard seams except for the ones given,
/// for the durability tests.
fn replace_with(path: &Path, keep: &KeepAs, link: &dyn Fn(&Path, &Path) -> std::io::Result<()>, sync_file: &dyn Fn(&Path) -> std::io::Result<()>, sync_dir: &dyn Fn(&Path) -> std::io::Result<()>) -> std::io::Result<ReplaceOutcome> {
    // Closure `|_path: &Path|`: an anonymous function, like a TS arrow function.
    let none = |_path: &Path| {};
    let swap = |first: &Path, second: &Path| -> std::io::Result<()> { return crate::journal_replace::exchange_paths(first, second); };
    // Borrow `&Seams`: lends the value without copying or giving it away.
    return replace_with_seams(path, "expected", "new text", keep, &Seams { between: &none, after_exchange: &none, swap: &swap, link, sync_file, sync_dir });
}

/// One step of a preservation,
/// in the order it happened:
/// a successful link and its new name,
/// or a directory barrier and whether it held.
#[derive(Debug)]
enum Event {
    /// A hard link was made with this new name.
    Linked(
        /// The new name.
        PathBuf,
    ),
    /// A directory barrier ran on this directory and held (`true`) or failed.
    Barrier(
        /// The directory.
        PathBuf,
        /// Whether it held.
        bool,
    ),
}

/// Like `replace_with`,
/// and also returns the inode the live path names right after the exchange,
/// which is the replacement's (9b 3.1).
fn replace_capturing(path: &Path, keep: &KeepAs, link: &dyn Fn(&Path, &Path) -> std::io::Result<()>, sync_file: &dyn Fn(&Path) -> std::io::Result<()>, sync_dir: &dyn Fn(&Path) -> std::io::Result<()>) -> (std::io::Result<ReplaceOutcome>, Option<u64>) {
    let replacement: Cell<Option<u64>> = Cell::new(None);
    let none = |_path: &Path| {};
    let capture = |live: &Path| { replacement.set(fs::metadata(live).ok().map(|found| { return found.ino(); })); };
    let swap = |first: &Path, second: &Path| -> std::io::Result<()> { return crate::journal_replace::exchange_paths(first, second); };
    let outcome = replace_with_seams(path, "expected", "new text", keep, &Seams { between: &none, after_exchange: &capture, swap: &swap, link, sync_file, sync_dir });
    return (outcome, replacement.get());
}

/// The real hard link.
fn real_link(existing: &Path, new: &Path) -> std::io::Result<()> {
    return fs::hard_link(existing, new);
}

/// The real data barrier.
fn real_sync_file(path: &Path) -> std::io::Result<()> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    return File::open(path)?.sync_all();
}

/// The real directory barrier.
fn real_sync_dir(path: &Path) -> std::io::Result<()> {
    return crate::journal::sync_directory(path);
}

/// Review finding (4b 2):
/// a failed data barrier on the displaced file is not turned into a settled outcome;
/// it is an error that
/// says where the file stays.
#[test]
fn a_failed_data_sync_of_the_displaced_file_is_an_error_naming_where_it_stays() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    let failing = |_path: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected EIO")); };

    // Borrow `&path`: lends the value without copying or giving it away.
    let result = replace_with(&path, &KeepAs::next_to_target(".displaced"), &real_link, &failing, &real_sync_dir);

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `message: String`.
    let message: String = result.expect_err("an unsettled replacement").to_string();
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("injected EIO") && message.contains(".shadow-"), "{message}");
}

/// Contract K2,
/// round 8 (acceptance row K2):
/// the entries of a kept copy are made durable from the kept directory up to the first directory that existed before the
/// transaction,
/// and the walk stops there:
/// nothing above that anchor is synchronized.
#[test]
fn the_walk_stops_at_the_first_directory_that_existed_before_the_transaction() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let anchor = root.path().join("a").join("b");
    fs::create_dir_all(&anchor).expect("a pre-existing anchor");
    let kept_dir = anchor.join("c").join("d");
    // Links and barriers in one ordered log (certification group 3): the kept entry must be made durable after its own link, and the
    // barriers of the new directory chain made before it do not do that.
    let synced: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let events: RefCell<Vec<Event>> = RefCell::new(Vec::new());
    // What each barrier's directory listed at the moment of the barrier (certification, second pass, K2): a barrier of a parent
    // makes the entry of a new child durable only if the child already exists in it.
    let listed: RefCell<Vec<(usize, PathBuf, Vec<String>)>> = RefCell::new(Vec::new());
    let record = |directory: &Path| -> std::io::Result<()> {
        synced.borrow_mut().push(directory.to_path_buf());
        let position: usize = events.borrow().len();
        events.borrow_mut().push(Event::Barrier(directory.to_path_buf(), true));
        let names: Vec<String> = fs::read_dir(directory).map(|entries| { return entries.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); }).unwrap_or_default();
        listed.borrow_mut().push((position, directory.to_path_buf(), names));
        return Ok(());
    };
    let logging_link = |existing: &Path, new: &Path| -> std::io::Result<()> {
        real_link(existing, new)?;
        events.borrow_mut().push(Event::Linked(new.to_path_buf()));
        return Ok(());
    };

    replace_with(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &logging_link, &real_sync_file, &record).expect("replaces");

    let log = events.borrow();
    let preserved_at: usize = log.iter().position(|event| { return matches!(event, Event::Linked(name) if name.starts_with(&kept_dir)); }).unwrap_or_else(|| { panic!("no link made a name in the kept directory: {log:?}"); });
    let resolved_kept: PathBuf = fs::canonicalize(&kept_dir).expect("resolves");
    assert!(
        log[preserved_at..].iter().any(|event| { return matches!(event, Event::Barrier(directory, true) if *directory == kept_dir || *directory == resolved_kept); }),
        "no successful barrier of the kept directory came after the link that preserved the file: {log:?}"
    );
    // Each new child directory entry is published and then covered, before the file is linked into the kept directory, by a barrier of
    // its parent that already lists it: `c` in the anchor, `d` in `c`. A parent that is synchronized only before the child exists
    // leaves the child's entry unprotected, and the climb after the link is a second, separate protection.
    let barriers = listed.borrow();
    for (parent, child) in [(anchor.clone(), "c"), (anchor.join("c"), "d")] {
        let resolved_parent: PathBuf = fs::canonicalize(&parent).expect("resolves");
        assert!(
            barriers.iter().any(|(position, directory, names)| { return *position < preserved_at && (*directory == parent || *directory == resolved_parent) && names.iter().any(|name| { return name == child; }); }),
            "no barrier of {} came after {child} was created in it and before the link: {barriers:?}",
            parent.display()
        );
    }
    // The climb after the preservation link covers the kept directory and every new directory above it up to the anchor.
    for needed in [kept_dir.clone(), anchor.join("c"), anchor.clone()] {
        let resolved: PathBuf = fs::canonicalize(&needed).expect("resolves");
        assert!(
            log[preserved_at..].iter().any(|event| { return matches!(event, Event::Barrier(directory, true) if *directory == needed || *directory == resolved); }),
            "no barrier of {} came after the link that preserved the file: {log:?}",
            needed.display()
        );
    }
    let calls = synced.borrow();
    let resolved_anchor: PathBuf = fs::canonicalize(&anchor).expect("resolves");
    for needed in [kept_dir.clone(), anchor.join("c"), anchor.clone()] {
        let resolved: PathBuf = fs::canonicalize(&needed).expect("resolves");
        assert!(calls.contains(&resolved) || calls.contains(&needed), "{} was not synced: {calls:?}", needed.display());
    }
    for above in [resolved_anchor.parent().expect("a parent").to_path_buf(), anchor.parent().expect("a parent").to_path_buf(), root.path().to_path_buf()] {
        assert!(!calls.contains(&above), "{} is above the anchor and was synced: {calls:?}", above.display());
    }
}

/// Contract assumption b,
/// the knowledge clause:
/// a directory this process knows it left unsynchronized in an earlier failed attempt is not taken as durable because it exists now;
/// the walk goes through it to the pre-existing directory above it.
#[test]
fn a_directory_left_unsynced_by_a_failed_attempt_is_not_taken_as_durable() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let anchor = root.path().join("a").join("b");
    fs::create_dir_all(&anchor).expect("a pre-existing anchor");
    let kept_dir = anchor.join("c");
    let failing = |_directory: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected sync failure")); };
    let first_attempt = crate::journal::create_directories_with(&kept_dir, &failing);
    assert!(first_attempt.is_err(), "the failed attempt should fail");
    assert!(kept_dir.exists(), "the directory exists although it was not made durable");
    let synced: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let record = |directory: &Path| -> std::io::Result<()> { synced.borrow_mut().push(directory.to_path_buf()); return Ok(()); };

    replace_with(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &real_link, &real_sync_file, &record).expect("replaces");

    let calls = synced.borrow();
    let resolved_anchor: PathBuf = fs::canonicalize(&anchor).expect("resolves");
    assert!(calls.contains(&resolved_anchor), "the walk stopped at a directory known to be unsynced: {calls:?}");

    let control = repository("expected");
    let control_path = control.path().join(CRATE_DIR).join("Cargo.lock");
    let control_kept = control.path().join("a").join("b").join("c2");
    crate::journal::create_directories_synced(&control_kept).expect("a directory made durably");
    let control_synced: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let control_record = |directory: &Path| -> std::io::Result<()> { control_synced.borrow_mut().push(directory.to_path_buf()); return Ok(()); };
    replace_with(&control_path, &KeepAs::in_directory(&control_kept, ".displaced"), &real_link, &real_sync_file, &control_record).expect("replaces");
    let control_anchor: PathBuf = fs::canonicalize(control.path().join("a").join("b")).expect("resolves");
    assert!(!control_synced.borrow().contains(&control_anchor), "a durably made directory was walked through");
}

/// Acceptance row K2:
/// a required directory barrier that fails is not reported as preservation,
/// and the displaced inode keeps the name it was linked under.
#[test]
fn a_failing_directory_barrier_is_not_reported_as_preserved() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let anchor = root.path().join("a").join("b");
    fs::create_dir_all(anchor.join("c")).expect("a pre-existing kept directory");
    let kept_dir = anchor.join("c");
    let resolved_kept: PathBuf = fs::canonicalize(&kept_dir).expect("resolves");
    let failing = move |directory: &Path| -> std::io::Result<()> {
        if directory == resolved_kept { return Err(std::io::Error::other("injected barrier failure")); }
        return Ok(());
    };

    let result = replace_with(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &real_link, &real_sync_file, &failing);

    let message: String = result.expect_err("a failed barrier is not a success").to_string();
    assert!(message.contains("injected barrier failure") || message.contains("not made durable"), "{message}");
    let names: Vec<String> = fs::read_dir(&kept_dir).expect("lists").flatten().map(|entry| { return entry.file_name().to_string_lossy().to_string(); }).collect();
    assert!(names.iter().any(|name| { return name.contains(".displaced"); }), "the displaced file lost its name: {names:?}");
}

/// Acceptance row K2:
/// an ancestor above the anchor that the user cannot open for reading is never opened.
#[test]
fn a_higher_ancestor_the_user_cannot_open_is_never_touched() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let above = root.path().join("a");
    let anchor = above.join("b");
    fs::create_dir_all(&anchor).expect("a pre-existing anchor");
    let kept_dir = anchor.join("c");
    fs::set_permissions(&above, fs::Permissions::from_mode(0o311)).expect("make the ancestor unreadable");
    if fs::read_dir(&above).is_ok() {
        fs::set_permissions(&above, fs::Permissions::from_mode(0o755)).expect("make it readable again");
        eprintln!("SKIPPED: this user opens a directory without the read permission (a privileged user), so the schedule cannot be made");
        return;
    }

    let result = replace_with(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &real_link, &real_sync_file, &real_sync_dir);

    fs::set_permissions(&above, fs::Permissions::from_mode(0o755)).expect("make it readable again");
    assert!(result.is_ok(), "{:?}", result.err().map(|error| { return error.to_string(); }));
}

/// Round 7 (6b 1),
/// kept under round 8:
/// a kept directory reached through a symbolic link is made durable through the directory it really is,
/// not through the link's spelling.
#[test]
fn a_kept_directory_behind_a_symbolic_link_is_synced_by_its_resolved_path() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let real = root.path().join("real");
    fs::create_dir_all(real.join("kept")).expect("the real kept directory");
    std::os::unix::fs::symlink(&real, root.path().join("link")).expect("a link to it");
    let synced: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let record = |directory: &Path| -> std::io::Result<()> { synced.borrow_mut().push(directory.to_path_buf()); return Ok(()); };

    replace_with(&path, &KeepAs::in_directory(&root.path().join("link").join("kept"), ".displaced"), &real_link, &real_sync_file, &record).expect("replaces");

    let resolved: PathBuf = fs::canonicalize(real.join("kept")).expect("resolves");
    let calls = synced.borrow();
    assert!(calls.contains(&resolved), "the directory the entry lives in was not synced: {calls:?}");
    assert!(!calls.contains(&root.path().join("link").join("kept")), "the link's spelling was synced: {calls:?}");
}

/// Review finding (4b 3):
/// a failure after the file was kept still names where it is kept.
#[test]
fn a_failure_after_the_file_was_kept_names_the_kept_path() {
    let root = repository("expected");
    let directory = root.path().join(CRATE_DIR);
    let path = directory.join("Cargo.lock");
    let failing_for_target = |dir: &Path| -> std::io::Result<()> {
        if dir == directory {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(std::io::Error::other("injected directory sync failure"));
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    };

    // Borrow `&path`: lends the value without copying or giving it away.
    let result = replace_with(&path, &KeepAs::next_to_target(".displaced"), &real_link, &real_sync_file, &failing_for_target);

    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `message: String`.
    let message: String = result.expect_err("an error").to_string();
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("Cargo.lock.displaced"), "the kept path is missing: {message}");
}

/// Review finding (4b 3):
/// restoring several manifests returns what it already did when a later one fails.
#[test]
fn restoring_manifests_returns_the_warnings_of_those_restored_before_a_failure() {
    use crate::manifest_edit::restore_manifests;
    let root = repository("lock");
    let first = root.path().join("a.toml");
    let second = root.path().join("b.toml");
    // Borrow `&first`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&first, "first rewritten").expect("a");
    // Borrow `&second`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&second, [0xff_u8, 0xfe_u8]).expect("b, not text");
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let edits = RefCell::new(vec![
        // `Some`: the present or the absent value of an `Option`.
        // `.clone()`: an independent owned copy.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: first.clone(), before: "first before".to_string(), produced: Some("first rewritten".to_string()) },
        // `None`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: second, before: "second before".to_string(), produced: None },
    ]);

    // Borrow `&edits`: lends the value without copying or giving it away.
    let (warnings, failure) = restore_manifests(&edits, &KeepAs::next_to_target(".displaced"));

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(failure.is_some());
    // Closure `|warning|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(warnings.iter().any(|warning| { return warning.contains("is kept as"); }), "{warnings:?}");
}

/// Round 6 (owner answer "Warn past a count,
/// never refuse"):
/// a kept folder past the count does not stop a new transaction;
/// the transaction carries a notice with the count,
/// the total size and the folder,
/// and nothing is deleted.
#[test]
fn a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses() {
    let root = repository("lock");
    let kept = root.path().join(crate::journal::QUARANTINE_DIR);
    fs::create_dir_all(&kept).expect("kept directory");
    for number in 0..=crate::journal::KEPT_FILES_WARNING_COUNT {
        fs::write(kept.join(format!("Cargo.lock.displaced.{number}")), "kept").expect("a kept file");
    }

    let transaction = begin(root.path(), &[snapshot("original")], false).expect("a large kept folder never refuses");

    let notice: String = transaction.kept_notice().unwrap_or_default().to_string();
    let count: usize = crate::journal::KEPT_FILES_WARNING_COUNT + 1;
    assert!(notice.contains(&format!("{count} files")), "count missing: {notice}");
    assert!(notice.contains(&format!("{} bytes", count * 4)), "total size missing: {notice}");
    assert!(notice.contains(&kept.display().to_string()), "folder missing: {notice}");
    assert_eq!(fs::read_dir(&kept).expect("list").count(), count, "a kept file was deleted");
}

/// Round 6:
/// a kept folder at or under the count adds no notice.
#[test]
fn a_kept_directory_at_the_count_adds_no_notice() {
    let root = repository("lock");
    let kept = root.path().join(crate::journal::QUARANTINE_DIR);
    fs::create_dir_all(&kept).expect("kept directory");
    fs::write(kept.join("Cargo.lock.displaced.0"), "kept").expect("a kept file");

    let transaction = begin(root.path(), &[snapshot("original")], false).expect("begin");

    assert!(transaction.kept_notice().is_none(), "{:?}", transaction.kept_notice());
}

/// Review finding (4b 3):
/// a temporary left by a run that was killed between the exchange and the kept link may be the only name of
/// a displaced file,
/// so recovery reports it and leaves it.
#[test]
fn a_leftover_temporary_of_a_killed_replacement_is_reported_and_kept() {
    let root = repository("original");
    // Borrow `&[`: lends the value without copying or giving it away.
    plant(root.path(), &[snapshot("original")], &[]);
    let leftover = root.path().join(CRATE_DIR).join(".Cargo.lock.shadow-1-2-3.tmp");
    // Borrow `&leftover`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&leftover, "somebody's displaced work").expect("a leftover");

    // Closure `|_before, _live|`: an anonymous function, like a TS arrow function.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let messages = recover(root.path(), &|_before: &[crate::crates::LockSnapshot], _live: &[crate::crates::LockSnapshot]| { return Vec::new(); });

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(leftover.exists(), "the leftover was deleted");
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages.iter().any(|message| { return message.contains(".Cargo.lock.shadow-1-2-3.tmp") && message.contains("may hold"); }), "{messages:?}");
}

/// Every name under a directory (recursively) that is a hard link of the inode.
fn names_of_inode(directory: &Path, inode: u64) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(directory).expect("lists").flatten() {
        let path: PathBuf = entry.path();
        let metadata = fs::symlink_metadata(&path).expect("examines");
        if metadata.is_dir() {
            found.extend(names_of_inode(&path, inode));
        } else if metadata.ino() == inode {
            found.push(path);
        }
    }
    return found;
}

/// Acceptance row R2:
/// the preferred kept directory refuses the link (`EMLINK`,
/// `ENOSPC`,
/// `EXDEV`,
/// `EACCES`),
/// and the protocol falls back to a name beside the target:
/// the displaced inode keeps a protected name.
#[test]
fn a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name() {
    for (label, code) in [("EMLINK", rustix::io::Errno::MLINK), ("ENOSPC", rustix::io::Errno::NOSPC), ("EXDEV", rustix::io::Errno::XDEV), ("EACCES", rustix::io::Errno::ACCESS)] {
        let root = repository("expected");
        let path = root.path().join(CRATE_DIR).join("Cargo.lock");
        let kept_dir = root.path().join("kept");
        let displaced: u64 = fs::metadata(&path).expect("examines").ino();
        // Each queued error must have been injected (certification group 3): the refusal is counted when it fires.
        let injected: Cell<u32> = Cell::new(0);
        let refuse_in_kept = |existing: &Path, new: &Path| -> std::io::Result<()> {
            if new.starts_with(&kept_dir) {
                injected.set(injected.get() + 1);
                return Err(std::io::Error::from_raw_os_error(code.raw_os_error()));
            }
            return fs::hard_link(existing, new);
        };

        // The replacement is captured at the exchange, so both original inodes can be inventoried afterwards.
        let (outcome, captured) = replace_capturing(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &refuse_in_kept, &real_sync_file, &real_sync_dir);

        assert!(outcome.is_ok(), "{label}: {:?}", outcome.err().map(|error| { return error.to_string(); }));
        assert!(injected.get() >= 1, "{label}: the refusal in the kept directory never fired, so the fallback was not exercised");
        let names: Vec<PathBuf> = names_of_inode(root.path(), displaced);
        assert!(!names.is_empty(), "{label}: the displaced inode lost its last name");
        assert!(names.iter().all(|name| { return !name.starts_with(&kept_dir); }), "{label}: {names:?}");
        let replacement: u64 = captured.unwrap_or_else(|| { panic!("{label}: the exchange was never reached"); });
        assert_ne!(replacement, displaced, "{label}: the replacement is the displaced inode");
        let replacement_names: Vec<PathBuf> = names_of_inode(root.path(), replacement);
        assert!(replacement_names.contains(&path), "{label}: the replacement inode is not at the live path: {replacement_names:?}");
    }
}

/// Acceptance row R2:
/// every link fails,
/// or a barrier fails:
/// the replacement is not reported as settled,
/// the displaced inode still has a name,
/// and the message says where it is.
#[test]
fn when_every_link_or_a_barrier_fails_the_displaced_inode_keeps_its_temporary_name_and_the_message_says_where() {
    let refuse_all = |_existing: &Path, _new: &Path| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(rustix::io::Errno::MLINK.raw_os_error())); };
    let failing_file = |_path: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected data barrier failure")); };
    let failing_dir = |_path: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected directory barrier failure")); };
    type Link<'a> = &'a dyn Fn(&Path, &Path) -> std::io::Result<()>;
    type Barrier<'a> = &'a dyn Fn(&Path) -> std::io::Result<()>;
    let cases: Vec<(&str, Link, Barrier, Barrier)> = vec![
        ("every link fails", &refuse_all, &real_sync_file, &real_sync_dir),
        ("the data barrier fails", &real_link, &failing_file, &real_sync_dir),
        ("the directory barrier fails", &real_link, &real_sync_file, &failing_dir),
    ];
    for (label, link, sync_file, sync_dir) in cases {
        let root = repository("expected");
        let path = root.path().join(CRATE_DIR).join("Cargo.lock");
        let displaced: u64 = fs::metadata(&path).expect("examines").ino();

        let (outcome, captured) = replace_capturing(&path, &KeepAs::next_to_target(".displaced"), link, sync_file, sync_dir);
        let replacement: u64 = captured.unwrap_or_else(|| { panic!("{label}: the exchange was never reached, so the replacement inode was not captured"); });

        // Every link failing is an outcome (the file stays under its temporary name and the path is returned); a failing barrier is an error.
        let message: String = match outcome {
            Ok(found) => {
                let shown: String = format!("{found:?}");
                assert!(shown.starts_with("Displaced("), "{label}: reported as settled: {shown}");
                shown
            }
            Err(error) => error.to_string(),
        };
        let names: Vec<PathBuf> = names_of_inode(root.path(), displaced);
        assert!(!names.is_empty(), "{label}: the displaced inode lost its last name");
        // The diagnostic names a pathname under which the displaced inode really survives, not a word.
        assert!(names.iter().any(|name| { return message.contains(&name.display().to_string()); }), "{label}: the message does not name a surviving path of the file ({names:?}): {message}");
        // The replacement inode captured at the exchange keeps a name too (R2: neither inode loses its last name), and it is a different
        // inode from the displaced one, so the live path cannot hold some other file while both assertions pass.
        assert_ne!(replacement, displaced, "{label}: the replacement is the displaced inode");
        let replacement_names: Vec<PathBuf> = names_of_inode(root.path(), replacement);
        assert!(!replacement_names.is_empty(), "{label}: the replacement inode lost its last name");
        assert!(replacement_names.contains(&path), "{label}: the replacement is not at the live path: {replacement_names:?}");
    }
}

/// Acceptance row R2 (stages and read-only transition):
/// the exchange itself fails,
/// or the file system turns read-only (`EROFS`) at the link:
/// the displaced inode keeps a name,
/// the live file is not lost,
/// and nothing is reported as settled.
#[test]
fn a_failing_exchange_or_a_read_only_link_keeps_both_inodes_named() {
    // The exchange fails: nothing moved, so the original is still the live file and the replacement text is nowhere.
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let original: u64 = fs::metadata(&path).expect("examines").ino();
    let none = |_path: &Path| {};
    let failing_swap = |_first: &Path, _second: &Path| -> std::io::Result<()> { return Err(std::io::Error::other("injected exchange failure")); };
    let outcome = replace_with_seams(&path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &Seams { between: &none, after_exchange: &none, swap: &failing_swap, link: &real_link, sync_file: &real_sync_file, sync_dir: &real_sync_dir });
    assert!(outcome.is_err(), "a failed exchange was reported as settled");
    assert_eq!(fs::read_to_string(&path).expect("the live file"), "expected", "the live file changed although the exchange failed");
    assert!(names_of_inode(root.path(), original).contains(&path), "the original lost its name");

    // The file system turns read-only at every link: the displaced file stays under its temporary name, and the live file has the
    // replacement text.
    let other = repository("expected");
    let other_path = other.path().join(CRATE_DIR).join("Cargo.lock");
    let displaced: u64 = fs::metadata(&other_path).expect("examines").ino();
    let read_only = |_existing: &Path, _new: &Path| -> std::io::Result<()> { return Err(std::io::Error::from_raw_os_error(rustix::io::Errno::ROFS.raw_os_error())); };
    let swap = |first: &Path, second: &Path| -> std::io::Result<()> { return crate::journal_replace::exchange_paths(first, second); };
    let settled = replace_with_seams(&other_path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &Seams { between: &none, after_exchange: &none, swap: &swap, link: &read_only, sync_file: &real_sync_file, sync_dir: &real_sync_dir });
    let shown: String = format!("{settled:?}");
    assert!(shown.contains("Displaced"), "a read-only file system was reported as settled: {shown}");
    assert!(!names_of_inode(other.path(), displaced).is_empty(), "the displaced inode lost its last name");
    assert_eq!(fs::read_to_string(&other_path).expect("the live file"), "new text", "the replacement is not live");
    // The replacement inode is the one captured at the exchange (not read from the live path afterwards, which would only show that
    // whatever is live is named where it is): a faulty cleanup that replaced it with a fresh inode of the same text would pass the
    // latter and fail this (certification, second pass).
    let captured: Cell<Option<u64>> = Cell::new(None);
    let capture = |live: &Path| { captured.set(Some(fs::metadata(live).expect("examines the live path at the exchange").ino())); };
    let third = repository("expected");
    let third_path = third.path().join(CRATE_DIR).join("Cargo.lock");
    let third_displaced: u64 = fs::metadata(&third_path).expect("examines").ino();
    let _ = replace_with_seams(&third_path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &Seams { between: &none, after_exchange: &capture, swap: &swap, link: &read_only, sync_file: &real_sync_file, sync_dir: &real_sync_dir });
    let replacement: u64 = captured.get().expect("the exchange was reached");
    assert_ne!(replacement, third_displaced, "the exchange put the displaced inode back");
    assert_eq!(fs::metadata(&third_path).expect("examines").ino(), replacement, "the live path holds another inode than the one exchanged in");
    assert!(names_of_inode(third.path(), replacement).contains(&third_path), "the replacement inode that was live lost its name");
    assert!(!names_of_inode(third.path(), third_displaced).is_empty(), "the displaced inode lost its last name");
}

/// Acceptance row R2 (relevant rename failure after a conflict):
/// another writer changes the target between the comparison and the exchange,
/// the exchange succeeds,
/// and the undo exchange fails:
/// the other writer's inode (now at the temporary name) and the replacement (live) both keep a name,
/// and the outcome names where the other writer's file stays.
#[test]
fn a_conflict_whose_undo_exchange_fails_keeps_the_other_writers_inode_named_and_reports_where() {
    let root = repository("expected");
    let path = root.path().join(CRATE_DIR).join("Cargo.lock");
    let writers: u64 = fs::metadata(&path).expect("examines").ino();
    // Another writer changes the file in place (same inode) after the first comparison.
    let other_writer = |live: &Path| { fs::write(live, "another writer's text").expect("the other writer writes"); };
    let captured: Cell<Option<u64>> = Cell::new(None);
    let capture = |live: &Path| { captured.set(Some(fs::metadata(live).expect("examines the live path at the exchange").ino())); };
    let calls: Cell<usize> = Cell::new(0);
    // The first exchange is real; the undo exchange (the second call) fails.
    let swap = |first: &Path, second: &Path| -> std::io::Result<()> {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            return crate::journal_replace::exchange_paths(first, second);
        }
        return Err(std::io::Error::other("injected undo failure"));
    };

    let outcome = replace_with_seams(&path, "expected", "new text", &KeepAs::next_to_target(".displaced"), &Seams { between: &other_writer, after_exchange: &capture, swap: &swap, link: &real_link, sync_file: &real_sync_file, sync_dir: &real_sync_dir }).expect("a failed undo is an outcome, not an error");

    assert_eq!(calls.get(), 2, "the undo exchange was not attempted");
    let ReplaceOutcome::Displaced(kept) = outcome else {
        panic!("expected the other writer's file reported as displaced, got {outcome:?}");
    };
    let replacement: u64 = captured.get().expect("the exchange was reached");
    assert_ne!(replacement, writers);
    assert_eq!(fs::read_to_string(&path).expect("the live file"), "new text", "the replacement is not live");
    assert_eq!(fs::metadata(&path).expect("examines").ino(), replacement);
    assert_eq!(fs::metadata(&kept).expect("the reported path exists").ino(), writers, "the reported path is not the other writer's inode");
    assert_eq!(fs::read_to_string(&kept).expect("reads the kept file"), "another writer's text", "the other writer's text was not kept");
    assert!(names_of_inode(root.path(), writers).contains(&kept), "the other writer's inode does not survive under the reported name");
    assert!(names_of_inode(root.path(), replacement).contains(&path), "the replacement inode lost its name");
}

/// Acceptance row K2 (each barrier of the creation chain):
/// every directory barrier of a preservation into a new nested kept directory,
/// failed one at a time:
/// the displaced inode keeps a protected name each time,
/// and preservation is reported only with the copy beside the target,
/// never through the directories whose barrier failed.
#[test]
fn failing_each_directory_barrier_of_a_preservation_in_turn_never_loses_the_name_or_reports_the_unsynced_chain() {
    let counted = {
        let root = repository("expected");
        let path = root.path().join(CRATE_DIR).join("Cargo.lock");
        let kept_dir = root.path().join("a").join("b").join("c").join("d");
        fs::create_dir_all(root.path().join("a")).expect("the pre-existing anchor");
        let calls: RefCell<usize> = RefCell::new(0);
        let count = |_directory: &Path| -> std::io::Result<()> { *calls.borrow_mut() += 1; return Ok(()); };
        replace_with(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &real_link, &real_sync_file, &count).expect("replaces");
        let total: usize = *calls.borrow();
        total
    };
    // Independent of the implementation's own count: the chain makes three new directories (b, c, d under the existing a), and each is
    // synchronized itself and listed by a synchronized parent, before the kept file is linked and synchronized.
    let new_directories: usize = 3;
    assert!(counted >= 2 * new_directories, "the chain should have a barrier for each new directory and for its parent: {counted}");
    let mut reported_preserved: usize = 0;
    for failing in 0..counted {
        let root = repository("expected");
        let path = root.path().join(CRATE_DIR).join("Cargo.lock");
        let kept_dir = root.path().join("a").join("b").join("c").join("d");
        fs::create_dir_all(root.path().join("a")).expect("the pre-existing anchor");
        let displaced: u64 = fs::metadata(&path).expect("examines").ino();
        let seen: RefCell<usize> = RefCell::new(0);
        // One ordered log of what happened: each successful link (its new name) and each directory barrier (its directory, and
        // whether it held), so a barrier can be required to come after the publication it protects (11b 2.2).
        let events: RefCell<Vec<Event>> = RefCell::new(Vec::new());
        let fail_nth = |directory: &Path| -> std::io::Result<()> {
            let index: usize = *seen.borrow();
            *seen.borrow_mut() += 1;
            if index == failing {
                events.borrow_mut().push(Event::Barrier(directory.to_path_buf(), false));
                return Err(std::io::Error::other("injected barrier failure"));
            }
            events.borrow_mut().push(Event::Barrier(directory.to_path_buf(), true));
            return Ok(());
        };
        let logging_link = |existing: &Path, new: &Path| -> std::io::Result<()> {
            real_link(existing, new)?;
            events.borrow_mut().push(Event::Linked(new.to_path_buf()));
            return Ok(());
        };

        let (outcome, captured) = replace_capturing(&path, &KeepAs::in_directory(&kept_dir, ".displaced"), &logging_link, &real_sync_file, &fail_nth);

        let names: Vec<PathBuf> = names_of_inode(root.path(), displaced);
        assert!(!names.is_empty(), "barrier {failing}: the displaced inode lost its last name");
        // Both inodes are inventoried after every failure (11b 2.2): the replacement stays at the live path.
        let replacement: u64 = captured.unwrap_or_else(|| { panic!("barrier {failing}: the exchange was never reached"); });
        assert!(names_of_inode(root.path(), replacement).contains(&path), "barrier {failing}: the replacement is not at the live path");
        if outcome.is_ok() {
            reported_preserved += 1;
            // Reported as preserved: the protected copy is the fallback beside the target (a `.displaced` name next to the lockfile,
            // not an exchange temporary and not a name in the new chain), and a barrier of that directory succeeded after the link
            // that published that name.
            let crate_dir: PathBuf = path.parent().expect("a parent").to_path_buf();
            let protected: Vec<&PathBuf> = names.iter().filter(|name| { return name.parent() == Some(crate_dir.as_path()) && name.file_name().is_some_and(|file| { return file.to_string_lossy().contains(".displaced"); }); }).collect();
            assert!(!protected.is_empty(), "barrier {failing}: reported as preserved without a protected name beside the target: {names:?}");
            let log = events.borrow();
            let linked_at: usize = log.iter().position(|event| { return matches!(event, Event::Linked(name) if protected.contains(&name)); }).unwrap_or_else(|| { panic!("barrier {failing}: the protected name was not published by a link: {log:?}"); });
            assert!(log[linked_at..].iter().any(|event| { return matches!(event, Event::Barrier(directory, true) if *directory == crate_dir); }), "barrier {failing}: no successful barrier of the target directory came after the link of the protected name: {log:?}");
        }
    }
    // The branch that checks the protected copy must have been exercised by at least one barrier of the chain.
    assert!(reported_preserved > 0, "no failed barrier ended in a reported preservation, so the fallback was never checked");
}

/// Acceptance rows A3 and A4 (certification,
/// second pass):
/// the whole order of finishing is observed in one ordered trace:
/// the output barriers,
/// the completion rename,
/// the successful barrier of the directory that lists the marker,
/// and only then the
/// removal (the cleanup).
/// A failure at each step stops the trace at that step,
/// with the failure kind the contract names:
/// the output barrier,
/// the rename and
/// the marker's directory barrier are mandatory failures that attempt no cleanup,
/// and a failed cleanup is cleanup-only and comes
/// after the successful barrier.
#[test]
fn finishing_is_observed_as_outputs_then_marker_then_marker_barrier_then_removal() {
    use crate::failpoints::{Point, arm};
    let outputs = ["output Cargo.lock", "output Cargo.toml"];
    let marker = ["marker renamed", "marker parent synced"];
    let cleanup = "cleanup attempted";
    // (fault, the events that must be seen, the failure is cleanup-only; None: the finish succeeds)
    let cases: Vec<(Option<Point>, Vec<&str>, Option<bool>)> = vec![
        (None, [outputs.to_vec(), marker.to_vec(), vec![cleanup]].concat(), None),
        (Some(Point::FinishDurability), vec![], Some(false)),
        (Some(Point::CompletionRename), outputs.to_vec(), Some(false)),
        (Some(Point::CompletionSync), [outputs.to_vec(), vec![marker[0]]].concat(), Some(false)),
        (Some(Point::FinishCleanup), [outputs.to_vec(), marker.to_vec(), vec![cleanup]].concat(), Some(true)),
    ];
    for (fault, expected, failure) in cases {
        let root = repository("final output");
        let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
        fs::write(&manifest, "final manifest").expect("manifest");
        let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin");
        transaction.add_manifests(&[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "before".to_string(), produced: None }]).expect("manifests");
        if let Some(point) = fault {
            arm(point);
        }
        let _ = crate::bounded_run::take_trace_for_test();
        let record = |path: &Path| {
            let name: String = path.file_name().map(|found| { return found.to_string_lossy().into_owned(); }).unwrap_or_default();
            crate::bounded_run::trace(format_args!("output {name}"));
            return Ok(());
        };

        let outcome = transaction.finish_with(&record);

        let seen: Vec<String> = crate::bounded_run::take_trace_for_test();
        assert_eq!(seen, expected, "{fault:?}: the finishing steps are not in the order outputs, marker, marker barrier, removal");
        if let Some(point) = fault {
            assert!(!crate::failpoints::fires(point), "{point:?}: the fault was never reached");
        }
        match (outcome, failure) {
            (Ok(()), None) => {}
            (Err(error), Some(cleanup_only)) => assert_eq!(error.cleanup_only(), cleanup_only, "{fault:?}: {error}"),
            (unexpected, _) => panic!("{fault:?}: the finish ended as {unexpected:?}"),
        }
    }
}

/// Acceptance row A4 (certification,
/// third pass):
/// the ordinary `finish()` makes the real outputs durable (the final lockfile and the
/// journaled manifest,
/// through `make_durable`) before the marker is renamed,
/// the marker's directory is synchronized,
/// and the journal
/// is removed.
/// The trace is produced by the production barrier itself,
/// not by a callback of the test.
#[test]
fn the_ordinary_finish_makes_the_lockfile_and_the_manifest_durable_before_the_marker() {
    let root = repository("final output");
    let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
    fs::write(&manifest, "final manifest").expect("manifest");
    let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin");
    transaction.add_manifests(&[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "before".to_string(), produced: None }]).expect("manifests");
    let _ = crate::bounded_run::take_trace_for_test();

    transaction.finish().expect("finish");

    let seen: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("durable ") || event.starts_with("marker ") || event.starts_with("cleanup "); }).collect();
    let expected: Vec<&str> = vec!["durable Cargo.lock", "durable Cargo.toml", "marker renamed", "marker parent synced", "cleanup attempted"];
    assert_eq!(seen, expected, "the ordinary finish did not make both outputs durable, in order, before the marker");
}

/// Acceptance row A4 (certification,
/// fourth pass):
/// the real output-file barrier and the real output-directory barrier of the
/// ordinary `finish()` are each failed on their own.
/// The fault is consumed at the production operation,
/// the failure is a mandatory
/// failure of the output phase,
/// no marker is renamed and no cleanup is attempted,
/// and the journal stays active.
/// The control without a
/// fault reaches the marker,
/// so the stop is the fault's.
#[test]
fn a_failed_output_file_or_directory_sync_of_the_ordinary_finish_is_mandatory_and_publishes_no_marker() {
    use crate::failpoints::{Point, arm, fires};
    for fault in [Point::OutputFileSync, Point::OutputDirectorySync] {
        let root = repository("final output");
        let manifest = root.path().join(CRATE_DIR).join("Cargo.toml");
        fs::write(&manifest, "final manifest").expect("manifest");
        let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin");
        transaction.add_manifests(&[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "before".to_string(), produced: None }]).expect("manifests");
        arm(fault);
        let _ = crate::bounded_run::take_trace_for_test();

        let outcome = transaction.finish();

        assert!(!fires(fault), "{fault:?}: the fault was never reached");
        let error = outcome.expect_err("a failed output barrier must fail the finish");
        assert!(!error.cleanup_only(), "{fault:?}: a failed output barrier was read as a cleanup: {error}");
        let seen: Vec<String> = crate::bounded_run::take_trace_for_test();
        assert!(!seen.iter().any(|event| { return event.starts_with("marker ") || event.starts_with("cleanup "); }), "{fault:?}: the marker or the cleanup went on after a failed output barrier: {seen:?}");
        let journals = crate::journal_support::journal_dirs(root.path());
        assert_eq!(journals.len(), 1, "{fault:?}");
        assert!(!journals[0].to_string_lossy().ends_with(".done"), "{fault:?}: the journal was marked done");
    }
}

/// The environment variable that gives the syscall helper the file it makes durable.
const FIXTURE_DURABLE_FILE: &str = "CARGO_SHADOW_FIXTURE_DURABLE_FILE";

/// The helper process of the syscall test:
/// it makes one file durable through the production barrier and exits.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of the_output_barrier_syncs_the_file_and_then_its_directory_as_the_kernel_sees_it"]
fn make_durable_fixture() {
    let Ok(file) = std::env::var(FIXTURE_DURABLE_FILE) else {
        return;
    };
    crate::journal::make_durable(Path::new(&file)).expect("makes the file durable");
}

/// Acceptance rows A4 and V5 (certification,
/// fourth pass):
/// the output barrier is observed at the system call.
/// No in-process
/// instrumentation can show that a regression kept its traces and failpoints and dropped the `fsync`,
/// so a helper process runs
/// `make_durable` under `strace` and the test reads the calls the kernel was given:
/// `fsync` of the file,
/// then `fsync` of the directory
/// that lists it,
/// in that order.
/// A host without `strace` cannot show this;
/// the test says so and stops (this host has it).
#[test]
fn the_output_barrier_syncs_the_file_and_then_its_directory_as_the_kernel_sees_it() {
    let found = std::process::Command::new("strace").arg("--version").output();
    if found.is_err() {
        crate::test_scratch::skip_or_fail("strace is not installed, so the system calls of the barrier cannot be observed here");
        return;
    }
    let root = repository("final output");
    let file: PathBuf = root.path().join(CRATE_DIR).join("Cargo.lock");
    let log: PathBuf = root.path().join("strace.log");

    let status = std::process::Command::new("strace")
        .args(["-f", "-y", "-e", "trace=fsync,fdatasync", "-o"])
        .arg(&log)
        .arg(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "journal_protocol_tests::make_durable_fixture", "--nocapture"])
        .env(FIXTURE_DURABLE_FILE, &file)
        .status()
        .expect("strace runs the helper");

    assert!(status.success(), "the helper or strace failed: {status:?}");
    let text: String = fs::read_to_string(&log).expect("the strace log");
    let calls: Vec<&str> = text.lines().filter(|line| { return line.contains("fsync(") || line.contains("fdatasync("); }).collect();
    let file_at: Option<usize> = calls.iter().position(|line| { return line.contains(&format!("<{}>", file.display())) && line.contains("= 0"); });
    let directory_at: Option<usize> = calls.iter().position(|line| { return line.contains(&format!("<{}>", file.parent().expect("a parent").display())) && line.contains("= 0"); });
    let (Some(first), Some(second)) = (file_at, directory_at) else {
        panic!("the kernel was not given an fsync of the file and of its directory: {calls:?}");
    };
    assert!(first < second, "the directory was synchronized before the file: {calls:?}");
}

/// Acceptance rows A4 and V5 (sixth pass,
/// group 5):
/// the manifest's own output barrier is failed on its own,
/// with a fault that names that file,
/// while the lockfile's barrier is left to succeed.
/// The finish stops as a mandatory failure of the output phase,
/// no marker is renamed,
/// no cleanup is attempted,
/// and the journal stays active.
/// A regression that stops propagating the manifest's barrier error still reaches the
/// marker (the lockfile's barrier is untouched) and turns this red.
#[test]
fn a_manifest_barrier_that_fails_on_its_own_stops_the_ordinary_finish() {
    for directory_barrier in [false, true] {
        let root = repository("final output");
        let manifest: PathBuf = root.path().join(CRATE_DIR).join("Cargo.toml");
        fs::write(&manifest, "final manifest").expect("manifest");
        let transaction = begin(root.path(), &[snapshot("original")], true).expect("begin");
        transaction.add_manifests(&[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "before".to_string(), produced: None }]).expect("manifests");
        if directory_barrier {
            crate::failpoints::arm_output_directory_sync_of(&manifest);
        } else {
            crate::failpoints::arm_output_file_sync_of(&manifest);
        }
        let _ = crate::bounded_run::take_trace_for_test();

        let outcome = transaction.finish();

        let (file_armed, directory_armed) = crate::failpoints::path_targets_armed();
        assert!(!file_armed && !directory_armed, "directory={directory_barrier}: the manifest's barrier was never reached");
        let error = outcome.expect_err("a failed manifest barrier must fail the finish");
        assert!(!error.cleanup_only(), "directory={directory_barrier}: the failure was read as a cleanup: {error}");
        let seen: Vec<String> = crate::bounded_run::take_trace_for_test();
        assert!(seen.iter().any(|event| { return event == "durable Cargo.lock"; }), "directory={directory_barrier}: the lockfile barrier did not succeed first, so the schedule is not the manifest's alone: {seen:?}");
        assert!(!seen.iter().any(|event| { return event.starts_with("marker ") || event.starts_with("cleanup "); }), "directory={directory_barrier}: the marker or the cleanup went on after a failed manifest barrier: {seen:?}");
        let journals: Vec<PathBuf> = crate::journal_support::journal_dirs(root.path());
        assert_eq!(journals.len(), 1, "directory={directory_barrier}");
        assert!(!journals[0].to_string_lossy().ends_with(".done"), "directory={directory_barrier}: the journal was marked done");

        // The control: the same schedule without the fault reaches the marker, so the stop above is the fault's.
        let control = repository("final output");
        let control_manifest: PathBuf = control.path().join(CRATE_DIR).join("Cargo.toml");
        fs::write(&control_manifest, "final manifest").expect("manifest");
        let control_transaction = begin(control.path(), &[snapshot("original")], true).expect("begin");
        control_transaction.add_manifests(&[ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: control_manifest.clone(), before: "before".to_string(), produced: None }]).expect("manifests");
        let _ = crate::bounded_run::take_trace_for_test();
        control_transaction.finish().expect("the control finishes");
        let control_seen: Vec<String> = crate::bounded_run::take_trace_for_test();
        assert!(control_seen.iter().any(|event| { return event == "durable Cargo.toml"; }) && control_seen.iter().any(|event| { return event == "marker renamed"; }), "the control did not reach the manifest barrier and the marker: {control_seen:?}");
    }
}

/// Acceptance row V5 (sixth pass,
/// group 5):
/// a recovery's manifest barrier is failed on its own,
/// with a fault that names that file:
/// the recovery is a mandatory failure,
/// names the manifest,
/// keeps the journal,
/// and the settled lockfile and manifest stay where the recovery put them.
/// A regression that skips the manifest's barrier keeps the journal's state unchanged here and spends no
/// targeted fault,
/// so the reached assertion fails.
#[test]
fn a_recovery_manifest_barrier_that_fails_on_its_own_is_mandatory_and_keeps_the_journal() {
    for directory_barrier in [false, true] {
        let root = repository("unchecked output of the killed run");
        let manifest: PathBuf = root.path().join(CRATE_DIR).join("Cargo.toml");
        fs::write(&manifest, "edited manifest of the killed run").expect("manifest");
        let manifests: Vec<ManifestEdit> = vec![ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: manifest.clone(), before: "manifest before".to_string(), produced: None }];
        let journal: PathBuf = plant(root.path(), &[snapshot("original")], &manifests);
        if directory_barrier {
            crate::failpoints::arm_output_directory_sync_of(&manifest);
        } else {
            crate::failpoints::arm_output_file_sync_of(&manifest);
        }

        let report = crate::journal_recover::recover_report(root.path(), &|_before: &[LockSnapshot], _live: &[LockSnapshot]| { return vec!["dep@1.0.1 refused".to_string()]; });

        let (file_armed, directory_armed) = crate::failpoints::path_targets_armed();
        assert!(!file_armed && !directory_armed, "directory={directory_barrier}: the manifest's barrier was never reached: {report:?}");
        assert!(report.mandatory_failure, "directory={directory_barrier}: a failed manifest barrier was not mandatory: {report:?}");
        assert!(report.messages.iter().any(|message| { return message.contains(&manifest.display().to_string()) && message.contains("could not be synced"); }), "directory={directory_barrier}: no message names the manifest's barrier: {report:?}");
        assert_eq!(crate::journal_support::journal_dirs(root.path()), vec![journal], "directory={directory_barrier}: the journal was not kept as it is");
        assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), "original", "directory={directory_barrier}: the settled lockfile did not stay");
        assert_eq!(fs::read_to_string(&manifest).expect("manifest"), "manifest before", "directory={directory_barrier}: the settled manifest did not stay");
    }
}

/// Acceptance row V5 (sixth pass,
/// group 5,
/// syscall observation):
/// the directory that lists the completion marker is synchronized by the kernel after the rename that publishes it,
/// and before the journal is removed.
/// The helper process runs the production `finish_journal` under `strace`, so no in-process
/// trace can stand in for the barrier: a regression that keeps the failpoint and the trace event but drops the
/// `sync_directory` call removes the fsync that this test requires between the rename and the removal.
#[test]
fn the_completion_parent_barrier_is_synchronized_by_the_kernel_before_the_journal_is_removed() {
    let found = std::process::Command::new("strace").arg("--version").output();
    if found.is_err() {
        crate::test_scratch::skip_or_fail("strace is not installed, so the system calls of the completion barrier cannot be observed here");
        return;
    }
    let root = repository("final output");
    let journals: PathBuf = root.path().join(".cache/cargo-shadow/journal");
    let journal: PathBuf = journals.join("completion");
    fs::create_dir_all(&journal).expect("a journal directory");
    let log: PathBuf = root.path().join("strace-completion.log");

    let status = std::process::Command::new("strace")
        .args(["-f", "-y", "-e", "trace=rename,renameat,renameat2,unlinkat,rmdir,fsync,fdatasync", "-o"])
        .arg(&log)
        .arg(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "journal_protocol_tests::completion_parent_barrier_fixture", "--nocapture"])
        .env(FIXTURE_COMPLETION_JOURNAL, &journal)
        .status()
        .expect("strace runs the helper");

    assert!(status.success(), "the helper or strace failed: {status:?}");
    let text: String = fs::read_to_string(&log).expect("the strace log");
    let calls: Vec<&str> = text.lines().filter(|line| { return line.contains("rename(") || line.contains("renameat") || line.contains("unlinkat(") || line.contains("rmdir(") || line.contains("fsync(") || line.contains("fdatasync("); }).collect();
    let renamed_at: Option<usize> = calls.iter().position(|line| { return line.contains(&format!("{}.done", journal.display())) && !line.contains("= -1"); });
    let Some(renamed_at) = renamed_at else {
        panic!("the completion marker was never renamed: {calls:?}");
    };
    let parent: String = format!("<{}>", journals.display());
    let removed_at: Option<usize> = calls.iter().position(|line| { return (line.contains("unlinkat(") || line.contains("rmdir(")) && line.contains(&format!("{}.done", journal.display())); });
    let synced_at: Option<usize> = calls.iter().enumerate().skip(renamed_at + 1).find(|(_, line)| { return (line.contains("fsync(") || line.contains("fdatasync(")) && line.contains(&parent) && line.contains("= 0"); }).map(|(index, _)| { return index; });
    let Some(synced_at) = synced_at else {
        panic!("no fsync of the directory that lists the marker came after the rename: {calls:?}");
    };
    assert!(removed_at.is_none_or(|removal| { return synced_at < removal; }), "the journal was removed before its marker's directory was synchronized: {calls:?}");
}

/// The environment variable that gives the completion-barrier helper its journal.
const FIXTURE_COMPLETION_JOURNAL: &str = "CARGO_SHADOW_FIXTURE_COMPLETION_JOURNAL";

/// The helper process of the completion-parent barrier test:
/// it finishes one journal through the production path and exits.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of the_completion_parent_barrier_is_synchronized_by_the_kernel_before_the_journal_is_removed"]
fn completion_parent_barrier_fixture() {
    let Ok(journal) = std::env::var(FIXTURE_COMPLETION_JOURNAL) else {
        return;
    };
    crate::journal_transaction::finish_journal(Path::new(&journal)).expect("finishes the journal");
}
