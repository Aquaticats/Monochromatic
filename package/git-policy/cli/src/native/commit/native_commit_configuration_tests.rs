//! Controls for the configuration native `git commit` carries into the shadow:
//! which keys decide what it writes,
//! the file the shadow's Git reads their values from,
//! and that real Git reads every value from it as it went in.

use super::*;
use std::ffi::OsString;
use std::path::PathBuf;
use crate::argument_limit::one_argument_limit;
use crate::test_support::{fixture, git_output, remove};

/// A snapshot of the listing Git prints for the given `scope`,
/// `key` and optional `value` records,
/// a record without a value being an entry Git printed with nothing after the key.
fn snapshot_records(records: &[(&str, &str, Option<&[u8]>)]) -> ConfigSnapshot {
    let mut listing: Vec<u8> = Vec::new();
    for (scope, key, value) in records {
        listing.extend_from_slice(scope.as_bytes());
        listing.push(0);
        listing.extend_from_slice(key.as_bytes());
        if let Some(text) = value {
            listing.push(b'\n');
            listing.extend_from_slice(text);
        }
        listing.push(0);
    }
    return ConfigSnapshot::parse(listing.as_slice(), true).expect("a listing of the printed shape");
}

/// `snapshot_records` for the text records of a listing.
fn snapshot(records: &[(&str, &str, Option<&str>)]) -> ConfigSnapshot {
    let owned: Vec<(&str, &str, Option<&[u8]>)> = records.iter().map(|(scope, key, value)| { return (*scope, *key, value.map(str::as_bytes)); }).collect();
    return snapshot_records(owned.as_slice());
}

/// The characters Git's configuration format treats specially in a value,
/// each paired with the key it travels under:
/// the tab,
/// the newline,
/// the backslash,
/// the double quote,
/// spaces at either end,
/// and several of them together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cases = [['commit.template', 'unused\ttemplate'], ['commit.cleanup', 'first\nsecond'], ...];
/// ```
const SPECIAL_VALUES: [(&str, &[u8]); 6] = [
    ("commit.template", b"unused\ttemplate"),
    ("commit.cleanup", b"first\nsecond"),
    ("core.commentchar", b"back\\slash"),
    ("user.signingkey", b"dou\"ble"),
    ("i18n.commitencoding", b" leading and trailing "),
    ("commit.signoff", b"tab\tand\nnewline\tboth"),
];

/// Every character the format treats specially reads back from the carried file as it went in,
/// compared by real Git against the original bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [key, value] of cases) assert.equal(await git(['-c', include, 'config', '-z', '--get', key]), value + '\0');
/// ```
#[test]
fn every_character_the_format_treats_specially_reads_back_from_the_carried_file() {
    let root: PathBuf = fixture("native-commit-configuration-specials");
    let records: Vec<(&str, &str, Option<&[u8]>)> = SPECIAL_VALUES
        .iter()
        .map(|(key, value)| { return ("local", *key, Some(*value)); })
        .collect();
    let found: ConfigSnapshot = snapshot_records(records.as_slice());
    let mut query: Vec<OsString> = native_commit_configuration(Some(&found), root.as_path()).expect("carried");
    query.extend([OsString::from("config"), OsString::from("-z"), OsString::from("--get")]);
    for (key, value) in SPECIAL_VALUES {
        let mut asked: Vec<OsString> = query.clone();
        asked.push(OsString::from(key));
        let output: std::process::Output = git_output(root.as_path(), asked.as_slice());
        assert!(output.status.success(), "real Git must read {key}: {}", String::from_utf8_lossy(&output.stderr));
        let mut expected: Vec<u8> = value.to_vec();
        expected.push(0);
        assert_eq!(output.stdout, expected, "{key} reads back as it went in");
    }
    remove(root.as_path());
}

