//! Controls for the wrapper's side of after-commit work.

use super::*;
use crate::service_client::Activation;
use crate::service_messages::Platform;
use crate::service_server::Bind;
use crate::test_support::{fixture, remove};
use crate::work_item::{PENDING, state_directory};
use std::time::Duration;

const OID: &str = "0123456789abcdef0123456789abcdef01234567";
const PARENT: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn handoff(root: &std::path::Path, activation: Activation) -> ServiceHandoff {
    return ServiceHandoff {
        config: ClientConfig {
            address: Bind::Path(root.join("no-service.sock")),
            activation,
            start_wait: Duration::from_millis(100),
            heartbeat_silence: Duration::from_secs(1),
            start_watch: None, keep_fresh: None, answer_ceiling: crate::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING, extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: crate::wait_ceilings::START_CEILING,
            waiting_notice_after: Duration::from_secs(60),
            units_record_dir: None,
        },
        repository: root.join("worktree"),
        git_dir: root.join("common"),
        reference: String::from("refs/heads/main"),
        parent: PARENT.to_string(),
    };
}

/// A handoff writes one pending record per handed-over kind,
/// named for the kind and the commit,
/// before it wakes anyone;
///  with no way to start a service it says so plainly and leaves the records for the next wake.
#[test]
fn the_records_are_written_before_the_wake_and_survive_a_failed_one() {
    let root: std::path::PathBuf = fixture("handoff-records");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let mut sink: Vec<u8> = Vec::new();
    let failure = hand_over(&handoff, OID, &mut sink).expect_err("no service can be started");
    // The commit landed, so the failure says the work was not handed over, never that the commit was not made.
    assert_eq!(failure, ClientFailure::WorkNotHandedOver(String::from("this account cannot start the commit service on demand")));
    let pending: std::path::PathBuf = state_directory(work_directory(handoff.git_dir.as_path()).as_path(), PENDING);
    let mut found: Vec<String> = std::fs::read_dir(pending).expect("pending").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    found.sort();
    // Each record is named by its own id, which for the push carries the branch as well as the commit.
    let mut expected: Vec<String> = HANDED_OVER
        .iter()
        .map(|kind| {
            return WorkItem { kind: *kind, repository: PathBuf::new(), git_dir: PathBuf::new(), branch_ref: handoff.reference.clone(), oid: String::from(OID), parent: String::from(PARENT), created: String::new(), tries: 0 }.id();
        })
        .collect();
    expected.sort();
    assert_eq!(found, expected);
    remove(root.as_path());
}

/// A second handoff of the same commit adds nothing (the record names are derived),
/// so a retry cannot duplicate work.
#[test]
fn a_repeated_handoff_adds_no_second_record() {
    let root: std::path::PathBuf = fixture("handoff-repeat");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let mut sink: Vec<u8> = Vec::new();
    let _first = hand_over(&handoff, OID, &mut sink);
    let _second = hand_over(&handoff, OID, &mut sink);
    let pending: std::path::PathBuf = state_directory(work_directory(handoff.git_dir.as_path()).as_path(), PENDING);
    assert_eq!(std::fs::read_dir(pending).expect("pending").count(), HANDED_OVER.len());
    remove(root.as_path());
}

