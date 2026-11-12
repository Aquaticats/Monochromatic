//! Controls for the caller's command-line configuration,
//! as the pairs the switch reads and the variables the Git LFS upload child reads.

use super::*;
use std::ffi::OsString;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// The value of one variable an answer names,
/// or nothing when it names none of that name.
fn named(variables: &[(OsString, OsString)], name: &str) -> Option<String> {
    return variables
        .iter()
        .find(|(key, _)| {
            return key == name;
        })
        .map(|(_, value)| {
            return value.to_string_lossy().into_owned();
        });
}

/// What:
///  The caller's pairs travel as the entries Git appends to `GIT_CONFIG_PARAMETERS`,
/// after the value the environment already holds,
/// and no numbered pair of the wrapper's own joins them.
/// Why:
///   Git reads that variable after the numbered pairs,
/// so an entry appended here decides over an inherited parameter;
///  numbering the caller's pairs instead let an inherited parameter decide the upload's server.
#[test]
fn the_callers_pairs_are_appended_to_the_parameters_the_environment_holds() {
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from(PARAMETERS_VARIABLE), OsString::from("'inherited.key'='inherited value'")),
        (OsString::from("GIT_CONFIG_COUNT"), OsString::from("3")),
    ];
    let variables = child_configuration_variables(words(&["-c", "a=b"]).as_slice(), environment.as_slice()).expect("no variable is named");
    assert_eq!(
        named(variables.as_slice(), PARAMETERS_VARIABLE).as_deref(),
        Some("'inherited.key'='inherited value' 'a'='b'"),
        "the inherited parameters come first and the caller's pair after them"
    );
    assert_eq!(named(variables.as_slice(), "GIT_CONFIG_COUNT"), None, "the caller's pair takes no numbered slot");
    assert!(child_configuration_variables(&[], &[]).expect("nothing to read").is_empty());
    // A variable the environment holds as the empty text names no entry, as Git reads it.
    let empty: Vec<(OsString, OsString)> = vec![(OsString::from(PARAMETERS_VARIABLE), OsString::new())];
    let from_empty = child_configuration_variables(words(&["-c", "a=b"]).as_slice(), empty.as_slice()).expect("nothing to answer");
    assert_eq!(named(from_empty.as_slice(), PARAMETERS_VARIABLE).as_deref(), Some("'a'='b'"));
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
    for prefix in [
        vec!["--config-env=remote.origin.pushurl=POLICY_PUSH_URL"],
        vec!["--config-env", "remote.origin.pushurl=POLICY_PUSH_URL"],
    ] {
        let answered = child_configuration_variables(words(prefix.as_slice()).as_slice(), environment.as_slice()).expect("the environment answers");
        assert_eq!(
            named(answered.as_slice(), PARAMETERS_VARIABLE).as_deref(),
            Some("'remote.origin.pushurl'='http://example.invalid/x'"),
            "{prefix:?}"
        );
    }
    // The last `=` names the variable and leaves the key before it, as Git reads the option.
    let split_at_last = command_line_pairs(words(&["--config-env=probe.key=x=POLICY_PUSH_URL"]).as_slice(), environment.as_slice())
        .expect("the environment answers");
    assert_eq!(split_at_last, vec![(OsString::from("probe.key=x"), OsString::from("http://example.invalid/x"))]);
    // A caller's `-c` keeps everything after its first `=`, and the pairs keep the order the caller gave them.
    let both_spellings = child_configuration_variables(
        words(&["-c", "first=a=b", "--config-env=second=POLICY_PUSH_URL"]).as_slice(),
        environment.as_slice(),
    )
    .expect("the environment answers");
    assert_eq!(
        named(both_spellings.as_slice(), PARAMETERS_VARIABLE).as_deref(),
        Some("'first'='a=b' 'second'='http://example.invalid/x'")
    );
    // Nothing answers for a variable the environment does not hold, a parameter with no `=`, an empty variable name, or an
    //  option with no value at all.
    for prefix in [
        vec!["--config-env=probe.key=MISSING"],
        vec!["--config-env=probe.key"],
        vec!["--config-env=probe.key="],
        vec!["--config-env"],
    ] {
        assert!(child_configuration_variables(words(prefix.as_slice()).as_slice(), environment.as_slice()).is_none(), "{prefix:?}");
        assert!(command_line_pairs(words(prefix.as_slice()).as_slice(), environment.as_slice()).is_none(), "{prefix:?}");
    }
}

/// What:
///  A setting holding a space or a single quote is quoted as Git quotes one,
/// so the child reads the value the caller gave.
/// Why:
///   The value is a word of a quoted list,
/// and an unquoted one would end at its space or lose its quote:
///  a push URL or a header with either would reach `git-lfs` as another value.
#[test]
fn a_setting_holding_a_space_or_a_quote_is_quoted_as_git_quotes_it() {
    let variables = child_configuration_variables(
        words(&["-c", "probe.spaced=http://a b.invalid/x", "-c", "probe.quoted=it's"]).as_slice(),
        &[],
    )
    .expect("nothing comes from a variable");
    assert_eq!(
        named(variables.as_slice(), PARAMETERS_VARIABLE).as_deref(),
        Some("'probe.spaced'='http://a b.invalid/x' 'probe.quoted'='it'\\''s'")
    );
}

/// What:
///  The directory,
/// worktree and namespace options become their variables,
/// in both spellings Git accepts.
/// Why:
///   The upload child starts `git-lfs`,
/// which asks its own Git for the repository,
/// and without these variables it would read another one.
#[test]
fn the_directory_options_become_their_variables() {
    let variables = child_configuration_variables(
        words(&["--git-dir=/g", "--work-tree", "/w", "--namespace=ns", "-C", "/elsewhere"]).as_slice(),
        &[],
    )
    .expect("no variable is named");
    assert_eq!(named(variables.as_slice(), "GIT_DIR").as_deref(), Some("/g"));
    assert_eq!(named(variables.as_slice(), "GIT_WORK_TREE").as_deref(), Some("/w"));
    assert_eq!(named(variables.as_slice(), "GIT_NAMESPACE").as_deref(), Some("ns"));
    assert_eq!(named(variables.as_slice(), PARAMETERS_VARIABLE), None, "a directory option sets no configuration");
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
    let variables = child_configuration_variables(vec![token].as_slice(), environment.as_slice()).expect("the environment answers");
    assert_eq!(
        named(variables.as_slice(), PARAMETERS_VARIABLE).as_deref(),
        Some("'remote.origin.pushurl'='http://example.invalid/x'")
    );
}

/// What:
///  A `-c` setting whose name or value is not text is carried as the bytes the caller gave.
/// Why:
///   Git applies those bytes,
/// and a decoded setting would hand `git-lfs` another key or another value than the push used.
#[cfg(unix)]
#[test]
fn a_setting_that_is_not_text_is_carried_as_its_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let mut setting: Vec<u8> = b"probe.key=".to_vec();
    setting.extend_from_slice(b"a\xffb");
    let carried = child_configuration_variables(vec![OsString::from("-c"), OsString::from_vec(setting)].as_slice(), &[])
        .expect("nothing comes from a variable");
    let rendered = carried
        .iter()
        .find(|(key, _)| {
            return key == PARAMETERS_VARIABLE;
        })
        .map(|(_, value)| {
            return value.as_encoded_bytes().to_vec();
        })
        .expect("the parameters are named");
    let mut expected: Vec<u8> = b"'probe.key'='a".to_vec();
    expected.push(0xff);
    expected.extend_from_slice(b"b'");
    assert_eq!(rendered, expected);
}
