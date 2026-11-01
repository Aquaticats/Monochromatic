//! What:
//!  Controls for finding the linter on PATH.
//! Why:
//!  The first usable absolute entry wins;
//!  relative entries,
//!  directories,
//!  plain files
//!      and an unset PATH never yield a program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(findLinter(`${empty}:${linterDir}`)).toBe(join(linterDir, 'monochromatic-lint'));
//! ```

/// The lookup under test.
use super::{LINTER_NAME, find_linter, linter_file_name};
use crate::test_support::{executable, fixture, remove};
use std::ffi::OsString;
use std::path::PathBuf;

/// Join directories into one PATH value.
fn path_of(directories: &[PathBuf]) -> OsString {
    return std::env::join_paths(directories).expect("joinable PATH");
}

/// The name is the bare linter name on Unix.
#[test]
fn the_name_is_the_linter_s() {
    assert_eq!(LINTER_NAME, "monochromatic-lint");
    assert_eq!(linter_file_name(), "monochromatic-lint");
}

/// The first absolute entry with an executable file wins;
///  earlier entries that hold a
/// directory,
///  a file without an execute bit,
///  or nothing are passed over,
///  and so is a
/// relative entry even when it holds a usable program.
#[test]
fn the_first_usable_absolute_entry_wins() {
    let root: PathBuf = fixture("lookup-order");
    for name in ["empty", "directory", "plain", "first", "second", "relative"] {
        std::fs::create_dir(root.join(name)).expect("entry");
    }
    std::fs::create_dir(root.join("directory/monochromatic-lint")).expect("directory entry");
    std::fs::write(root.join("plain/monochromatic-lint"), b"#!/bin/sh\n").expect("plain");
    executable(
        root.join("first/monochromatic-lint").as_path(),
        b"#!/bin/sh\n",
    );
    executable(
        root.join("second/monochromatic-lint").as_path(),
        b"#!/bin/sh\n",
    );
    executable(
        root.join("relative/monochromatic-lint").as_path(),
        b"#!/bin/sh\n",
    );
    // A relative entry is resolved against the current directory by a shell; never here.
    let relative: PathBuf = PathBuf::from("relative");
    let path: OsString = path_of(&[
        relative,
        root.join("empty"),
        root.join("directory"),
        root.join("plain"),
        root.join("first"),
        root.join("second"),
    ]);
    assert_eq!(
        find_linter(Some(path.as_os_str())),
        Some(root.join("first/monochromatic-lint"))
    );
    remove(root.as_path());
}

/// A symbolic link to the executable is followed,
///  as running it would follow it.
#[test]
fn a_link_to_the_linter_is_found() {
    let root: PathBuf = fixture("lookup-link");
    std::fs::create_dir(root.join("bin")).expect("bin");
    executable(root.join("real-linter").as_path(), b"#!/bin/sh\n");
    std::os::unix::fs::symlink(
        root.join("real-linter"),
        root.join("bin/monochromatic-lint"),
    )
    .expect("link");
    let path: OsString = path_of(&[root.join("bin")]);
    assert_eq!(
        find_linter(Some(path.as_os_str())),
        Some(root.join("bin/monochromatic-lint"))
    );
    remove(root.as_path());
}

/// No usable entry,
///  an empty PATH,
///  a PATH of relative entries and an unset PATH find nothing.
#[test]
fn nothing_usable_finds_nothing() {
    let root: PathBuf = fixture("lookup-none");
    std::fs::create_dir(root.join("empty")).expect("empty");
    // A linter beside the PATH entries, but not in one, is not found.
    executable(root.join("monochromatic-lint").as_path(), b"#!/bin/sh\n");
    let empty_entry: OsString = path_of(&[root.join("empty")]);
    for path in [
        empty_entry,
        OsString::new(),
        OsString::from("relative:./also:"),
    ] {
        assert_eq!(find_linter(Some(path.as_os_str())), None, "{path:?}");
    }
    assert_eq!(find_linter(None), None);
    remove(root.as_path());
}
