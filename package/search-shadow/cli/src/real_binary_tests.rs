//! What:
//! Unit tests of the PATH lookup in `real_binary.rs`.
//! Why:
//! The lookup decides which program actually runs,
//! so a wrong answer would either loop the shadow into itself or run the wrong tool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.unit.test.ts
//! describe('findInPath', () => { it('finds the first executable outside the skipped path', () => { ... }); });
//! ```

/// What:
/// `use crate::real_binary::find_in_path;` imports the function under test from this crate (`crate` means
/// "the package I am in").
/// Why:
/// The tests call it directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { findInPath } from './real-binary.ts';
/// ```
use crate::real_binary::find_in_path;

/// Imports the file-system module (write fixture files / set permissions / make symlinks).
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (needed to mark a fixture file executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `fn write_tool(directory: &Path, name: &str, mode: u32)` declares a helper that writes a one-line shell script
/// named `name` in `directory` and sets its Unix permission bits to `mode`
/// (`u32` is an unsigned 32-bit integer;
/// the siblings are `i32` / `u64`).
/// Why:
/// Each test needs runnable and non-runnable fixture files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeTool(directory: string, name: string, mode: number): void {
///   fs.writeFileSync(path.join(directory, name), '#!/bin/sh\n');
///   fs.chmodSync(path.join(directory, name), mode);
/// }
/// ```
fn write_tool(directory: &Path, name: &str, mode: u32) {
    // `.expect("...")` unwraps a `Result` and, on error, stops the test with that message (tests may panic on purpose).
    fs::write(directory.join(name), "#!/bin/sh\n").expect("write the fixture tool");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(mode)).expect("set the fixture mode");
}

/// What:
/// `#[test]` is an attribute marking the next function as a test case the test runner should run.
/// The function takes nothing and passes if it does not panic.
/// Why:
/// Rust's built-in test runner (here driven by nextest) discovers tests by this attribute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('finds the first executable outside the skipped path', () => { ... });
/// ```
#[test]
fn finds_the_first_executable_on_path() {
    // `scratch()` makes a directory that is deleted when the variable goes out of scope.
    let first = crate::test_scratch::scratch();
    let second = crate::test_scratch::scratch();
    write_tool(first.path(), "rg", 0o755);
    write_tool(second.path(), "rg", 0o755);
    let path_variable = format!("{}:{}", first.path().display(), second.path().display());

    // `Path::new("")` is an empty path, which matches no candidate, so nothing is skipped.
    let found = find_in_path("rg", &path_variable, Path::new(""));

    // `assert_eq!` stops the test with a message when the two values differ (a macro, hence `!`).
    assert_eq!(found, Some(first.path().join("rg")));
}

/// The lookup must not return a file without execute bits.
#[test]
fn skips_a_file_that_is_not_executable() {
    let plain = crate::test_scratch::scratch();
    let real = crate::test_scratch::scratch();
    write_tool(plain.path(), "rg", 0o644);
    write_tool(real.path(), "rg", 0o755);
    let path_variable = format!("{}:{}", plain.path().display(), real.path().display());

    let found = find_in_path("rg", &path_variable, Path::new(""));

    assert_eq!(found, Some(real.path().join("rg")));
}

/// A symlink that resolves to the skipped path (the shadow itself) must be passed over,
/// which is what stops the loop.
#[test]
fn skips_a_link_that_resolves_to_the_shadow() {
    let shadow_directory = crate::test_scratch::scratch();
    let real_directory = crate::test_scratch::scratch();
    write_tool(shadow_directory.path(), "search-shadow", 0o755);
    write_tool(real_directory.path(), "rg", 0o755);
    let shadow_executable = shadow_directory.path().join("search-shadow");
    std::os::unix::fs::symlink(&shadow_executable, shadow_directory.path().join("rg")).expect("make the shadow link");
    let path_variable = format!("{}:{}", shadow_directory.path().display(), real_directory.path().display());

    // The skip value is the resolved path of the shadow executable, as the shim computes it.
    let skip = fs::canonicalize(&shadow_executable).expect("resolve the shadow");
    let found = find_in_path("rg", &path_variable, &skip);

    assert_eq!(found, Some(real_directory.path().join("rg")));
}

/// Empty entries and missing directories and directories named like the tool must not stop the search.
#[test]
fn ignores_empty_entries_missing_directories_and_directories_named_like_the_tool() {
    let trap = crate::test_scratch::scratch();
    let real = crate::test_scratch::scratch();
    fs::create_dir(trap.path().join("rg")).expect("make a directory named like the tool");
    write_tool(real.path(), "rg", 0o755);
    let path_variable = format!("::/does/not/exist:{}:{}", trap.path().display(), real.path().display());

    let found = find_in_path("rg", &path_variable, Path::new(""));

    assert_eq!(found, Some(real.path().join("rg")));
}

/// Nothing runnable on PATH gives `None`,
/// which the shim reports as "not found".
#[test]
fn answers_none_when_no_directory_holds_the_tool() {
    let empty = crate::test_scratch::scratch();
    let path_variable = format!("{}", empty.path().display());

    let found = find_in_path("rg", &path_variable, Path::new(""));

    assert_eq!(found, None);
}
