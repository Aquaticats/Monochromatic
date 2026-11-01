//! Controls for the service protocol: messages survive a round trip, and every refusal the decoder promises is real.

use super::*;

const BLOB: &str = "0123456789abcdef0123456789abcdef01234567";

fn request() -> CommitRequest {
    return CommitRequest {
        id: String::from("0a1b2c3d-4e5f"),
        repository: PathBuf::from("/work/repo"),
        common_dir: PathBuf::from("/work/repo/.git"),
        reference: String::from("refs/heads/main"),
        base: String::from(BLOB),
        changes: vec![
            Change { path: String::from("src/a \"b\".txt"), mode: ALLOWED_MODES[0], blob: Some(String::from(BLOB)) },
            Change { path: String::from("old.txt"), mode: ALLOWED_MODES[0], blob: None },
        ],
        message: String::from("subject\n\nbody with \"quotes\" and a\ttab\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/work/repo/.git/index"),
    };
}

/// A commit request survives encoding, with awkward text in the path and the message.
#[test]
fn a_commit_request_round_trips() {
    let sent: CommitRequest = request();
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent.clone())));
    assert!(!line.contains('\n'), "one line per message");
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Commit(Box::new(sent))));
}

/// Every service message survives encoding, and every failure code is reachable on the wire.
#[test]
fn service_messages_round_trip() {
    let mut messages: Vec<ServiceMessage> = vec![
        ServiceMessage::Welcome { version: PROTOCOL_VERSION },
        ServiceMessage::Position { number: 3, of: 7 },
        ServiceMessage::Landed { id: String::from("x"), commit: String::from(BLOB), already: true },
        ServiceMessage::RetryPrepare { id: String::from("x"), base: String::from(BLOB) },
    ];
    messages.extend(FailureCode::ALL.map(|code| { return ServiceMessage::Failed { id: String::from("x"), code }; }));
    for message in messages {
        assert_eq!(decode_service(encode_service(&message).as_str()), Ok(message.clone()), "{message:?}");
    }
}

/// Unknown words, wrong versions and truncated or oversized lines are refused, never guessed.
#[test]
fn damaged_lines_are_refused() {
    let good: String = encode_client(&ClientMessage::Commit(Box::new(request())));
    assert!(decode_client(&good[..good.len() / 2]).is_err());
    assert!(decode_client("").is_err());
    assert!(decode_client(good.replace(&format!("\"version\":{PROTOCOL_VERSION}"), "\"version\":999").as_str()).is_err());
    assert!(decode_client(good.replace("\"commit\"", "\"surprise\"").as_str()).is_err());
    assert!(decode_client("x".repeat(MAX_LINE_BYTES + 1).as_str()).is_err());
    assert!(decode_service("{\"type\":\"failed\",\"id\":\"x\",\"code\":\"made-up\"}").is_err());
}

/// Hostile paths, modes, object names and identities are refused.
#[test]
fn unacceptable_requests_are_refused() {
    let hostile_paths = ["/etc/passwd", "../up", "a/../b", "a//b", "a/./b", ".git/config", "dir/.git/x", "", "a\nb"];
    for path in hostile_paths {
        let mut bad: CommitRequest = request();
        bad.changes[0].path = String::from(path);
        assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad))).as_str()).is_err(), "{path:?}");
    }
    let mut bad_mode: CommitRequest = request();
    bad_mode.changes[0].mode = 0o040_000;
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad_mode))).as_str()).is_err());
    let mut bad_blob: CommitRequest = request();
    bad_blob.changes[0].blob = Some(String::from("not-an-object"));
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad_blob))).as_str()).is_err());
    let mut bad_id: CommitRequest = request();
    bad_id.id = String::from("a b");
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad_id))).as_str()).is_err());
    let mut relative: CommitRequest = request();
    relative.repository = PathBuf::from("relative/repo");
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(relative))).as_str()).is_err());
    let mut empty: CommitRequest = request();
    empty.changes.clear();
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(empty))).as_str()).is_ok(), "an empty commit changes no path");
    let mut many: CommitRequest = request();
    many.changes = (0..=MAX_CHANGES).map(|n| { return Change { path: format!("f{n}"), mode: ALLOWED_MODES[0], blob: None }; }).collect();
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(many))).as_str()).is_err());
}

