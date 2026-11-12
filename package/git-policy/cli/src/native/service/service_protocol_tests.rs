//! Controls for the service protocol:
//! messages survive a round trip,
//! and every refusal the decoder promises is real.

use super::*;
use crate::command_test_support::{fixture, remove};
use crate::service_protocol::{LandingFigures, LandingSync};
use crate::service_review_key::ReviewKey;
use std::path::PathBuf;

const BLOB: &str = "0123456789abcdef0123456789abcdef01234567";

fn request() -> CommitRequest {
    return CommitRequest {
        id: String::from("0a1b2c3d-4e5f"),
        repository: PathBuf::from("/work/repo"),
        common_dir: PathBuf::from("/work/repo/.git"),
        git_dir: PathBuf::from("/work/repo/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: String::from(BLOB),
        changes: vec![
            Change { path: PathBuf::from("src/a \"b\".txt"), mode: ALLOWED_MODES[0], blob: Some(String::from(BLOB)) },
            Change { path: PathBuf::from("old.txt"), mode: ALLOWED_MODES[0], blob: None },
        ],
        message: String::from("subject\n\nbody with \"quotes\" and a\ttab\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/work/repo/.git/index"),
        scratch: PathBuf::from("/work/repo/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-0a1b2c3d-4e5f-0"),
        captured: 7_000_000_000,
        holder: String::new(),
    };
}

/// A commit request survives encoding,
/// with awkward text in the path and the message.
#[test]
fn a_commit_request_round_trips() {
    let sent: CommitRequest = request();
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent.clone())));
    assert!(!line.contains('\n'), "one line per message");
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Commit(Box::new(sent))));
}

/// What:
///  A change whose path is not text survives the socket.
/// Why:
///   A commit's changed paths travel as JSON,
/// and a name that lost its bytes would reach the service as another name,
/// or as none;
/// the plain field beside the bytes stays for a reader that predates it.
#[test]
fn a_change_path_that_is_not_text_round_trips() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let name: PathBuf = PathBuf::from(OsStr::from_bytes(b"caf\xff.txt"));
    let mut sent: CommitRequest = request();
    sent.changes = vec![
        Change { path: name.clone(), mode: ALLOWED_MODES[0], blob: Some(String::from(BLOB)) },
        Change { path: PathBuf::from("gone.txt"), mode: ALLOWED_MODES[0], blob: None },
    ];
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent.clone())));
    let value: serde_json::Value = serde_json::from_str(line.as_str()).expect("one JSON object");
    assert_eq!(value["changes"][0]["path"].as_str(), Some("caf\u{fffd}.txt"), "the plain field is the readable spelling");
    assert_eq!(value["changes"][0]["pathHex"].as_str(), Some("636166ff2e747874"), "the bytes are the name");
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Commit(Box::new(sent))));
}

/// What:
///  A commit request speaks the protocol version that carries a changed path's bytes,
///  and a line of the version before it is no request this release serves.
/// Why:
///   A service built before the bytes travelled ignores `pathHex` and commits the readable spelling of the name,
/// so the version both sides compare is what keeps such a service from ever being handed the request:
/// the release text alone cannot,
/// because two builds of one crate version name the same release.
#[test]
fn a_commit_request_carries_the_version_that_knows_a_changed_paths_bytes() {
    let sent: CommitRequest = request();
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent)));
    let mut value: serde_json::Value = serde_json::from_str(line.as_str()).expect("one JSON object");
    assert_eq!(value["version"].as_u64(), Some(u64::from(PROTOCOL_VERSION)), "the request names this release's protocol");
    // The version of the build that ignores `pathHex`: a request in it is served, so this release must not speak it.
    value["version"] = serde_json::Value::from(7);
    assert_eq!(decode_client(value.to_string().as_str()), Err(String::from("unsupported version")), "the version before refuses it");
}

