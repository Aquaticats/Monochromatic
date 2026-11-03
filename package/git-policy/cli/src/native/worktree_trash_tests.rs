//! The leave-out of a snapshot of the main checkout (on btrfs), the hand-over of the trash to the service, and the service's
//!  handler: what is set aside, what is kept, what the work item carries, and what the handler refuses to touch.

use super::*;
use crate::claim_time::ClaimTime;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::iso_time::now_iso;
use crate::service_client::START_WAIT;
use crate::service_provision::ProvisionHost;
use crate::service_route::Route;
use crate::service_work::{Verdict, WorkHandlers, drain, work_directory};
use crate::work_item::{WorkItem, WorkKind, enqueue};
use crate::test_support::{fixture, git, git_context, remove};
use crate::work_item::{DONE, state_directory};
use crate::service_messages::UPDATING_NOTICE;
use crate::service_protocol::{ClientMessage, PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_client, encode_service};
use crate::service_server::Bind;
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked};
use crate::worktree_snapshot::take_snapshot;
use crate::worktree_snapshot_source::choose_snapshot_source;

/// Write a file, making its directories.
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full: PathBuf = root.join(path);
    std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
    std::fs::write(full, bytes).expect("file");
}

/// A main checkout that is a subvolume, with ignored state of every shape the rule tells apart, a tracked directory whose name
///  the rule would leave out, one nested worktree that is an ordinary directory and one that is a subvolume.
fn main_with_state(root: &Path) -> PathBuf {
    let repo: PathBuf = subvolume_repository(root);
    tracked(repo.as_path(), "src/.cache/tracked.txt", b"tracked inside a directory named like a cache\n");
    commit_all(repo.as_path(), "tracked cache name");
    std::fs::write(repo.join(".git/info/exclude"), b".claude/\nnode_modules/\ntarget/\n.cache/\nclone/\npackage/\n").expect("exclude");
    put(repo.as_path(), "node_modules/pkg/index.js", b"module\n");
    put(repo.as_path(), "node_modules/.cache/jiti/x.js", b"cache\n");
    put(repo.as_path(), "node_modules/.monochromatic/log.jsonl", b"log\n");
    put(repo.as_path(), "target/debug/deps/libz.rlib", b"rlib\n");
    put(repo.as_path(), "target/debug/incremental/state", b"incremental\n");
    put(repo.as_path(), "package/a/.cache/typescript/info", b"cache\n");
    put(repo.as_path(), ".cache/forbidden-strings.rules.txt", b"rules\n");
    put(repo.as_path(), "clone/.git/HEAD", b"ref: refs/heads/main\n");
    put(repo.as_path(), "clone/file", b"another checkout\n");
    std::fs::create_dir_all(repo.join(".claude/worktrees")).expect("worktrees directory");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", ".claude/worktrees/ordinary"]);
    subvolume_create_at(repo.join(".claude/worktrees/volume").as_path()).expect("nested subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", ".claude/worktrees/volume"]);
    return repo;
}