/// A review record is left only where the worktree keeps a shadow log;
/// elsewhere the handed-over kinds are all there is.
#[test]
fn a_review_record_is_left_only_where_the_repository_keeps_a_shadow_log() {
    let root: std::path::PathBuf = fixture("handoff-review");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let pending: std::path::PathBuf = state_directory(work_directory(handoff.git_dir.as_path()).as_path(), PENDING);
    let mut sink: Vec<u8> = Vec::new();
    let _without = hand_over(&handoff, OID, &mut sink);
    assert!(!pending.join(format!("{}-{OID}", WorkKind::Review.name())).exists());
    let log: std::path::PathBuf = handoff.repository.join(crate::standards_review_input::TRACKED_LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directory");
    std::fs::write(log.as_path(), "").expect("log");
    let _with = hand_over(&handoff, PARENT, &mut sink);
    assert!(pending.join(format!("{}-{PARENT}", WorkKind::Review.name())).exists());
    assert_eq!(std::fs::read_dir(pending).expect("pending").count(), 2 * HANDED_OVER.len() + 1);
    remove(root.as_path());
}

/// Every file under a directory,
/// recursively.
fn files_under(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(listing) = std::fs::read_dir(directory) {
        for entry in listing.flatten() {
            let path: std::path::PathBuf = entry.path();
            if path.is_dir() {
                found.extend(files_under(path.as_path()));
            } else {
                found.push(path);
            }
        }
    }
    return found;
}

/// Whether `needle` occurs in `bytes`.
fn contains_bytes(bytes: &[u8], needle: &[u8]) -> bool {
    return bytes.windows(needle.len()).any(|window| { return window == needle; });
}

/// The credential never reaches a file:
/// every byte under the worktree and the Git directory is scanned,
/// after records
/// were written,
/// and a positive control shows the scan reads the records.
#[test]
fn the_on_disk_work_items_never_hold_the_key() {
    const SECRET: &str = "test-key-not-a-secret-on-disk";
    let root: std::path::PathBuf = fixture("handoff-key-disk");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let log: std::path::PathBuf = handoff.repository.join(crate::standards_review_input::TRACKED_LOG_RELATIVE);
    std::fs::create_dir_all(log.parent().expect("parent")).expect("directory");
    std::fs::write(log.as_path(), "").expect("log");
    let key: ReviewKey = ReviewKey::new(SECRET).expect("a key");
    let mut sink: Vec<u8> = Vec::new();
    let _result = hand_over_with_key(&handoff, OID, Some(&key), &mut sink);
    let files: Vec<std::path::PathBuf> = files_under(root.as_path());
    let records: usize = files.iter().filter(|path| { return path.to_string_lossy().contains(OID); }).count();
    assert_eq!(records, HANDED_OVER.len() + 1, "the scan sees the records, review included");
    for path in &files {
        let bytes: Vec<u8> = std::fs::read(path).expect("a readable file");
        assert!(!contains_bytes(bytes.as_slice(), SECRET.as_bytes()), "{} holds the key", path.display());
    }
    assert!(!String::from_utf8_lossy(sink.as_slice()).contains(SECRET), "the notices never print the key");
    remove(root.as_path());
}

/// The wake message that reaches the service carries the key,
/// and carries none when there is none.
#[test]
fn the_wake_message_carries_the_key_to_the_service() {
    const SECRET: &str = "test-key-not-a-secret-on-the-wire";
    let root: std::path::PathBuf = fixture("handoff-key-wire");
    let name: String = format!("cli-git-test-wake-key-{}", std::process::id());
    let mut config: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    config.config.address = Bind::Abstract(name.clone());
    let listener: std::os::unix::net::UnixListener = crate::service_server::bind(&Bind::Abstract(name)).expect("fake service socket");
    let received: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::clone(&received);
    let server: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        use std::io::{BufRead, Write};
        for _ in 0..2 {
            let Ok((mut stream, _)) = listener.accept() else { return };
            let welcome: String = crate::service_protocol::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: crate::service_protocol::PROTOCOL_VERSION, release: Some(String::from(crate::service_protocol::RELEASE)) });
            let _written = writeln!(stream, "{welcome}");
            let mut line: String = String::new();
            let _read = std::io::BufReader::new(&stream).read_line(&mut line);
            sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(line);
        }
    });
    let mut notices: Vec<u8> = Vec::new();
    let key: ReviewKey = ReviewKey::new(SECRET).expect("a key");
    hand_over_with_key(&config, OID, Some(&key), &mut notices).expect("woken with a key");
    hand_over_with_key(&config, PARENT, None, &mut notices).expect("woken without a key");
    let _joined = server.join();
    let lines: Vec<String> = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert_eq!(lines.len(), 2);
    match crate::service_protocol::decode_client(lines[0].trim_end()) {
        Ok(crate::service_protocol::ClientMessage::Work { review_key, git_dir }) => {
            assert_eq!(review_key, Some(key));
            assert_eq!(git_dir, config.git_dir);
        }
        other => panic!("not a work message: {other:?}"),
    }
    assert!(matches!(crate::service_protocol::decode_client(lines[1].trim_end()), Ok(crate::service_protocol::ClientMessage::Work { review_key: None, .. })));
    remove(root.as_path());
}

/// What:
///  A gate that blocks the backup forgets the record an earlier release named as well as this release's.
/// Why:
///   `promote_the_push` moves a held record of an earlier release,
/// named by the kind and the commit alone,
///  into `pending` under that name;
/// a blocked gate that removed only this release's name would leave it there,
/// and the next wake
///  would push a commit the gate kept local.
#[test]
fn a_blocked_gate_forgets_a_record_an_earlier_release_named() {
    let root: std::path::PathBuf = fixture("handoff-blocked-gate-earlier-name");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let item: WorkItem = push_item(&handoff, OID);
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let pending: std::path::PathBuf = state_directory(work.as_path(), PENDING);
    std::fs::create_dir_all(pending.as_path()).expect("a state directory");
    std::fs::write(pending.join(item.id_before_the_branch()), item.encode()).expect("a record an earlier release left");
    forget_the_push(&handoff, OID, "final-newline: the file does not end with a newline");
    assert!(!pending.join(item.id()).exists(), "no record waits under this release's name");
    assert!(!pending.join(item.id_before_the_branch()).exists(), "the record an earlier release named is forgotten too");
    assert_eq!(crate::work_item::hold(work.as_path(), &item), Ok(()), "a later hold asks nothing of a blocked id");
    assert!(!state_directory(work.as_path(), crate::work_item::HELD).join(item.id()).exists(), "the blocked verdict stands, so the push is not held again");
    // The verdict keeps the branch and the gate's words, because the next push of that branch reads them to know where it must stop
    //  and every command reads them to say why.
    let verdicts: Vec<crate::work_item_holding::BlockedVerdict> = crate::work_item_holding::blocked_verdicts(work.as_path());
    assert_eq!(
        verdicts,
        vec![crate::work_item_holding::BlockedVerdict {
            branch_ref: String::from("refs/heads/main"),
            commit: String::from(OID),
            reason: String::from("final-newline: the file does not end with a newline"),
        }],
        "one verdict, with its branch, its commit and its words"
    );
    remove(root.as_path());
}

