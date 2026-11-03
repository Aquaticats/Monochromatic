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
    let code: i32 = run_with_placeholder_lock(&lock, true, &mut || { return 0; });

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
    let code: i32 = run_with_placeholder_lock(&lock, false, &mut || {
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
    let code: i32 = run_with_placeholder_lock(&lock, false, &mut || { return 1; });

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
    let code: i32 = run_with_placeholder_lock(&lock, false, &mut || {
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
    let code: i32 = run_with_placeholder_lock(&lock, true, &mut || { return 0; });

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(code, 0);
    // Borrow `&lock`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&lock).expect("read"), REAL_LOCK);
}

/// Review finding (2c 5): the cleanup reads the stand-in and then removes the path. A real lockfile published between the two must
/// survive.
#[test]
fn a_lockfile_published_between_the_cleanup_decision_and_the_removal_survives() {
    use crate::shadow_update::run_with_placeholder_lock_hooked;
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    let publish = |path: &std::path::Path| { fs::write(path, REAL_LOCK).expect("another writer publishes a real lockfile"); };

    let code: i32 = run_with_placeholder_lock_hooked(&lock, false, &mut || { return 1; }, &publish);

    assert_eq!(code, 1);
    assert_eq!(fs::read_to_string(&lock).expect("the real lockfile must survive"), REAL_LOCK);
}
