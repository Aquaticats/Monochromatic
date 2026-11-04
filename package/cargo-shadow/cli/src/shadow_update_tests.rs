//! The unit tests of the stand-in lockfile that a full update of a root without a lockfile uses.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fs;

// Imports `crate::shadow_update::run_with_placeholder_...`.
use crate::shadow_update::run_with_placeholder_lock;
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

const REAL_LOCK: &str = "version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\n";

/// Review finding (dry run):
/// a dry run restores the stand-in and succeeds,
/// and must leave no lockfile behind.
#[test]
fn a_successful_dry_run_leaves_no_lockfile_behind() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");

    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let code: i32 = run_with_placeholder_lock(&lock, &root.path().join("kept"), true, &mut || { return 0; });

    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert_eq!(code, 0);
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!lock.exists(), "the stand-in of a dry run was left behind");
}

/// A real update keeps the lockfile it produced.
#[test]
fn a_successful_update_keeps_the_lockfile_it_produced() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    let written = lock.clone();

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock(&lock, &root.path().join("kept"), false, &mut || {
        // Borrow `&written`: lends the value without copying or giving it away.
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        fs::write(&written, REAL_LOCK).expect("cargo writes the lock");
        return 0;
    });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 0);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&lock).expect("read"), REAL_LOCK);
}

/// A refused update removes its own stand-in.
#[test]
fn a_refused_update_removes_its_own_stand_in() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock(&lock, &root.path().join("kept"), false, &mut || { return 1; });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 1);
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!lock.exists());
}

/// Review finding (cleanup deletes another writer's file):
/// another writer publishes a lockfile while the update runs,
/// and
/// the update then fails.
/// The cleanup removes only the stand-in it wrote.
#[test]
fn a_refused_update_does_not_delete_a_lockfile_another_writer_published() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    // `.clone()`: an independent owned copy.
    let theirs = lock.clone();

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock(&lock, &root.path().join("kept"), false, &mut || {
        // Borrow `&theirs`: lends the value without copying or giving it away.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(&theirs, REAL_LOCK).expect("the other writer");
        return 1;
    });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 1);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&lock).expect("the other writer's lock survives"), REAL_LOCK);
}

/// Review finding (truncation):
/// a lockfile that already exists is never replaced by the stand-in.
#[test]
fn an_existing_lockfile_is_never_touched() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&lock, REAL_LOCK).expect("an existing lock");

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock(&lock, &root.path().join("kept"), true, &mut || { return 0; });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 0);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&lock).expect("read"), REAL_LOCK);
}

/// Review finding (2c 5):
/// the cleanup reads the stand-in and then removes the path.
/// A real lockfile published between the two must
/// survive.
#[test]
fn a_lockfile_published_between_the_cleanup_decision_and_the_removal_survives() {
    use crate::shadow_update::run_with_placeholder_lock_hooked;
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let publish = |path: &std::path::Path| { fs::write(path, REAL_LOCK).expect("another writer publishes a real lockfile"); };

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock_hooked(&lock, &root.path().join("kept"), false, &mut || { return 1; }, &publish);

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 1);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&lock).expect("the real lockfile must survive"), REAL_LOCK);
}

/// Review finding (3c P1-3):
/// a writer that holds the stand-in open when it is cleaned up keeps what it writes afterwards;
/// the stand-in is kept under a name,
/// never unlinked while it can still be written.
#[test]
fn a_writer_that_holds_the_stand_in_open_keeps_what_it_writes_afterwards() {
    use crate::shadow_update::run_with_placeholder_lock_hooked;
    use std::io::Write;
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // Typed binding: `held: std::cell::RefCell<Option<fs::File>>`.
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
    let held: std::cell::RefCell<Option<fs::File>> = std::cell::RefCell::new(None);
    // `Some`: the present or the absent value of an `Option`.
    // Closure `|path: &std::path::Path|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let open = |path: &std::path::Path| { *held.borrow_mut() = Some(fs::OpenOptions::new().write(true).open(path).expect("open the stand-in")); };

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock_hooked(&lock, &root.path().join("kept"), false, &mut || { return 1; }, &open);
    {
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        let mut slot = held.borrow_mut();
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        let file = slot.as_mut().expect("held");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        file.set_len(0).expect("truncate");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        file.write_all(REAL_LOCK.as_bytes()).expect("the late write");
    }

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 1);
    // `let mut`: a binding that may change.
    // Typed binding: `found: bool`.
    let mut found: bool = false;
    for directory in [root.path().to_path_buf(), root.path().join("kept")] {
        // Borrow `&directory`: lends the value without copying or giving it away.
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        for entry in fs::read_dir(&directory).into_iter().flatten().flatten() {
            // Closure `|text|`: an anonymous function, like a TS arrow function.
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
            // Why:      They express one step of a pipeline without an index loop.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // list.map(f).filter(g)
            // ```
            if fs::read_to_string(entry.path()).map(|text| { return text == REAL_LOCK; }).unwrap_or(false) {
                found = true;
            }
        }
    }
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(found, "the late write has no name left");
}

/// Review finding (4c F3):
/// a real lockfile that cleanup puts back is also given a name in the kept directory,
/// so a writer that holds
/// its inode and writes after another writer replaced the live path still has a named file.
#[test]
fn a_real_lockfile_that_cleanup_puts_back_is_also_kept_by_name() {
    use crate::shadow_update::run_with_placeholder_lock_hooked;
    use std::io::Write;
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    // `None`: the present or the absent value of an `Option`.
    // Typed binding: `held: std::cell::RefCell<Option<fs::File>>`.
    // Constructor `RefCell::new(`: an empty owned collection or a boxed value.
    let held: std::cell::RefCell<Option<fs::File>> = std::cell::RefCell::new(None);
    // Closure `|path: &std::path::Path|`: an anonymous function, like a TS arrow function.
    let publish_and_hold = |path: &std::path::Path| {
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // `let mut`: a binding that may change.
        let mut file = fs::OpenOptions::new().write(true).open(path).expect("open the stand-in");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        file.set_len(0).expect("truncate");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        file.write_all(REAL_LOCK.as_bytes()).expect("a real lockfile written through the held descriptor");
        // `Some`: the present or the absent value of an `Option`.
        *held.borrow_mut() = Some(file);
    };

    // Borrow `&lock`: lends the value without copying or giving it away.
    // Typed binding: `code: i32`.
    let code: i32 = run_with_placeholder_lock_hooked(&lock, &root.path().join("kept"), false, &mut || { return 1; }, &publish_and_hold);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(&lock).expect("another writer replaces the live path");
    {
        // `let mut`: a binding that may change.
        let mut slot = held.borrow_mut();
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        let file = slot.as_mut().expect("held");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        file.write_all(b"\nlate").expect("the late write");
    }

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 1);
    // Typed binding: `late: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let late: String = format!("{REAL_LOCK}\nlate");
    // `let mut`: a binding that may change.
    // Typed binding: `found: bool`.
    let mut found: bool = false;
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    for entry in fs::read_dir(root.path().join("kept")).into_iter().flatten().flatten() {
        // Closure `|text|`: an anonymous function, like a TS arrow function.
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Adapter `.map(`: transforms or selects items with a closure.
        if fs::read_to_string(entry.path()).map(|text| { return text == late; }).unwrap_or(false) {
            found = true;
        }
    }
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(found, "the late write has no name left in the kept directory");
}