/// An empty subsection is a subsection,
/// not a name starting with a dot:
/// `[trailer ""]` is how Git spells the key `trailer..key`,
/// and no variable name may start with a dot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(await git(['-c', include, 'config', '-z', '--get', 'trailer..key']), Buffer.from('X-Empty\0'));
/// ```
#[test]
fn an_empty_subsection_is_written_as_one() {
    let root: PathBuf = fixture("native-commit-configuration-empty-subsection");
    let found: ConfigSnapshot = snapshot(&[("local", "trailer..key", Some("X-Empty"))]);
    let mut query: Vec<OsString> = native_commit_configuration(Some(&found), root.as_path()).expect("carried");
    let text: String = String::from_utf8(std::fs::read(root.join(NATIVE_COMMIT_CONFIGURATION_NAME)).expect("the carried file"))
        .expect("the file is text");
    assert_eq!(text.lines().collect::<Vec<&str>>(), vec!["[trailer \"\"]", "\tkey = \"X-Empty\""]);
    query.extend([OsString::from("config"), OsString::from("-z"), OsString::from("--get"), OsString::from("trailer..key")]);
    let output: std::process::Output = git_output(root.as_path(), query.as_slice());
    assert!(output.status.success(), "real Git must read trailer..key: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"X-Empty\0");
    remove(root.as_path());
}

/// A setting Git's format cannot state is left out of the carried file rather than written as one Git refuses,
/// so the settings beside it still travel and the commit still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a key with no section, a name with a space or an `=` are all left out, and `commit.verbose` still travels
/// ```
#[test]
fn a_setting_the_format_cannot_state_is_left_out() {
    let root: PathBuf = fixture("native-commit-configuration-unrepresentable");
    let found: ConfigSnapshot = snapshot_records(&[
        ("local", "commitcleanup", Some(b"no section at all")),
        ("local", "commit cleanup", Some(b"a space where a name is read")),
        ("local", "commit.cleanup=verbatim", Some(b"an equals sign in the name")),
        ("local", "commit.verbose", Some(b"kept")),
    ]);
    // The arguments the shadow is given, kept so the production path is the one that writes the file.
    let _arguments: Vec<OsString> = native_commit_configuration(Some(&found), root.as_path()).expect("carried");
    let file: PathBuf = root.join(NATIVE_COMMIT_CONFIGURATION_NAME);
    let text: String = String::from_utf8(std::fs::read(file.as_path()).expect("the carried file")).expect("the file is text");
    assert_eq!(text.lines().collect::<Vec<&str>>(), vec!["[commit]", "\tverbose = \"kept\""]);
    // The file alone, so the listing holds what the file states and not the settings that included it.
    let query: Vec<OsString> = vec![OsString::from("config"), OsString::from("-z"), OsString::from("--file"), file.into_os_string(), OsString::from("--list")];
    let output: std::process::Output = git_output(root.as_path(), query.as_slice());
    assert!(output.status.success(), "real Git must read the file: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"commit.verbose\nkept\0");
    remove(root.as_path());
}

/// A value holding a NUL cannot be stated in Git's format,
/// and Git's own listing cannot carry one either:
/// the listing ends a record at a NUL,
/// so a value holding one is a listing the snapshot parser refuses whole.
/// The writer therefore leaves such a value out rather than writing a file Git refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(pushEntry('commit.template', Buffer.from('kept\0')), false);
/// ```
#[test]
fn a_value_holding_a_nul_cannot_be_stated_and_is_left_out() {
    let mut listing: Vec<u8> = Vec::new();
    listing.extend_from_slice(b"local\0commit.template\nkept\0rest\0");
    assert!(ConfigSnapshot::parse(listing.as_slice(), true).is_none(), "Git's listing cannot hold a NUL in a value");
    let mut text: Vec<u8> = Vec::new();
    assert!(!push_entry(&mut text, "commit.template", Some(b"kept\x00rest")), "a NUL value is left out");
    assert!(text.is_empty(), "nothing is written for it");
}

/// The key criterion takes the signing and message keys,
/// every trailer,
/// and nothing else.
#[test]
fn the_keys_are_the_signing_and_message_keys_and_trailers() {
    for key in [
        "commit.gpgsign",
        "user.signingkey",
        "commit.template",
        "core.commentchar",
        "trailer.reviewed-by.value",
    ] {
        assert!(is_native_commit_key(key), "{key}");
    }
    // Every key `git_gpg_config` in Git 2.56.0's `gpg-interface.c` reads,
    // because a format whose program is missing signs with the default program instead.
    for key in [
        "gpg.format",
        "gpg.program",
        "gpg.openpgp.program",
        "gpg.x509.program",
        "gpg.ssh.program",
        "gpg.mintrustlevel",
        "gpg.ssh.defaultkeycommand",
        "gpg.ssh.allowedsignersfile",
        "gpg.ssh.revocationfile",
    ] {
        assert!(is_native_commit_key(key), "{key}");
    }
    for key in ["core.filemode", "user.name", "commit.gpgsign2", "gpg", "gpgx.format"] {
        assert!(!is_native_commit_key(key), "{key}");
    }
}

/// The carrier names one file,
/// and that file states every entry of every native key in the order Git resolved them,
/// an unrelated key left out,
/// and a key written without a value kept valueless.
#[test]
fn every_entry_of_a_native_key_reaches_the_carried_file_in_order() {
    let root: PathBuf = fixture("native-commit-configuration-order");
    let found: ConfigSnapshot = snapshot(&[
        ("system", "gpg.format", Some("openpgp")),
        ("global", "gpg.program", Some("/legacy/gpg")),
        ("local", "gpg.openpgp.program", Some("/usr/bin/gpg")),
        ("local", "commit.gpgsign", Some("false")),
        ("local", "commit.gpgsign", Some("true")),
        ("local", "user.signingkey", Some("/keys/sign")),
        ("local", "trailer.reviewed-by.value", Some("Fixture")),
        ("local", "commit.verbose", None),
        ("local", "core.filemode", Some("true")),
    ]);
    let arguments: Vec<OsString> = native_commit_configuration(Some(&found), root.as_path()).expect("carried");
    let file: PathBuf = root.join(NATIVE_COMMIT_CONFIGURATION_NAME);
    let mut setting: Vec<u8> = b"include.path=".to_vec();
    setting.extend_from_slice(file.as_os_str().as_encoded_bytes());
    assert_eq!(arguments, vec![OsString::from("-c"), os_from_path_bytes(setting.as_slice())]);
    let text: String = String::from_utf8(std::fs::read(file).expect("the carried file")).expect("the file is text");
    assert_eq!(
        text.lines().collect::<Vec<&str>>(),
        vec![
            "[gpg]",
            "\tformat = \"openpgp\"",
            "[gpg]",
            "\tprogram = \"/legacy/gpg\"",
            "[gpg \"openpgp\"]",
            "\tprogram = \"/usr/bin/gpg\"",
            "[commit]",
            "\tgpgsign = \"false\"",
            "[commit]",
            "\tgpgsign = \"true\"",
            "[user]",
            "\tsigningkey = \"/keys/sign\"",
            "[trailer \"reviewed-by\"]",
            "\tvalue = \"Fixture\"",
            "[commit]",
            "\tverbose",
        ]
    );
    remove(root.as_path());
}

/// Real Git reads every carried value from the file as it went in:
/// a subsection that itself holds an equals sign,
/// a value longer than one argument can carry,
/// and a value that is not text.
#[test]
fn real_git_reads_every_carried_value_from_the_file() {
    let root: PathBuf = fixture("native-commit-configuration-values");
    let long: Vec<u8> = vec![b'x'; one_argument_limit() * 2];
    let not_text: [u8; 3] = [0xff, b'/', b't'];
    let found: ConfigSnapshot = snapshot_records(&[
        ("local", "trailer.review=bot.key", Some(b"Reviewed-by")),
        ("local", "trailer.review=bot.cmd", Some(b"printf reviewer")),
        ("local", "trailer.unused.key", Some(long.as_slice())),
        ("local", "commit.template", Some(&not_text)),
    ]);
    let mut asked: Vec<OsString> = native_commit_configuration(Some(&found), root.as_path()).expect("carried");
    asked.extend([OsString::from("config"), OsString::from("--get")]);
    for (key, expected) in [
        ("trailer.review=bot.key", b"Reviewed-by".as_slice()),
        ("trailer.review=bot.cmd", b"printf reviewer".as_slice()),
        ("trailer.unused.key", long.as_slice()),
        ("commit.template", &not_text[..]),
    ] {
        let mut query: Vec<OsString> = asked.clone();
        query.push(OsString::from(key));
        let output: std::process::Output = git_output(root.as_path(), query.as_slice());
        assert!(output.status.success(), "real Git must read {key}: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout), expected, "{key}");
    }
    remove(root.as_path());
}

/// No snapshot,
/// and a snapshot without a native key,
/// add no setting and write no file,
/// so a command that could not read the configuration still runs.
#[test]
fn no_native_entry_adds_no_setting() {
    let root: PathBuf = fixture("native-commit-configuration-absent");
    let unrelated: ConfigSnapshot = snapshot(&[("local", "core.filemode", Some("true"))]);
    assert_eq!(native_commit_configuration(None, root.as_path()).expect("nothing to carry"), Vec::<OsString>::new());
    assert_eq!(native_commit_configuration(Some(&unrelated), root.as_path()).expect("nothing to carry"), Vec::<OsString>::new());
    assert!(!root.join(NATIVE_COMMIT_CONFIGURATION_NAME).exists(), "no file is written when nothing travels");
    remove(root.as_path());
}

/// A scratch directory that cannot take the file is an error,
/// which the preparation reports instead of starting Git without the person's configuration.
#[test]
fn a_directory_that_cannot_take_the_file_is_an_error() {
    let root: PathBuf = fixture("native-commit-configuration-unwritable");
    let found: ConfigSnapshot = snapshot(&[("local", "commit.cleanup", Some("verbatim"))]);
    assert!(native_commit_configuration(Some(&found), root.join("absent").as_path()).is_err());
    remove(root.as_path());
}
