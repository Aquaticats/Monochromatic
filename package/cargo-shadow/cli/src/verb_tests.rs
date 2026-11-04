//! The unit tests of the command-word rules:
//! which cargo command lines get the install step.

use crate::verb::{CATALOG_WARNING_VERBS, INSTALL_STEP_VERBS, Treatment, VALUE_OPTIONS, find_verb, treatment_of};

use std::ffi::OsString;

fn arguments(words: &[&str]) -> Vec<OsString> {
    let mut list: Vec<OsString> = Vec::new();
    for word in words {
        list.push(OsString::from(word));
    }
    return list;
}

#[test]
fn every_resolving_verb_alone_gets_the_install_step() {
    assert!(!INSTALL_STEP_VERBS.is_empty());
    for verb in INSTALL_STEP_VERBS {
        assert_eq!(treatment_of(&arguments(&[verb])), Treatment::InstallStep, "verb {verb}");
    }
}

#[test]
fn the_short_aliases_of_the_builtin_commands_get_the_install_step() {
    for alias in ["b", "c", "d", "r", "t"] {
        assert_eq!(treatment_of(&arguments(&[alias])), Treatment::InstallStep, "alias {alias}");
    }
}

#[test]
fn add_and_remove_with_their_aliases_get_the_catalog_warning() {
    assert!(!CATALOG_WARNING_VERBS.is_empty());
    for verb in CATALOG_WARNING_VERBS {
        assert_eq!(treatment_of(&arguments(&[verb])), Treatment::CatalogWarning, "verb {verb}");
    }
}

#[test]
fn a_command_that_resolves_nothing_passes_straight_through() {
    let verbs: [&str; 14] = [
        "fmt", "clean", "locate-project", "pkgid", "read-manifest", "verify-project", "config", "new", "init", "search",
        "install", "uninstall", "login", "unknown-alias",
    ];
    for verb in verbs {
        assert_eq!(treatment_of(&arguments(&[verb])), Treatment::PassThrough, "verb {verb}");
    }
}

#[test]
fn the_lock_writers_of_cargo_itself_are_answered_as_the_pnpm_commands_they_stand_for() {
    assert_eq!(treatment_of(&arguments(&["update"])), Treatment::PnpmUpdate);
    assert_eq!(treatment_of(&arguments(&["generate-lockfile"])), Treatment::PnpmUpdate);
    assert_eq!(treatment_of(&arguments(&["update", "--help"])), Treatment::PassThrough);
}

#[test]
fn a_leading_toolchain_word_and_global_options_are_skipped() {
    let command_lines: [&[&str]; 8] = [
        &["+nightly", "check"],
        &["+1.98.1", "-q", "build"],
        &["--color", "never", "test"],
        &["--color=never", "test"],
        &["-C", "../x", "check"],
        &["--config", "build.jobs=2", "check"],
        &["-Z", "unstable-options", "check"],
        &["-Zunstable-options", "check"],
    ];
    for command_line in command_lines {
        assert_eq!(treatment_of(&arguments(command_line)), Treatment::InstallStep, "{command_line:?}");
    }
}

#[test]
fn the_value_of_a_value_option_is_never_taken_for_the_verb() {
    // `check` here is the value of `--config`, so the verb is `fmt`.
    assert_eq!(find_verb(&arguments(&["--config", "check", "fmt"])), Some("fmt".to_string()));
    assert_eq!(treatment_of(&arguments(&["--config", "check", "fmt"])), Treatment::PassThrough);
}

#[test]
fn every_value_option_consumes_exactly_the_next_word() {
    assert!(!VALUE_OPTIONS.is_empty());
    for option in VALUE_OPTIONS {
        assert_eq!(find_verb(&arguments(&[option, "value", "check"])), Some("check".to_string()), "option {option}");
    }
}

#[test]
fn a_help_or_version_word_before_the_double_dash_cancels_everything() {
    let command_lines: [&[&str]; 6] = [
        &["check", "--help"],
        &["build", "-h"],
        &["--help", "check"],
        &["check", "--version"],
        &["-V"],
        &["help", "check"],
    ];
    for command_line in command_lines {
        assert_eq!(treatment_of(&arguments(command_line)), Treatment::PassThrough, "{command_line:?}");
    }
}

#[test]
fn everything_after_a_double_dash_belongs_to_the_program_and_is_not_read() {
    assert_eq!(treatment_of(&arguments(&["run", "--", "--help"])), Treatment::InstallStep);
    assert_eq!(treatment_of(&arguments(&["fmt", "--", "check"])), Treatment::PassThrough);
    assert_eq!(find_verb(&arguments(&["--", "check"])), None);
}

#[test]
fn metadata_resolves_unless_it_is_asked_for_no_dependencies() {
    assert_eq!(treatment_of(&arguments(&["metadata"])), Treatment::InstallStep);
    assert_eq!(treatment_of(&arguments(&["metadata", "--no-deps"])), Treatment::PassThrough);
}

#[test]
fn no_verb_means_pass_through() {
    assert_eq!(find_verb(&arguments(&[])), None);
    assert_eq!(treatment_of(&arguments(&[])), Treatment::PassThrough);
    assert_eq!(treatment_of(&arguments(&["-q", "--locked"])), Treatment::PassThrough);
}
