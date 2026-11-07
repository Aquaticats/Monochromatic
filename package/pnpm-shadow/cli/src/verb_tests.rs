//! The unit tests of the command-word rules:
//! which pnpm command lines the manifest warning belongs to.

/// Imports the functions and constants under test.
use crate::verb::{INSTALL_CLASS_VERBS, VALUE_OPTIONS, find_verb, warning_makes_sense};

/// Imports the owned operating-system text type.
use std::ffi::OsString;

fn arguments(words: &[&str]) -> Vec<OsString> {
    let mut list: Vec<OsString> = Vec::new();
    for word in words {
        list.push(OsString::from(word));
    }
    return list;
}

#[test]
fn every_install_class_verb_alone_makes_the_warning_make_sense() {
    for verb in INSTALL_CLASS_VERBS {
        assert!(warning_makes_sense(&arguments(&[verb])), "verb {verb}");
    }
}

#[test]
fn a_command_that_does_not_act_on_specifiers_does_not() {
    let verbs: [&str; 14] = [
        "run", "list", "ls", "root", "bin", "exec", "dlx", "test", "start", "fetch", "prune", "outdated", "config", "why",
    ];
    for verb in verbs {
        assert!(!warning_makes_sense(&arguments(&[verb])), "verb {verb}");
    }
}

#[test]
fn options_before_the_verb_are_skipped_including_the_values_of_value_options() {
    let command_lines: [&[&str]; 7] = [
        &["--filter", "app", "add", "zod"],
        &["-F", "app", "add", "zod"],
        &["-C", "../x", "install"],
        &["--dir=../x", "install"],
        &["-r", "install"],
        &["-w", "--no-progress", "update"],
        &["--reporter", "silent", "--registry", "https://example.invalid", "i"],
    ];
    for command_line in command_lines {
        assert!(warning_makes_sense(&arguments(command_line)), "{command_line:?}");
    }
}

#[test]
fn the_value_of_a_value_option_is_never_taken_for_the_verb() {
    // `install` here is the filter's value, so the verb is `list`.
    assert_eq!(find_verb(&arguments(&["--filter", "install", "list"])), Some("list".to_string()));
    assert!(!warning_makes_sense(&arguments(&["--filter", "install", "list"])));
}

#[test]
fn every_value_option_consumes_exactly_the_next_word() {
    for option in VALUE_OPTIONS {
        assert_eq!(find_verb(&arguments(&[option, "value", "add"])), Some("add".to_string()), "option {option}");
    }
}

#[test]
fn recursive_words_are_looked_through_once() {
    assert!(warning_makes_sense(&arguments(&["recursive", "install"])));
    assert!(warning_makes_sense(&arguments(&["m", "i"])));
    assert!(!warning_makes_sense(&arguments(&["recursive", "run", "build"])));
}

#[test]
fn a_help_or_version_word_anywhere_before_the_double_dash_cancels_the_warning() {
    let command_lines: [&[&str]; 5] = [
        &["install", "--help"],
        &["add", "-h"],
        &["--help", "install"],
        &["install", "--version"],
        &["install", "-v"],
    ];
    for command_line in command_lines {
        assert!(!warning_makes_sense(&arguments(command_line)), "{command_line:?}");
    }
}

#[test]
fn everything_after_a_double_dash_belongs_to_a_script_and_is_not_read() {
    // The verb is `run`, so no warning; and a help word after `--` does not cancel an install.
    assert!(!warning_makes_sense(&arguments(&["run", "x", "--", "install"])));
    assert!(warning_makes_sense(&arguments(&["add", "--", "--help"])));
    assert_eq!(find_verb(&arguments(&["--", "add"])), None);
}

#[test]
fn no_verb_means_no_warning() {
    assert_eq!(find_verb(&arguments(&[])), None);
    assert!(!warning_makes_sense(&arguments(&[])));
    assert!(!warning_makes_sense(&arguments(&["-r", "--no-progress"])));
}
