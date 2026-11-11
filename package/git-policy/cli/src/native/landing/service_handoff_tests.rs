//! Controls for the wrapper's side of after-commit work.

use super::*;
use crate::service_client::Activation;
use crate::service_messages::Platform;
use crate::service_server::Bind;
use crate::test_support::{fixture, remove, spawn_owned};
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
    assert_eq!(crate::work_item::hold(work.as_path(), &item, ""), Ok(()), "a later hold asks nothing of a blocked id");
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

/// The wake a control gives:
/// a control of the line reaches no service,
/// and a record it settles stays runnable either way.
fn the_service_is_not_woken() -> crate::held_push_release_wakes_the_service::NothingWoken {
    return crate::held_push_release_wakes_the_service::NothingWoken;
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
    crate::work_item::hold(work.as_path(), &item, "").expect("the push is held");
    let held: std::path::PathBuf = state_directory(work.as_path(), crate::work_item::HELD).join(item.id());
    let done: std::path::PathBuf = state_directory(work.as_path(), crate::work_item::DONE).join(item.id());
    // The state the record describes: Git's record of the branch's remote does not hold the commit, so the line stands.
    assert_eq!(held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(false), &the_service_is_not_woken()).len(), 1);
    assert!(held.exists(), "a proof that says no settles nothing");
    // A later push of the branch carried the commit: the record is settled where a finished push goes, and nothing is said.
    assert_eq!(held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(true), &the_service_is_not_woken()), Vec::<String>::new(), "a commit the remote holds is not held");
    assert!(!held.exists(), "the record left the held state");
    assert!(done.exists(), "and is known to the queue as a push that was made");
    // Settled once, so a second command says nothing either.
    assert_eq!(held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(true), &the_service_is_not_woken()), Vec::<String>::new());
    remove(root.as_path());
}

/// What:
///  A holder text naming a process that has ended,
/// spelled as `this_command` spells one.
/// Why:
///  The settlement turns on that proof,
/// and a process a control starts and waits for is the one holder it can
///  make without a race.
fn a_command_that_has_ended() -> String {
    // Only a child that outlives the read of its identity has one: a child that has already ended, or a zombie the
    // parent has not reaped yet, answers "gone" (`process_identity.rs`), and a `true` raced this read in about one
    // run in ten.
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("30")).expect("a child starts");
    let pid: u32 = child.id();
    let birth: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("the table is readable").expect("the child runs");
    let _ended = child.kill();
    let _reaped = child.wait();
    return format!("{}/{pid}/{birth}", crate::claim_time::current_boot_id());
}

/// What:
///  A held push record whose command left a verdict,
/// as the command wrote it before it settled the record itself.
/// Why:
///  A command killed between its gate and its promote or forget leaves exactly this,
/// and the next command is what
///  settles it.
fn held_with_verdict(handoff: &ServiceHandoff, commit: &str, holding: &Holding) -> (WorkItem, std::path::PathBuf) {
    let item: WorkItem = push_item(handoff, commit);
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let path: std::path::PathBuf = state_directory(work.as_path(), crate::work_item::HELD).join(item.id());
    std::fs::create_dir_all(path.parent().expect("the held directory")).expect("a state directory");
    let record: String = crate::held_push_holder_and_verdict::with_holding(item.encode().as_str(), holding).expect("one record");
    std::fs::write(path.as_path(), record.as_bytes()).expect("a record a killed command left");
    return (item, path);
}

/// What:
///  The wake a control records,
/// so the command that released a push can be shown to have told the service.
/// Why:
///   A released record that nobody is woken for waits for the next commit,
/// which is the wait this settlement exists to end.
#[derive(Default)]
struct Woken(std::cell::RefCell<Vec<std::path::PathBuf>>);

impl crate::held_push_release_wakes_the_service::WakeTheService for Woken {
    fn wake(&self, git_dir: &std::path::Path) {
        self.0.borrow_mut().push(git_dir.to_path_buf());
    }
}

/// What:
///  A held push whose command recorded a pass and then died is released by the next command,
/// and the service is woken for it.
/// Why:
///   The commit's checks cleared it,
/// so the hold may end,
/// and the command that would have promoted the record is gone:
///  the words the record holds are the only statement that its gate approved the backup.
#[test]
fn a_held_push_whose_command_left_a_pass_is_released_by_the_next_command() {
    let root: std::path::PathBuf = fixture("handoff-held-pass");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let dead: String = a_command_that_has_ended();
    let holding: Holding = Holding { holder: dead, verdict: Verdict::Passed, reason: String::new() };
    let (item, held) = held_with_verdict(&handoff, OID, &holding);
    let woken: Woken = Woken::default();
    let said: Vec<String> = held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(false), &woken);
    assert_eq!(said.len(), 1, "the released push is named once: {said:?}");
    assert!(said[0].contains(OID), "{}", said[0]);
    assert!(said[0].contains("is queued for its push from refs/heads/main"), "the release is said as a queued push: {}", said[0]);
    assert!(!held.exists(), "the record is no longer held");
    assert!(state_directory(work.as_path(), PENDING).join(item.id()).exists(), "and waits for the service in `pending`");
    assert_eq!(woken.0.borrow().len(), 1, "the service was told to look");
    remove(root.as_path());
}

