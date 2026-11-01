//! Controls for raw commit objects:
//!  parsing, the byte-exact unsigned rewrite, dropped headers, and identity variables.

use super::*;

/// A commit with the given extra header lines between `committer` and the message.
fn commit_bytes(extra: &str, message: &[u8]) -> Vec<u8> {
    // `mut` lets the extra header lines and the blank separator be appended in order.
    let mut header: String = String::from(
        "tree 1111111111111111111111111111111111111111\nparent 2222222222222222222222222222222222222222\nauthor Ada Lovelace <ada@example.test> 1700000000 +0100\ncommitter Grace Hopper <grace@example.test> 1700000100 -0500\n",
    );
    header.push_str(extra);
    if !extra.is_empty() {
        header.push('\n');
    }
    header.push('\n');
    let mut bytes: Vec<u8> = header.into_bytes();
    bytes.extend_from_slice(message);
    return bytes;
}

/// Tree, encoding-free headers, and the message parse exactly as the object holds them.
#[test]
fn parsing_reads_the_tree_and_keeps_the_message_bytes() {
    let commit: RawCommit = parse_raw_commit(&commit_bytes("", b"subject\n\nbody\n")).expect("commit");
    assert_eq!(commit.tree_oid, "1111111111111111111111111111111111111111");
    assert_eq!(commit.message, b"subject\n\nbody\n".to_vec());
    assert_eq!(commit.encoding, None);
    assert!(!commit.signed);
    assert_eq!(commit.headers.len(), 4);
}

/// A signed commit is recognized by its signature header, whose continuation lines stay attached.
#[test]
fn a_signature_header_keeps_its_continuation_lines() {
    let extra: &str = "gpgsig -----BEGIN SSH SIGNATURE-----\n abc\n -----END SSH SIGNATURE-----";
    let commit: RawCommit = parse_raw_commit(&commit_bytes(extra, b"signed\n")).expect("commit");
    assert!(commit.signed);
    let signature: &RawCommitHeader = commit.headers.iter().find(|header| { return header.name == "gpgsig"; }).expect("gpgsig");
    assert!(signature.bytes.ends_with(b" -----END SSH SIGNATURE-----\n"));
    assert!(parse_raw_commit(&commit_bytes("gpgsig-sha256 x", b"m\n")).expect("commit").signed);
}

/// An `encoding` header is read as its value, and a non-UTF-8 message stays exact.
#[test]
fn an_encoding_header_is_read_and_the_message_stays_exact() {
    let message: &[u8] = b"caf\xe9\n";
    let commit: RawCommit = parse_raw_commit(&commit_bytes("encoding ISO-8859-1", message)).expect("commit");
    assert_eq!(commit.encoding.as_deref(), Some("ISO-8859-1"));
    assert_eq!(commit.message, message.to_vec());
}

/// An object without a blank line has no message; one without a tree header fails.
#[test]
fn objects_without_a_blank_line_or_a_tree_are_handled() {
    let headless: RawCommit = parse_raw_commit(b"tree 3333\n").expect("commit");
    assert!(headless.message.is_empty());
    let error: ReplayObjectError = parse_raw_commit(b"author x <x@y> 1 +0000\n\nm\n").expect_err("no tree");
    assert_eq!(error.0, "Commit object has no tree header.");
    assert!(parse_raw_commit(b"").is_err());
}

/// The rewrite replaces `tree` and `parent`, keeps every other header and the message bytes,
///  and emits the new parent right after the tree.
#[test]
fn rewriting_replaces_only_tree_and_parent() {
    let original: Vec<u8> = commit_bytes("x-custom  value\n second", b"caf\xe9 message\n");
    let commit: RawCommit = parse_raw_commit(&original).expect("commit");
    let rewritten: Vec<u8> = rewrite_commit_object(&commit, "4444", "5555");
    let text: String = String::from_utf8_lossy(&rewritten).into_owned();
    assert!(text.starts_with("tree 4444\nparent 5555\nauthor Ada Lovelace"), "{text}");
    assert!(rewritten.ends_with(b"\ncaf\xe9 message\n"));
    assert!(text.contains("x-custom  value\n second\n"));
    assert!(!text.contains("2222"));
}

/// An unborn branch has no parent header in the prepared object, so the rewrite inserts one after the tree.
#[test]
fn an_unborn_parent_is_inserted_after_the_tree() {
    let bytes: Vec<u8> = b"tree 1111\nauthor A <a@b> 1 +0000\ncommitter A <a@b> 1 +0000\n\nm\n".to_vec();
    let commit: RawCommit = parse_raw_commit(&bytes).expect("commit");
    let rewritten: Vec<u8> = rewrite_commit_object(&commit, "9999", "8888");
    assert!(rewritten.starts_with(b"tree 9999\nparent 8888\nauthor A"));
}

/// Custom headers are named once each, in object order; rebuilt headers are never listed.
#[test]
fn dropped_headers_are_the_custom_names_in_order() {
    let commit: RawCommit = parse_raw_commit(&commit_bytes("zeta a\nalpha b\nzeta c\ngpgsig x", b"m\n")).expect("commit");
    assert_eq!(dropped_replay_headers(&commit), vec![String::from("zeta"), String::from("alpha")]);
    let plain: RawCommit = parse_raw_commit(&commit_bytes("", b"m\n")).expect("commit");
    assert!(dropped_replay_headers(&plain).is_empty());
}

/// The author variables carry the name without trailing space, the email, and the date as `@<seconds> <zone>`.
#[test]
fn identity_variables_follow_the_raw_header() {
    let commit: RawCommit = parse_raw_commit(&commit_bytes("", b"m\n")).expect("commit");
    assert_eq!(
        identity_environment(&commit, "author").expect("author"),
        vec![
            (String::from("GIT_AUTHOR_NAME"), String::from("Ada Lovelace")),
            (String::from("GIT_AUTHOR_EMAIL"), String::from("ada@example.test")),
            (String::from("GIT_AUTHOR_DATE"), String::from("@1700000000 +0100")),
        ]
    );
    let committer: Vec<(String, String)> = identity_environment(&commit, "committer").expect("committer");
    assert_eq!(committer[2].1, "@1700000100 -0500");
}

/// A missing identity header or one without an email fails.
#[test]
fn malformed_identities_fail() {
    let bare: RawCommit = parse_raw_commit(b"tree 1\ncommitter A 1 +0000\n\nm\n").expect("commit");
    assert_eq!(
        identity_environment(&bare, "committer").expect_err("no email").0,
        "Commit committer header is malformed: \"A 1 +0000\""
    );
    assert_eq!(
        identity_environment(&bare, "author").expect_err("absent").0,
        "Commit object has no author header."
    );
    let open_only: RawCommit = parse_raw_commit(b"tree 1\nauthor A <a 1 +0000\n\nm\n").expect("commit");
    assert!(identity_environment(&open_only, "author").is_err());
}

/// A header with no value keeps its whole name, so the name is the bytes before the newline.
#[test]
fn a_header_without_a_value_keeps_its_whole_name() {
    let commit: RawCommit = parse_raw_commit(b"tree 1111\nfoo\n\nmessage\n").expect("commit");
    assert_eq!(dropped_replay_headers(&commit), vec![String::from("foo")]);
}

/// The object error displays exactly its message.
#[test]
fn the_object_error_displays_its_message() {
    let error: ReplayObjectError = ReplayObjectError(String::from("Commit object has no tree header."));
    assert_eq!(error.to_string(), "Commit object has no tree header.");
}
