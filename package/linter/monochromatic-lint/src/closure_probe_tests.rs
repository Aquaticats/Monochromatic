//! What:
//!  Controls of the machine prerequisite checks.
//! Why:
//!  A missing prerequisite must skip with an explanation,
//! a present one must pass,
//! and no check may run anything the
//!  manifest chose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a missing program skips', () => expect(prepare([executables(['x'])], dir, cargo, search)).toEqual(skipped(...)));
//! ```

use super::{PackageSetup, Readiness, check_executables, find_executable, prepare, qmake_setup};
use crate::{closure_manifest::Prerequisite, test_fs::Fixture};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Write an empty file,
/// executable on Unix.
fn write_program(directory: &Path, name: &str, executable: bool) -> PathBuf {
    std::fs::create_dir_all(directory).expect("directory");
    let path: PathBuf = directory.join(name);
    std::fs::write(&path, "").expect("program file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode: u32 = if executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("mode");
    }
    #[cfg(not(unix))]
    let _unused: bool = executable;
    return path;
}

/// A search list of the given directories.
fn search_of(directories: &[&Path]) -> OsString {
    return std::env::join_paths(directories).expect("joinable directories");
}

/// A program in a directory with a space is found;
/// a name that exists only as a directory is not.
#[test]
fn finds_a_program_in_a_directory_with_a_space() {
    let fixture: Fixture = Fixture::new();
    let bin: PathBuf = fixture.path.join("my tools");
    let program: PathBuf = write_program(&bin, "zig", true);
    std::fs::create_dir(bin.join("folder")).expect("folder");
    assert_eq!(find_executable("zig", &search_of(&[&bin])), Some(program));
    assert_eq!(find_executable("folder", &search_of(&[&bin])), None);
    assert_eq!(find_executable("absent", &search_of(&[&bin])), None);
}

/// A file without the execute bit is not a program (Unix).
#[cfg(unix)]
#[test]
fn a_file_without_the_execute_bit_is_not_found() {
    let fixture: Fixture = Fixture::new();
    write_program(&fixture.path, "plain", false);
    assert_eq!(find_executable("plain", &search_of(&[&fixture.path])), None);
}

/// The first directory of the search list wins.
#[test]
fn the_first_directory_wins() {
    let fixture: Fixture = Fixture::new();
    let first: PathBuf = fixture.path.join("first");
    let second: PathBuf = fixture.path.join("second");
    let expected: PathBuf = write_program(&first, "tool", true);
    write_program(&second, "tool", true);
    assert_eq!(
        find_executable("tool", &search_of(&[&first, &second])),
        Some(expected)
    );
}

/// An empty search list finds nothing.
#[test]
fn an_empty_search_finds_nothing() {
    assert_eq!(find_executable("tool", &OsString::new()), None);
}

/// Missing programs are named in the reason.
#[test]
fn check_executables_names_the_first_missing_program() {
    let fixture: Fixture = Fixture::new();
    write_program(&fixture.path, "present", true);
    let names: Vec<String> = vec![String::from("present"), String::from("absent-tool")];
    let reason: String =
        check_executables(names.as_slice(), &search_of(&[&fixture.path])).expect("a skip");
    assert!(reason.contains("absent-tool"), "{reason}");
    assert_eq!(
        check_executables(&names[..1], &search_of(&[&fixture.path])),
        None
    );
}

/// `qmake6` is preferred over `qmake`,
/// and an existing `QMAKE` file is kept.
#[test]
fn qmake_prefers_qmake6_and_keeps_an_existing_variable() {
    let fixture: Fixture = Fixture::new();
    let six: PathBuf = write_program(&fixture.path, "qmake6", true);
    write_program(&fixture.path, "qmake", true);
    let search: OsString = search_of(&[&fixture.path]);
    assert_eq!(
        qmake_setup(&search, None),
        Ok(vec![(
            String::from("QMAKE"),
            six.to_string_lossy().into_owned()
        )])
    );
    let owned: PathBuf = write_program(&fixture.path.join("owned"), "qmake-custom", true);
    assert_eq!(
        qmake_setup(&search, Some(owned.into_os_string())),
        Ok(Vec::new())
    );
    assert!(qmake_setup(&search, Some(OsString::from("/nonexistent/qmake"))).is_ok());
}

