//! The leave-out of a snapshot of the main checkout (on btrfs), the hand-over of the trash to the service, and the service's
//!  handler: what is set aside, what is kept, what the work item carries, and what the handler refuses to touch.

use super::*;
use crate::claim_time::ClaimTime;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::service_work::{WorkHandlers, drain};
use crate::test_support::{fixture, git, git_context, remove};
use crate::work_item::{DONE, state_directory};
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

/// Take a snapshot of the main checkout for a new worktree `topic` and return what the controls inspect.
fn snapshot_of(root: &Path, repo: &Path) -> (PathBuf, crate::worktree_snapshot_source::SnapshotSource) {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", "../topic"]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("../topic")).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: std::fs::File = crate::fs_beneath::open_root(repo.join(".git/worktrees/topic").as_path()).expect("administrative directory");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect("a snapshot source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice()).map_err(|failure| { return failure.reason; }).expect("snapshot");
    return (stage, source);
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
    let (stage, source) = snapshot_of(root.as_path(), repo.as_path());
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
    let (stage, mut source) = snapshot_of(root.as_path(), repo.as_path());
    source.git_dir = repo.join(".git/worktrees/ordinary");
    let before: usize = std::fs::read_dir(stage.join("node_modules")).expect("listing").count();
    assert_eq!(prune_snapshot(stage.as_path(), &source).expect("pruned"), 0);
    assert_eq!(std::fs::read_dir(stage.join("node_modules")).expect("listing").count(), before);
    remove(root.as_path());
}

/// Without a route to the service the trash is emptied in the foreground, and the work item written for it is withdrawn.
#[test]
fn a_trash_the_service_cannot_be_reached_for_is_emptied_here() {
    let root: PathBuf = fixture("trash-fallback");
    let worktree: PathBuf = root.join("topic");
    let git_dir: PathBuf = root.join("admin");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect(".git file");
    let name: String = "a".repeat(40);
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/file").as_str(), b"left out\n");
    let mut notices: Vec<u8> = Vec::new();
    dispose_pending_trash_with(Path::new("/usr/bin/git"), worktree.as_path(), git_dir.as_path(), &mut notices, &|_git: &Path, _dir: &Path| { return None; });
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
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(worktree.join(".git"), b"gitdir: /nowhere\n").expect(".git file");
    let name: String = "b".repeat(40);
    let item = |oid: &str, parent: &str| -> WorkItem {
        return WorkItem { kind: WorkKind::Trash, repository: worktree.display().to_string(), git_dir: String::from("/x"), branch_ref: String::from(TRASH_REFERENCE), oid: String::from(oid), parent: String::from(parent), created: now_iso(), tries: 0 };
    };
    // Absent is done.
    assert_eq!(empty_trash(&item(name.as_str(), name.as_str())), Verdict::Done);
    // A name that is not forty lowercase hexadecimal digits, or a parent that differs, is refused before any path is built.
    for (oid, parent) in [("../../etc", "../../etc"), ("B".repeat(40).as_str(), "B".repeat(40).as_str()), (name.as_str(), "other")] {
        assert!(matches!(empty_trash(&item(oid, parent)), Verdict::Failed(_)), "{oid}");
    }
    // A directory is emptied, and the parent goes with the last one.
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/deep/file").as_str(), b"x");
    assert_eq!(empty_trash(&item(name.as_str(), name.as_str())), Verdict::Done);
    assert!(!worktree.join(TRASH_PARENT).exists());
    // A link at the name is left alone, and what it points at is untouched.
    let target: PathBuf = root.join("precious");
    put(target.as_path(), "file", b"keep");
    std::fs::create_dir_all(worktree.join(TRASH_PARENT)).expect("parent");
    std::os::unix::fs::symlink(&target, worktree.join(TRASH_PARENT).join(&name)).expect("link");
    assert!(matches!(empty_trash(&item(name.as_str(), name.as_str())), Verdict::Failed(_)));
    assert!(target.join("file").exists());
    // A worktree that lost its .git is left alone.
    std::fs::remove_file(worktree.join(TRASH_PARENT).join(&name)).expect("remove link");
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/file").as_str(), b"x");
    std::fs::remove_file(worktree.join(".git")).expect("remove .git");
    assert!(matches!(empty_trash(&item(name.as_str(), name.as_str())), Verdict::Failed(_)));
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
    std::fs::write(worktree.join(".git"), b"gitdir: /nowhere\n").expect(".git file");
    let name: String = "c".repeat(40);
    put(worktree.as_path(), format!("{TRASH_PARENT}/{name}/d0/file").as_str(), b"x");
    let item: WorkItem = WorkItem { kind: WorkKind::Trash, repository: worktree.display().to_string(), git_dir: git_dir.display().to_string(), branch_ref: String::from(TRASH_REFERENCE), oid: name.clone(), parent: name.clone(), created: now_iso(), tries: 0 };
    enqueue(work_directory(git_dir.as_path()).as_path(), &item).expect("enqueued");
    let _next = drain(git_dir.as_path(), &Handler, &ClaimTime::read());
    assert!(!worktree.join(TRASH_PARENT).exists(), "emptied by the drain");
    assert!(state_directory(work_directory(git_dir.as_path()).as_path(), DONE).read_dir().expect("done listing").count() > 0, "settled as done");
    remove(root.as_path());
}
