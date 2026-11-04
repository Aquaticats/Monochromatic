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
const NAME: &str = "cargo";

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
    let shadow: PathBuf = shadow_directory.path().join("cargo-shadow");
    std::fs::write(&shadow, "#!/bin/sh\n").expect("write the shadow");
    std::fs::set_permissions(&shadow, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    symlink(&shadow, link_directory.path().join(NAME)).expect("link the shadow as cargo");
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

/// The text of a launch record (`mpm-shadow`'s format) whose `real` binary is `real`; built field by field because the `json!` macro
/// unwraps a conversion that clippy.toml disallows.
fn launch_record(real: &Path) -> String {
    let mut fields = serde_json::Map::new();
    fields.insert("toolId".to_string(), serde_json::Value::String("x".to_string()));
    fields.insert("lockFile".to_string(), serde_json::Value::String("/x".to_string()));
    fields.insert("real".to_string(), serde_json::Value::String(real.to_string_lossy().to_string()));
    fields.insert("scope".to_string(), serde_json::Value::String("repo".to_string()));
    return serde_json::Value::Object(fields).to_string();
}

/// A launcher in `directory`: an executable named `launcher_name` with a `<launcher_name>.launch` record whose `real` is `target`,
/// and a `cargo` link to it in `links`; returns the link.
fn write_launcher_link(directory: &Path, links: &Path, launcher_name: &str, target: &Path) -> PathBuf {
    let launcher: PathBuf = directory.join(launcher_name);
    std::fs::write(&launcher, "#!/bin/sh\n").expect("write the launcher");
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    let record: String = launch_record(target);
    std::fs::write(directory.join(format!("{launcher_name}.launch")), record).expect("write the record");
    let link: PathBuf = links.join(NAME);
    symlink(&launcher, &link).expect("link the launcher");
    return link;
}

/// A candidate that is a launcher whose record names the shadow (through a `current` link, as the toolchain's tree has it) is skipped
/// before it is run, and so is a launcher that leads to the shadow through another launcher.
#[test]
fn a_launcher_whose_record_names_the_shadow_is_skipped_directly_and_through_another_launcher() {
    let tree = scratch();
    let launchers = scratch();
    let links = scratch();
    let other_links = scratch();
    let real_directory = scratch();
    std::fs::create_dir_all(tree.path().join("0.1.0").join("bin")).expect("the tree");
    let shadow: PathBuf = tree.path().join("0.1.0").join("bin").join("cargo-shadow");
    std::fs::write(&shadow, "shadow").expect("the shadow binary");
    symlink("0.1.0", tree.path().join("current")).expect("the current link");
    let named: PathBuf = tree.path().join("current").join("bin").join("cargo-shadow");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);

    write_launcher_link(launchers.path(), links.path(), "direct", &named);
    let direct = find_in_path(NAME, &path_value(&[links.path(), real_directory.path()]), &skip);
    assert_eq!(direct, Some(expected.clone()), "a launcher whose record names the shadow was taken for the real cargo");

    // A second launcher whose record names the first one.
    let first_launcher: PathBuf = launchers.path().join("direct");
    write_launcher_link(launchers.path(), other_links.path(), "outer", &first_launcher);
    let through = find_in_path(NAME, &path_value(&[other_links.path(), real_directory.path()]), &skip);
    assert_eq!(through, Some(expected), "a launcher that leads to the shadow through another launcher was taken for the real cargo");
}

