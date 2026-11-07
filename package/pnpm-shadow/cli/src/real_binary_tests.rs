//! The unit tests of the PATH lookup:
//! the first executable wins,
//! non-executables and the shadow itself are skipped.

/// Imports the function under test.
use crate::real_binary::find_in_path;
/// Imports the scratch directories.
use crate::test_scratch::scratch;

/// Imports the owner-only and executable mode helpers.
use std::os::unix::fs::{PermissionsExt, symlink};
/// Imports the path types.
use std::path::{Path, PathBuf};

/// The program name the tests look for.
const NAME: &str = "pnpm";

/// The mode of a file the owner may run.
const EXECUTABLE_MODE: u32 = 0o755;

/// The mode of a file nobody may run.
const PLAIN_MODE: u32 = 0o644;

/// Writes a file named `NAME` in `directory` with the given mode and returns its path.
fn write_program(directory: &Path, mode: u32) -> PathBuf {
    let path: PathBuf = directory.join(NAME);
    std::fs::write(&path, "#!/bin/sh\n").expect("write the program");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("set the mode");
    return path;
}

/// Joins directories into a PATH value.
fn path_value(directories: &[&Path]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for directory in directories {
        parts.push(directory.to_string_lossy().to_string());
    }
    return parts.join(":");
}

#[test]
fn the_first_executable_in_path_order_wins() {
    let first = scratch();
    let second = scratch();
    let expected: PathBuf = write_program(first.path(), EXECUTABLE_MODE);
    write_program(second.path(), EXECUTABLE_MODE);

    let found = find_in_path(NAME, &path_value(&[first.path(), second.path()]), &[]);

    assert_eq!(found, Some(expected));
}

#[test]
fn a_file_without_an_execute_bit_is_skipped() {
    let plain = scratch();
    let runnable = scratch();
    write_program(plain.path(), PLAIN_MODE);
    let expected: PathBuf = write_program(runnable.path(), EXECUTABLE_MODE);

    let found = find_in_path(NAME, &path_value(&[plain.path(), runnable.path()]), &[]);

    assert_eq!(found, Some(expected));
}

#[test]
fn a_candidate_that_resolves_to_the_shadow_is_skipped_under_any_name() {
    let shadow_directory = scratch();
    let link_directory = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = shadow_directory.path().join("pnpm-shadow");
    std::fs::write(&shadow, "#!/bin/sh\n").expect("write the shadow");
    std::fs::set_permissions(&shadow, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    symlink(&shadow, link_directory.path().join(NAME)).expect("link the shadow as pnpm");
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let resolved_shadow: PathBuf = std::fs::canonicalize(&shadow).expect("resolve the shadow");

    let found = find_in_path(
        NAME,
        &path_value(&[link_directory.path(), real_directory.path()]),
        &[resolved_shadow],
    );

    assert_eq!(found, Some(expected));
}

#[test]
fn an_empty_path_entry_is_ignored_and_a_missing_program_is_none() {
    let empty = scratch();

    let found = find_in_path(NAME, &format!(":{}:", empty.path().display()), &[]);

    assert_eq!(found, None);
}

#[test]
fn every_resolved_path_in_the_skip_list_is_skipped_not_only_the_first() {
    let launcher_directory = scratch();
    let tried_directory = scratch();
    let real_directory = scratch();
    let launcher: PathBuf = write_program(launcher_directory.path(), EXECUTABLE_MODE);
    let tried: PathBuf = write_program(tried_directory.path(), EXECUTABLE_MODE);
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let skip: Vec<PathBuf> = vec![
        std::fs::canonicalize(&launcher).expect("resolve the launcher"),
        std::fs::canonicalize(&tried).expect("resolve the tried candidate"),
    ];

    let found = find_in_path(
        NAME,
        &path_value(&[launcher_directory.path(), tried_directory.path(), real_directory.path()]),
        &skip,
    );

    assert_eq!(found, Some(expected));
}
