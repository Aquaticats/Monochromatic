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
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(empty))).as_str()).is_err());
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
    let sent: ClientMessage = ClientMessage::Work { common_dir: PathBuf::from("/srv/repo/.git") };
    assert_eq!(decode_client(encode_client(&sent).as_str()), Ok(sent.clone()));
    let line: String = encode_client(&sent);
    for bad in ["relative/.git", "", "\\u0000/.git"] {
        let hostile: String = line.replace("/srv/repo/.git", bad);
        assert!(decode_client(hostile.as_str()).is_err(), "{bad:?}");
    }
    let missing: String = format!("{{\"type\":\"work\",\"version\":{PROTOCOL_VERSION}}}");
    assert!(decode_client(missing.as_str()).is_err());
}
