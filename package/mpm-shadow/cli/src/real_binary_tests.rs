//! What:
//! Unit tests of the PATH lookup in `real_binary.rs`.
//! Why:
//! The lookup decides which program actually runs,
//! so a wrong answer would either loop the shadow into itself
//!           or run the wrong program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.unit.test.ts
//! describe('findInPath', () => { it('finds the first executable outside the skipped path', () => { ... }); });
//! ```

/// Imports the function under test from this crate.
use crate::real_binary::find_in_path;

/// Imports the scratch-directory helper (see `test_scratch.rs`).
use crate::test_scratch::scratch;

/// Imports the file-system module (writes fixture scripts,
/// sets permissions,
/// makes symlinks).
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (marks a fixture executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// What:
/// `fn write_tool(directory: &Path, name: &str, mode: u32)` declares a helper writing a one-line script named
///           `name` in `directory` with Unix permission bits `mode` (`u32`:
/// unsigned 32-bit integer,
/// siblings `i32`,
///           `u64`).
/// Why:
/// Each test needs runnable and non-runnable fixture files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeTool(directory: string, name: string, mode: number): void
/// ```
fn write_tool(directory: &Path, name: &str, mode: u32) {
    fs::write(directory.join(name), "#!/bin/sh\n").expect("write the fixture tool");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(mode)).expect("set the fixture mode");
}

/// The first executable in PATH order is returned.
#[test]
fn finds_the_first_executable_in_path_order() {
    let first = scratch();
    let second = scratch();
    write_tool(first.path(), "mpm", 0o755);
    write_tool(second.path(), "mpm", 0o755);
    let path_variable: String = format!("{}:{}", first.path().display(), second.path().display());

    // `Some(first.path().join("mpm"))` is the present case of `Option` wrapping the expected path.
    assert_eq!(find_in_path("mpm", &path_variable, Path::new("/nowhere")), Some(first.path().join("mpm")));
}

/// A file without an execute bit is not a candidate.
#[test]
fn skips_files_that_are_not_executable() {
    let first = scratch();
    let second = scratch();
    write_tool(first.path(), "mpm", 0o644);
    write_tool(second.path(), "mpm", 0o755);
    let path_variable: String = format!("{}:{}", first.path().display(), second.path().display());

    assert_eq!(find_in_path("mpm", &path_variable, Path::new("/nowhere")), Some(second.path().join("mpm")));
}

/// Any candidate that resolves to the shadow itself is skipped,
/// however many links lead to it.
#[test]
fn skips_every_link_that_resolves_to_the_shadow() {
    let shadow_home = scratch();
    let first_links = scratch();
    let second_links = scratch();
    let real = scratch();
    write_tool(shadow_home.path(), "mpm-shadow", 0o755);
    write_tool(real.path(), "mpm", 0o755);
    std::os::unix::fs::symlink(shadow_home.path().join("mpm-shadow"), first_links.path().join("mpm")).expect("link");
    std::os::unix::fs::symlink(shadow_home.path().join("mpm-shadow"), second_links.path().join("mpm")).expect("link");
    let skip: PathBuf = fs::canonicalize(shadow_home.path().join("mpm-shadow")).expect("resolve the shadow");
    let path_variable: String =
        format!("{}:{}:{}", first_links.path().display(), second_links.path().display(), real.path().display());

    assert_eq!(find_in_path("mpm", &path_variable, &skip), Some(real.path().join("mpm")));
}

/// Nothing runnable on PATH gives `None`;
/// empty PATH entries are ignored.
#[test]
fn returns_none_when_nothing_is_found() {
    let empty = scratch();
    let path_variable: String = format!(":{}", empty.path().display());
    assert_eq!(find_in_path("mpm", &path_variable, Path::new("/nowhere")), None);
}