/// Plant a trash directory `name` below the trash parent of `worktree` with `count` entries, each holding a file, and return
///  the manifest a leave-out would have written for it.
fn plant_trash(worktree: &Path, name: &str, count: usize) -> TrashManifest {
    for index in 0..count {
        put(worktree, format!("{TRASH_PARENT}/{name}/{}/file", entry_name(index)).as_str(), b"left out\n");
    }
    let root: std::fs::File = open_root(worktree).expect("the worktree");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("plain parent");
    let parent: std::fs::File = open_directory_beneath(&root, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = open_directory_at(&parent, std::ffi::OsStr::new(name)).expect("the trash");
    let entries: Vec<crate::fs_beneath::ObjectId> = (0..count).map(|index: usize| { return identity_at(&trash, std::ffi::OsStr::new(entry_name(index).as_str())).expect("entry identity"); }).collect();
    return TrashManifest { name: String::from(name), trash: identity_of(&trash).expect("trash identity"), entries };
}

/// A work item for the trash a manifest describes, as the hand-over writes it.
fn item_of(worktree: &Path, git_dir: &Path, manifest: &TrashManifest) -> WorkItem {
    return WorkItem { kind: WorkKind::Trash, repository: worktree.display().to_string(), git_dir: git_dir.display().to_string(), branch_ref: String::from(TRASH_REFERENCE), oid: manifest.name.clone(), parent: manifest.encode(), created: now_iso(), tries: 0 };
}

/// Take a snapshot of the main checkout for a new worktree `topic` and return what the controls inspect.
fn snapshot_of(root: &Path, repo: &Path) -> (PathBuf, crate::worktree_snapshot_source::SnapshotSource, Option<TrashManifest>) {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("../topic")).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: std::fs::File = crate::fs_beneath::open_root(repo.join(".git/worktrees/topic").as_path()).expect("administrative directory");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect("a snapshot source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let taken = take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice()).map_err(|failure| { return failure.reason; }).expect("snapshot");
    return (stage, source, taken.trash);
}

/// The trash directory a stage holds, if any.
fn trash_of(stage: &Path) -> Option<PathBuf> {
    let parent: PathBuf = stage.join(TRASH_PARENT);
    return std::fs::read_dir(parent).ok()?.flatten().map(|entry: std::fs::DirEntry| { return entry.path(); }).next();
}

/// A snapshot of the main checkout sets aside the ordinary nested worktree (its pointer unlinked first), the other checkout and
///  the caches and run records, keeps what a new worktree needs and every tracked directory, and the main checkout is not
///  blocked by the ordinary nested worktree.
#[test]
fn btrfs_a_snapshot_of_main_sets_aside_exactly_what_the_rule_leaves_out() {
    let Some(root) = btrfs_fixture("trash-snapshot") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, source, _trash) = snapshot_of(root.as_path(), repo.as_path());
    assert!(source.trash_ignored, "the exclude file ignores the place the trash is made");
    for kept in ["node_modules/pkg/index.js", "target/debug/deps/libz.rlib", ".cache/forbidden-strings.rules.txt", "ignored/pkg/index.js", "src/.cache/tracked.txt"] {
        assert!(stage.join(kept).exists(), "{kept} stays in the new worktree");
    }
    for gone in ["node_modules/.cache", "node_modules/.monochromatic", "target/debug/incremental", "package/a/.cache", "clone", ".claude/worktrees/ordinary"] {
        assert!(std::fs::symlink_metadata(stage.join(gone)).is_err(), "{gone} is not in the new worktree any more");
    }
    let trash: PathBuf = trash_of(stage.as_path()).expect("a trash directory");
    assert!(is_trash_name(trash.file_name().expect("name").to_str().expect("utf8")));
    let aside: Vec<String> = std::fs::read_dir(&trash).expect("trash listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(aside.len(), 6, "{aside:?}: the ordinary worktree, the clone, and four left-out directories");
    // The pointer of the copied ordinary worktree is gone before anything else could use it.
    for entry in std::fs::read_dir(&trash).expect("trash listing").flatten() {
        let pointer: PathBuf = entry.path().join(".git");
        assert!(!pointer.is_file(), "no copied .git file survives: {}", pointer.display());
    }
    remove(root.as_path());
}

/// The blocker that declined a snapshot over an ordinary nested worktree no longer applies to the main checkout, and still
///  applies to a linked worktree.
#[test]
fn btrfs_the_ordinary_nested_worktree_does_not_block_a_snapshot_of_main() {
    let Some(root) = btrfs_fixture("trash-blocker") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let plan = plan_to(std::fs::canonicalize(repo.join("../topic")).expect("destination"));
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("the main checkout is a source");
    assert_eq!(crate::worktree_snapshot_source::snapshot_blocker(&source), None);
    let mut as_linked = source.clone();
    as_linked.git_dir = repo.join(".git/worktrees/ordinary");
    assert!(crate::worktree_snapshot_source::snapshot_blocker(&as_linked).is_some_and(|reason: String| { return reason.contains("ordinary directory"); }));
    remove(root.as_path());
}

/// A linked worktree sets nothing aside, whatever it holds.
#[test]
fn btrfs_a_linked_source_sets_nothing_aside() {
    let Some(root) = btrfs_fixture("trash-linked") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, mut source, _trash) = snapshot_of(root.as_path(), repo.as_path());
    source.git_dir = repo.join(".git/worktrees/ordinary");
    let before: usize = std::fs::read_dir(stage.join("node_modules")).expect("listing").count();
    assert_eq!(prune_snapshot(stage.as_path(), &source).expect("pruned").count, 0);
    assert_eq!(std::fs::read_dir(stage.join("node_modules")).expect("listing").count(), before);
    remove(root.as_path());
}

/// Without a route to the service the trash is emptied in the foreground, and the work item written for it is withdrawn once
///  it is gone.
#[test]
fn a_trash_the_service_cannot_be_reached_for_is_emptied_here() {
    let root: PathBuf = fixture("trash-fallback");
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let name: String = "a".repeat(40);
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 2);
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return None; }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT }).expect("emptied here");
    assert!(!worktree.join(TRASH_PARENT).join(&name).exists(), "emptied in the foreground");
    assert!(!worktree.join(TRASH_PARENT).exists(), "and its parent goes with the last one");
    let pending: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(!pending.exists(), "the work item written before the route was tried is withdrawn when the trash is emptied here");
    remove(root.as_path());
}