/// Every service message survives encoding,
/// and every failure code is reachable on the wire.
#[test]
fn service_messages_round_trip() {
    let mut messages: Vec<ServiceMessage> = vec![
        ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) },
        ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: None },
        ServiceMessage::Position { number: 3, of: 7 },
        ServiceMessage::Heartbeat,
        ServiceMessage::Withdrawn { id: String::from("x") },
        ServiceMessage::Landed { id: String::from("x"), commit: String::from(BLOB), already: true, landing: None },
        ServiceMessage::Landed {
            id: String::from("x"),
            commit: String::from(BLOB),
            already: false,
            landing: Some(LandingFigures {
                lock_hold_ms: 47,
                queued_behind: 3,
                reads_ms: 53,
                reads: vec![(String::from("branch"), 2), (String::from("witness"), 5)],
                syncs: vec![LandingSync { what: String::from("built-index"), waited: true, ms: 7 }, LandingSync { what: String::from("intent-file"), waited: false, ms: 0 }],
            }),
        },
        ServiceMessage::RetryPrepare { id: String::from("x"), base: String::from(BLOB), landed: Vec::new() },
        ServiceMessage::RetryPrepare {
            id: String::from("x"),
            base: String::from(BLOB),
            landed: vec![LandedCapture { commit: String::from(BLOB), captured: 5 }, LandedCapture { commit: String::from(BLOB), captured: 0 }],
        },
    ];
    messages.extend(FailureCode::ALL.map(|code| { return ServiceMessage::Failed { id: String::from("x"), code }; }));
    for message in messages {
        assert_eq!(decode_service(encode_service(&message).as_str()), Ok(message.clone()), "{message:?}");
    }
}

/// What:
///  An answer of a release that sends no landing figures still decodes,
///  as an answer with none.
/// Why:
///   A client of this release may meet a service of an older one until that service retires;
///  the field was added without a new protocol version,
///  so its absence must read as no figures and never as an error.
#[test]
fn an_answer_without_landing_figures_decodes_as_none() {
    let line: String = format!("{{\"type\":\"landed\",\"id\":\"x\",\"commit\":\"{BLOB}\",\"already\":false}}");
    assert_eq!(
        decode_service(line.as_str()),
        Ok(ServiceMessage::Landed { id: String::from("x"), commit: String::from(BLOB), already: false, landing: None })
    );
}

/// Landing figures that are not whole numbers or that name no reads are refused,
/// never guessed.
#[test]
fn damaged_landing_figures_are_refused() {
    let line: String =
        format!("{{\"type\":\"landed\",\"id\":\"x\",\"commit\":\"{BLOB}\",\"already\":false,\"landing\":{{\"lockHoldMs\":-1,\"queuedBehind\":0,\"readsMs\":0,\"reads\":{{}},\"syncs\":[]}}}}");
    assert!(decode_service(line.as_str()).is_err(), "a negative figure is not a whole number");
    let no_reads: String =
        format!("{{\"type\":\"landed\",\"id\":\"x\",\"commit\":\"{BLOB}\",\"already\":false,\"landing\":{{\"lockHoldMs\":1,\"queuedBehind\":0,\"readsMs\":0,\"syncs\":[]}}}}");
    assert!(decode_service(no_reads.as_str()).is_err(), "figures that name no reads are refused");
}

/// Unknown words,
/// wrong versions and truncated or oversized lines are refused,
/// never guessed.
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

/// Hostile paths,
/// modes,
/// object names and identities are refused.
#[test]
fn unacceptable_requests_are_refused() {
    let hostile_paths = ["/etc/passwd", "../up", "a/../b", "a//b", "a/./b", ".git/config", "dir/.git/x", "", "a\nb"];
    for path in hostile_paths {
        let mut bad: CommitRequest = request();
        bad.changes[0].path = PathBuf::from(path);
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
    many.changes = (0..=MAX_CHANGES).map(|n| { return Change { path: PathBuf::from(format!("f{n}")), mode: ALLOWED_MODES[0], blob: None }; }).collect();
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(many))).as_str()).is_err());
}

/// What:
///  A request names the scratch tree the service removes,
///  and no other directory can be named in its place.
/// Why:
///   The service removes that tree once the landing is over,
///  so a request that could name any path would have the service delete what it did not make.
///  The tree is the one the wrapper created,
///  named directly inside the worktree's repository scratch directory or its Git directory.
#[test]
fn a_request_names_only_the_wrapper_scratch_tree_it_wants_removed() {
    let sent: CommitRequest = request();
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent.clone())));
    let value: serde_json::Value = serde_json::from_str(line.as_str()).expect("one JSON object");
    assert_eq!(value["scratch"].as_str(), sent.scratch.to_str(), "the request says where its tree is");
    let without: serde_json::Value = {
        let mut fields: serde_json::Map<String, serde_json::Value> = value.as_object().expect("an object").clone();
        fields.remove("scratch");
        fields.remove("scratchHex");
        serde_json::Value::Object(fields)
    };
    assert!(decode_client(without.to_string().as_str()).is_err(), "a request that names no tree");
    for refused in [
        PathBuf::from("/etc/passwd"),
        PathBuf::from("/work/repo/.git/notes"),
        PathBuf::from("/work/repo/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-"),
        PathBuf::from("/work/repo/.git/deeper/monochromatic-git-policy-cli-service-0a1b"),
    ] {
        let mut bad: CommitRequest = request();
        bad.scratch = refused.clone();
        assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(bad))).as_str()).is_err(), "{}", refused.display());
    }
}