/// What:
///  A held push whose command recorded a refusal and then died is settled into that verdict.
/// Why:
///   A commit a policy blocked stays local,
/// so the record must not become runnable,
/// and the verdict the next push of the
///  branch stops before must be written from the words the record kept.
#[test]
fn a_held_push_whose_command_left_a_refusal_is_settled_into_that_verdict() {
    let root: std::path::PathBuf = fixture("handoff-held-refusal");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let holding: Holding = Holding {
        holder: a_command_that_has_ended(),
        verdict: Verdict::Blocked,
        reason: String::from("final-newline: the file does not end with a newline"),
    };
    let (item, held) = held_with_verdict(&handoff, OID, &holding);
    let woken: Woken = Woken::default();
    let said: Vec<String> = held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(false), &woken);
    assert!(said.is_empty(), "a refused push is not said as held: {said:?}");
    assert!(!held.exists(), "the record left the held state");
    assert!(!state_directory(work.as_path(), PENDING).join(item.id()).exists(), "and never becomes runnable");
    assert!(woken.0.borrow().is_empty(), "nothing is pushed, so nothing is woken");
    let verdicts: Vec<crate::work_item_holding::BlockedVerdict> = crate::work_item_holding::blocked_verdicts(work.as_path());
    assert_eq!(
        verdicts,
        vec![crate::work_item_holding::BlockedVerdict {
            branch_ref: String::from("refs/heads/main"),
            commit: String::from(OID),
            reason: String::from("final-newline: the file does not end with a newline"),
        }],
        "the verdict the next push of the branch stops before, with the gate's own words"
    );
    remove(root.as_path());
}

/// What:
///  A held push whose command died before any verdict names that command and what sends the commit.
/// Why:
///   The commit stays local,
/// as the owner decided,
/// and the line must say why nothing here publishes it and what the person
///  can do instead,
/// rather than promising a next push that nothing in this worktree would make.
#[test]
fn a_held_push_whose_command_is_gone_says_so_and_names_what_sends_it() {
    let root: std::path::PathBuf = fixture("handoff-held-gone");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let dead: String = a_command_that_has_ended();
    let pid: i64 = pid_of(dead.as_str()).expect("the holder names a process");
    let holding: Holding = Holding { holder: dead, verdict: Verdict::Undecided, reason: String::new() };
    let (_item, held) = held_with_verdict(&handoff, OID, &holding);
    let woken: Woken = Woken::default();
    let said: Vec<String> = held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(false), &woken);
    assert_eq!(said.len(), 1, "one line: {said:?}");
    assert!(said[0].contains(format!("process {pid}").as_str()), "the command is named by its number: {}", said[0]);
    assert!(said[0].contains("ended before its checks ran"), "{}", said[0]);
    assert!(said[0].contains("a push you make of refs/heads/main sends it"), "the remedy is named: {}", said[0]);
    assert!(held.exists(), "no verdict was written, so nothing here publishes the commit");
    assert!(woken.0.borrow().is_empty(), "and nothing is woken for it");
    remove(root.as_path());
}

/// What:
///  A held push whose command still runs is left to that command.
/// Why:
///   The wait is that command's own,
/// and a later command that settled the record would decide a gate another process
///  is running.
#[test]
fn a_held_push_whose_command_still_runs_is_left_to_it() {
    let root: std::path::PathBuf = fixture("handoff-held-live");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let holding: Holding = Holding { holder: crate::held_push_holder_and_verdict::this_command(), verdict: Verdict::Passed, reason: String::new() };
    let (_item, held) = held_with_verdict(&handoff, OID, &holding);
    let woken: Woken = Woken::default();
    let said: Vec<String> = held_push_notices(handoff.git_dir.as_path(), &TheRemoteHolds(false), &woken);
    assert_eq!(said.len(), 1, "one line: {said:?}");
    assert!(said[0].contains(format!("process {}", std::process::id()).as_str()), "{}", said[0]);
    assert!(said[0].contains("has not finished its checks"), "{}", said[0]);
    assert!(held.exists(), "the record stays where the command that decided it left it");
    assert!(woken.0.borrow().is_empty(), "and nothing is woken by another command");
    remove(root.as_path());
}

/// What:
///  A retry that finishes a commit takes the first try's hold over,
/// names itself as the command that decided,
/// and makes the record runnable.
/// Why:
///   The service held the push for the command that landed the commit,
/// and that command never reached its gate;
/// the record
///  is keyed by the kind and the commit,
/// so the command that finishes the commit finds it and is the one a later look
///  can prove gone.
#[test]
fn a_retry_that_finishes_the_commit_takes_the_first_try_s_hold_over() {
    let root: std::path::PathBuf = fixture("handoff-retry-takes-over");
    let handoff: ServiceHandoff = handoff(root.as_path(), Activation::Unsupported(Platform::LinuxWithoutUserServices));
    let work: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let item: WorkItem = push_item(&handoff, OID);
    let held: std::path::PathBuf = state_directory(work.as_path(), crate::work_item::HELD).join(item.id());
    let dead: String = a_command_that_has_ended();
    crate::work_item::hold(work.as_path(), &item, dead.as_str()).expect("the first try's push is held");
    let before: String = std::fs::read_to_string(held.as_path()).expect("the held record");
    assert!(before.contains(dead.as_str()), "the record names the command that landed the commit: {before}");
    let _promoted: () = promote_the_push(&handoff, OID).expect("the record is made runnable");
    assert!(!held.exists(), "the first try's hold is gone");
    let pending: std::path::PathBuf = state_directory(work.as_path(), PENDING).join(item.id());
    let after: String = std::fs::read_to_string(pending.as_path()).expect("the runnable record");
    let holding: Holding = holding_in(after.as_str());
    assert_eq!(holding.verdict, Verdict::Passed, "the retry wrote the verdict it recorded: {after}");
    assert_eq!(holding.holder, crate::held_push_holder_and_verdict::this_command(), "and named itself as the command that decided: {after}");
    let written: WorkItem = WorkItem::decode(after.as_str()).expect("the item the record states");
    assert_eq!(written.id(), item.id(), "the record is the first try's own, not a second one");
    remove(root.as_path());
}
