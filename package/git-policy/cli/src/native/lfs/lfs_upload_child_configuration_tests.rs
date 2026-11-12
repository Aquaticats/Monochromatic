//! Controls for the caller's command-line configuration,
//! as the variables the Git LFS upload child reads.

use super::*;
use std::ffi::OsString;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// Pairs the environment already holds come first,
/// so the global options win as they do in Git.
#[test]
fn the_variables_continue_after_the_environments_own_pairs() {
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_CONFIG_COUNT"), OsString::from("3"))];
    let variables = prefix_variables(words(&["-c", "a=b"]).as_slice(), environment.as_slice()).expect("no configuration comes from a variable");
    let named = |name: &str| -> Option<String> {
        return variables
            .iter()
            .find(|(key, _)| {
                return key == name;
            })
            .map(|(_, value)| {
                return value.to_string_lossy().into_owned();
            });
    };
    assert_eq!(named("GIT_CONFIG_COUNT").as_deref(), Some("4"));
    assert_eq!(named("GIT_CONFIG_KEY_3").as_deref(), Some("a"));
    assert_eq!(named("GIT_CONFIG_VALUE_3").as_deref(), Some("b"));
    assert!(prefix_variables(&[], &[]).expect("nothing to read").is_empty());
}

/// What:
///  A configuration the environment names becomes a pair whose variable is the text after the last `=`.
/// Why:
///   Git 2.56.0 reads `--config-env=<name>=<envvar>` that way,
/// leaving an invalid key before a second `=`
///  (`command_status_tests.rs` holds the real-Git control),
/// and a pair the caller spells with `-c` keeps everything after its first `=`.
///  The two spellings keep the order the caller gave them.
#[test]
fn a_configuration_from_the_environment_becomes_a_pair() {
    let environment: Vec<(OsString, OsString)> =
        vec![(OsString::from("POLICY_PUSH_URL"), OsString::from("http://example.invalid/x"))];
    let named = |variables: &[(OsString, OsString)], name: &str| -> Option<String> {
        return variables
            .iter()
            .find(|(key, _)| {
                return key == name;
            })
            .map(|(_, value)| {
                return value.to_string_lossy().into_owned();
            });
    };
    for prefix in [
        vec!["--config-env=remote.origin.pushurl=POLICY_PUSH_URL"],
        vec!["--config-env", "remote.origin.pushurl=POLICY_PUSH_URL"],
    ] {
        let answered = prefix_variables(words(prefix.as_slice()).as_slice(), environment.as_slice()).expect("the environment answers");
        assert_eq!(named(answered.as_slice(), "GIT_CONFIG_COUNT").as_deref(), Some("1"), "{prefix:?}");
        assert_eq!(named(answered.as_slice(), "GIT_CONFIG_KEY_0").as_deref(), Some("remote.origin.pushurl"), "{prefix:?}");
        assert_eq!(named(answered.as_slice(), "GIT_CONFIG_VALUE_0").as_deref(), Some("http://example.invalid/x"), "{prefix:?}");
    }
    // The last `=` names the variable and leaves the key before it, as Git reads the option.
    let split_at_last = prefix_variables(words(&["--config-env=probe.key=x=POLICY_PUSH_URL"]).as_slice(), environment.as_slice())
        .expect("the environment answers");
    assert_eq!(named(split_at_last.as_slice(), "GIT_CONFIG_KEY_0").as_deref(), Some("probe.key=x"));
    // A caller's `-c` keeps everything after its first `=`, and the pairs keep the order the caller gave them.
    let both_spellings = prefix_variables(
        words(&["-c", "first=a=b", "--config-env=second=POLICY_PUSH_URL"]).as_slice(),
        environment.as_slice(),
    )
    .expect("the environment answers");
    assert_eq!(named(both_spellings.as_slice(), "GIT_CONFIG_COUNT").as_deref(), Some("2"));
    assert_eq!(named(both_spellings.as_slice(), "GIT_CONFIG_KEY_0").as_deref(), Some("first"));
    assert_eq!(named(both_spellings.as_slice(), "GIT_CONFIG_VALUE_0").as_deref(), Some("a=b"));
    assert_eq!(named(both_spellings.as_slice(), "GIT_CONFIG_KEY_1").as_deref(), Some("second"));
    // Nothing answers for a variable the environment does not hold, a parameter with no `=`, an empty variable name, or an
    //  option with no value at all.
    for prefix in [
        vec!["--config-env=probe.key=MISSING"],
        vec!["--config-env=probe.key"],
        vec!["--config-env=probe.key="],
        vec!["--config-env"],
    ] {
        assert!(prefix_variables(words(prefix.as_slice()).as_slice(), environment.as_slice()).is_none(), "{prefix:?}");
    }
}

/// What:
///  A variable whose name is not text is answered for by the bytes the operating system holds,
/// as Git answers it.
/// Why:
///   Decoding the name would answer for another variable or for none,
/// and either answer is a configuration the
///  upload never receives.
#[cfg(unix)]
#[test]
fn a_variable_name_that_is_not_text_is_answered_for_by_its_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let awkward: Vec<u8> = b"POLICY\xff_URL".to_vec();
    let environment: Vec<(OsString, OsString)> =
        vec![(OsString::from_vec(awkward.clone()), OsString::from("http://example.invalid/x"))];
    let mut token: OsString = OsString::from("--config-env=remote.origin.pushurl=");
    token.push(OsString::from_vec(awkward));
    let variables = prefix_variables(vec![token].as_slice(), environment.as_slice()).expect("the environment answers");
    let named = |name: &str| -> Option<String> {
        return variables
            .iter()
            .find(|(key, _)| {
                return key == name;
            })
            .map(|(_, value)| {
                return value.to_string_lossy().into_owned();
            });
    };
    assert_eq!(named("GIT_CONFIG_KEY_0").as_deref(), Some("remote.origin.pushurl"));
    assert_eq!(named("GIT_CONFIG_VALUE_0").as_deref(), Some("http://example.invalid/x"));
}