/// The service's handler empties the one directory a well-formed item names, and refuses everything else.
#[test]
fn the_handler_empties_only_the_trash_it_is_told_about() {
    let root: PathBuf = fixture("trash-handler");
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let name: String = "b".repeat(40);
    // Absent is done.
    let absent: TrashManifest = TrashManifest { name: name.clone(), trash: crate::fs_beneath::ObjectId { device: 1, inode: 1, birth: None }, entries: Vec::new() };
    assert_eq!(empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &absent)), Verdict::Done);
    // A name that is not forty lowercase hexadecimal digits, or a record that does not say which directory it is, is refused
    //  before any path is built.
    for (oid, parent) in [("../../etc", absent.encode()), ("B".repeat(40).as_str(), absent.encode()), (name.as_str(), String::from("other"))] {
        let mut item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &absent);
        item.oid = String::from(oid);
        item.parent = parent;
        assert!(matches!(empty_trash(&item), Verdict::Failed(_)), "{oid}");
    }
    // A directory is emptied, and the parent goes with the last one.
    let planted: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 1);
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/deep/file").as_str(), b"x");
    assert_eq!(empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &planted)), Verdict::Done);
    assert!(!worktree.join(TRASH_PARENT).exists());
    // A link at the name is left alone, and what it points at is untouched.
    let target: PathBuf = root.join("precious");
    put(target.as_path(), "file", b"keep");
    std::fs::create_dir_all(worktree.join(TRASH_PARENT)).expect("parent");
    std::os::unix::fs::symlink(&target, worktree.join(TRASH_PARENT).join(&name)).expect("link");
    assert!(matches!(empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &planted)), Verdict::Failed(_)));
    assert!(target.join("file").exists());
    // A worktree that lost its .git is left alone.
    std::fs::remove_file(worktree.join(TRASH_PARENT).join(&name)).expect("remove link");
    let again: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 1);
    std::fs::remove_file(worktree.join(".git")).expect("remove .git");
    assert!(matches!(empty_trash(&item_of(worktree.as_path(), git_dir.as_path(), &again)), Verdict::Failed(_)));
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d0/file").exists());
    remove(root.as_path());
}

/// The service's drain takes a trash item through the handler and settles it as done.
#[test]
fn the_drain_empties_a_trash_item() {
    struct Handler;
    impl WorkHandlers for Handler {
        fn push(&self, _items: &[WorkItem]) -> Verdict {
            return Verdict::Done;
        }
        fn maintenance(&self, _item: &WorkItem) -> Verdict {
            return Verdict::Done;
        }
        fn review(&self, _item: &WorkItem) -> Verdict {
            return Verdict::Done;
        }
        fn trash(&self, item: &WorkItem) -> Verdict {
            return empty_trash(item);
        }
    }
    let root: PathBuf = fixture("trash-drain");
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let name: String = "c".repeat(40);
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 1);
    let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
    enqueue(work_directory(git_dir.as_path()).as_path(), &item).expect("enqueued");
    let _next = drain(git_dir.as_path(), &Handler, &ClaimTime::read());
    assert!(!worktree.join(TRASH_PARENT).exists(), "emptied by the drain");
    assert!(state_directory(work_directory(git_dir.as_path()).as_path(), DONE).read_dir().expect("done listing").count() > 0, "settled as done");
    remove(root.as_path());
}

/// What a fixture service did: every line it was sent, in order.
#[cfg(target_os = "linux")]
struct FixtureService {
    /// Raised to end the accept loop.
    finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// The loop, returning the non-empty lines it read.
    thread: std::thread::JoinHandle<Vec<String>>,
}

/// Serve `name` as a service of a release other than this one; when `retires` is true it is replaced by a service of this
///  release as soon as it has been asked to retire, else it goes on answering as the earlier release.
#[cfg(target_os = "linux")]
fn serve_earlier_release(name: &str, retires: bool) -> FixtureService {
    use std::os::linux::net::SocketAddrExt;
    use std::io::BufRead;
    let socket = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind_addr(&socket).expect("bind");
    listener.set_nonblocking(true).expect("non-blocking");
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread = std::thread::spawn({
        let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
        move || {
            let mut heard: Vec<String> = Vec::new();
            let mut replaced: bool = false;
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                let Ok((stream, _)) = listener.accept() else {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    continue;
                };
                stream.set_nonblocking(false).expect("blocking");
                let release: String = if replaced { String::from(RELEASE) } else { String::from("0.0.1-earlier") };
                let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(release) });
                greeting.push('\n');
                let _sent = std::io::Write::write_all(&mut &stream, greeting.as_bytes());
                let mut raw: String = String::new();
                let _read = std::io::BufReader::new(&stream).read_line(&mut raw);
                let line: String = raw.trim_end().to_string();
                if line == encode_client(&ClientMessage::Retire) && retires {
                    replaced = true;
                }
                if !line.is_empty() {
                    heard.push(line);
                }
            }
            return heard;
        }
    });
    return FixtureService { finished, thread };
}

