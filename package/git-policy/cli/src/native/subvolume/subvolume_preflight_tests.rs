//! Controls for the refusals of the migration:
//! the paths of a layout,
//! every condition that stops a migration with its
//!  cause and remedy,
//! the leftovers of an earlier run that the journal explains,
//! and the refusals of the rollback and of
//!  the deletion of the old tree.
//! They run on the stand-in platform,
//! so no btrfs is needed.

use super::*;
use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_test_support::PlainPlatform;
use crate::test_support::{fixture, remove};
use std::fs::File;

/// A directory with a `.git` directory,
/// as a bare-minimum checkout,
/// and its layout.
fn checkout(name: &str) -> (PathBuf, Layout) {
    let parent: PathBuf = fixture(name);
    let root: PathBuf = parent.join("checkout");
    std::fs::create_dir_all(root.join(".git")).expect("git directory");
    let layout: Layout = layout_of(root.as_path()).expect("layout");
    return (parent, layout);
}

/// The journal of a layout,
/// empty.
fn empty_journal(layout: &Layout) -> Journal {
    return Journal::load(layout.journal_path.as_path()).expect("journal");
}

/// The causes of the refusals,
/// joined for substring checks.
fn causes(found: &[Refusal]) -> String {
    return found.iter().map(|refusal: &Refusal| { return refusal.cause.as_str(); }).collect::<Vec<&str>>().join("\n");
}

/// A platform that cannot identify any filesystem.
struct Blind(PlainPlatform);

/// Everything is delegated except the filesystem question,
/// which fails.
impl Platform for Blind {
    fn filesystem_facts(&self, path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
        return Err(PrimitiveError { operation: "statfs", path: path.to_path_buf(), errno: libc::EIO });
    }
    fn is_subvolume_root(&self, path: &Path) -> Result<bool, PrimitiveError> {
        return self.0.is_subvolume_root(path);
    }
    fn create_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.0.create_subvolume(path);
    }
    fn destroy_subvolume(&self, path: &Path) -> Result<(), PrimitiveError> {
        return self.0.destroy_subvolume(path);
    }
    fn removal_allowed(&self, path: &Path) -> bool {
        return self.0.removal_allowed(path);
    }
    fn is_mount_point(&self, path: &Path) -> bool {
        return self.0.is_mount_point(path);
    }
    fn clone_file(&self, source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
        return self.0.clone_file(source, destination, path);
    }
    fn exchange(&self, first: &Path, second: &Path) -> Result<(), PrimitiveError> {
        return self.0.exchange(first, second);
    }
}

/// The names beside the checkout derive from the checkout's own name,
/// and a root with no name has no layout.
#[test]
fn the_layout_derives_every_path_from_the_checkout_name() {
    let layout: Layout = layout_of(Path::new("/work/repo")).expect("layout");
    assert_eq!(layout.parent, PathBuf::from("/work"));
    assert_eq!(layout.new_path, PathBuf::from("/work/repo.subvol-new"));
    assert_eq!(layout.old_path, PathBuf::from("/work/repo.before-subvolume"));
    assert_eq!(layout.journal_path, PathBuf::from("/work/.cli-git-migrate-repo.journal.json"));
    assert_eq!(layout.unreadable_path, PathBuf::from("/work/.cli-git-migrate-repo.unreadable.txt"));
    assert_eq!(layout_of(Path::new("/")), None);
    assert_eq!(layout_of(Path::new("repo")).expect("relative").parent, PathBuf::from(""));
}

/// A plain checkout on a btrfs-looking filesystem with room has nothing in its way.
#[test]
fn a_ready_checkout_has_no_refusal() {
    let (parent, layout) = checkout("preflight-ready");
    let found: Vec<Refusal> = migrate_refusals(&PlainPlatform::new(), &layout, &empty_journal(&layout));
    assert_eq!(found, Vec::<Refusal>::new());
    remove(parent.as_path());
}