/// A launcher whose record names another program is a real cargo for this purpose, a record that cannot be read is not a launcher,
/// and a record chain that never ends is skipped instead of followed without end.
#[test]
fn a_launcher_of_another_program_a_broken_record_and_a_record_loop() {
    let launchers = scratch();
    let links = scratch();
    let elsewhere = scratch();
    let shadow = scratch();
    let shadow_binary: PathBuf = shadow.path().join("cargo-shadow");
    std::fs::write(&shadow_binary, "shadow").expect("the shadow binary");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow_binary).expect("resolves")];
    let other_program: PathBuf = write_program(elsewhere.path(), EXECUTABLE_MODE);

    // Names another program: not skipped.
    let link: PathBuf = write_launcher_link(launchers.path(), links.path(), "other", &other_program);
    assert_eq!(find_in_path(NAME, &path_value(&[links.path()]), &skip), Some(link.clone()), "a launcher of another program was skipped");

    // A record that is not JSON: not a launcher, so not skipped.
    std::fs::write(launchers.path().join("other.launch"), "not json").expect("break the record");
    assert_eq!(find_in_path(NAME, &path_value(&[links.path()]), &skip), Some(link), "a broken record made the candidate skipped");

    // Two launchers whose records name each other: the walk ends and the candidate is skipped.
    let looping = scratch();
    let loop_links = scratch();
    let a: PathBuf = looping.path().join("a");
    let b: PathBuf = looping.path().join("b");
    for (launcher, target) in [(&a, &b), (&b, &a)] {
        std::fs::write(launcher, "#!/bin/sh\n").expect("write a launcher");
        std::fs::set_permissions(launcher, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
        let record: String = launch_record(target);
        let mut name = launcher.as_os_str().to_os_string();
        name.push(".launch");
        std::fs::write(name, record).expect("write the record");
    }
    symlink(&a, loop_links.path().join(NAME)).expect("link");
    assert_eq!(find_in_path(NAME, &path_value(&[loop_links.path()]), &skip), None, "a loop of launcher records was followed or taken for a real cargo");
}

/// Third certification pass (the launcher row): a hard link to the shadow resolves to another pathname than the shadow's own and is
/// still the same executable, so a PATH candidate that is a hard link to it is skipped by file identity, not taken for the real cargo.
/// A symlink-only fixture cannot show this: a symlink canonicalizes to the shadow's own path.
#[test]
fn a_hard_link_to_the_shadow_is_skipped_by_file_identity() {
    let shadow_directory = scratch();
    let alias_directory = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = shadow_directory.path().join(NAME);
    std::fs::write(&shadow, "#!/bin/sh\n").expect("the shadow");
    std::fs::set_permissions(&shadow, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).expect("set the mode");
    let alias: PathBuf = alias_directory.path().join(NAME);
    std::fs::hard_link(&shadow, &alias).expect("a hard link of the shadow");
    assert_ne!(std::fs::canonicalize(&alias).expect("resolves"), std::fs::canonicalize(&shadow).expect("resolves"), "the alias must resolve to another pathname for this test to mean anything");
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];

    let found = find_in_path(NAME, &path_value(&[shadow_directory.path(), alias_directory.path(), real_directory.path()]), &skip);

    assert_eq!(found, Some(expected), "a hard link to the shadow was taken for the real cargo");
}

/// Third certification pass: a launcher whose record names a hard-linked alias of the shadow leads back to the shadow, so it is
/// skipped, directly and through a second launcher.
#[test]
fn a_launcher_record_that_names_a_hard_linked_alias_of_the_shadow_is_skipped() {
    let tree = scratch();
    let alias_directory = scratch();
    let launchers = scratch();
    let links = scratch();
    let real_directory = scratch();
    let shadow: PathBuf = tree.path().join("cargo-shadow");
    std::fs::write(&shadow, "shadow").expect("the shadow binary");
    let alias: PathBuf = alias_directory.path().join("alias-of-the-shadow");
    std::fs::hard_link(&shadow, &alias).expect("a hard link of the shadow");
    let skip: Vec<PathBuf> = vec![std::fs::canonicalize(&shadow).expect("resolves")];
    let expected: PathBuf = write_program(real_directory.path(), EXECUTABLE_MODE);
    write_launcher_link(launchers.path(), links.path(), "direct", &alias);

    let found = find_in_path(NAME, &path_value(&[links.path(), real_directory.path()]), &skip);

    assert_eq!(found, Some(expected), "a launcher whose record names a hard link of the shadow was taken for the real cargo");
}