/// A push record the wrapper promoted and no service has taken says the push is queued,
/// and one that has just been queued is a push in flight.
#[test]
fn a_push_record_no_service_has_taken_is_named_as_queued_once_it_has_waited() {
    let root: std::path::PathBuf = fixture("handoff-queued-push");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let item: WorkItem = push_item(&handoff, OID);
    crate::work_item::enqueue(work.as_path(), &item).expect("the push record is queued");
    let now: std::time::SystemTime = std::time::SystemTime::now();
    assert!(
        queued_push_notices(handoff.git_dir.as_path(), now).is_empty(),
        "a record younger than the bound is a push in flight, and nothing is said about it"
    );
    let waited: std::time::SystemTime = now + PUSH_QUEUED_AFTER + Duration::from_secs(1);
    let said: Vec<String> = queued_push_notices(handoff.git_dir.as_path(), waited);
    assert_eq!(said.len(), 1, "one line: {said:?}");
    assert!(said[0].contains(OID), "the line names the commit: {}", said[0]);
    assert!(said[0].contains("is queued for its push from refs/heads/main"), "{}", said[0]);
    assert!(said[0].contains("has not been made yet"), "the line claims only what is known: {}", said[0]);
    assert!(said[0].contains("needs no further commit"), "{}", said[0]);
    // A second waiting push is counted rather than named: the list of a container's unserved commits has no end.
    let second: WorkItem = push_item(&handoff, PARENT);
    crate::work_item::enqueue(work.as_path(), &second).expect("a second push record is queued");
    let more: Vec<String> = queued_push_notices(handoff.git_dir.as_path(), waited);
    assert_eq!(more.len(), 2, "the oldest is named and the rest counted: {more:?}");
    assert_eq!(more[0], said[0], "the oldest keeps its line");
    assert!(more[1].contains("1 more commit is queued for its push the same way"), "{}", more[1]);
    // The line is the record's: a service that takes it leaves nothing to name.
    let claimed: Option<crate::work_item::Claimed> =
        crate::work_item::claim_next_of(work.as_path(), &crate::claim_time::ClaimTime::read(), &[WorkKind::Push]).expect("a claim");
    assert!(claimed.is_some(), "the service took the oldest record");
    let taken: Vec<String> = queued_push_notices(handoff.git_dir.as_path(), waited);
    assert_eq!(taken.len(), 1, "a claimed record is no longer waiting: {taken:?}");
    assert!(!taken[0].contains(OID), "the claimed record is not named: {}", taken[0]);
    remove(root.as_path());
}

/// What a command is told about a held record the repository proves the remote already holds.
struct TheRemoteHolds(bool);

impl crate::held_push_commit_on_the_remote::RemoteRecord for TheRemoteHolds {
    fn on_the_remote(&self, _reference: &str, _commit: &str) -> OnTheRemote {
        return if self.0 { OnTheRemote::There } else { OnTheRemote::NotThere };
    }
}

/// A held push whose commit a later push carried is settled as done and no longer said,
/// and one the remote does not hold keeps its line.
#[test]
fn a_held_push_the_remote_already_holds_is_settled_and_not_said() {
    let root: std::path::PathBuf = fixture("handoff-held-settled");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let item: WorkItem = push_item(&handoff, OID);
    crate::work_item::hold(work.as_path(), &item).expect("the push is held");
    let held: std::path::PathBuf = state_directory(work.as_path(), crate::work_item::HELD).join(item.id());
    let done: std::path::PathBuf = state_directory(work.as_path(), crate::work_item::DONE).join(item.id());
    // The state the record describes: Git's record of the branch's remote does not hold the commit, so the line stands.
    assert_eq!(held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(false)).len(), 1);
    assert!(held.exists(), "a proof that says no settles nothing");
    // A later push of the branch carried the commit: the record is settled where a finished push goes, and nothing is said.
    assert_eq!(held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(true)), Vec::<String>::new(), "a commit the remote holds is not held");
    assert!(!held.exists(), "the record left the held state");
    assert!(done.exists(), "and is known to the queue as a push that was made");
    // Settled once, so a second command says nothing either.
    assert_eq!(held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(true)), Vec::<String>::new());
    remove(root.as_path());
}