/// A missing root and a root that is a link or a file are refused before anything else is asked.
#[test]
fn a_missing_root_and_a_non_directory_root_are_refused_alone() {
    let (parent, layout) = checkout("preflight-root-shapes");
    let platform: PlainPlatform = PlainPlatform::new();
    let missing: Layout = layout_of(parent.join("absent").as_path()).expect("layout");
    let found: Vec<Refusal> = migrate_refusals(&platform, &missing, &empty_journal(&missing));
    assert_eq!(found.len(), 1);
    assert!(found[0].cause.contains("cannot be examined") && found[0].remedy.contains("main checkout"), "{found:?}");
    std::os::unix::fs::symlink(layout.root.as_path(), parent.join("link")).expect("link");
    let linked: Layout = layout_of(parent.join("link").as_path()).expect("layout");
    let through_link: Vec<Refusal> = migrate_refusals(&platform, &linked, &empty_journal(&linked));
    assert_eq!(through_link.len(), 1);
    assert!(through_link[0].cause.contains("symbolic link or a file"), "{through_link:?}");
    remove(parent.as_path());
}

/// A filesystem that is not btrfs is named (overlayfs separately),
/// with a remedy;
/// one that cannot be identified is
///  reported with the operating system's words.
#[test]
fn the_filesystem_conditions_are_named() {
    let (parent, layout) = checkout("preflight-filesystem");
    let journal: Journal = empty_journal(&layout);
    let mut platform: PlainPlatform = PlainPlatform::new();
    platform.kind = FilesystemKind::Overlayfs;
    let text: String = causes(migrate_refusals(&platform, &layout, &journal).as_slice());
    assert!(text.contains("is on overlayfs, not on btrfs"), "{text}");
    platform.kind = FilesystemKind::Other;
    let found: Vec<Refusal> = migrate_refusals(&platform, &layout, &journal);
    assert!(causes(found.as_slice()).contains("a filesystem other than btrfs") && found[0].remedy.contains("move the checkout to a btrfs filesystem"), "{found:?}");
    let blind: Vec<Refusal> = migrate_refusals(&Blind(PlainPlatform::new()), &layout, &journal);
    assert!(causes(blind.as_slice()).contains("cannot be identified") && blind[0].remedy.contains("can read it"), "{blind:?}");
    remove(parent.as_path());
}

/// The requirement grows with the entries of the checkout:
/// one byte under it is refused with the entry count and both
///  numbers,
/// exactly the requirement is accepted.
#[test]
fn free_space_is_compared_with_the_requirement_of_the_entries() {
    let (parent, layout) = checkout("preflight-space");
    let journal: Journal = empty_journal(&layout);
    let files: u64 = 7;
    for number in 0..files {
        std::fs::write(layout.root.join(format!("file-{number}")), b"x").expect("file");
    }
    // The `.git` directory of the fixture is an entry too.
    let entries: u64 = files + 1;
    let needed: u64 = required_free_bytes(entries);
    let mut platform: PlainPlatform = PlainPlatform::new();
    platform.free = Some(needed - 1);
    let found: Vec<Refusal> = migrate_refusals(&platform, &layout, &journal);
    let text: String = causes(found.as_slice());
    assert!(text.contains(format!("{} bytes free", needed - 1).as_str()), "{text}");
    assert!(text.contains(format!("{needed} bytes").as_str()), "{text}");
    assert!(text.contains(format!("{entries} entries").as_str()), "{text}");
    platform.free = Some(needed);
    assert_eq!(migrate_refusals(&platform, &layout, &journal), Vec::<Refusal>::new());
    remove(parent.as_path());
}

/// A small checkout does not need a fixed gibibytes:
/// the need is the entries times the measured cost of one.
#[test]
fn a_small_checkout_needs_only_the_space_of_its_entries() {
    let (parent, layout) = checkout("preflight-small");
    let journal: Journal = empty_journal(&layout);
    let mut platform: PlainPlatform = PlainPlatform::new();
    platform.free = Some(required_free_bytes(1) * 2);
    assert_eq!(migrate_refusals(&platform, &layout, &journal), Vec::<Refusal>::new());
    remove(parent.as_path());
}