/// The retire message needs the same version as a commit.
#[test]
fn retire_round_trips() {
    assert_eq!(decode_client(encode_client(&ClientMessage::Retire).as_str()), Ok(ClientMessage::Retire));
}

/// A work message survives encoding; a relative or empty directory, and a missing one, are refused.
#[test]
fn a_work_message_round_trips_and_hostile_directories_are_refused() {
    let sent: ClientMessage = ClientMessage::Work { common_dir: PathBuf::from("/srv/repo/.git"), review_key: None };
    assert_eq!(decode_client(encode_client(&sent).as_str()), Ok(sent.clone()));
    let line: String = encode_client(&sent);
    for bad in ["relative/.git", "", "\\u0000/.git"] {
        let hostile: String = line.replace("/srv/repo/.git", bad);
        assert!(decode_client(hostile.as_str()).is_err(), "{bad:?}");
    }
    let missing: String = format!("{{\"type\":\"work\",\"version\":{PROTOCOL_VERSION}}}");
    assert!(decode_client(missing.as_str()).is_err());
}

/// A commit built elsewhere is named whole, alone or with an amend, never together with a tree; a name that is no object is refused.
#[test]
fn a_prebuilt_commit_is_named_whole_and_never_with_a_tree() {
    let mut plain: CommitRequest = request();
    plain.commit = Some(String::from(BLOB));
    let line: String = encode_client(&ClientMessage::Commit(Box::new(plain.clone())));
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Commit(Box::new(plain.clone()))));
    assert!(plain.lands_alone());
    let mut amend: CommitRequest = plain.clone();
    amend.amend = true;
    assert_eq!(decode_client(encode_client(&ClientMessage::Commit(Box::new(amend.clone()))).as_str()), Ok(ClientMessage::Commit(Box::new(amend))));
    let mut with_tree: CommitRequest = plain.clone();
    with_tree.tree = Some(String::from(BLOB));
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(with_tree))).as_str()).is_err(), "a tree beside a prebuilt commit");
    let mut bad: CommitRequest = plain;
    bad.commit = Some(String::from("not-an-object"));
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad))).as_str()).is_err(), "a commit that is no object name");
    assert!(!request().lands_alone());
}

/// An amend names its tree and nothing else does: the two fields survive encoding together, and a request that has one
///  without the other is refused.
#[test]
fn an_amend_carries_its_tree_and_only_an_amend_does() {
    let mut amend: CommitRequest = request();
    amend.amend = true;
    amend.tree = Some(String::from(BLOB));
    let line: String = encode_client(&ClientMessage::Commit(Box::new(amend.clone())));
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Commit(Box::new(amend.clone()))));
    let mut without_tree: CommitRequest = amend.clone();
    without_tree.tree = None;
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(without_tree))).as_str()).is_err(), "an amend without a tree");
    let mut plain_with_tree: CommitRequest = request();
    plain_with_tree.tree = Some(String::from(BLOB));
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(plain_with_tree))).as_str()).is_err(), "a tree without an amend");
    let mut bad_tree: CommitRequest = amend;
    bad_tree.tree = Some(String::from("not-an-object"));
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad_tree))).as_str()).is_err(), "a tree that is no object name");
}

/// A work message carries the review key when it has one and nothing otherwise; a key of another type is refused, and no
/// printed form of the message shows the key.
#[test]
fn a_work_message_carries_the_review_key_and_hides_it_when_printed() {
    const SECRET: &str = "test-key-not-a-secret";
    let with_key: ClientMessage = ClientMessage::Work { common_dir: PathBuf::from("/srv/repo/.git"), review_key: ReviewKey::new(SECRET) };
    let line: String = encode_client(&with_key);
    assert!(line.contains(SECRET), "the key travels in the message");
    assert_eq!(decode_client(line.as_str()), Ok(with_key.clone()));
    assert!(!format!("{with_key:?}").contains(SECRET));
    let without_key: ClientMessage = ClientMessage::Work { common_dir: PathBuf::from("/srv/repo/.git"), review_key: None };
    assert!(!encode_client(&without_key).contains("reviewKey"));
    let wrong_type: String = line.replace(&format!("\"{SECRET}\""), "42");
    assert!(decode_client(wrong_type.as_str()).is_err());
    let empty: String = line.replace(SECRET, "");
    assert_eq!(decode_client(empty.as_str()), Ok(without_key), "an empty key is no key");
}
