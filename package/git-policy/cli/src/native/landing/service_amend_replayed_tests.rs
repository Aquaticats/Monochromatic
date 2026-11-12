//! Controls for the commit an amend makes when the branch moved under it.

use super::*;

#[test]
fn a_stored_amend_keeps_its_identities_its_message_and_its_extra_headers() {
    let raw: &[u8] = b"tree 1111111111111111111111111111111111111111\nparent 2222222222222222222222222222222222222222\nauthor A U Thor <a@example.invalid> 1700000000 +0100\ncommitter C O Mitter <c@example.invalid> 1700000100 +0100\nencoding ISO-8859-1\n\nmessage body\n";
    let fields: AmendedCommit = stored_commit_fields(raw).expect("the fields of a commit");
    assert!(matches!(fields.author.as_str(), "A U Thor <a@example.invalid> 1700000000 +0100"));
    assert!(matches!(fields.committer.as_str(), "C O Mitter <c@example.invalid> 1700000100 +0100"));
    assert!(matches!(fields.message.as_slice(), b"message body\n"));
    assert_eq!(fields.extra_headers, vec!["encoding ISO-8859-1".to_string()]);
    assert!(!fields.signed);
}

#[test]
fn a_signature_is_recorded_and_not_carried() {
    let raw: &[u8] = b"tree 1111111111111111111111111111111111111111\nparent 2222222222222222222222222222222222222222\nauthor A <a@example.invalid> 1700000000 +0100\ncommitter C <c@example.invalid> 1700000100 +0100\ngpgsig -----BEGIN PGP SIGNATURE-----\n \n abc\n -----END PGP SIGNATURE-----\n\nmessage\n";
    let fields: AmendedCommit = stored_commit_fields(raw).expect("the fields of a signed commit");
    assert!(fields.signed);
    assert!(fields.extra_headers.is_empty(), "the signature's own lines belong to the signature");
}

#[test]
fn a_commit_without_an_identity_is_no_amend_the_wrapper_can_make() {
    assert_eq!(stored_commit_fields(b"tree 1111111111111111111111111111111111111111\n\nbody\n"), None);
    assert_eq!(stored_commit_fields(b"no header block"), None);
}

#[test]
fn the_message_of_a_service_built_amend_ends_with_one_line_break() {
    let made: Embedded = Embedded {
        message: String::from("subject"),
        author: String::from("A <a@example.invalid> 1700000000 +0100"),
        committer: String::from("C <c@example.invalid> 1700000100 +0100"),
        source: None,
        quiet: true,
    };
    let fields: AmendedCommit = amended_from_embedded(&made);
    assert!(matches!(fields.message.as_slice(), b"subject\n"));
    assert!(fields.quiet());
    assert_eq!(fields.subject(), "subject");
}

/// The signing option of a command line,
/// as the text `git commit-tree` is given.
fn named_signing_option(words: &[&str]) -> Option<String> {
    let arguments: Vec<std::ffi::OsString> = words.iter().map(std::ffi::OsString::from).collect();
    return signing_option(arguments.as_slice()).map(String::from);
}

/// What:
///  The signing option of a command line in the spelling `git commit-tree` takes,
///  including a key named with the long option or inside a short cluster.
/// Why:
///  A commit made again is signed with the key the person chose (rule: what the wrapper passes to
///  Git means what the person typed), and `--gpg-sign=KEY` names one as `-SKEY` does;
///  the long option alone names the default key,
///  which `commit-tree` takes as `-S`.
///  A value that spells the option and a path after the options end are neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(namedSigningOption(['--gpg-sign=KEY'])).toBe(Some('-SKEY'));
/// ```
#[test]
fn the_signing_option_of_a_command_line_is_the_key_it_names() {
    assert_eq!(named_signing_option(&["--message=x", "-SABCDEF"]), Some(String::from("-SABCDEF")));
    assert_eq!(named_signing_option(&["--message=x"]), None);
    // The long option names the default key, and a key with `=`.
    assert_eq!(named_signing_option(&["--gpg-sign"]), Some(String::from("-S")));
    assert_eq!(named_signing_option(&["--gpg-sign=ABCDEF"]), Some(String::from("-SABCDEF")));
    // Every unambiguous prefix of it counts, as it does for Git.
    assert_eq!(named_signing_option(&["--gpg-s=ABCDEF"]), Some(String::from("-SABCDEF")));
    // A short cluster carries the letter, and what follows it is the key.
    assert_eq!(named_signing_option(&["-qSABCDEF"]), Some(String::from("-SABCDEF")));
    // A value that spells the option, and a path after the options end, are neither.
    assert_eq!(named_signing_option(&["-m", "-SABCDEF"]), None);
    assert_eq!(named_signing_option(&["-m", "x", "--", "-SABCDEF"]), None);
    assert_eq!(named_signing_option(&["-m", "x", "--end-of-options", "-SABCDEF"]), None);
    // The last request wins, as it does in Git.
    assert_eq!(named_signing_option(&["-SABCDEF", "--no-gpg-sign"]), None);
}