/// The cost of an entry multiplies by the entries and cannot overflow on an absurd count.
#[test]
fn the_requirement_is_linear_in_entries_and_saturates() {
    assert_eq!(required_free_bytes(0), 0);
    assert_eq!(required_free_bytes(10), required_free_bytes(1) * 10);
    assert_eq!(required_free_bytes(u64::MAX), u64::MAX);
}

/// A mount point cannot be swapped,
/// and a root on another device than its parent says so.
#[test]
fn a_mount_point_and_a_root_on_another_device_are_refused() {
    let (parent, layout) = checkout("preflight-mount");
    let mut platform: PlainPlatform = PlainPlatform::new();
    platform.mount_point = true;
    let text: String = causes(migrate_refusals(&platform, &layout, &empty_journal(&layout)).as_slice());
    assert!(text.contains("is a mount point"), "{text}");
    remove(parent.as_path());
    let shared_memory: &Path = Path::new("/dev/shm");
    let on_other_device: bool = match (std::fs::metadata(shared_memory), std::fs::metadata("/dev")) {
        (Ok(inner), Ok(outer)) => std::os::unix::fs::MetadataExt::dev(&inner) != std::os::unix::fs::MetadataExt::dev(&outer),
        _ => false,
    };
    if !on_other_device {
        eprintln!("DEVICE CONTROL NOT RUN: /dev/shm is not a separate device here, so no directory on another device than its parent was available");
        return;
    }
    let device_layout: Layout = layout_of(shared_memory).expect("layout");
    let device_text: String = causes(migrate_refusals(&PlainPlatform::new(), &device_layout, &empty_journal(&device_layout)).as_slice());
    assert!(device_text.contains("are on different devices"), "{device_text}");
}

/// A checkout that is a subvolume already,
/// with no record of this command,
/// is not migrated again;
/// a recorded exchange
///  explains it.
#[test]
fn an_unrecorded_subvolume_root_is_refused_and_a_recorded_one_is_not() {
    let (parent, layout) = checkout("preflight-already");
    let platform: PlainPlatform = PlainPlatform::new();
    platform.register(layout.root.as_path());
    let mut journal: Journal = empty_journal(&layout);
    let text: String = causes(migrate_refusals(&platform, &layout, &journal).as_slice());
    assert!(text.contains("already a subvolume"), "{text}");
    journal.mark("exchange", "started").expect("mark");
    assert_eq!(migrate_refusals(&platform, &layout, &journal), Vec::<Refusal>::new());
    remove(parent.as_path());
}

/// The new tree and the old tree beside the checkout are refused unless the journal says this command made them.
#[test]
fn leftovers_beside_the_checkout_are_refused_unless_the_journal_explains_them() {
    let (parent, layout) = checkout("preflight-leftovers");
    let platform: PlainPlatform = PlainPlatform::new();
    std::fs::create_dir(layout.new_path.as_path()).expect("new tree");
    std::fs::create_dir(layout.old_path.as_path()).expect("old tree");
    let mut journal: Journal = empty_journal(&layout);
    let found: Vec<Refusal> = migrate_refusals(&platform, &layout, &journal);
    let text: String = causes(found.as_slice());
    assert!(text.contains(layout.new_path.to_str().expect("path")) && text.contains("no migration journal"), "{text}");
    assert!(text.contains(layout.old_path.to_str().expect("path")) && text.contains("where the original tree would be moved"), "{text}");
    assert!(found.iter().any(|refusal: &Refusal| { return refusal.remedy.contains("--remove-old"); }));
    journal.mark("create", "started").expect("mark");
    let explained: String = causes(migrate_refusals(&platform, &layout, &journal).as_slice());
    assert!(!explained.contains("no migration journal") && explained.contains("where the original tree would be moved"), "{explained}");
    journal.mark("exchange", "started").expect("mark");
    assert_eq!(migrate_refusals(&platform, &layout, &journal), Vec::<Refusal>::new());
    remove(parent.as_path());
}

