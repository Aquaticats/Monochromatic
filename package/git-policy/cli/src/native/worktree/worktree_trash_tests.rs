//! The leave-out of a snapshot of the main checkout (on btrfs),
//! the hand-over of the trash to the service,
//! and the service's
//!  handler:
//! what is set aside,
//! what is kept,
//! what the work item carries,
//! and what the handler refuses to touch.

use super::*;
use crate::worktree_trash_ignore_rule::trash_location_is_ignored;
use crate::claim_time::ClaimTime;
use crate::fs_beneath::{identity_at, identity_of, open_directory_at};
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::iso_time::now_iso;
use crate::service_work::{GitHandlers, Verdict, WorkHandlers, drain, work_directory};
use crate::work_item::{WorkItem, WorkKind, enqueue};
use crate::test_support::{age, fixture, git, git_context, remove, repository};
use crate::worktree_copy_selection::STAGE_PREFIX;
use crate::work_item::{DONE, state_directory};
use crate::worktree_ladder_test_support::{commit_all, main_identity, plan_to, subvolume_repository, tracked};
use crate::worktree_snapshot::take_snapshot;
use crate::worktree_snapshot_source::choose_snapshot_source;

/// Write a file,
/// making its directories.
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full: PathBuf = root.join(path);
    std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
    std::fs::write(full, bytes).expect("file");
}

/// A main checkout that is a subvolume,
/// with ignored state of every shape the rule tells apart,
/// a tracked directory whose name
///  the rule would leave out,
/// one nested worktree that is an ordinary directory and one that is a subvolume.
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

/// Plant a trash directory `name` below the trash parent of `worktree` with `count` entries,
/// each holding a file,
/// and return
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

/// A work item for the trash a manifest describes,
/// as the hand-over writes it.
fn item_of(worktree: &Path, git_dir: &Path, manifest: &TrashManifest) -> WorkItem {
    return WorkItem { kind: WorkKind::Trash, repository: worktree.to_path_buf(), git_dir: git_dir.to_path_buf(), branch_ref: String::from(TRASH_REFERENCE), oid: manifest.name.clone(), parent: manifest.encode(), created: now_iso(), tries: 0 };
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
    let taken = take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice(), &crate::fixture_content_decision::content_environment()).map_err(|failure| { return failure.reason; }).expect("snapshot");
    return (stage, source, taken.trash);
}

/// The trash directory a stage holds,
/// if any.
fn trash_of(stage: &Path) -> Option<PathBuf> {
    let parent: PathBuf = stage.join(TRASH_PARENT);
    return std::fs::read_dir(parent).ok()?.flatten().map(|entry: std::fs::DirEntry| { return entry.path(); }).next();
}

/// A snapshot of the main checkout sets aside the ordinary nested worktree (its pointer unlinked first),
/// the other checkout and
///  the caches and run records,
/// keeps what a new worktree needs and every tracked directory,
/// and the main checkout is not
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
    assert!(aside.contains(&String::from(crate::worktree_trash_own_manifest::MANIFEST_FILE_NAME)), "{aside:?}: the trash says what it is");
    let held: usize = aside.iter().filter(|name: &&String| return name.as_str() != crate::worktree_trash_own_manifest::MANIFEST_FILE_NAME).count();
    assert_eq!(held, 8, "{aside:?}: the ordinary worktree, the clone, four left-out directories, the worktree folder whose stand-in went with it, and the snapshot's own .git");
    // The pointer of the copied ordinary worktree is gone before anything else could use it.
    for entry in std::fs::read_dir(&trash).expect("trash listing").flatten() {
        let pointer: PathBuf = entry.path().join(".git");
        assert!(!pointer.is_file(), "no copied .git file survives: {}", pointer.display());
    }
    remove(root.as_path());
}

/// The blocker that declined a snapshot over an ordinary nested worktree no longer applies to the main checkout,
/// and still
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

/// A linked worktree sets nothing aside by the main checkout's rule,
/// whatever it holds,
/// and the
///  only thing it moves is the trash an earlier add left.
#[test]
fn btrfs_a_linked_source_sets_nothing_aside() {
    let Some(root) = btrfs_fixture("trash-linked") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let (stage, mut source, _trash) = snapshot_of(root.as_path(), repo.as_path());
    source.git_dir = repo.join(".git/worktrees/ordinary");
    let before: usize = std::fs::read_dir(stage.join("node_modules")).expect("listing").count();
    let pruned: Pruned = prune_snapshot(stage.as_path(), &source).expect("pruned");
    assert_eq!(std::fs::read_dir(stage.join("node_modules")).expect("listing").count(), before, "a linked source's dependencies are not set aside");
    // The one directory that moves is the trash the earlier prune in this control made, which no snapshot may carry on.
    assert_eq!(pruned.count, 1, "only the trash an earlier add left is moved");
    remove(root.as_path());
}