/// What:
///  A commit request names its tree on the wire,
///  and a line of the version before that field is no request this release serves.
/// Why:
///   A service built before the field works the tree out from the request's index,
///  which is the worktree's real index and not a tree,
///  so it would land the commit and leave the tree to the age sweep.
///  The version is what keeps such a service from being handed the request.
#[test]
fn a_commit_request_carries_the_version_that_names_its_tree() {
    let sent: CommitRequest = request();
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent)));
    let mut value: serde_json::Value = serde_json::from_str(line.as_str()).expect("one JSON object");
    assert_eq!(value["version"].as_u64(), Some(u64::from(PROTOCOL_VERSION)), "the request names this release's protocol");
    // The version before the field, which served a request without it.
    value["version"] = serde_json::Value::from(8);
    assert_eq!(decode_client(value.to_string().as_str()), Err(String::from("unsupported version")), "the version before refuses it");
}

/// The retire message round trips,
/// and is accepted from a client of another protocol version,
/// which is how a service of
///  another protocol is replaced;
/// a commit from that client is still refused.
#[test]
fn retire_round_trips_and_crosses_protocol_versions() {
    let line: String = encode_client(&ClientMessage::Retire);
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Retire));
    let other: String = line.replace(&format!("\"version\":{PROTOCOL_VERSION}"), "\"version\":999");
    assert_eq!(decode_client(other.as_str()), Ok(ClientMessage::Retire));
}

/// A retire written for a service of another protocol version carries that version,
/// which is what a service of an earlier
///  release requires of it,
/// and this build's reader accepts it too.
#[test]
fn retire_is_written_in_the_version_the_service_announced() {
    let earlier: u32 = PROTOCOL_VERSION - 1;
    let line: String = encode_retire(earlier);
    assert!(line.contains(&format!("\"version\":{earlier}")), "{line}");
    assert_eq!(decode_client(line.as_str()), Ok(ClientMessage::Retire));
    assert_eq!(encode_retire(PROTOCOL_VERSION), encode_client(&ClientMessage::Retire));
}

/// A welcome without a release,
/// as the services before the field send it,
/// reads as no release.
#[test]
fn a_welcome_without_a_release_reads_as_none() {
    let line: String = format!("{{\"type\":\"welcome\",\"version\":{PROTOCOL_VERSION}}}");
    assert_eq!(decode_service(line.as_str()), Ok(ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: None }));
}

/// A work message survives encoding;
/// a relative or empty directory,
/// and a missing one,
/// are refused.
#[test]
fn a_work_message_round_trips_and_hostile_directories_are_refused() {
    let sent: ClientMessage = ClientMessage::Work { git_dir: PathBuf::from("/srv/repo/.git"), review_key: None };
    assert_eq!(decode_client(encode_client(&sent).as_str()), Ok(sent.clone()));
    for bad in ["relative/.git", "", "/\u{0}/.git"] {
        let hostile: ClientMessage = ClientMessage::Work { git_dir: PathBuf::from(bad), review_key: None };
        assert!(decode_client(encode_client(&hostile).as_str()).is_err(), "{bad:?}");
    }
    let missing: String = format!("{{\"type\":\"work\",\"version\":{PROTOCOL_VERSION}}}");
    assert!(decode_client(missing.as_str()).is_err());
}

