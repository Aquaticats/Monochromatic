//! Controls of the relink record:
//! what it carries,
//! what it refuses,
//! and what a hand-over leaves behind when no service can be reached.

use super::{RELINK_REFERENCE, RelinkEntry, Reach, RelinkTool, decode, encode, hand_over_with, item_of, object_name_of};
use crate::claim_time::ClaimTime;
use crate::iso_time::now_iso;
use crate::service_route::Route;
use crate::service_server::Bind;
use crate::service_work::work_directory;
use crate::test_support::{fixture, git_context, remove};
use crate::work_item::{Claimed, DONE, Enqueued, PENDING, WorkItem, WorkKind, claim_next_of, enqueue, settle, state_directory};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// One worktree that needs both tools,
/// so the payload holds more than one name.
fn entry(root: &Path) -> RelinkEntry {
    return RelinkEntry { root: root.to_path_buf(), search_path: None, search_from: None, tools: vec![RelinkTool::Pnpm, RelinkTool::Toolchain] };
}

/// A route finder that names no service,
/// as a session with no unit for this repository does.
fn no_service(_git: &Path, _git_dir: &Path) -> Option<Route> {
    return None;
}

/// A provisioner that has nothing to set up.
fn nothing_to_set_up(_host: &crate::service_provision::ProvisionHost) -> Result<(), crate::service_messages::ClientFailure> {
    return Ok(());
}

/// The reach of a control:
/// a route finder that finds nothing and a provisioner that sets nothing up,
/// so no service and no session bus is reached.
fn no_reach() -> Reach<'static> {
    return Reach { router: &no_service, provisioner: &nothing_to_set_up, start_wait: std::time::Duration::from_millis(200) };
}

/// What:
///  A worktree that needs both tools round-trips through the payload,
/// with the order the report has always named them in,
/// with the person's search path,
/// and with the directory that search path was resolved in.
/// Why:
///   The record is the only thing the service reads,
/// so what it carries must survive being written and read back;
/// a search path and the directory it was resolved in are two facts,
/// and a reader that lost the second would read a relative entry from its own working directory.
#[test]
fn both_tools_round_trip_through_the_payload() {
    let written: RelinkEntry = RelinkEntry {
        root: PathBuf::from("/w/topic"),
        search_path: Some(OsString::from("bin:/usr/bin:/bin")),
        search_from: Some(PathBuf::from("/home/someone/src/project")),
        tools: vec![RelinkTool::Pnpm, RelinkTool::Toolchain],
    };
    let text: String = encode(&written).expect("the payload");
    assert!(text.contains("searchFrom"), "the record does not name the directory its search path was resolved in: {text}");
    assert_eq!(decode(text.as_str()).expect("the entry"), written);
    let without: RelinkEntry = entry(Path::new("/w/topic"));
    assert_eq!(decode(encode(&without).expect("the payload").as_str()).expect("the entry"), without);
}

/// What:
///  A payload of no tool,
/// of a tool this release does not know,
/// or of text that is not the object,
/// is refused.
/// Why:
///   A record a person or another release wrote must be reported rather than acted on,
/// and a record of no tool would make the service start nothing.
#[test]
fn a_payload_that_names_no_known_tool_is_refused() {
    for text in [
        "{\"root\":\"/w\",\"tools\":[]}",
        "{\"root\":\"/w\",\"tools\":[\"npm\"]}",
        "{\"root\":\"/w\",\"tools\":[7]}",
        "{\"tools\":[\"pnpm\"]}",
        "{\"root\":\"/w\"}",
        "[]",
        "{}",
        "not json",
    ] {
        let refused: Result<RelinkEntry, String> = decode(text);
        assert!(refused.is_err(), "{text} was accepted as {refused:?}");
    }
    assert!(encode(&RelinkEntry { root: PathBuf::from("/w"), search_path: None, search_from: None, tools: Vec::new() }).is_err(), "a relink of no tool is no work");
}