/// A `.git` that is not a directory (a linked worktree,
/// or nothing) is refused with the way forward and nothing else
///  about Git is asked.
#[test]
fn a_checkout_without_a_git_directory_is_refused() {
    let (parent, layout) = checkout("preflight-gitfile");
    std::fs::remove_dir(layout.root.join(".git")).expect("remove");
    std::fs::write(layout.root.join(".git"), b"gitdir: elsewhere\n").expect("gitfile");
    let found: Vec<Refusal> = migrate_refusals(&PlainPlatform::new(), &layout, &empty_journal(&layout));
    assert_eq!(found.len(), 1);
    assert!(found[0].cause.contains("not a directory") && found[0].remedy.contains("not from a linked worktree"), "{found:?}");
    remove(parent.as_path());
}

/// Every in-progress name refuses in the main Git directory and in a linked worktree's administrative directory,
/// and the
///  refusal names the file.
#[test]
fn every_in_progress_marker_is_refused_where_git_keeps_it() {
    let (parent, layout) = checkout("preflight-in-progress");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = layout.root.join(".git/worktrees/linked");
    std::fs::create_dir_all(admin.as_path()).expect("admin directory");
    for directory in [layout.root.join(".git"), admin.clone()] {
        for name in IN_PROGRESS_NAMES {
            let marker: PathBuf = directory.join(name);
            std::fs::write(marker.as_path(), b"").expect("marker");
            let found: Vec<Refusal> = migrate_refusals(&platform, &layout, &empty_journal(&layout));
            assert_eq!(found.len(), 1, "{name}: {found:?}");
            assert!(found[0].cause.contains(marker.to_str().expect("path")) && found[0].remedy.contains("Finish or abort"), "{name}: {found:?}");
            std::fs::remove_file(marker.as_path()).expect("remove marker");
        }
    }
    remove(parent.as_path());
}

/// What:
///  A bisect that was killed before it opened its log is refused,
///  in the main Git directory and in a linked worktree's administrative directory.
/// Why:
///  `git bisect start` writes `BISECT_START`, then `BISECT_NAMES`,
/// and opens `BISECT_LOG` only after both
///  (measured with Git 2.56 under `strace`),
///  so a process killed between those writes leaves a bisect in progress whose only mark is a file written before the kill,
///  and a preflight that knew only the log migrated the checkout with that bisect still unfinished.
#[test]
fn a_bisect_killed_before_its_log_is_refused() {
    let (parent, layout) = checkout("preflight-bisect-killed");
    let admin: PathBuf = layout.root.join(".git/worktrees/linked");
    std::fs::create_dir_all(admin.as_path()).expect("admin directory");
    for directory in [layout.root.join(".git"), admin.clone()] {
        let start: PathBuf = directory.join("BISECT_START");
        std::fs::write(start.as_path(), b"main\n").expect("the file Git writes before its log");
        let found: Vec<Refusal> = migrate_refusals(&PlainPlatform::new(), &layout, &empty_journal(&layout));
        assert_eq!(found.len(), 1, "{}: {found:?}", start.display());
        assert!(found[0].cause.contains("BISECT_START"), "{found:?}");
        std::fs::remove_file(start.as_path()).expect("remove the mark");
    }
    remove(parent.as_path());
}

/// The migration's own locks (the main one and a linked worktree's) are accepted when they hold the journal's token,
/// and
///  no other lock is:
/// an empty file,
/// another migration's token,
/// or a Git command's text are all someone else's.
#[test]
fn the_migrations_own_lock_is_accepted_only_when_it_holds_its_token() {
    let (parent, layout) = checkout("preflight-own-lock");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = layout.root.join(".git/worktrees/linked");
    std::fs::create_dir_all(admin.as_path()).expect("admin directory");
    let journal: Journal = empty_journal(&layout);
    let token: String = token_of(&journal);
    for lock in [layout.root.join(".git/index.lock"), admin.join("index.lock")] {
        std::fs::write(lock.as_path(), b"").expect("an empty lock");
        assert_eq!(migrate_refusals(&platform, &layout, &journal).len(), 1, "an empty lock is someone else's");
        std::fs::write(lock.as_path(), b"monochromatic-git-policy-cli migrate-to-subvolume another start time\n").expect("another migration's lock");
        assert_eq!(migrate_refusals(&platform, &layout, &journal).len(), 1, "another migration's lock is someone else's");
        std::fs::write(lock.as_path(), token.as_bytes()).expect("our lock");
        assert_eq!(migrate_refusals(&platform, &layout, &journal), Vec::<Refusal>::new(), "{}", lock.display());
        std::fs::remove_file(lock.as_path()).expect("remove");
    }
    remove(parent.as_path());
}

