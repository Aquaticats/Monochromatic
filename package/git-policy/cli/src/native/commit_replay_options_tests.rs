//! Controls for the replay options: every branch of the scan changes the answer.

use super::*;

/// Arguments as owned text.
fn args(values: &[&str]) -> Vec<String> {
    return values.iter().map(|value| { return String::from(*value); }).collect();
}

/// A signing option that looks like a flag but is a message or author value is a value, not an option.
#[test]
fn an_option_value_is_never_read_as_an_option() {
    assert_eq!(replay_options(&args(&["-m", "--gpg-sign=k"])).signing_key, None);
    assert_eq!(replay_options(&args(&["--message", "--gpg-sign=k"])).signing_key, None);
    assert_eq!(replay_options(&args(&["--author", "--gpg-sign=k"])).signing_key, None);
}

/// Options after the pathspec separator are pathspecs.
#[test]
fn the_pathspec_separator_ends_the_options() {
    assert_eq!(replay_options(&args(&["-m", "x", "--", "--gpg-sign=k"])).signing_key, None);
    assert_eq!(replay_options(&args(&["--", "--gpg-sign=key"])).signing_key, None);
}

/// The signing key comes from `--gpg-sign=<key>` and is absent for a bare `-S` or `--gpg-sign`.
///  The incumbent's argument normalization splits an attached `-S<key>` into `-S` and a separate token, so it
///  reads no key from that spelling either; this control pins that shared behavior (checked against the incumbent).
#[test]
fn the_signing_key_is_read_from_the_long_spelling_only() {
    assert_eq!(replay_options(&args(&["--gpg-sign=def"])).signing_key, Some(String::from("def")));
    assert_eq!(replay_options(&args(&["-S"])).signing_key, None);
    assert_eq!(replay_options(&args(&["--gpg-sign"])).signing_key, None);
    assert_eq!(replay_options(&args(&["-Sabc", "-m", "x"])).signing_key, None);
    assert_eq!(replay_options(&args(&["-aSabc"])).signing_key, None);
}

/// A later message option keeps the key, a later bare `-S` clears it (the incumbent's rule), and an option value is
///  consumed once: the token after it is an option again.
#[test]
fn later_options_keep_or_clear_the_key_and_a_value_is_consumed_once() {
    assert_eq!(replay_options(&args(&["--gpg-sign=k", "-m", "x"])).signing_key, Some(String::from("k")));
    assert_eq!(replay_options(&args(&["--gpg-sign=k", "-S"])).signing_key, None);
    assert_eq!(replay_options(&args(&["-m", "x", "--gpg-sign=k"])).signing_key, Some(String::from("k")));
}

/// A pathspec before an option is not a cluster: it consumes nothing.
#[test]
fn a_pathspec_is_not_a_cluster() {
    assert_eq!(replay_options(&args(&["a.txt", "--gpg-sign=k"])).signing_key, Some(String::from("k")));
    assert_eq!(replay_options(&args(&["-", "--gpg-sign=k"])).signing_key, Some(String::from("k")));
}

/// A long option is never read as a cluster of short letters: `--no-edit` and `--dry-run` contain an `n`.
#[test]
fn a_long_option_is_not_a_cluster() {
    assert_eq!(replay_options(&args(&["--no-edit", "--gpg-sign=k"])).signing_key, Some(String::from("k")));
    assert_eq!(replay_options(&args(&["--dry-run", "-m", "x"])).signing_key, None);
}
