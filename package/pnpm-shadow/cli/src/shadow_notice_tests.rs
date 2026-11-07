//! The unit tests of the notice:
//! which command lines get one,
//! and what it says.

/// Imports the functions and constants under test.
use crate::shadow_notice::{HELP_WORDS, TopLevelRequest, VERSION_WORDS, help_section, notice_text, top_level_request, version_line};

/// Imports the owned operating-system text type.
use std::ffi::OsString;
/// Imports the path type.
use std::path::Path;

fn arguments(words: &[&str]) -> Vec<OsString> {
    let mut list: Vec<OsString> = Vec::new();
    for word in words {
        list.push(OsString::from(word));
    }
    return list;
}

#[test]
fn every_version_word_alone_is_a_version_request() {
    for word in VERSION_WORDS {
        assert_eq!(top_level_request(&arguments(&[word])), TopLevelRequest::Version, "word {word}");
    }
}

#[test]
fn every_help_word_alone_is_a_help_request() {
    for word in HELP_WORDS {
        assert_eq!(top_level_request(&arguments(&[word])), TopLevelRequest::Help, "word {word}");
    }
}

#[test]
fn a_command_line_with_more_than_one_word_or_none_is_never_a_top_level_request() {
    let command_lines: [&[&str]; 5] = [
        &[],
        &["install", "--help"],
        &["help", "install"],
        &["--dir", "x", "--version"],
        &["--version", "--json"],
    ];
    for command_line in command_lines {
        assert_eq!(top_level_request(&arguments(command_line)), TopLevelRequest::Other, "{command_line:?}");
    }
}

#[test]
fn an_ordinary_command_is_not_a_top_level_request() {
    assert_eq!(top_level_request(&arguments(&["install"])), TopLevelRequest::Other);
}

#[test]
fn the_version_line_names_the_wrapper_its_version_and_the_real_pnpm() {
    let line: String = version_line(Path::new("/real/bin/pnpm"));

    assert!(line.starts_with("pnpm-shadow "));
    assert!(line.contains(env!("CARGO_PKG_VERSION")));
    assert!(line.contains("/real/bin/pnpm"));
    assert!(line.ends_with('\n'));
}

#[test]
fn the_help_section_starts_after_a_blank_line_and_repeats_the_version_line() {
    let real = Path::new("/real/bin/pnpm");

    let section: String = help_section(real);

    assert!(section.starts_with('\n'));
    assert!(section.contains(&version_line(real)));
    assert!(section.contains("never blocks pnpm"));
}

#[test]
fn only_a_version_or_help_request_gets_a_notice() {
    let real = Path::new("/real/bin/pnpm");

    assert_eq!(notice_text(TopLevelRequest::Version, real), Some(version_line(real)));
    assert_eq!(notice_text(TopLevelRequest::Help, real), Some(help_section(real)));
    assert_eq!(notice_text(TopLevelRequest::Other, real), None);
}
