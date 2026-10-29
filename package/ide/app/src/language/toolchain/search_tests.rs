//! Lookup order,
//!  shim refusal,
//!  and `PATH` construction on disposable directories.

use super::{Missed, find_program_in, is_shim, search_path};
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
};

fn executable(path: &Path) {
    fs::create_dir_all(path.parent().expect("parent")).expect("directory");
    fs::write(path, "#!/bin/sh\n").expect("program");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("mode");
}

#[test]
fn toolchain_directories_win_over_the_application_path() {
    let root = crate::test_scratch::scratch().expect("directory");
    let tools = root.path().join("tools/bin");
    let stale = root.path().join("stale/bin");
    executable(&tools.join("taplo"));
    executable(&stale.join("taplo"));
    let directories = [tools];
    let found = find_program_in("taplo", root.path(), &directories, &[stale]).expect("found");
    assert_eq!(found, directories[0].join("taplo"));
}

#[test]
fn the_application_path_is_searched_when_the_toolchain_has_no_such_program() {
    let root = crate::test_scratch::scratch().expect("directory");
    let tools = root.path().join("tools/bin");
    let stale = root.path().join("stale/bin");
    fs::create_dir_all(&tools).expect("directory");
    executable(&stale.join("marksman"));
    let inherited = [stale];
    let found = find_program_in("marksman", root.path(), &[tools], &inherited).expect("found");
    assert_eq!(found, inherited[0].join("marksman"));
}

#[test]
fn a_program_that_is_nowhere_is_missed_without_a_shim() {
    let root = crate::test_scratch::scratch().expect("directory");
    let missed = find_program_in("absent", root.path(), &[], &[]).expect_err("missed");
    assert_eq!(missed, Missed { shim: None });
}

#[test]
fn a_file_without_execute_permission_is_not_a_program() {
    let root = crate::test_scratch::scratch().expect("directory");
    let directory = root.path().join("bin");
    fs::create_dir_all(&directory).expect("directory");
    fs::write(directory.join("plain"), "").expect("file");
    assert!(find_program_in("plain", root.path(), &[directory], &[]).is_err());
}

#[test]
fn relative_directories_are_never_searched() {
    let root = crate::test_scratch::scratch().expect("directory");
    executable(&root.path().join("bin/taplo"));
    let relative = [PathBuf::from("bin")];
    assert!(find_program_in("taplo", root.path(), &relative, &relative).is_err());
}

#[test]
fn a_command_with_a_separator_is_taken_relative_to_the_project() {
    let root = crate::test_scratch::scratch().expect("directory");
    executable(&root.path().join("node_modules/.bin/server"));
    let found = find_program_in("node_modules/.bin/server", root.path(), &[], &[]).expect("found");
    assert_eq!(found, root.path().join("node_modules/.bin/server"));
    assert!(find_program_in("node_modules/.bin/other", root.path(), &[], &[]).is_err());
}

#[test]
fn a_shim_is_a_link_to_the_mise_program_and_nothing_else_is() {
    let root = crate::test_scratch::scratch().expect("directory");
    let mise = root.path().join("bin/mise");
    executable(&mise);
    let shims = root.path().join("shims");
    fs::create_dir_all(&shims).expect("directory");
    symlink(&mise, shims.join("taplo")).expect("shim");
    let other = root.path().join("bin/other-tool");
    executable(&other);
    symlink(&other, shims.join("linked-elsewhere")).expect("link");
    assert!(is_shim(&shims.join("taplo")));
    assert!(!is_shim(&shims.join("linked-elsewhere")));
    assert!(!is_shim(&other));
    // A regular file that happens to be called mise is mise, not a shim of something else.
    assert!(!is_shim(&mise));
    assert!(!is_shim(&root.path().join("missing")));
}

#[test]
fn a_shim_is_skipped_and_named_and_a_real_program_after_it_is_found() {
    let root = crate::test_scratch::scratch().expect("directory");
    let mise = root.path().join("bin/mise");
    executable(&mise);
    let shims = root.path().join("shims");
    fs::create_dir_all(&shims).expect("directory");
    symlink(&mise, shims.join("taplo")).expect("shim");
    let only_shims = [shims.clone()];
    let missed = find_program_in("taplo", root.path(), &[], &only_shims).expect_err("shim only");
    assert_eq!(
        missed,
        Missed {
            shim: Some(shims.join("taplo"))
        }
    );
    let real = root.path().join("install/bin");
    executable(&real.join("taplo"));
    let found = find_program_in("taplo", root.path(), &[], &[shims, real.clone()]).expect("found");
    assert_eq!(found, real.join("taplo"));
}

#[test]
fn the_path_lists_toolchain_directories_before_the_inherited_ones() {
    let directories = [
        PathBuf::from("/tools/node/bin"),
        PathBuf::from("/tools/taplo"),
    ];
    let path = search_path(&directories, Some(OsString::from("/usr/bin:/bin"))).expect("path");
    assert_eq!(path, "/tools/node/bin:/tools/taplo:/usr/bin:/bin");
    assert_eq!(
        search_path(&directories, None).expect("path"),
        "/tools/node/bin:/tools/taplo"
    );
}

#[test]
fn directories_that_cannot_stand_in_a_path_are_left_out_and_the_others_kept_whole() {
    let directories = [
        PathBuf::from("/tools/with:colon/bin"),
        PathBuf::from("/tools/with space/bin"),
        PathBuf::from("/tools/it's \"quoted\"/bin"),
        PathBuf::from("/tools/-dash/bin"),
        PathBuf::from("/tools/new\nline/bin"),
    ];
    let path = search_path(&directories, Some(OsString::from("/usr/bin"))).expect("path");
    let entries: Vec<&str> = path.split(':').collect();
    assert_eq!(
        entries,
        [
            "/tools/with space/bin",
            "/tools/it's \"quoted\"/bin",
            "/tools/-dash/bin",
            "/tools/new\nline/bin",
            "/usr/bin"
        ]
    );
}
