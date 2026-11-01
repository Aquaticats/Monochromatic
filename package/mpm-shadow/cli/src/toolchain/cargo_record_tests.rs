//! Unit tests of the reader of cargo's `.crates.toml` record:
//! the shapes cargo writes (one binary on a line,
//! a list with one binary per line),
//! and a file that is missing,
//! empty or not a record.

use std::fs;

use super::cargo_record::{RecordedInstall, install_naming_bin, parse_recorded_installs, read_recorded_installs};

/// The record's shape as measured on 2026-10-31:
/// a header,
/// single-binary entries,
/// and a list with one binary per line.
const RECORD: &str = r#"[v1]
"cargo-ndk 4.1.2 (registry+https://github.com/rust-lang/crates.io-index)" = [
    "cargo-ndk",
    "cargo-ndk-env",
]
"monochromatic-git-policy-cli 0.4.5 (registry+https://github.com/rust-lang/crates.io-index)" = ["cli-git-native"]
"monochromatic-mpm-shadow 0.2.0 (registry+https://github.com/rust-lang/crates.io-index)" = ["mpm-shadow"]
"#;

fn install(crate_name: &str, version: &str, bins: &[&str]) -> RecordedInstall {
    return RecordedInstall {
        crate_name: crate_name.to_string(),
        version: version.to_string(),
        bins: bins.iter().map(|bin| return (*bin).to_string()).collect(),
    };
}

#[test]
fn single_line_and_multi_line_entries_are_both_read_with_their_binaries() {
    let installs = parse_recorded_installs(RECORD);
    assert_eq!(
        installs,
        vec![
            install("cargo-ndk", "4.1.2", &["cargo-ndk", "cargo-ndk-env"]),
            install("monochromatic-git-policy-cli", "0.4.5", &["cli-git-native"]),
            install("monochromatic-mpm-shadow", "0.2.0", &["mpm-shadow"]),
        ]
    );
}

#[test]
fn the_install_naming_a_binary_is_found_and_an_unnamed_binary_has_none() {
    let installs = parse_recorded_installs(RECORD);
    let found = install_naming_bin(&installs, "cargo-ndk-env").expect("named by the second line of the list");
    assert_eq!(found.crate_name, "cargo-ndk");
    assert!(install_naming_bin(&installs, "search-shadow").is_none(), "no entry names it");
}

#[test]
fn text_that_is_not_a_record_names_nothing() {
    for text in ["", "[v1]\n", "not a record at all\n\"only a quoted line\"\n", "\"nameonly\" = [\"a\"]\n"] {
        assert!(parse_recorded_installs(text).is_empty(), "{text:?}");
    }
}

#[test]
fn a_missing_record_file_names_nothing_and_a_present_one_is_read_from_the_cargo_home() {
    let directory = crate::test_scratch::scratch();
    assert!(read_recorded_installs(directory.path()).is_empty(), "no .crates.toml in the directory");
    fs::write(directory.path().join(".crates.toml"), RECORD).expect("write the record");
    assert_eq!(read_recorded_installs(directory.path()).len(), 3);
}