/// A warm message round-trips,
/// and a relative,
/// empty or NUL-holding directory or repository,
/// and a missing one,
/// are refused.
#[test]
fn a_warm_message_round_trips_and_hostile_paths_are_refused() {
    let sent: ClientMessage = ClientMessage::Warm { git_dir: PathBuf::from("/srv/repo/.git"), repository: PathBuf::from("/srv/repo") };
    assert_eq!(decode_client(encode_client(&sent).as_str()), Ok(sent.clone()));
    for bad in ["relative/.git", "", "/\u{0}/.git"] {
        let hostile: ClientMessage = ClientMessage::Warm { git_dir: PathBuf::from(bad), repository: PathBuf::from("/srv/repo") };
        assert!(decode_client(encode_client(&hostile).as_str()).is_err(), "{bad:?}");
        let hostile_repository: ClientMessage = ClientMessage::Warm { git_dir: PathBuf::from("/srv/repo/.git"), repository: PathBuf::from(bad) };
        assert!(decode_client(encode_client(&hostile_repository).as_str()).is_err(), "{bad:?}");
    }
    let missing: String = format!("{{\"type\":\"warm\",\"version\":{PROTOCOL_VERSION},\"gitDir\":\"/srv/repo/.git\"}}");
    assert!(decode_client(missing.as_str()).is_err());
}

/// What:
///  Every message that names a directory round-trips one whose path is not text,
/// byte for byte.
/// Why:
///   The wrapper sends the worktree it commits in over the socket,
/// and the service acts on the path it reads:
/// a spelling with replacement characters names a directory that is not the person's,
/// so the commit would fail there.
#[test]
fn a_message_of_a_repository_that_is_not_text_round_trips() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let git_dir: PathBuf = PathBuf::from(OsStr::from_bytes(b"/srv/r\xffpo/.git"));
    let repository: PathBuf = PathBuf::from(OsStr::from_bytes(b"/srv/r\xffpo"));
    let mut sent: CommitRequest = request();
    sent.repository = repository.clone();
    sent.common_dir = git_dir.clone();
    sent.git_dir = git_dir.clone();
    sent.index = git_dir.join("index");
    sent.scratch = git_dir.join("monochromatic-git-policy-cli-service-0a1b2c3d");
    // A cleanup belongs to a commit built elsewhere, which is the only request that names one.
    sent.commit = Some(String::from(BLOB));
    sent.cleanup = Some(Cleanup { git_dir: git_dir.clone(), scratch: git_dir.join("monochromatic-git-policy-cli-service-0a1b2c3d"), reftable: false });
    for message in [
        ClientMessage::Work { git_dir: git_dir.clone(), review_key: None },
        ClientMessage::Warm { git_dir: git_dir.clone(), repository },
        ClientMessage::Commit(Box::new(sent)),
    ] {
        assert_eq!(decode_client(encode_client(&message).as_str()), Ok(message.clone()), "{message:?}");
    }
}

/// What:
///  A line written by a release that spelled its directories as plain text reads as those directories.
/// Why:
///   A wrapper of a release that sends only the plain fields must still reach a service of this one,
/// and the plain field is the whole path whenever the path is text.
#[test]
fn a_line_of_an_earlier_release_is_read_as_its_plain_paths() {
    let warm: String = format!("{{\"type\":\"warm\",\"version\":{PROTOCOL_VERSION},\"gitDir\":\"/srv/repo/.git\",\"repository\":\"/srv/repo\"}}");
    assert_eq!(decode_client(warm.as_str()), Ok(ClientMessage::Warm { git_dir: PathBuf::from("/srv/repo/.git"), repository: PathBuf::from("/srv/repo") }));
    let work: String = format!("{{\"type\":\"work\",\"version\":{PROTOCOL_VERSION},\"gitDir\":\"/srv/repo/.git\"}}");
    assert_eq!(decode_client(work.as_str()), Ok(ClientMessage::Work { git_dir: PathBuf::from("/srv/repo/.git"), review_key: None }));
}

/// A commit built elsewhere is named whole,
/// alone or with an amend,
/// never together with a tree;
/// a name that is no object is refused.
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

/// An amend names its tree and nothing else does:
/// the two fields survive encoding together,
/// and a request that has one
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