/// Unfinished commit-service work in either queue,
/// of the main Git directory or of a linked worktree's,
/// is refused;
/// an
///  empty queue directory is not.
#[test]
fn unfinished_service_work_is_refused_in_both_queues() {
    let (parent, layout) = checkout("preflight-queues");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = layout.root.join(".git/worktrees/linked");
    std::fs::create_dir_all(admin.as_path()).expect("admin directory");
    for directory in [layout.root.join(".git"), admin] {
        for queue in ["pending", "claimed"] {
            let path: PathBuf = directory.join("monochromatic-git-policy-cli/work").join(queue);
            std::fs::create_dir_all(path.as_path()).expect("queue");
            assert_eq!(migrate_refusals(&platform, &layout, &empty_journal(&layout)), Vec::<Refusal>::new(), "an empty {queue}");
            std::fs::write(path.join("work-1"), b"x").expect("work record");
            let found: Vec<Refusal> = migrate_refusals(&platform, &layout, &empty_journal(&layout));
            assert_eq!(found.len(), 1, "{queue}: {found:?}");
            assert!(found[0].cause.contains(path.to_str().expect("path")) && found[0].remedy.contains("service status"), "{found:?}");
            std::fs::remove_file(path.join("work-1")).expect("remove record");
        }
    }
    remove(parent.as_path());
}

/// A rollback needs a journal;
/// with one it has no refusal.
#[test]
fn a_rollback_needs_a_journal() {
    let (parent, layout) = checkout("preflight-rollback");
    let mut journal: Journal = empty_journal(&layout);
    let found: Vec<Refusal> = rollback_refusals(&layout, &journal);
    assert_eq!(found.len(), 1);
    assert!(found[0].cause.contains("no migration journal") && found[0].remedy.contains("mv --exchange"), "{found:?}");
    journal.mark("create", "started").expect("mark");
    assert_eq!(rollback_refusals(&layout, &journal), Vec::<Refusal>::new());
    remove(parent.as_path());
}

/// The deletion of the old tree needs a finished migration and an old tree;
/// each missing part is its own refusal.
#[test]
fn deleting_the_old_tree_needs_a_finished_migration_and_an_old_tree() {
    let (parent, layout) = checkout("preflight-remove-old");
    let mut journal: Journal = empty_journal(&layout);
    let found: Vec<Refusal> = remove_old_refusals(&layout, &journal);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].cause.contains("not recorded as finished") && found[0].remedy.contains("--rollback"), "{found:?}");
    assert!(found[1].cause.contains("does not exist") && found[1].remedy.contains("removed already"), "{found:?}");
    journal.mark("verify", "done").expect("mark");
    assert_eq!(remove_old_refusals(&layout, &journal).len(), 1);
    std::fs::create_dir(layout.old_path.as_path()).expect("old tree");
    assert_eq!(remove_old_refusals(&layout, &journal), Vec::<Refusal>::new());
    journal.mark("verify", "started").expect("mark");
    assert_eq!(remove_old_refusals(&layout, &journal).len(), 1, "a started verification is not finished");
    remove(parent.as_path());
}

/// Free space only matters where the copy writes btrfs metadata,
/// so a filesystem that is not btrfs is refused for that and not
///  also for space,
/// and its tree is not listed to be counted.
#[test]
fn free_space_is_not_checked_where_the_filesystem_is_not_btrfs() {
    let (parent, layout) = checkout("preflight-space-other");
    let mut platform: PlainPlatform = PlainPlatform::new();
    platform.kind = FilesystemKind::Other;
    platform.free = Some(0);
    let text: String = causes(migrate_refusals(&platform, &layout, &empty_journal(&layout)).as_slice());
    assert!(text.contains("a filesystem other than btrfs"), "{text}");
    assert!(!text.contains("bytes free"), "{text}");
    remove(parent.as_path());
}
