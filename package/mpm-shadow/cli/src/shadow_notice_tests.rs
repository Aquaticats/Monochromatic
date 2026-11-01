//! Unit tests of the top-level scan and of the text added after mpm's version and help:
//! each option form click accepts before the subcommand,
//! and what must pass through untouched.

use std::ffi::OsString;
use std::path::Path;

use super::{TopLevelRequest, help_section, top_level_request, version_line};

fn request(words: &[&str]) -> TopLevelRequest {
    let arguments: Vec<OsString> = words.iter().map(|word| return OsString::from(*word)).collect();
    return top_level_request(&arguments);
}

#[test]
fn a_top_level_version_or_help_is_recognised_in_every_spelling_mpm_has() {
    assert_eq!(request(&["--version"]), TopLevelRequest::Version);
    assert_eq!(request(&["--help"]), TopLevelRequest::Help);
    assert_eq!(request(&["-h"]), TopLevelRequest::Help);
    assert_eq!(request(&["-vvh"]), TopLevelRequest::Help, "h inside a cluster of short flags");
    assert_eq!(request(&["--no-color", "--version"]), TopLevelRequest::Version);
    assert_eq!(request(&["--brew", "--no-apt", "-d", "--help"]), TopLevelRequest::Help, "manager selectors and flags");
}

#[test]
fn values_of_options_are_skipped_so_they_are_never_taken_for_the_subcommand_or_a_flag() {
    assert_eq!(request(&["--verbosity", "DEBUG", "--version"]), TopLevelRequest::Version);
    assert_eq!(request(&["--verbosity=DEBUG", "--help"]), TopLevelRequest::Help);
    assert_eq!(request(&["-t", "30", "--version"]), TopLevelRequest::Version);
    assert_eq!(request(&["-t30", "--version"]), TopLevelRequest::Version, "a value attached to a short option");
    assert_eq!(request(&["-vvj", "auto", "-h"]), TopLevelRequest::Help, "a cluster that ends in a value option");
    assert_eq!(request(&["--color", "never", "--version"]), TopLevelRequest::Version);
    assert_eq!(request(&["--color", "--version"]), TopLevelRequest::Version, "a bare --color has no value");
    assert_eq!(request(&["--config", "--help", "--version"]), TopLevelRequest::Version, "--help here is --config's value");
}

#[test]
fn a_subcommand_ends_the_scan_so_its_own_help_and_version_pass_through() {
    assert_eq!(request(&["install", "--help"]), TopLevelRequest::Other);
    assert_eq!(request(&["help", "install"]), TopLevelRequest::Other);
    assert_eq!(request(&["--verbose", "outdated", "--version"]), TopLevelRequest::Other);
    assert_eq!(request(&["--brew", "list", "-h"]), TopLevelRequest::Other);
    assert_eq!(request(&[]), TopLevelRequest::Other);
}

#[test]
fn another_output_that_comes_first_keeps_the_text_off_and_the_first_eager_option_wins() {
    assert_eq!(request(&["--help-format", "json", "--help"]), TopLevelRequest::Other);
    assert_eq!(request(&["--help-format=man", "--help"]), TopLevelRequest::Other);
    assert_eq!(request(&["--tree", "--version"]), TopLevelRequest::Other);
    assert_eq!(request(&["--man"]), TopLevelRequest::Other);
    assert_eq!(request(&["--export-config", "toml", "--version"]), TopLevelRequest::Other);
    assert_eq!(request(&["--version", "--tree"]), TopLevelRequest::Version, "--version came first");
    assert_eq!(request(&["--help", "--version"]), TopLevelRequest::Help);
}

#[test]
fn a_command_line_the_scan_does_not_understand_adds_nothing() {
    assert_eq!(request(&["-V"]), TopLevelRequest::Other, "mpm has no -V");
    assert_eq!(request(&["-z", "--version"]), TopLevelRequest::Other, "an unknown short option");
    assert_eq!(request(&["--", "--version"]), TopLevelRequest::Other);
    assert_eq!(request(&["-"]), TopLevelRequest::Other);
    let non_utf8: Vec<OsString> = vec![std::os::unix::ffi::OsStringExt::from_vec(vec![0x66, 0xff, 0x6f])];
    assert_eq!(top_level_request(&non_utf8), TopLevelRequest::Other);
    assert_eq!(request(&["--verbosity"]), TopLevelRequest::Other, "a value option with no value");
}

#[test]
fn the_added_text_names_the_shadow_its_version_and_the_real_mpm() {
    let line = version_line(Path::new("/real/bin/mpm"));
    assert_eq!(line, format!("mpm-shadow {}, shadowing mpm at /real/bin/mpm\n", env!("CARGO_PKG_VERSION")));
    assert_eq!(line.matches('\n').count(), 1, "exactly one line");
    let section = help_section(Path::new("/real/bin/mpm"));
    assert!(section.starts_with("\nmpm-shadow "), "{section}");
    for expected in [env!("CARGO_PKG_VERSION"), "/real/bin/mpm", "shadow of mpm", "`monochromatic-` crates", "deleted stale"] {
        assert!(section.contains(expected), "{expected} missing from:\n{section}");
    }
}