/// What:
///  A stage container of another command is set aside by name,
/// wherever it sits below the snapshot,
/// its whole payload with it.
/// Why:
///   A container is an ordinary directory holding the ignored state of another worktree
/// (credentials included),
/// and its
///  destination lies below the same folder as this add's,
/// so a snapshot carries it whole and nothing names it
///  afterwards:
/// a `git add --all` in the new worktree would commit it.
///   The walk needs no btrfs to decide what it leaves out,
/// so the control builds the snapshot's shape in a disposable directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { count } = await pruneSnapshot(stage, source)
/// expect(count).toBe(1)
/// expect(existsSync(join(stage, '.claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543'))).toBe(false)
/// ```
#[test]
fn a_stage_container_below_the_snapshot_is_set_aside() {
    let root: PathBuf = fixture("trash-stage-container");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("kept.txt"), b"kept\n").expect("a tracked file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=kept"]);
    let stage: PathBuf = root.join("stage");
    let container: PathBuf = stage.join(".claude/worktrees").join(format!("{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543"));
    std::fs::create_dir_all(container.join("payload/node_modules/pkg")).expect("the other command's stage");
    std::fs::write(container.join("payload/node_modules/pkg/index.js"), b"another worktree's dependency\n").expect("the other command's payload");
    std::fs::write(stage.join("kept.txt"), b"kept\n").expect("the tracked file in the snapshot");
    let source: SnapshotSource = SnapshotSource {
        root: repo.clone(),
        git_dir: repo.join(".git"),
        common_dir: repo.join(".git"),
        clean: crate::worktree_clean::CleanSource {
            head_commit: String::new(),
            head_tree: Vec::new(),
            tracked: 0,
            index_seconds: 0,
            hash_length: 20,
            attribute_blobs: Vec::new(),
            index_bytes: std::fs::read(repo.join(".git/index")).expect("the index"),
        },
        nested: Vec::new(),
        config: crate::config_snapshot::read_config_snapshot(&git_context(), repo.as_path()).expect("configuration"),
        trash_ignored: false,
        dirty: None,
    };
    let pruned: Pruned = prune_snapshot(stage.as_path(), &source).expect("the leave-out runs");
    assert_eq!(pruned.count, 1, "the container is the one directory set aside");
    assert!(!container.exists(), "the other command's stage container is not carried");
    assert!(stage.join("kept.txt").exists(), "the tracked file stays");
    remove(root.as_path());
}

