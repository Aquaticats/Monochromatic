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