/// A worktree with one trash directory in it, the admin directory it points at, and the manifest of that trash.
#[cfg(target_os = "linux")]
fn worktree_with_trash(label: &str, name: &str) -> (PathBuf, PathBuf, PathBuf, TrashManifest) {
    let root: PathBuf = fixture(label);
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name, 1);
    return (root, worktree, git_dir, manifest);
}

/// A service of an earlier release that never retires is not given the work item: it is asked only to retire, the trash is
///  emptied here, and no record is left for a later service to find.
#[cfg(target_os = "linux")]
#[test]
fn a_service_of_an_earlier_release_that_will_not_retire_is_never_handed_trash() {
    let name: String = "e".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_trash("trash-earlier-stubborn", name.as_str());
    let address: String = format!("trash-earlier-stubborn-{}", std::process::id());
    let service: FixtureService = serve_earlier_release(address.as_str(), false);
    let mut notices: Vec<u8> = Vec::new();
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return Some(route.clone()); }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: std::time::Duration::from_millis(400) }).expect("emptied here");
    service.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let heard: Vec<String> = service.thread.join().expect("service thread");
    assert!(!heard.is_empty() && heard.iter().all(|line: &String| { return *line == encode_client(&ClientMessage::Retire); }), "only retires were sent: {heard:?}");
    assert!(String::from_utf8_lossy(notices.as_slice()).contains(UPDATING_NOTICE), "the person is told the service is being replaced");
    assert!(!worktree.join(TRASH_PARENT).exists(), "the trash is emptied in the foreground");
    let pending: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(!pending.exists(), "no item an earlier service could never claim is left");
    remove(root.as_path());
}

/// A service of an earlier release that retires is replaced before the hand-over: the work item reaches the current service,
///  after the retire, and the trash stays for that service to empty.
#[cfg(target_os = "linux")]
#[test]
fn a_service_of_an_earlier_release_is_replaced_before_trash_is_handed_over() {
    let name: String = "f".repeat(40);
    let (root, worktree, git_dir, manifest) = worktree_with_trash("trash-earlier-retires", name.as_str());
    let address: String = format!("trash-earlier-retires-{}", std::process::id());
    let service: FixtureService = serve_earlier_release(address.as_str(), true);
    let mut notices: Vec<u8> = Vec::new();
    let route: Route = Route { address: Bind::Abstract(address), provision: None };
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &manifest, &mut notices, &Reach { router: &|_git: &Path, _dir: &Path| { return Some(route.clone()); }, provisioner: &|_host: &ProvisionHost| { return Ok(()); }, start_wait: START_WAIT }).expect("handed over");
    service.finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let heard: Vec<String> = service.thread.join().expect("service thread");
    let retire: String = encode_client(&ClientMessage::Retire);
    let work: String = encode_client(&ClientMessage::Work { git_dir: git_dir.clone(), review_key: None });
    let retired_at: usize = heard.iter().position(|line: &String| { return *line == retire; }).expect("the earlier service was asked to retire");
    let worked_at: usize = heard.iter().position(|line: &String| { return *line == work; }).expect("the current service was woken");
    assert!(retired_at < worked_at, "the retire comes first: {heard:?}");
    assert_eq!(heard.iter().filter(|line: &&String| { return **line == work; }).count(), 1, "the earlier release is never sent the work message: {heard:?}");
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d0/file").exists(), "the current service empties it, not the wrapper");
    let pending: PathBuf = state_directory(work_directory(git_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    assert!(pending.exists(), "the item waits for the current service to claim it");
    remove(root.as_path());
}

/// Controls for the findings of the cross-vendor review of the leave-out and its disposal.
#[path = "worktree_trash_safety_tests.rs"]
mod safety;

/// What:
///  The `check-ignore` start that asks whether the trash location is ignored.
/// Why:
///   The owner's rule is that no Git start of the wrapper gets `--quiet` or `-q`; the answer is the exit status in both forms
///  (Git 2.56.0 `builtin/check-ignore.c` gates only the printing on the option).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await trashLocationIsIgnored(recordingGit, repo)).toBe(true); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_trash_location_question_starts_check_ignore_without_quiet() {
    let root: PathBuf = fixture("trash-check-ignore-no-quiet");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), format!("{TRASH_PARENT}/\n").as_bytes()).expect("gitignore");
    let (context, log) = crate::recording_git::recording_context(root.as_path());
    assert!(trash_location_is_ignored(&context, repo.as_path()), "the location is ignored");
    let asks: Vec<String> = crate::recording_git::recorded_starts(log.as_path()).into_iter().filter(|start| { return start.contains("check-ignore"); }).collect();
    assert_eq!(asks.len(), 1, "one start: {asks:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(root.as_path());
}