/// What:
///  A snapshot taken from a linked worktree sets another command's stage container aside,
/// while everything that worktree
///  ignores itself stays.
/// Why:
///   A linked worktree has no leave-out rule of its own,
/// so the stage rule is the one thing the leave-out decides there;
///  the
///  container holds another worktree's ignored state,
/// credentials included,
/// and nothing would name it in the new
///  worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeSnapshot({ source: linkedSource, plan, stage, admin, pointer });
/// expect(existsSync(join(stage, '.claude/worktrees/.cli-git-worktree-copy-<token>'))).toBe(false);
/// expect(existsSync(join(stage, 'node_modules/pkg/index.js'))).toBe(true);
/// ```
#[test]
fn btrfs_a_snapshot_of_a_linked_worktree_sets_another_commands_stage_aside() {
    let Some(root) = btrfs_fixture("trash-linked-stage") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    tracked(repo.as_path(), "tracked.txt", b"tracked\n");
    commit_all(repo.as_path(), "first");
    std::fs::write(repo.join(".git/info/exclude"), b".claude/\nnode_modules/\n").expect("exclude");
    let linked: PathBuf = root.join("linked");
    subvolume_create_at(linked.as_path()).expect("the linked worktree is a subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "linked", "../linked"]);
    // The checkout wrote every file as new as its index; only an aged tree is provably clean.
    for name in [".gitignore", "tracked.txt", "shared.txt", "src/deep/leaf.txt", "new/added.txt"] {
        age(linked.join(name).as_path());
    }
    git(linked.as_path(), &["update-index", "--refresh"]);
    put(linked.as_path(), "node_modules/pkg/index.js", b"module the linked worktree ignores\n");
    let container: String = format!(".claude/worktrees/{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543");
    put(linked.as_path(), format!("{container}/payload/credentials").as_str(), b"another worktree's token\n");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(root.join("topic")).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: std::fs::File = crate::fs_beneath::open_root(repo.join(".git/worktrees/topic").as_path()).expect("administrative directory");
    let identity = crate::worktree_identity::WorktreeIdentity::LinkedWorktree { common_dir: repo.join(".git"), git_dir: repo.join(".git/worktrees/linked"), worktree_root: linked.clone() };
    let plan = plan_to(destination);
    let source: SnapshotSource = choose_snapshot_source(&git_context(), &identity, &plan).expect("a linked worktree is a snapshot source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-linked");
    take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice(), &crate::fixture_content_decision::content_environment()).map_err(|failure| { return failure.reason; }).expect("snapshot");
    assert!(std::fs::symlink_metadata(stage.join(container.as_str())).is_err(), "the other command's stage container is not carried");
    assert!(stage.join("node_modules/pkg/index.js").is_file(), "what the linked worktree ignores stays");
    assert!(linked.join(container.as_str()).join("payload/credentials").is_file(), "the other command's payload is left where it was");
    remove(root.as_path());
}

/// What:
///  The trash a snapshot made is left in the new worktree by the add,
/// which hands it to the commit service with
///  one line to the person,
/// and the service's emptying of it is what deletes anything.
/// Why:
///   The trash sits inside the new worktree,
/// so a target commit that stops ignoring `.claude/` would make it
///  stageable;
/// what keeps it out of the index is the repository's own exclude line,
/// never an emptying the person waits for
///  (`worktree_trash_exclude_line.rs`).
/// The add's own work is the record,
/// and the emptying belongs to the service that runs
///  it one worktree's trashes at a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverTrash({ destination, commonDir, manifest, notices });
/// expect(existsSync(join(worktree, TRASH_PARENT, name))).toBe(true); expect(pendingRecord(commonDir, name)).toBeDefined();
/// ```
#[test]
fn the_add_hands_the_trash_over_and_leaves_it_standing() {
    let root: PathBuf = fixture("trash-hand-over");
    let worktree: PathBuf = root.join("topic");
    let common_dir: PathBuf = root.join("common");
    std::fs::create_dir_all(common_dir.join("worktrees").join("topic")).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", common_dir.join("worktrees").join("topic").display())).expect(".git file");
    let name: String = "a".repeat(40);
    let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 2);
    let pending_trash: PendingTrash = PendingTrash { destination: worktree.clone(), common_dir: common_dir.clone(), manifest: manifest.clone(), taken_over: 0, quiet: false };
    let mut notices: Vec<u8> = Vec::new();
    let _why: String = hand_over_with(&git_context(), &pending_trash, &mut notices, &Reach { router: &|_git: &Path, _directory: &Path| return None, provisioner: &|_host: &crate::service_provision::ProvisionHost| return Ok(()), start_wait: std::time::Duration::from_millis(400) })
        .expect_err("a session with no service is reported to the caller");
    assert!(worktree.join(TRASH_PARENT).join(&name).join("d0/file").is_file(), "the add deletes nothing");
    let record: PathBuf = state_directory(work_directory(common_dir.as_path()).as_path(), "pending").join(format!("trash-{name}"));
    let item: WorkItem = WorkItem::decode(std::fs::read_to_string(record.as_path()).expect("the record").as_str()).expect("a work item");
    assert_eq!(item.git_dir, common_dir, "the record belongs to the repository, so one service empties its trashes one at a time");
    assert_eq!(item.parent, manifest.encode(), "and it carries the identities the leave-out wrote");
    assert_eq!(empty_trash(&item), Verdict::Done, "the handler is what deletes a trash");
    assert!(!worktree.join(TRASH_PARENT).exists(), "and the trash goes, with the parent that held only it");
    assert!(String::from_utf8_lossy(notices.as_slice()).is_empty(), "a hand-over that reached no service says nothing of it; its caller reports the reason");
    remove(root.as_path());
}

/// The service's handler empties the one directory a well-formed item names,
/// and refuses everything else.
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