/// A work message carries the review key when it has one and nothing otherwise;
/// a key of another type is refused,
/// and no
/// printed form of the message shows the key.
#[test]
fn a_work_message_carries_the_review_key_and_hides_it_when_printed() {
    const SECRET: &str = "test-key-not-a-secret";
    let with_key: ClientMessage = ClientMessage::Work { git_dir: PathBuf::from("/srv/repo/.git"), review_key: ReviewKey::new(SECRET) };
    let line: String = encode_client(&with_key);
    assert!(line.contains(SECRET), "the key travels in the message");
    assert_eq!(decode_client(line.as_str()), Ok(with_key.clone()));
    assert!(!format!("{with_key:?}").contains(SECRET));
    let without_key: ClientMessage = ClientMessage::Work { git_dir: PathBuf::from("/srv/repo/.git"), review_key: None };
    assert!(!encode_client(&without_key).contains("reviewKey"));
    let wrong_type: String = line.replace(&format!("\"{SECRET}\""), "42");
    assert!(decode_client(wrong_type.as_str()).is_err());
    let empty: String = line.replace(SECRET, "");
    assert_eq!(decode_client(empty.as_str()), Ok(without_key), "an empty key is no key");
}

/// A service that sends no list of landed captures,
/// or a damaged one,
/// tells nothing about their order.
#[test]
fn a_retry_without_a_usable_list_of_landed_captures_reads_as_an_empty_one() {
    let bare: String = format!("{{\"type\":\"retry-prepare\",\"id\":\"x\",\"base\":\"{BLOB}\"}}");
    let damaged: String = format!("{{\"type\":\"retry-prepare\",\"id\":\"x\",\"base\":\"{BLOB}\",\"landed\":[{{\"commit\":\"nope\",\"captured\":1}},{{\"commit\":\"{BLOB}\"}}]}}");
    for line in [bare, damaged] {
        assert_eq!(decode_service(line.as_str()), Ok(ServiceMessage::RetryPrepare { id: String::from("x"), base: String::from(BLOB), landed: Vec::new() }));
    }
}

/// A request from a client that sends no capture stamp reads as an unknown one.
#[test]
fn a_request_without_a_capture_stamp_has_an_unknown_one() {
    let mut value: serde_json::Value = serde_json::from_str(encode_client(&ClientMessage::Commit(Box::new(request()))).as_str()).expect("json");
    value.as_object_mut().expect("object").remove("captured");
    let Ok(ClientMessage::Commit(decoded)) = decode_client(value.to_string().as_str()) else { panic!("the request is refused") };
    assert_eq!(decoded.captured, 0);
}

/// A cleanup and a request's scratch tree name the wrapper's own directory names directly inside the
///  repository scratch directory of the worktree,
/// or directly inside the worktree's own Git directory as a release before this one made it,
/// and nothing else,
/// which is the check `the_tree_of` asks before the service removes a tree.
#[test]
fn a_scratch_path_names_only_a_scratch_directory_of_the_worktree() {
    let root: PathBuf = fixture("protocol-scratch-names");
    let common: PathBuf = root.join("repo/.git");
    let git_dir: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(&git_dir).expect("worktree git directory");
    std::fs::write(git_dir.join("commondir"), b"../..\n").expect("commondir");
    let swept: PathBuf = common.join("monochromatic-git-policy-cli-scratch");
    let accepted = |path: PathBuf| -> bool { return is_scratch_path(path.as_path(), git_dir.as_path()); };
    let name: &str = "monochromatic-git-policy-cli-service-0a1b-2c";
    assert!(accepted(swept.join(name)), "the place this release makes the tree in");
    assert!(accepted(git_dir.join(name)), "the place a release before this one made it in");
    for refused in [
        swept.join("monochromatic-git-policy-cli-service-"),
        swept.join("monochromatic-git-policy-cli-service-a b"),
        swept.join("monochromatic-git-policy-cli-service-../x"),
        swept.join("notes"),
        swept.join("cli-git-input-0a1b"),
        swept.join("deeper").join(name),
        git_dir.join("monochromatic-git-policy-cli-service-"),
        git_dir.join("notes"),
        git_dir.join("index"),
        git_dir.join("deeper").join(name),
        common.join(name),
        common.join("worktrees/other").join(name),
        root.join("elsewhere").join(name),
        PathBuf::from("monochromatic-git-policy-cli-service-x"),
        PathBuf::from("/"),
        PathBuf::from(format!("{}/{name}\u{0}", swept.display())),
    ] {
        assert!(!accepted(refused.clone()), "{}", refused.display());
    }
    assert!(!accepted(root.join("precious")), "a directory the wrapper did not make is never accepted");
    remove(root.as_path());
}

