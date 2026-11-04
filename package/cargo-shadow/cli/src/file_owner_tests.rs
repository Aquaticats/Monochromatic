//! The unit tests of the ownership evidence for a file a child may have written.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fs;
// Imports `std::time::Duration`.
// What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
// Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: T[] / Set / Map / number
// ```
use std::time::Duration;

// Imports `crate::file_owner::{Claim, claim_output, ob...`.
use crate::file_owner::{Claim, claim_output, observe, timed};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

/// Longer than the margin the evidence allows before a window starts and the clock tick of a file system,
/// so a later write is later.
const AFTER_THE_WINDOW: Duration = Duration::from_millis(60);

#[test]
fn a_file_the_child_rewrote_inside_its_window_is_the_childs_output() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
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
    fs::write(&path, "before").expect("lock");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let (_, window) = timed(|| { fs::write(&path, "child output").expect("the child writes"); });

    // Borrow `&path`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
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
    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Ours("child output".to_string()));
}

#[test]
fn a_file_equal_to_its_snapshot_is_unchanged_and_a_missing_one_is_absent() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("lock");
    let (_, window) = timed(|| {});

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Unchanged);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(&path).expect("remove");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Absent);
}

/// Review finding (2b 1,
/// 2c 4):
/// another writer publishes after the child returned and before the output is recorded.
/// The file differs from its snapshot,
/// but its change time is later than the window,
/// so it is not the child's output.
#[test]
fn a_file_published_after_the_child_returned_is_another_writers() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("lock");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let (_, window) = timed(|| { fs::write(&path, "child output").expect("the child writes"); });
    std::thread::sleep(AFTER_THE_WINDOW);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "published by another writer").expect("another writer");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Foreign);
}

/// The same for a file renamed into place after the child returned:
/// a rename moves the change time even when the source is old.
#[test]
fn a_file_renamed_into_place_after_the_child_returned_is_another_writers() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    let prepared = root.path().join("prepared");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "before").expect("lock");
    // Borrow `&prepared`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&prepared, "an editor's save").expect("the editor prepares its file earlier");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let (_, window) = timed(|| { fs::write(&path, "child output").expect("the child writes"); });
    std::thread::sleep(AFTER_THE_WINDOW);
    // Borrow `&prepared`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::rename(&prepared, &path).expect("the editor publishes by rename");

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(claim_output(&path, "before", window).expect("claims"), Claim::Foreign);
}

/// Review finding (2b 5):
/// an unreadable file is an error,
/// never an empty one.
#[test]
fn a_file_that_is_not_utf8_text_is_an_error_not_an_empty_file() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, [0xff, 0xfe, 0x00]).expect("binary");
    let (_, window) = timed(|| {});

    // Borrow `&path`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(observe(&path).is_err());
    // Borrow `&path`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(claim_output(&path, "", window).is_err());
}

/// Review finding (3b 8):
/// a window whose end lies before its start (the clock stepped back during the call) establishes nothing,
/// so a changed file is treated as the call's output (it is restored with its displaced copy kept),
/// never left live as foreign.
#[test]
fn a_window_that_ends_before_it_starts_claims_the_changed_file() {
    let root = scratch();
    let path = root.path().join("Cargo.lock");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&path, "changed").expect("lock");
    let inverted = crate::file_owner::Window { start_ns: crate::file_owner::clock_ns() + 3_600_000_000_000, end_ns: crate::file_owner::clock_ns() - 3_600_000_000_000 };

    // Borrow `&path`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(claim_output(&path, "before", inverted).expect("claims"), Claim::Ours("changed".to_string()));
}