/// What:
///  The two faces of the service's trash handler empty the trash a task and a plain thread ask about,
/// and they answer the same.
/// Why:
///   The drain of a round is a task of the runtime,
/// and the synchronous face enters the runtime for the entry removals,
///  which is refused from inside it:
/// a task that asked it kept the trash and settled the item as failed (measured on
///  `the_drain_empties_a_trash_item` before this face existed).
/// So the two faces are asked about one fixture each,
/// and each must empty its own trash and answer done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(handlers.trash(threadItem)).toBe('done'); expect(await handlers.trashAwaiting(taskItem)).toBe('done')
/// ```
#[test]
fn the_two_faces_of_the_trash_handler_empty_what_a_task_and_a_thread_ask_about() {
    let root: PathBuf = fixture("trash-handler-faces");
    std::fs::create_dir_all(root.as_path()).expect("the fixture's folder");
    let name: String = "d".repeat(40);
    let mut planted: Vec<PathBuf> = Vec::new();
    for face in ["thread", "task"] {
        let worktree: PathBuf = root.join(face).join("topic");
        let git_dir: PathBuf = root.join(face).join("admin");
        std::fs::create_dir_all(&git_dir).expect("admin");
        std::fs::create_dir_all(&worktree).expect("worktree");
        std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display()).as_str()).expect(".git file");
        let manifest: TrashManifest = plant_trash(worktree.as_path(), name.as_str(), 2);
        let item: WorkItem = item_of(worktree.as_path(), git_dir.as_path(), &manifest);
        let handlers: GitHandlers = GitHandlers::new(git_context());
        let answered: Verdict = if face == "thread" {
            handlers.trash(&item)
        } else {
            // The task of the runtime a drain runs in, which cannot enter it.
            crate::merge_second_pass::tests::await_it(handlers.trash_awaiting(&item))
        };
        assert_eq!(answered, Verdict::Done, "the {face} face empties the trash it is told about");
        assert!(!worktree.join(TRASH_PARENT).exists(), "the {face} face emptied {TRASH_PARENT} and its parent");
        planted.push(worktree);
    }
    assert_eq!(planted.len(), 2);
    remove(root.as_path());
}

/// Controls for the findings of the cross-vendor review of the leave-out and its disposal.
#[path = "worktree_trash_safety_tests.rs"]
mod safety;

/// What:
///  The `check-ignore` start that asks whether the trash location is ignored.
/// Why:
///   The owner's rule is that no Git start of the wrapper gets `--quiet` or `-q`;
/// the answer is the exit status in both forms
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

/// What:
///  A trash an earlier add left in the source,
/// and that says what it is,
/// is not carried into the new snapshot:
/// its directories are moved into this add's trash instead,
/// so the emptying covers them.
/// Why:
///   An add killed after its swap and before its record leaves `.claude/cli-git-trash` in the worktree,
/// holding what
///  it left out,
/// another command's stage container with its credentials among them.
/// The disposal is handed only the manifest
///  of the add that runs,
/// so a carried trash would never be emptied there:
/// the walk skipped the whole trash parent,
///  and the copy reached the new worktree.
/// The trash's own manifest is what proves the wrapper made it and says which
///  directories are its own,
/// so it is moved into this add's trash and emptied with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantTrash(repo, inherited); writeTrashManifest(repo, inherited);
/// const { stage, trash } = await snapshotOf(repo);
/// expect(existsSync(join(stage, TRASH_PARENT, inherited))).toBe(false); expect(trash.entries.length).toBeGreaterThan(0);
/// ```
#[test]
fn btrfs_a_snapshot_does_not_carry_an_inherited_trash_forward() {
    let Some(root) = btrfs_fixture("trash-inherited") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let inherited: String = "b".repeat(40);
    let inherited_manifest: TrashManifest = plant_trash(repo.as_path(), inherited.as_str(), 1);
    crate::worktree_trash_own_manifest::write(repo.as_path(), &inherited_manifest).expect("the earlier trash says what it is");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    assert!(std::fs::symlink_metadata(stage.join(TRASH_PARENT).join(&inherited)).is_err(), "the inherited trash is not carried into the new worktree");
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    assert!(!manifest.entries.is_empty(), "what the inherited trash held was moved into this add's trash, so the emptying covers it");
    remove(root.as_path());
}

