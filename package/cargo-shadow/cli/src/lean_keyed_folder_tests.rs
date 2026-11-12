//! The tests of the toolchain-keyed build folder:
//! the base is read from the user's template,
//! the toolchain from the arguments or the environment,
//! and the key is stable and differs by workspace and by toolchain.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::lean_keyed_folder::{base_of_template, configured_base, keyed_build_dir, leave_ledger, read_ledger, toolchain_of, workspace_key};
use crate::lean_test_support::{make_pipe, within};
use crate::test_scratch::scratch;

#[test]
fn the_base_is_what_precedes_the_workspace_hash_and_nothing_else_is_keyed() {
    assert_eq!(base_of_template("/var/mnt/encrypted/cargo-build/{workspace-path-hash}"), Some(PathBuf::from("/var/mnt/encrypted/cargo-build")));
    assert_eq!(base_of_template("{cargo-cache-home}/build/{workspace-path-hash}"), None);
    assert_eq!(base_of_template("relative/{workspace-path-hash}"), None);
    assert_eq!(base_of_template("/fixed/dir"), None);
    assert_eq!(base_of_template("/x/{workspace-path-hash}/after"), None);
}

#[test]
fn the_configuration_file_is_read_for_its_build_dir() {
    let home = scratch();
    assert_eq!(configured_base(home.path()), None);
    std::fs::write(home.path().join("config.toml"), "[build]\nbuild-dir = \"/b/{workspace-path-hash}\"\n").expect("config");
    assert_eq!(configured_base(home.path()), Some(PathBuf::from("/b")));
    std::fs::write(home.path().join("config.toml"), "[build]\njobs = 2\n").expect("config");
    assert_eq!(configured_base(home.path()), None);
}

#[test]
fn a_plus_argument_beats_the_environment_and_odd_names_are_refused() {
    let plus: Vec<OsString> = vec![OsString::from("+nightly-a"), OsString::from("check")];
    let plain: Vec<OsString> = vec![OsString::from("check")];
    assert_eq!(toolchain_of(&plus, Some("nightly-b")), Some(String::from("nightly-a")));
    assert_eq!(toolchain_of(&plain, Some("nightly-b")), Some(String::from("nightly-b")));
    assert_eq!(toolchain_of(&plain, None), None);
    for odd in ["../x", "a/b", ".hidden", ""] {
        assert_eq!(toolchain_of(&plain, Some(odd)), None, "{odd:?}");
    }
}

#[test]
fn the_key_is_stable_and_differs_by_workspace_and_by_toolchain() {
    let one: PathBuf = PathBuf::from("/work/one");
    let two: PathBuf = PathBuf::from("/work/two");
    assert_eq!(workspace_key(&one), workspace_key(&one));
    assert_ne!(workspace_key(&one), workspace_key(&two));
    let (shard, key) = workspace_key(&one);
    assert_eq!((shard.len(), key.len()), (2, 14));
    let base: &Path = Path::new("/b");
    assert_ne!(keyed_build_dir(base, "nightly-a", &one), keyed_build_dir(base, "nightly-b", &one));
    assert!(keyed_build_dir(base, "nightly-a", &one).starts_with("/b/toolchain/nightly-a"));
}

#[test]
fn the_ledger_is_written_once_and_read_back() {
    let folder = scratch();
    let inner: PathBuf = folder.path().join("keyed");
    leave_ledger(&inner, Path::new("/work/one"));
    leave_ledger(&inner, Path::new("/work/two"));
    assert_eq!(read_ledger(&inner), Some(PathBuf::from("/work/one")));
}

/// A pipe where a Cargo configuration would be is no configured base and holds nothing:
/// the name is a person's file, so the open refuses what is not a regular file.
#[test]
fn a_pipe_at_a_cargo_configuration_is_no_base_and_does_not_hold_the_run() {
    let root = scratch();
    let home: PathBuf = root.path().join("cargo-home");
    std::fs::create_dir_all(&home).expect("cargo home");
    make_pipe(&home.join("config.toml"));
    make_pipe(&home.join("config"));
    let named_home: PathBuf = home.clone();
    let base: Option<PathBuf> = within(move || { return configured_base(&named_home); });
    assert_eq!(base, None);
}
