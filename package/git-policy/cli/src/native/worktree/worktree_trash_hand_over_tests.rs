//! The hand-over of a snapshot's trash:
//! the record it writes,
//! the service it wakes,
//! what it says to the person,
//! and what a standing trash needs to stay out of the index.

use super::{Reach, hand_over_with};
use crate::fs_beneath::{ObjectId, identity_at, identity_of, open_directory_at, open_directory_beneath, open_root, plain_components};
use crate::service_protocol::{ClientMessage, PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_client, encode_service};
use crate::service_route::Route;
use crate::service_server::Bind;
use crate::service_work::{Verdict, work_directory};
use crate::test_support::{fixture, git_context, remove};
use crate::work_item::{DONE, FAILED, PENDING, WorkItem, WorkKind, state_directory};
use crate::worktree_trash::{PendingTrash, TRASH_PARENT, TRASH_REFERENCE, TrashManifest, empty_trash, entry_name};
use std::path::{Path, PathBuf};

/// A worktree whose `.git` file names a directory below the common one,
/// with a trash of `count` entries in it,
/// and the
///  manifest a leave-out would have written for it.
fn worktree_with_planted_trash(label: &str, name: &str, count: usize) -> (PathBuf, PathBuf, PathBuf, TrashManifest) {
    let root: PathBuf = fixture(label);
    let common: PathBuf = root.join("common");
    let git_dir: PathBuf = common.join("worktrees").join("topic");
    let worktree: PathBuf = root.join("topic");
    std::fs::create_dir_all(&git_dir).expect("the administrative directory");
    std::fs::create_dir_all(&worktree).expect("the worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect("the .git file");
    for index in 0..count {
        let entry: PathBuf = worktree.join(TRASH_PARENT).join(name).join(entry_name(index));
        std::fs::create_dir_all(entry.join("payload")).expect("a trash entry");
        std::fs::write(entry.join("payload/credentials"), b"another worktree's token\n").expect("an entry file");
    }
    let held: std::fs::File = open_root(worktree.as_path()).expect("the worktree");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("a plain parent");
    let parent: std::fs::File = open_directory_beneath(&held, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = open_directory_at(&parent, std::ffi::OsStr::new(name)).expect("the trash");
    let entries: Vec<ObjectId> = (0..count).map(|index: usize| { return identity_at(&trash, std::ffi::OsStr::new(entry_name(index).as_str())).expect("an entry identity"); }).collect();
    let manifest: TrashManifest = TrashManifest { name: String::from(name), trash: identity_of(&trash).expect("the trash identity"), entries };
    return (root, worktree, common, manifest);
}

/// The trash of a snapshot waiting to be handed over.
fn pending_of(worktree: &Path, common: &Path, manifest: TrashManifest) -> PendingTrash {
    return PendingTrash { destination: worktree.to_path_buf(), common_dir: common.to_path_buf(), manifest, taken_over: 0, quiet: false };
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

/// The record a hand-over writes for the trash a manifest names,
/// read back.
fn record_of(common: &Path, manifest: &TrashManifest) -> WorkItem {
    let path: PathBuf = state_directory(work_directory(common).as_path(), PENDING).join(format!("trash-{}", manifest.name));
    let text: String = std::fs::read_to_string(path.as_path()).expect("the record the hand-over wrote");
    return WorkItem::decode(text.as_str()).expect("the record is a work item");
}

/// The reach of a control:
/// a route finder and a provisioner,
/// with a start wait short enough that a service which never answers cannot hold the
///  control.
fn reach<'a>(router: crate::worktree_trash_hand_over::Router<'a>, provisioner: crate::worktree_trash_hand_over::Provisioner<'a>) -> Reach<'a> {
    return Reach { router, provisioner, start_wait: std::time::Duration::from_millis(400) };
}

/// What:
///  A trash the service cannot be told about is still named by a record,
/// which sits under the repository's common
///  directory,
/// and the trash is left in the worktree for that service to empty.
/// Why:
///   The record is the whole difference between a trash that is emptied later and one nobody knows
/// about;
/// the common directory is where it must be,
/// because one repository's service drains one work
///  directory after another and so empties one trash at a time.
#[test]
fn a_hand_over_with_no_service_leaves_the_trash_and_its_record_under_the_common_directory() {
    let name: String = "1".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-no-service", name.as_str(), 2);
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let mut notices: Vec<u8> = Vec::new();
    let reason: String = hand_over_with(&git_context(), &pending, &mut notices, &reach(&no_service, &nothing_to_set_up))
        .expect_err("a session with no service is reported, never swallowed");
    assert!(reason.contains(worktree.display().to_string().as_str()), "the reason names the worktree: {reason}");
    assert!(reason.contains("no commit service can be started from this session"), "the reason says why nothing empties it now: {reason}");
    assert!(reason.contains(worktree.join(TRASH_PARENT).join(&name).display().to_string().as_str()), "the reason names the trash: {reason}");
    assert_eq!(notices, Vec::<u8>::new(), "the failure is reported by its caller, and one line is enough");
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d1/payload/credentials").is_file(), "the trash stays where the leave-out put it");
    let item: WorkItem = record_of(common.as_path(), &manifest);
    assert_eq!(item.kind, WorkKind::Trash);
    assert_eq!(item.repository, worktree, "the record names the worktree that holds the trash");
    assert_eq!(item.git_dir, common, "and the common directory whose service empties it");
    assert_eq!(item.branch_ref, TRASH_REFERENCE);
    assert_eq!(item.oid, name);
    assert_eq!(item.parent, manifest.encode(), "the record carries the identities the leave-out wrote");
    remove(root.as_path());
}

/// What:
///  A trash whose record the service gave up on is not handed to a service again,
///  and the answer names the record that holds the failure.
/// Why:
///   The id of a record is the trash's name,
/// so offering the same trash again finds the settled record and writes nothing;
/// waking a
///  service for it would tell the service about work it never claims,
/// and the person would be told the trash
///  empties when only they can remove it.
#[test]
fn a_trash_whose_record_the_service_gave_up_on_is_not_handed_over_again() {
    let name: String = "5".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-given-up", name.as_str(), 1);
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    crate::worktree_trash::record_the_trash(&pending).expect("the record the add writes before it returns");
    let item: WorkItem = record_of(common.as_path(), &manifest);
    let waiting: PathBuf = state_directory(work_directory(common.as_path()).as_path(), PENDING).join(item.id());
    let failed: PathBuf = state_directory(work_directory(common.as_path()).as_path(), FAILED).join(item.id());
    std::fs::rename(waiting.as_path(), failed.as_path()).expect("the service gave up on it");
    let mut notices: Vec<u8> = Vec::new();
    let reason: String = hand_over_with(&git_context(), &pending, &mut notices, &reach(&no_service, &nothing_to_set_up)).expect_err("a record the service gave up on is reported, never swallowed");
    assert!(reason.contains("gave up"), "the reason says the service gave up on the record: {reason}");
    assert!(reason.contains(failed.display().to_string().as_str()), "the reason names the record that holds the failure: {reason}");
    assert!(reason.contains("only you will remove them"), "the reason says who removes the trash: {reason}");
    assert!(!reason.contains("no commit service can be started from this session"), "no service was woken for a record it never claims: {reason}");
    assert!(!waiting.exists() && failed.is_file(), "the settled record stays where it is");
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d0/payload/credentials").is_file(), "the trash stays where the leave-out put it");
    remove(root.as_path());
}

/// What:
///  A routine trash handed over with a service in reach is left standing for it,
/// the service is woken for the repository's
///  common directory,
/// and the person reads nothing.
/// Why:
///   The wake names the common directory because that is the work directory the record is in and the one the
///  service drains;
/// a wake that named the new worktree would leave the record where nothing looks for it.
/// Every add now hands the
///  snapshot's stand-ins and its own `.git` copy over,
/// so a line about a trash the add made itself would print on every add and
///  tell nobody anything.
#[cfg(target_os = "linux")]
#[test]
fn a_routine_hand_over_wakes_the_service_and_says_nothing() {
    let name: String = "2".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-woken", name.as_str(), 3);
    let address: String = format!("trash-hand-over-{}", std::process::id());
    let service: FixtureService = serve_this_release(address.as_str());
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let mut notices: Vec<u8> = Vec::new();
    let router = |_git: &Path, _directory: &Path| return Some(route.clone());
    hand_over_with(&git_context(), &pending, &mut notices, &reach(&router, &nothing_to_set_up)).expect("handed over");
    let heard: Vec<String> = service.finish();
    let work: String = encode_client(&ClientMessage::Work { git_dir: common.clone(), review_key: None });
    assert!(heard.contains(&work), "the service was woken for the common directory: {heard:?}");
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d0").is_dir(), "the add returns without emptying anything");
    assert_eq!(notices, Vec::<u8>::new(), "a trash the add made itself is said by its note alone: {:?}", String::from_utf8_lossy(notices.as_slice()));
    remove(root.as_path());
}

/// What:
///  A hand-over that took over trash an earlier add left says so once,
/// in the singular when it hands one directory over.
/// Why:
///   Nothing of this add would ever empty a trash another add left,
/// and no other command mentions it,
/// so the person
///  hears about it once,
/// here;
/// a routine trash is said by the snapshot's own note and needs no line of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pending.takenOver = 1; const said = handOverWith(pending); expect(said).toContain('the 1 directory left out of the new worktree');
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_hand_over_that_took_over_an_earlier_adds_trash_says_so_once() {
    let name: String = "8".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-taken-over", name.as_str(), 1);
    let address: String = format!("trash-hand-over-taken-over-{}", std::process::id());
    let service: FixtureService = serve_this_release(address.as_str());
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    let router = |_git: &Path, _directory: &Path| return Some(route.clone());
    let mut pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    pending.taken_over = 1;
    let mut notices: Vec<u8> = Vec::new();
    hand_over_with(&git_context(), &pending, &mut notices, &reach(&router, &nothing_to_set_up)).expect("handed over");
    let _heard: Vec<String> = service.finish();
    let said: String = String::from_utf8(notices).expect("utf8");
    assert_eq!(said.lines().count(), 1, "one line: {said:?}");
    assert!(said.contains("took over one directory that an earlier add left"), "the line names the trash an earlier add left: {said:?}");
    assert!(said.contains("1 directory left out of the new worktree") && said.contains(" was handed to the commit service"), "one directory reads in the singular, and every verb agrees with it: {said:?}");
    assert!(said.contains(worktree.display().to_string().as_str()), "and it names the worktree: {said:?}");
    remove(root.as_path());
}

/// A `--quiet` add says nothing about the hand-over and writes the same record.
#[test]
fn a_quiet_add_says_nothing_and_hands_the_same_record_over() {
    let name: String = "3".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-quiet", name.as_str(), 1);
    let mut pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    pending.quiet = true;
    let mut notices: Vec<u8> = Vec::new();
    let _reason: String = hand_over_with(&git_context(), &pending, &mut notices, &reach(&no_service, &nothing_to_set_up)).expect_err("no service in reach");
    assert_eq!(notices, Vec::<u8>::new(), "a quiet add prints nothing of the hand-over");
    assert_eq!(record_of(common.as_path(), &manifest).oid, name, "the record is written all the same");
    remove(root.as_path());
}

/// What:
///  A provisioning failure keeps the record,
/// so the trash is emptied when the repository's service next runs,
///  and the reason is the manager's own,
/// not an internal term.
/// Why:
///   The units of a repository whose service was never installed are what the provisioner sets up;
/// a manager that
///  refuses is a state the person can repair,
/// and the trash must not become anonymous because of it.
#[test]
fn a_provisioning_failure_keeps_the_record_and_names_the_managers_reason() {
    let name: String = "4".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-provision", name.as_str(), 1);
    let route: Route = Route { address: Bind::Abstract(String::from("trash-hand-over-never-bound")), provision: Some(unreachable_host()) };
    let router = |_git: &Path, _directory: &Path| return Some(route.clone());
    let refusing = |_host: &crate::service_provision::ProvisionHost| return Err(crate::service_messages::ClientFailure::CouldNotStart(String::from("the session bus refused the call")));
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let mut notices: Vec<u8> = Vec::new();
    let mut reach = reach(&router, &refusing);
    reach.start_wait = std::time::Duration::from_millis(400);
    let reason: String = hand_over_with(&git_context(), &pending, &mut notices, &reach).expect_err("a refused manager is reported");
    assert!(reason.contains("the session bus refused the call"), "the reason is the manager's: {reason}");
    assert!(reason.contains("when it next runs for this repository"), "and the record still empties it later: {reason}");
    let item: WorkItem = record_of(common.as_path(), &manifest);
    assert_eq!(item.oid, name);
    assert!(worktree.join(TRASH_PARENT).join(&name).is_dir(), "the trash stays for that run");
    remove(root.as_path());
}

/// What:
///  The record a hand-over writes is emptied by the service's handler,
/// and the worktree's pointer into the common
///  directory is the proof the handler asks for.
/// Why:
///   The record names the common directory,
/// while the trash sits in a linked worktree whose `.git` file points at
///  `<common>/worktrees/<name>`;
/// the handler must accept exactly that and nothing else.
#[test]
fn the_handlers_record_of_a_linked_worktree_is_emptied_from_the_common_directory() {
    let name: String = "5".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-emptied", name.as_str(), 2);
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let mut notices: Vec<u8> = Vec::new();
    let _reason: String = hand_over_with(&git_context(), &pending, &mut notices, &reach(&no_service, &nothing_to_set_up)).expect_err("no service in reach");
    let item: WorkItem = record_of(common.as_path(), &manifest);
    let verdict: Verdict = empty_trash(&item);
    assert_eq!(verdict, Verdict::Done, "the handler empties the trash its record names");
    assert!(!worktree.join(TRASH_PARENT).exists(), "and the trash goes, with the parent that held only it");
    assert!(!state_directory(work_directory(common.as_path()).as_path(), DONE).join(item.id()).exists(), "the drain settles the record, and this face does not");
    remove(root.as_path());
}

/// What:
///  A record whose worktree is gone is done,
/// not failed.
/// Why:
///   A removal takes the worktree and its trash together,
/// so a record that outlived it names nothing on this disk;
/// a failure
///  would report a trash nobody can find.
#[test]
fn the_handler_is_done_with_a_record_whose_worktree_was_removed() {
    let name: String = "6".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-gone", name.as_str(), 1);
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let mut notices: Vec<u8> = Vec::new();
    let _reason: String = hand_over_with(&git_context(), &pending, &mut notices, &reach(&no_service, &nothing_to_set_up)).expect_err("no service in reach");
    let item: WorkItem = record_of(common.as_path(), &manifest);
    std::fs::remove_dir_all(worktree.as_path()).expect("the worktree is removed");
    assert_eq!(empty_trash(&item), Verdict::Done, "nothing is left to empty, so nothing failed");
    remove(root.as_path());
}

/// A provisioning host that no control reaches:
/// the provisioner of a control is a closure.
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

/// What a fixture service did:
/// every line it read,
/// in order.
#[cfg(target_os = "linux")]
struct FixtureService {
    /// Raised to end the accept loop.
    finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The loop,
    /// answering the lines it read.
    thread: std::thread::JoinHandle<Vec<String>>,
}

/// Methods over a fixture service.
#[cfg(target_os = "linux")]
impl FixtureService {
    /// Stop the loop and answer the lines it read.
    fn finish(self) -> Vec<String> {
        self.finished.store(true, std::sync::atomic::Ordering::SeqCst);
        return self.thread.join().expect("the service thread");
    }
}

/// Serve `name` as a service of this release on an abstract socket,
/// which greets and then reads one line per connection.
/// Why:
///   The wake speaks the service's own protocol,
/// and a control must hear the line the wrapper sent without a
///  real unit and a real service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const service = serveThisRelease('trash-hand-over-1234'); // Welcome then one line per connection
/// ```
#[cfg(target_os = "linux")]
fn serve_this_release(name: &str) -> FixtureService {
    use std::os::linux::net::SocketAddrExt;
    use std::io::BufRead;
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
            let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) });
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

/// What:
///  A service that reads the add's line and then keeps sending messages without closing cannot hold the hand-over:
///  the hand-over returns within its own bound.
/// Why:
///   The wake reads until the service closes the connection,
/// which the service does once it has read the line.
/// A service
///  that keeps sending heartbeats instead resets the silence bound on every read,
/// so a loop that only reads has no end,
/// and the
///  whole `git worktree add` waits behind it:
/// the owner's rule is "nothing should ever wait forever or indefinitely".
/// The
///  record is already written when the wake starts,
/// so a wake that reaches its bound costs the person nothing it was going
///  to get.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = withinSeconds(30, handOverWith({ trash, reach: chattyService() }));
/// expect(said).toBeDefined();
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_service_that_keeps_talking_does_not_hold_the_hand_over() {
    let name: String = "7".repeat(40);
    let (root, worktree, common, manifest) = worktree_with_planted_trash("hand-over-chatty", name.as_str(), 1);
    let address: String = format!("trash-hand-over-chatty-{}", std::process::id());
    let service: ChattyService = serve_heartbeats(address.as_str());
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    let pending: PendingTrash = pending_of(worktree.as_path(), common.as_path(), manifest.clone());
    let router = move |_git: &Path, _directory: &Path| return Some(route.clone());
    let (sender, receiver) = std::sync::mpsc::channel::<Result<(), String>>();
    let hand_over = std::thread::spawn(move || {
        let mut notices: Vec<u8> = Vec::new();
        let outcome: Result<(), String> = hand_over_with(&git_context(), &pending, &mut notices, &reach(&router, &nothing_to_set_up));
        let _sent = sender.send(outcome);
    });
    let outcome: Result<(), String> = receiver.recv_timeout(std::time::Duration::from_secs(30)).expect("the hand-over returns within its own bound, whatever the service keeps sending");
    assert!(outcome.is_ok(), "the line reached the service, which is all the wake asks of it: {outcome:?}");
    let _joined = hand_over.join();
    service.finish();
    remove(root.as_path());
}

/// A fixture service that greets,
/// reads one line,
/// and then keeps sending heartbeats without closing the connection.
#[cfg(target_os = "linux")]
struct ChattyService {
    /// Raised to end the heartbeat loop.
    finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The accept loop.
    thread: std::thread::JoinHandle<()>,
}

/// Methods over the chatty fixture service.
#[cfg(target_os = "linux")]
impl ChattyService {
    /// Stop the loop and wait for it.
    fn finish(self) {
        self.finished.store(true, std::sync::atomic::Ordering::SeqCst);
        let _joined = self.thread.join();
    }
}

/// Serve `name` as a service of this release on an abstract socket,
/// which greets,
/// reads one line,
/// and then sends a
///  heartbeat every few milliseconds until it is stopped or the client hangs up.
/// Why:
///   A service that never closes a connection it has read is the state the wake must be bounded against,
/// and a
///  fixture is cheaper and more deterministic than a wedged release service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serveHeartbeats = (name) => { onConnect: write(welcome); readLine(); while (!stopped) { write(heartbeat); await sleep(5); } };
/// ```
#[cfg(target_os = "linux")]
fn serve_heartbeats(name: &str) -> ChattyService {
    use std::os::linux::net::SocketAddrExt;
    use std::io::BufRead;
    let socket = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("a name");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind_addr(&socket).expect("bind");
    listener.set_nonblocking(true).expect("non-blocking");
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
    let thread = std::thread::spawn(move || {
        while !stop.load(std::sync::atomic::Ordering::SeqCst) {
            let Ok((stream, _)) = listener.accept() else {
                std::thread::sleep(std::time::Duration::from_millis(5));
                continue;
            };
            stream.set_nonblocking(false).expect("blocking");
            let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) });
            greeting.push('\n');
            if std::io::Write::write_all(&mut &stream, greeting.as_bytes()).is_err() {
                continue;
            }
            let mut raw: String = String::new();
            if std::io::BufReader::new(&stream).read_line(&mut raw).is_err() {
                continue;
            }
            let mut beat: String = encode_service(&ServiceMessage::Heartbeat);
            beat.push('\n');
            while !stop.load(std::sync::atomic::Ordering::SeqCst) && std::io::Write::write_all(&mut &stream, beat.as_bytes()).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    });
    return ChattyService { finished, thread };
}