/// The service removes what a request's cleanup names,
/// so only a commit's own scratch directory is accepted there,
/// in either of the two places the wrapper makes one,
/// and every other directory is refused at the decoder.
#[test]
fn a_cleanup_names_only_a_scratch_directory_of_the_worktree() {
    let root: PathBuf = fixture("protocol-cleanup-names");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b-2c";
    let swept: PathBuf = common.join("monochromatic-git-policy-cli-scratch");
    let asked = |scratch: PathBuf| -> String {
        let mut request: CommitRequest = request();
        request.git_dir = common.clone();
        request.common_dir = common.clone();
        request.repository = root.join("repo");
        request.index = common.join("index");
        request.scratch = common.join(name);
        request.commit = Some(String::from(BLOB));
        request.base = String::from(BLOB);
        request.cleanup = Some(Cleanup { git_dir: common.clone(), scratch, reftable: false });
        return encode_client(&ClientMessage::Commit(Box::new(request)));
    };
    let accepted = |scratch: PathBuf| -> bool { return decode_client(asked(scratch).as_str()).is_ok(); };
    assert!(accepted(swept.join(name)), "the place this release makes the tree in");
    assert!(accepted(common.join(name)), "the place a release before this one made it in");
    for refused in [
        swept.join("monochromatic-git-policy-cli-service-"),
        swept.join("monochromatic-git-policy-cli-service-a b"),
        swept.join("notes"),
        swept.join("deeper").join(name),
        common.join("index"),
        common.join("objects"),
        common.join(name).join("deeper"),
        root.join("repo").join(name),
        root.join("elsewhere").join(name),
        PathBuf::from("/etc/passwd"),
        PathBuf::from(name),
    ] {
        assert!(!accepted(refused.clone()), "{}", refused.display());
    }
    // A cleanup belongs to a commit built elsewhere, so a request that names no commit names no cleanup either.
    let mut without: CommitRequest = request();
    without.commit = None;
    without.cleanup = Some(Cleanup { git_dir: common.clone(), scratch: swept.join(name), reftable: false });
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(without))).as_str()).is_err());
    // A cleanup whose own Git directory is relative cannot name a place inside it.
    let mut relative: CommitRequest = request();
    relative.commit = Some(String::from(BLOB));
    relative.cleanup = Some(Cleanup { git_dir: PathBuf::from("repo/.git"), scratch: swept.join(name), reftable: false });
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(relative))).as_str()).is_err());
    remove(root.as_path());
}

/// A commit request carries the worktree's own Git directory,
/// which a request without it,
/// or with a relative one,
/// is refused for.
#[test]
fn a_commit_request_names_its_worktrees_git_directory() {
    let sent: CommitRequest = request();
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent.clone())));
    assert!(line.contains(&format!("\"gitDir\":\"{}\"", sent.git_dir.display())));
    let mut value: serde_json::Value = serde_json::from_str(line.as_str()).expect("a JSON line");
    let fields = value.as_object_mut().expect("an object");
    fields.remove("gitDir");
    fields.remove("gitDirHex");
    assert!(decode_client(value.to_string().as_str()).is_err(), "missing");
    let mut relative: CommitRequest = sent;
    relative.git_dir = PathBuf::from("relative/.git");
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(relative))).as_str()).is_err(), "relative");
}

/// A request that changes as many paths as one may,
/// each path as long as the 99th percentile of the paths of this repository's
///  commits,
/// still fits the line the service accepts;
/// so the change bound and the line bound agree for ordinary paths.
#[test]
fn a_request_of_the_most_changes_at_the_ninety_ninth_percentile_path_fits_one_line() {
    let mut sent: CommitRequest = request();
    let path: String = "p".repeat(P99_CHANGED_PATH_CHARACTERS);
    sent.changes = (0..MAX_CHANGES).map(|_number: usize| { return Change { path: PathBuf::from(path.clone()), mode: ALLOWED_MODES[0], blob: Some(String::from(BLOB)) }; }).collect();
    assert_eq!(sent.changes[0].path.to_str(), Some(path.as_str()), "the control's names are text, so they take the plain field only");
    let line: String = encode_client(&ClientMessage::Commit(Box::new(sent)));
    assert!(line.len() <= MAX_LINE_BYTES, "{} bytes must fit {MAX_LINE_BYTES}", line.len());
}