/// Without any qmake the package is skipped with a reason.
#[test]
fn qmake_without_a_program_is_skipped() {
    let fixture: Fixture = Fixture::new();
    assert!(qmake_setup(&search_of(&[&fixture.path]), None).is_err());
}

/// A module that no machine has is a skip,
/// whether pkg-config is installed (it fails) or not (it does not run).
#[test]
fn a_missing_system_library_skips() {
    let fixture: Fixture = Fixture::new();
    let prerequisites: Vec<Prerequisite> = vec![Prerequisite::PkgConfig(vec![String::from(
        "monochromatic-lint-no-such-module-0",
    )])];
    let readiness: Readiness = prepare(
        prerequisites.as_slice(),
        &fixture.path,
        Path::new("cargo"),
        &OsString::new(),
    );
    match readiness {
        Readiness::Skipped(reason) => assert!(
            reason.contains("monochromatic-lint-no-such-module-0"),
            "{reason}"
        ),
        Readiness::Ready(_) => panic!("a module that does not exist was accepted"),
    }
}

/// No prerequisite is ready with an empty setup.
#[test]
fn no_prerequisites_is_ready_and_empty() {
    let fixture: Fixture = Fixture::new();
    assert_eq!(
        prepare(&[], &fixture.path, Path::new("cargo"), &OsString::new()),
        Readiness::Ready(PackageSetup::default())
    );
}

/// The first missing prerequisite decides,
/// and the programs of earlier ones do not mask it.
#[test]
fn the_first_missing_prerequisite_decides_the_skip() {
    let fixture: Fixture = Fixture::new();
    write_program(&fixture.path, "present", true);
    let prerequisites: Vec<Prerequisite> = vec![
        Prerequisite::Executables(vec![String::from("present")]),
        Prerequisite::Executables(vec![String::from("gone-one")]),
        Prerequisite::Executables(vec![String::from("gone-two")]),
    ];
    let readiness: Readiness = prepare(
        prerequisites.as_slice(),
        &fixture.path,
        Path::new("cargo"),
        &search_of(&[&fixture.path]),
    );
    match readiness {
        Readiness::Skipped(reason) => {
            assert!(
                reason.contains("gone-one") && !reason.contains("gone-two"),
                "{reason}"
            );
        }
        Readiness::Ready(_) => panic!("missing programs were accepted"),
    }
}

/// Qmake and executables together produce the variable once everything is present.
#[test]
fn a_ready_package_carries_the_qmake_variable() {
    let fixture: Fixture = Fixture::new();
    let qmake: PathBuf = write_program(&fixture.path, "qmake6", true);
    write_program(&fixture.path, "cmake", true);
    let prerequisites: Vec<Prerequisite> = vec![
        Prerequisite::Executables(vec![String::from("cmake")]),
        Prerequisite::Qmake,
    ];
    let readiness: Readiness = prepare(
        prerequisites.as_slice(),
        &fixture.path,
        Path::new("cargo"),
        &search_of(&[&fixture.path]),
    );
    // A QMAKE the test environment already owns would be kept instead, which is the same outcome for the package.
    match readiness {
        Readiness::Ready(setup) => {
            let owned: bool = std::env::var_os("QMAKE").is_some_and(file_exists);
            assert!(
                owned
                    || setup.environment
                        == vec![(String::from("QMAKE"), qmake.to_string_lossy().into_owned())],
                "{setup:?}"
            );
        }
        Readiness::Skipped(reason) => panic!("{reason}"),
    }
}

/// Whether a path variable names an existing file.
fn file_exists(value: OsString) -> bool {
    return Path::new(&value).is_file();
}
