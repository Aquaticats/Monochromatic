//! The unit tests of the option reading:
//! the command line is read,
//! never changed.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::options::{asserts_lock, change_dir, manifest_path};

fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

#[test]
fn the_manifest_path_is_read_in_both_spellings_and_the_last_wins() {
    assert_eq!(manifest_path(&arguments(&["check", "--manifest-path", "a/Cargo.toml"])), Some(PathBuf::from("a/Cargo.toml")));
    assert_eq!(manifest_path(&arguments(&["check", "--manifest-path=b/Cargo.toml"])), Some(PathBuf::from("b/Cargo.toml")));
    assert_eq!(
        manifest_path(&arguments(&["--manifest-path", "a/Cargo.toml", "check", "--manifest-path=b/Cargo.toml"])),
        Some(PathBuf::from("b/Cargo.toml"))
    );
    assert_eq!(manifest_path(&arguments(&["check"])), None);
}

#[test]
fn an_option_after_the_double_dash_belongs_to_the_program_and_is_not_read() {
    assert_eq!(manifest_path(&arguments(&["run", "--", "--manifest-path", "x"])), None);
    assert!(!asserts_lock(&arguments(&["run", "--", "--locked"])));
}

#[test]
fn the_change_dir_option_is_read() {
    assert_eq!(change_dir(&arguments(&["-C", "../x", "check"])), Some(PathBuf::from("../x")));
    assert_eq!(change_dir(&arguments(&["check"])), None);
}

#[test]
fn locked_and_frozen_assert_the_lock() {
    assert!(asserts_lock(&arguments(&["check", "--locked"])));
    assert!(asserts_lock(&arguments(&["--frozen", "check"])));
    assert!(!asserts_lock(&arguments(&["check", "--offline"])));
}

/// Review finding (supported spellings):
/// `cargo update -m <path>` is the short spelling of `--manifest-path` (shown by
/// `cargo update --help`),
/// so the workspace it selects is the one in the path.
#[test]
fn the_short_manifest_path_of_update_selects_the_workspace() {
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
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
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    assert_eq!(manifest_path(&arguments(&["update", "-m", "../b/Cargo.toml"])), Some(PathBuf::from("../b/Cargo.toml")));
}

/// Review finding (2c 9):
/// the attached short form `-m<path>` selects the workspace of `cargo update`.
#[test]
fn the_attached_short_manifest_path_of_update_selects_the_workspace() {
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&arguments`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(manifest_path(&arguments(&["update", "-m../b/Cargo.toml"])), Some(PathBuf::from("../b/Cargo.toml")));
}