/// What:
///  The most changes at the path length the line ceiling was measured for fit one line while their names are text,
///  and do not fit once each name must carry its bytes.
/// Why:
///   This is the size the preparation measures before it decides whether the service can carry the request
/// (`commit_request_fits`):
/// a request that does not fit is one the wrapper builds itself,
/// so the boundary is the one a person's commit obeys.
#[test]
fn the_most_changes_fit_one_line_only_while_their_names_are_text() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let stem: String = "p".repeat(P99_CHANGED_PATH_CHARACTERS - 5);
    let of_names = |ending: Option<u8>| -> Vec<Change> {
        return (0..MAX_CHANGES)
            .map(|number| {
                let mut bytes: Vec<u8> = format!("{number:04}-{stem}").into_bytes();
                if let Some(byte) = ending {
                    bytes.push(byte);
                }
                return Change { path: PathBuf::from(OsStr::from_bytes(bytes.as_slice())), mode: ALLOWED_MODES[0], blob: Some(String::from(BLOB)) };
            })
            .collect();
    };
    let mut sent: CommitRequest = request();
    sent.changes = of_names(Some(0xff));
    assert!(!commit_request_fits(&sent), "the bytes beside the readable spelling are more than one line holds");
    sent.changes = of_names(None);
    assert!(commit_request_fits(&sent), "a text name of the same length fits the line the ceiling was measured for");
}

/// The most changes a request may carry is the most the decoder accepts,
/// and one more is refused.
#[test]
fn the_decoder_accepts_the_most_changes_and_refuses_one_more() {
    let mut sent: CommitRequest = request();
    sent.changes = (0..MAX_CHANGES).map(|number: usize| { return Change { path: PathBuf::from(format!("f{number}")), mode: ALLOWED_MODES[0], blob: Some(String::from(BLOB)) }; }).collect();
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(sent.clone()))).as_str()).is_ok());
    sent.changes.push(Change { path: PathBuf::from("one-more"), mode: ALLOWED_MODES[0], blob: None });
    assert!(decode_client(encode_client(&ClientMessage::Commit(Box::new(sent))).as_str()).is_err());
}

/// A withdrawal round trips,
/// and an empty or oversized identity in one is refused.
#[test]
fn a_withdrawal_round_trips_and_a_hostile_identity_is_refused() {
    let sent: ClientMessage = ClientMessage::Withdraw { id: String::from("0f0e0d0c-aaaa-bbbb-cccc-000000000001") };
    assert_eq!(decode_client(encode_client(&sent).as_str()), Ok(sent));
    for id in [String::new(), "x".repeat(MAX_ID_BYTES + 1)] {
        let line: String = encode_client(&ClientMessage::Withdraw { id });
        assert!(decode_client(line.as_str()).is_err(), "{line}");
    }
}

/// A `waiting` line that says the client's request is inside the stuck branch write is the `waiting` message with one extra field,
/// so a client of this release reads it as `InsideWrite` and a client that does not know the field reads the same line as the
/// `waiting` notice it is (type and text unchanged);
/// and a `waiting` line without the field,
/// which is all an older service sends,
/// is a
/// plain `Waiting`.
/// The protocol version stays,
/// because neither side needs the other to know the field.
#[test]
fn an_inside_write_notice_is_a_waiting_line_with_one_extra_field() {
    let inside: ServiceMessage = ServiceMessage::InsideWrite { text: String::from("monochromatic-git-policy-cli: the commit service has been inside a write") };
    let line: String = encode_service(&inside);
    assert_eq!(decode_service(line.as_str()), Ok(inside));
    let value: serde_json::Value = serde_json::from_str(line.as_str()).expect("a JSON line");
    assert_eq!(value.get("type").and_then(serde_json::Value::as_str), Some("waiting"));
    assert_eq!(value.get("text").and_then(serde_json::Value::as_str), Some("monochromatic-git-policy-cli: the commit service has been inside a write"));
    assert_eq!(value.get("inside_write").and_then(serde_json::Value::as_bool), Some(true));
    // What an older service sends, and what the same line reads as when the field is false.
    for older in [r#"{"type":"waiting","text":"t"}"#, r#"{"type":"waiting","text":"t","inside_write":false}"#] {
        assert_eq!(decode_service(older), Ok(ServiceMessage::Waiting { text: String::from("t") }), "{older}");
    }
}