/// What:
///  Two worktrees of the same commit are told apart by their object names.
/// Why:
///   A record's id is its idempotency key,
/// so a shared object name would drop the second worktree's relink as already known,
/// and a work item's object name must be forty hexadecimal digits to be acted on.
#[test]
fn two_worktrees_get_their_own_object_name() {
    let moment: &str = "2026-11-11T00:00:00.000Z";
    let first: String = object_name_of(Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/one"), moment);
    let second: String = object_name_of(Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/two"), moment);
    let later: String = object_name_of(Path::new("/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/one"), "2026-11-11T00:00:00.001Z");
    assert_ne!(first, later, "two hand-overs of one path are two records");
    assert_eq!(first.len(), 40, "{first}");
    assert!(first.as_bytes().iter().all(u8::is_ascii_hexdigit), "{first}");
    assert_ne!(first, second);
}

/// What:
///  The record names the worktree it relinks,
/// the common directory whose service drains it,
/// and the tools the add decided on.
/// Why:
///   `is_acceptable` refuses a record whose Git directory is not the one it was read from,
/// and the service finds the work to do from these fields alone.
#[test]
fn the_record_names_the_worktree_the_common_directory_and_the_tools() {
    let root: PathBuf = fixture("relink-item");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let written: RelinkEntry = entry(destination.as_path());
    let item: WorkItem = item_of(common.as_path(), destination.as_path(), &written).expect("the record");
    assert_eq!(item.kind, WorkKind::Relink);
    assert_eq!(item.repository, destination);
    assert_eq!(item.git_dir, common);
    assert_eq!(item.branch_ref, RELINK_REFERENCE);
    assert_eq!(item.oid, object_name_of(destination.as_path(), item.created.as_str()));
    assert_eq!(decode(item.parent.as_str()).expect("the entry"), written);
    remove(root.as_path());
}

/// What:
///  A hand-over that finds no service takes its record back,
/// and says why it did.
/// Why:
///   The add relinks the worktree itself when no service took the record,
/// so the record owes nothing and must go:
/// a record left behind names a path and nothing else,
/// and a service that drains it later would start the install in whatever directory holds that path then,
/// which need not be the worktree this record was written for.
#[test]
fn a_hand_over_with_no_service_takes_its_record_back() {
    let root: PathBuf = fixture("relink-no-service");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let written: RelinkEntry = entry(destination.as_path());
    let told: Result<(), String> = hand_over_with(&git_context(), common.as_path(), destination.as_path(), &written, &no_reach());
    let reason: String = told.expect_err("a session with no service is reported");
    assert!(reason.contains("no commit service can be started from this session"), "{reason}");
    let pending: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING);
    assert_eq!(pending_names(pending.as_path()), Vec::<String>::new(), "the record the add did the work for is still queued");
    remove(root.as_path());
}

/// The names of the records queued in a work directory,
/// sorted,
/// so a control can say what a hand-over left there.
fn pending_names(pending: &Path) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(pending) else { return Vec::new() };
    let mut names: Vec<String> = listing
        .flatten()
        .map(|item| return item.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    return names;
}

/// What:
///  Wait,
/// bounded,
///  for the clock to show another millisecond.
/// Why:
///   A record's object name carries the moment the add asked for the relink,
/// so a control that stands for two adds at one path needs two moments,
/// and a clock that does not move would make the control prove nothing.
fn after_the_clock_moves(moment: &str) {
    let deadline: Instant = Instant::now() + Duration::from_secs(30);
    while now_iso() == moment {
        assert!(Instant::now() < deadline, "the clock did not move past {moment}");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// What:
///  A worktree added again at a path whose earlier relink settled gets a record of its own.
/// Why:
///   A record's id is its idempotency key,
/// and the queue answers an id it already knows as known in every state.
/// An id made of the path alone would therefore answer the second add with the first add's settled
/// record:
/// no pending relink would exist,
/// and the add would report a repair that can never arrive.
#[test]
fn a_second_worktree_at_a_settled_path_gets_its_own_record() {
    let root: PathBuf = fixture("relink-reused-path");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let work: PathBuf = work_directory(common.as_path());
    let first: WorkItem = item_of(common.as_path(), destination.as_path(), &entry(destination.as_path())).expect("the first record");
    assert_eq!(enqueue(work.as_path(), &first), Ok(Enqueued::Added));
    let claimed: Claimed = claim_next_of(work.as_path(), &ClaimTime::read(), &[WorkKind::Relink])
        .expect("the claim")
        .expect("a relink record to claim");
    settle(work.as_path(), &claimed, DONE).expect("the first relink settles");
    after_the_clock_moves(first.created.as_str());
    let second: WorkItem = item_of(common.as_path(), destination.as_path(), &entry(destination.as_path())).expect("the second record");
    assert_ne!(first.created, second.created, "the two adds are two moments");
    assert_ne!(first.id(), second.id(), "the second add is other work than the settled one");
    assert_eq!(enqueue(work.as_path(), &second), Ok(Enqueued::Added), "a settled record answered the second add as known");
    assert_eq!(pending_names(state_directory(work.as_path(), PENDING).as_path()), vec![second.id()], "the second add has no pending relink");
    remove(root.as_path());
}

/// What:
///  A hand-over a service answers leaves its record queued and the relinking is that service's work.
/// Why:
///   The add never waits for the relink,
/// so the line it prints promises a repair that only a queued record and a woken service can keep.
#[test]
fn a_hand_over_a_service_answers_keeps_its_record_queued() {
    let root: PathBuf = fixture("relink-taken-by-a-service");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let address: String = format!("relink-hand-over-{}", std::process::id());
    let service: FixtureService = serve_this_release(address.as_str());
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    let router = |_git: &Path, _directory: &Path| return Some(route.clone());
    let written: RelinkEntry = entry(destination.as_path());
    hand_over_with(&git_context(), common.as_path(), destination.as_path(), &written, &reach(&router, &nothing_to_set_up)).expect("a service took the record");
    let heard: Vec<String> = service.finish();
    let work: String = crate::service_protocol::encode_client(&crate::service_protocol::ClientMessage::Work { git_dir: common.clone(), review_key: None });
    assert!(heard.contains(&work), "the service was woken for the repository's common directory: {heard:?}");
    let queued: Vec<String> = pending_names(state_directory(work_directory(common.as_path()).as_path(), PENDING).as_path());
    assert_eq!(queued.len(), 1, "the service drains the record: {queued:?}");
    remove(root.as_path());
}

/// What:
///  A hand-over whose record another party took is that party's work,
/// and the add relinks nothing.
/// Why:
///   The claim moves the record out of `pending` in one rename,
/// so the hand-over's own unlink finds nothing;
/// running the install beside the service that holds the record would make two installs write one
/// `node_modules` at once.
#[test]
fn a_hand_over_whose_record_was_taken_is_the_services_work() {
    let root: PathBuf = fixture("relink-taken-elsewhere");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let pending: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING);
    let route: Route = Route { address: Bind::Abstract(format!("relink-never-bound-{}", std::process::id())), provision: Some(unreachable_host()) };
    let router = |_git: &Path, _directory: &Path| return Some(route.clone());
    // The claim a service makes between the write and the unlink: the record leaves `pending`.
    let claimed_from: PathBuf = pending.clone();
    let claiming_pending = move |_host: &crate::service_provision::ProvisionHost| {
        for name in pending_names(claimed_from.as_path()) {
            std::fs::remove_file(claimed_from.join(name)).expect("the record leaves pending");
        }
        return Ok(());
    };
    let written: RelinkEntry = entry(destination.as_path());
    hand_over_with(&git_context(), common.as_path(), destination.as_path(), &written, &reach(&router, &claiming_pending)).expect("the record the service took is its work");
    assert_eq!(pending_names(pending.as_path()), Vec::<String>::new(), "nothing is left queued");
    remove(root.as_path());
}

/// What:
///  A record that cannot be taken back is named,
/// and the add relinks the worktree anyway.
/// Why:
///   A worktree whose copied links were never relinked is wrong for the person's next command,
/// so a directory that refuses the unlink makes the add do the work and leaves the record to say,
/// in the log,
/// that the service may do it again.
#[test]
fn a_hand_over_whose_record_cannot_be_taken_back_says_it_stays() {
    let root: PathBuf = fixture("relink-record-stays");
    let common: PathBuf = root.join("common");
    let destination: PathBuf = root.join("topic");
    std::fs::create_dir_all(&common).expect("the common directory");
    std::fs::create_dir_all(&destination).expect("the worktree");
    let pending: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING);
    let route: Route = Route { address: Bind::Abstract(format!("relink-unremovable-{}", std::process::id())), provision: Some(unreachable_host()) };
    let router = |_git: &Path, _directory: &Path| return Some(route.clone());
    // A directory that refuses the unlink, as one whose permissions somebody changed does.
    let protected: PathBuf = pending.clone();
    let protecting_the_record = move |_host: &crate::service_provision::ProvisionHost| {
        std::fs::set_permissions(protected.as_path(), std::fs::Permissions::from_mode(0o500)).expect("a directory that refuses writes");
        return Ok(());
    };
    let written: RelinkEntry = entry(destination.as_path());
    let reason: String = hand_over_with(&git_context(), common.as_path(), destination.as_path(), &written, &reach(&router, &protecting_the_record))
        .expect_err("a record that stays is not a hand-over");
    std::fs::set_permissions(pending.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the fixture is removable again");
    assert!(reason.contains("stays"), "the reason names the record that stays: {reason}");
    assert_eq!(pending_names(pending.as_path()).len(), 1, "the record is still queued");
    remove(root.as_path());
}

/// A reach of a control whose provisioner sets nothing up but records what it saw.
fn reach<'a>(router: crate::worktree_trash_hand_over::Router<'a>, provisioner: crate::worktree_trash_hand_over::Provisioner<'a>) -> Reach<'a> {
    return Reach { router, provisioner, start_wait: std::time::Duration::from_millis(400) };
}

/// A host a control never reaches,
/// so a route that names one still fails its start.
fn unreachable_host() -> crate::service_provision::ProvisionHost {
    return crate::service_provision::ProvisionHost {
        runtime_dir: PathBuf::from("/nonexistent-runtime"),
        user_unit_dir: None,
        bus: PathBuf::from("/nonexistent-bus"),
        service_binary: PathBuf::from("/nonexistent-service"),
        git: PathBuf::from("/nonexistent-git"),
        instance: String::from("fixture-instance"),
        git_dir: PathBuf::new(),
        retirement_wait: std::time::Duration::from_millis(10),
    };
}

/// What a fixture service read:
/// every line,
/// in order.
struct FixtureService {
    /// Raised to end the accept loop.
    finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The loop,
    /// answering the lines it read.
    thread: std::thread::JoinHandle<Vec<String>>,
}

impl FixtureService {
    /// Stop the loop and answer the lines it read.
    fn finish(self) -> Vec<String> {
        self.finished.store(true, std::sync::atomic::Ordering::SeqCst);
        return self.thread.join().expect("the service thread");
    }
}

/// What:
///  Serve `name` as a service of this release on an abstract socket,
///  which greets and then reads one line per connection.
/// Why:
///   The wake speaks the service's own protocol,
/// and a control must hear the line the wrapper sent without a real unit and a real service.
fn serve_this_release(name: &str) -> FixtureService {
    use std::io::BufRead;
    use std::os::linux::net::SocketAddrExt;
    let socket = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("a name");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind_addr(&socket).expect("bind");
    listener.set_nonblocking(true).expect("non-blocking");
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
    let thread = std::thread::spawn(move || {
        let mut heard: Vec<String> = Vec::new();
        while !stop.load(std::sync::atomic::Ordering::SeqCst) {
            let Ok((stream, _)) = listener.accept() else {
                std::thread::sleep(std::time::Duration::from_millis(5));
                continue;
            };
            stream.set_nonblocking(false).expect("blocking");
            let mut greeting: String = crate::service_protocol::encode_service(&crate::service_protocol::ServiceMessage::Welcome {
                version: crate::service_protocol::PROTOCOL_VERSION,
                release: Some(String::from(crate::service_protocol::RELEASE)),
            });
            greeting.push('\n');
            let _sent = std::io::Write::write_all(&mut &stream, greeting.as_bytes());
            let mut raw: String = String::new();
            let _read = std::io::BufReader::new(&stream).read_line(&mut raw);
            let line: String = raw.trim_end().to_string();
            if !line.is_empty() {
                heard.push(line);
            }
        }
        return heard;
    });
    return FixtureService { finished, thread };
}
