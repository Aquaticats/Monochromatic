//! Controls for the configuration native `git commit` carries into the shadow:
//! which keys decide what it writes,
//! and which value of each reaches the command line.

use super::*;
use std::ffi::OsString;

/// A snapshot of the listing Git prints for the given `scope`,
/// `key` and optional `value` records.
fn snapshot(records: &[(&str, &str, Option<&str>)]) -> ConfigSnapshot {
    let mut listing: Vec<u8> = Vec::new();
    for (scope, key, value) in records {
        listing.extend_from_slice(scope.as_bytes());
        listing.push(0);
        listing.extend_from_slice(key.as_bytes());
        if let Some(text) = value {
            listing.push(b'\n');
            listing.extend_from_slice(text.as_bytes());
        }
        listing.push(0);
    }
    return ConfigSnapshot::parse(listing.as_slice(), true).expect("a listing of the printed shape");
}

/// The key criterion takes the signing and message keys,
/// every trailer,
/// and nothing else.
#[test]
fn the_keys_are_the_signing_and_message_keys_and_trailers() {
    for key in ["commit.gpgsign", "user.signingkey", "gpg.format", "gpg.openpgp.program", "commit.template", "core.commentchar", "trailer.reviewed-by.value"] {
        assert!(is_native_commit_key(key), "{key}");
    }
    for key in ["core.filemode", "user.name", "commit.gpgsign2", "gpg.ssh.allowedsignersfile.bak"] {
        assert!(!is_native_commit_key(key), "{key}");
    }
}

/// Every entry of a native key travels in the order Git resolved it,
/// an unrelated key is left out,
/// and a key written without a value stays valueless.
#[test]
fn every_entry_of_a_native_key_reaches_the_command_line_in_order() {
    let found: ConfigSnapshot = snapshot(&[
        ("system", "gpg.format", Some("openpgp")),
        ("local", "commit.gpgsign", Some("false")),
        ("local", "commit.gpgsign", Some("true")),
        ("local", "user.signingkey", Some("/keys/sign")),
        ("local", "gpg.format", Some("ssh")),
        ("local", "trailer.reviewed-by.value", Some("Fixture")),
        ("local", "commit.verbose", None),
        ("local", "core.filemode", Some("true")),
    ]);
    assert_eq!(
        native_commit_configuration(Some(&found)),
        vec![
            OsString::from("-c"),
            OsString::from("gpg.format=openpgp"),
            OsString::from("-c"),
            OsString::from("commit.gpgsign=false"),
            OsString::from("-c"),
            OsString::from("commit.gpgsign=true"),
            OsString::from("-c"),
            OsString::from("user.signingkey=/keys/sign"),
            OsString::from("-c"),
            OsString::from("gpg.format=ssh"),
            OsString::from("-c"),
            OsString::from("trailer.reviewed-by.value=Fixture"),
            OsString::from("-c"),
            OsString::from("commit.verbose"),
        ]
    );
}

/// No snapshot adds no setting,
/// so a command that could not read the configuration still runs.
#[test]
fn no_snapshot_adds_no_setting() {
    assert_eq!(native_commit_configuration(None), Vec::<OsString>::new());
}
