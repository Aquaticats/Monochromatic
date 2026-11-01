//! Controls for the replay options: every branch of the scan changes the answer.

use super::*;

/// Arguments as owned text.
fn args(values: &[&str]) -> Vec<String> {
    return values.iter().map(|value| { return String::from(*value); }).collect();
}

/// `-n` clusters and the long forms decide `no_verify`; the latest decision wins.
#[test]
fn no_verify_follows_the_latest_decision() {
    assert!(replay_options(&args(&["-n", "-m", "x"])).no_verify);
    assert!(replay_options(&args(&["-an", "-m", "x"])).no_verify);
    assert!(replay_options(&args(&["--no-verify", "-m", "x"])).no_verify);
    assert!(!replay_options(&args(&["--no-verify", "--verify"])).no_verify);
    assert!(!replay_options(&args(&["-m", "x"])).no_verify);
}

/// A message that looks like a flag is a value, not an option.
#[test]
fn an_option_value_is_never_read_as_an_option() {
    assert!(!replay_options(&args(&["-m", "-n"])).no_verify);
    assert!(!replay_options(&args(&["--message", "--no-verify"])).no_verify);
    assert!(!replay_options(&args(&["--author", "--no-verify"])).no_verify);
}

/// Options after the pathspec separator are pathspecs.
#[test]
fn the_pathspec_separator_ends_the_options() {
    assert!(!replay_options(&args(&["-m", "x", "--", "--no-verify"])).no_verify);
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
    assert_eq!(replay_options(&args(&["-nSabc"])).signing_key, None);
    assert!(replay_options(&args(&["-nSabc"])).no_verify);
}

/// A later message option keeps the key, a later bare `-S` clears it (the incumbent's rule), and an option value is
///  consumed once: the token after it is an option again.
#[test]
fn later_options_keep_or_clear_the_key_and_a_value_is_consumed_once() {
    assert_eq!(replay_options(&args(&["--gpg-sign=k", "-m", "x"])).signing_key, Some(String::from("k")));
    assert_eq!(replay_options(&args(&["--gpg-sign=k", "-S"])).signing_key, None);
    assert!(replay_options(&args(&["-m", "x", "-n"])).no_verify);
    assert!(replay_options(&args(&["--message", "x", "--no-verify"])).no_verify);
}

/// A pathspec before an option is not a cluster: it consumes nothing.
#[test]
fn a_pathspec_is_not_a_cluster() {
    assert!(replay_options(&args(&["a.txt", "-n"])).no_verify);
    assert!(replay_options(&args(&["tmp", "-n"])).no_verify);
    assert!(replay_options(&args(&["-", "-n"])).no_verify);
}

/// A long option is never read as a cluster of short letters: `--no-edit` and `--dry-run` contain an `n`.
#[test]
fn a_long_option_is_not_a_cluster() {
    assert!(!replay_options(&args(&["--no-edit"])).no_verify);
    assert!(!replay_options(&args(&["--dry-run", "-m", "x"])).no_verify);
}