/// What:
///  Whether any file named `wanted` lies below `root`,
/// walked once,
/// breadth first.
/// Why:
///   The trash renames each directory it sets aside to a fresh name,
/// so a control cannot name the path a
///  container would have there;
/// finding its file says whether it reached the trash at all.
fn holds_a_named_file(root: &Path, wanted: &str) -> bool {
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(listed) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in listed.flatten() {
            if entry.file_name().to_string_lossy() == wanted {
                return true;
            }
            if entry.file_type().is_ok_and(|kind: std::fs::FileType| { return kind.is_dir(); }) {
                pending.push(entry.path());
            }
        }
    }
    return false;
}

/// What:
///  A stage container of another command is removed before the snapshot is installed,
/// not renamed into the
///  trash that waits for its emptying.
/// Why:
///   A container holds another worktree's ignored state,
/// credentials included,
/// and the trash stands inside the
///  new worktree from the install until the emptying runs;
/// a command that runs `git add --all` in that window stages
///  the credentials,
/// and emptying the trash afterwards does not take the blobs out of the index.
/// What the leave-out
///  moves for the caches,
/// the other checkouts and the run records still waits for the emptying,
/// so the add pays only
///  the container's removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stage, trash } = await snapshotOf(repo); // prune, before the swap
/// expect(existsSync(join(stage, container))).toBe(false); expect(findIn(join(stage, TRASH_PARENT), 'credentials')).toEqual([]);
/// expect(existsSync(join(stage, TRASH_PARENT, trash.name))).toBe(true); // the cache still waits there
/// ```
#[test]
fn btrfs_a_stage_container_is_removed_before_the_snapshot_is_installed() {
    let Some(root) = btrfs_fixture("trash-container-before-install") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let container: String = format!(".claude/worktrees/{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543");
    put(repo.as_path(), format!("{container}/payload/credentials").as_str(), b"another worktree's token\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    assert!(std::fs::symlink_metadata(stage.join(container.as_str())).is_err(), "the container is gone before the worktree is installed");
    let manifest: TrashManifest = trash.expect("the leave-out made a trash");
    let trash_dir: PathBuf = stage.join(TRASH_PARENT).join(&manifest.name);
    assert!(trash_dir.is_dir(), "the trash waits for the emptying that runs before the command returns");
    assert!(!holds_a_named_file(stage.join(TRASH_PARENT).as_path(), "credentials"), "the container's credentials are nowhere in the trash");
    assert!(trash_dir.is_dir() && std::fs::read_dir(&trash_dir).expect("listing").count() > 0, "what the leave-out set aside for the cache still waits in the trash");
    remove(root.as_path());
}

/// What:
///  A stage container inside a nested worktree that is moved whole is removed with it,
/// so the trash never holds
///  another command's credentials and a kill before the emptying cannot leave them stageable.
/// Why:
///   The leave-out moves a registered nested worktree that is an ordinary directory as one directory,
/// and the
///  check that removes a container looks only at the name it was handed:
/// a container inside the moved
///  directory has a parent named for the worktree,
/// so nothing removed it and the trash carried it into the new
///  worktree,
/// where a target commit that stops ignoring the path makes it stageable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// put(repo, '.claude/worktrees/ordinary/.claude/worktrees/.cli-git-worktree-copy-<token>/payload/credentials', 'token');
/// const { stage } = await snapshotOf(repo);
/// expect(holdsANamedFile(join(stage, TRASH_PARENT), 'credentials')).toBe(false);
/// ```
#[test]
fn btrfs_a_container_inside_a_nested_worktree_moved_whole_is_removed_with_it() {
    let Some(root) = btrfs_fixture("trash-container-in-nested") else { return };
    let repo: PathBuf = main_with_state(root.as_path());
    let inside: String = format!(".claude/worktrees/ordinary/.claude/worktrees/{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543");
    put(repo.as_path(), format!("{inside}/payload/credentials").as_str(), b"another worktree's token\n");
    let (stage, _source, trash) = snapshot_of(root.as_path(), repo.as_path());
    assert!(std::fs::symlink_metadata(stage.join(".claude/worktrees/ordinary")).is_err(), "the nested worktree itself is still set aside");
    assert!(!holds_a_named_file(stage.join(TRASH_PARENT).as_path(), "credentials"), "the container inside it is not in the trash");
    assert!(trash.is_some(), "the leave-out made a trash for the rest");
    remove(root.as_path());
}
