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

#[test]
fn the_signing_option_of_a_command_line_is_the_key_it_names() {
    let named: Vec<std::ffi::OsString> = vec![std::ffi::OsString::from("--message=x"), std::ffi::OsString::from("-SABCDEF")];
    assert_eq!(signing_option(named.as_slice()), Some("-SABCDEF"));
    let plain: Vec<std::ffi::OsString> = vec![std::ffi::OsString::from("--message=x")];
    assert_eq!(signing_option(plain.as_slice()), None);
    let long: Vec<std::ffi::OsString> = vec![std::ffi::OsString::from("--gpg-sign")];
    assert_eq!(signing_option(long.as_slice()), None, "a long option is not the short spelling `commit-tree` takes");
}
