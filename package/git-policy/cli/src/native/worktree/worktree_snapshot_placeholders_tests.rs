//! Controls for the stand-in removal of the snapshot rung:
//! which folders are listed,
//! what is a stand-in,
//! and what is left
//!  alone,
//! for a folder the snapshot does not carry,
//! a folder outside the source and a folder reached through a link;
//! and the snapshot rung of a source whose worktree folder holds nested subvolumes,
//! which a snapshot shows as stand-ins.

use super::*;
use crate::worktree_snapshot_ahead::AheadService;
use crate::fixture_content_decision::content_environment;
use crate::fs_beneath::{identity_at, identity_of, open_root};
use crate::fs_btrfs_ioctl::{snapshot_create_at, subvolume_create_at};
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, git, git_context, remove};
use crate::worktree_ladder_run::discard_stage;
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};
use crate::worktree_snapshot::take_snapshot;
use crate::worktree_snapshot_source::{SnapshotSource, choose_snapshot_source};
use crate::worktree_trash::{TRASH_PARENT, TrashManifest, entry_name, prune_snapshot};

/// A main checkout that is a subvolume with `count` worktrees in its worktree folder,
/// each a subvolume of its own,
/// and the repository's own exclude line that makes the trash place ignored.
/// Why:
///   A snapshot carries a stand-in where the source holds a nested subvolume,
/// so this is what the folders this step reads
///  hold in the state that matters.
fn main_with_nested_subvolumes(root: &Path, count: usize) -> PathBuf {
    let repo: PathBuf = subvolume_repository(root);
    std::fs::write(repo.join(".git/info/exclude"), b".claude/\ntarget/\n").expect("exclude");
    std::fs::create_dir_all(repo.join(".claude/worktrees")).expect("the worktree folder");
    for index in 0..count {
        let nested: PathBuf = repo.join(".claude/worktrees").join(format!("w{index}"));
        subvolume_create_at(nested.as_path()).expect("a nested subvolume");
        git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", format!(".claude/worktrees/w{index}").as_str()]);
    }
    return repo;
}

/// Ignored state the main-checkout rule sets aside,
/// so the leave-out makes a trash of its own.
fn with_run_state(repo: &Path) {
    let state: PathBuf = repo.join("target/debug/incremental/state");
    std::fs::create_dir_all(state.parent().expect("parent")).expect("directories");
    std::fs::write(state.as_path(), b"incremental\n").expect("file");
}

/// Take the snapshot the snapshot rung takes for the worktree `destination` of the repository,
/// and answer the stage and what the add would hand the service.
/// `trash_ignored` says whether Git ignores the place a snapshot sets its left-out directories aside.
fn snapshot_into(root: &Path, repo: &Path, destination: &str, trash_ignored: bool) -> (PathBuf, Option<TrashManifest>) {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", destination]);
    let made: PathBuf = std::fs::canonicalize(repo.join(destination)).expect("destination");
    let pointer: Vec<u8> = std::fs::read(made.join(".git")).expect("pointer");
    let admin: File = open_root(repo.join(".git/worktrees/topic").as_path()).expect("administrative directory");
    let plan = plan_to(made);
    let mut source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect("a snapshot source");
    source.trash_ignored = trash_ignored;
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let taken = take_snapshot(&source, &plan, stage.as_path(), &admin, pointer.as_slice(), &content_environment(), &AheadService::nothing()).map_err(|failure| { return failure.reason; }).expect("snapshot");
    return (stage, taken.trash);
}

/// The names of a directory,
/// for the listings these controls read.
fn names_in(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory).expect("listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    names.sort();
    return names;
}

/// The inode number of a path,
/// read without following a link.
fn inode_of(path: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt as _;
    return std::fs::symlink_metadata(path).expect("the entry").ino();
}

/// A held folder of a fixture that holds an ordinary empty directory,
/// a file,
/// a directory with a file in it and a link,
///  below a held handle of the same fixture root.
fn folder_of_ordinary_entries(root: &Path) -> (File, File) {
    std::fs::create_dir_all(root.join("folder/empty")).expect("an empty directory");
    std::fs::write(root.join("folder/plain"), b"a file").expect("a file");
    std::fs::create_dir_all(root.join("folder/full")).expect("a full directory");
    std::fs::write(root.join("folder/full/inner"), b"someone's data").expect("a file in it");
    std::fs::create_dir_all(root.join("elsewhere/inner")).expect("a directory outside the folder");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("folder/link")).expect("a link");
    return (open_root(root).expect("the fixture"), open_root(root.join("folder").as_path()).expect("the folder"));
}

/// The parent of each registered worktree and of the destination is named once,
/// and a worktree outside the source names no
///  folder of the snapshot.
#[test]
fn the_folders_that_hold_worktrees_are_each_named_once_and_only_below_the_source() {
    let source_root: &Path = Path::new("/repo");
    let nested: Vec<PathBuf> = vec![PathBuf::from("/repo/.claude/worktrees/first"), PathBuf::from("/repo/.claude/worktrees/second"), PathBuf::from("/elsewhere/worktree")];
    let destination: &Path = Path::new("/repo/.claude/worktrees/topic");
    assert_eq!(worktree_folders(nested.as_slice(), destination, source_root), vec![PathBuf::from("/repo/.claude/worktrees")]);
    let none: Vec<PathBuf> = Vec::new();
    assert_eq!(worktree_folders(none.as_slice(), Path::new("/repo-parent/topic"), source_root), Vec::<PathBuf>::new(), "a destination beside the source is in no snapshot of it");
}

/// A source of a fixture that is nothing more than the root a folder is read below,
/// with the trash place Git ignores or not.
/// Why:
///   The folder step reads the source root,
/// the registered worktrees and whether Git ignores the trash place,
/// so a
///  fixture that names only those three exercises the step without a repository.
fn source_of(root: &Path, trash_ignored: bool) -> SnapshotSource {
    return SnapshotSource {
        root: root.to_path_buf(),
        git_dir: root.join(".git"),
        common_dir: root.join(".git"),
        clean: crate::worktree_clean::CleanSource { head_commit: String::new(), head_tree: Vec::new(), tracked: 0, index_seconds: 0, hash_length: 20, attribute_blobs: Vec::new(), index_bytes: Vec::new() },
        nested: Vec::new(),
        config: crate::config_snapshot::ConfigSnapshot::parse(&[], true).expect("an empty listing"),
        trash_ignored,
        dirty: None,
    };
}

/// No ordinary entry is a stand-in:
/// an empty directory,
/// a file,
/// a directory that holds a file and a link all stay where they
///  are,
/// and the folder is not taken.
#[test]
fn no_ordinary_entry_is_a_stand_in() {
    let root: PathBuf = fixture("snapshot-placeholders-ordinary");
    let (held, folder) = folder_of_ordinary_entries(root.as_path());
    for name in ["empty", "plain", "full", "link", "missing"] {
        assert!(!is_stand_in(&folder, OsStr::new(name)), "{name} is no stand-in");
        assert!(!is_an_empty_stand_in(&folder, OsStr::new(name)), "{name} is no empty stand-in");
    }
    let source: SnapshotSource = source_of(root.as_path(), false);
    let plan = plan_to(root.join("folder/topic"));
    let mut set_aside: HandOver = HandOver::starting(None);
    assert_eq!(remove_snapshot_placeholders(&held, &source, &plan, &mut set_aside), Ok(()), "nothing is to be emptied");
    assert_eq!(set_aside.finish(), None, "and no trash is made");
    assert_eq!(names_in(root.join("folder").as_path()), vec!["empty", "full", "link", "plain"], "every ordinary entry stays");
    assert_eq!(std::fs::read(root.join("folder/full/inner")).expect("the data"), b"someone's data", "nothing that holds something is removed");
    remove(root.as_path());
}

/// A folder the snapshot does not carry and a folder outside the source are nothing to do.
#[test]
fn a_folder_the_snapshot_does_not_carry_and_a_folder_outside_the_source_are_left_alone() {
    let root: PathBuf = fixture("snapshot-placeholders-absent");
    let (held, _folder) = folder_of_ordinary_entries(root.as_path());
    assert!(open_below_stand_ins(&held, &[OsString::from("missing"), OsString::from("deep")], "missing/deep").expect("nothing is opened").is_none(), "a folder the snapshot does not carry is nothing to do");
    let source: SnapshotSource = source_of(root.as_path(), false);
    let plan = plan_to(root.join("folder/topic"));
    let outside: SnapshotSource = source_of(root.join("elsewhere").as_path(), false);
    let mut set_aside: HandOver = HandOver::starting(None);
    assert_eq!(remove_snapshot_placeholders(&held, &source, &plan, &mut set_aside), Ok(()));
    let mut elsewhere: HandOver = HandOver::starting(None);
    assert_eq!(remove_snapshot_placeholders(&held, &outside, &plan, &mut elsewhere), Ok(()), "a folder outside the source names no folder of the snapshot");
    assert!(root.join("folder/empty").exists());
    remove(root.as_path());
}

/// A folder that is reached through a link is an error,
/// so nothing is listed or removed in the tree the link points at.
#[test]
fn a_folder_reached_through_a_link_is_refused() {
    let root: PathBuf = fixture("snapshot-placeholders-link");
    let (held, _folder) = folder_of_ordinary_entries(root.as_path());
    let error: String = open_below_stand_ins(&held, &[OsString::from("folder"), OsString::from("link"), OsString::from("inner")], "folder/link/inner").expect_err("refused");
    assert!(error.contains("link"), "{error}");
    let source: SnapshotSource = source_of(root.as_path(), false);
    let plan = plan_to(root.join("folder/link/inner/topic"));
    let mut set_aside: HandOver = HandOver::starting(None);
    let refused: String = remove_snapshot_placeholders(&held, &source, &plan, &mut set_aside).expect_err("refused");
    assert!(refused.contains("link"), "{refused}");
    assert!(root.join("elsewhere/inner").exists(), "the tree the link points at is untouched");
    remove(root.as_path());
}

/// A worktree folder that holds nothing but the stand-ins of nested subvolumes leaves the snapshot whole:
/// one entry of the trash holds the folder the snapshot carried,
/// a fresh empty folder takes its place,
/// and no
///  stand-in is removed inside the add.
/// Why:
///   Every removal of a stand-in inside the add is one btrfs metadata change,
/// and the adds wait on each other for
///  them;
/// the fold a snapshot shows of a repository with many worktrees is one such stand-in per worktree.
#[test]
fn btrfs_a_worktree_folder_that_holds_only_stand_ins_goes_into_the_trash_whole() {
    let Some(root) = btrfs_fixture("placeholders-whole-folder") else { return };
    let repo: PathBuf = main_with_nested_subvolumes(root.as_path(), 5);
    let (stage, trash) = snapshot_into(root.as_path(), repo.as_path(), ".claude/worktrees/topic", true);
    let manifest: TrashManifest = trash.expect("the stand-ins are handed to the trash the leave-out made");
    let trash_dir: PathBuf = stage.join(TRASH_PARENT).join(manifest.name.as_str());
    // The snapshot's own `.git` is the last entry the add sets aside, so the folder is the one before it.
    let last: usize = manifest.entries.len() - 2;
    let held: Vec<String> = names_in(trash_dir.join(entry_name(last)).as_path());
    assert_eq!(held, vec!["w0", "w1", "w2", "w3", "w4"], "every stand-in went with the folder");
    let trash_root: File = open_root(trash_dir.as_path()).expect("the trash");
    assert_eq!(identity_at(&trash_root, OsStr::new(entry_name(last).as_str())).expect("the entry"), manifest.entries[last], "the entry the manifest names is the folder that went in");
    let fresh: PathBuf = stage.join(".claude/worktrees");
    assert!(fresh.is_dir(), "a fresh worktree folder takes its place");
    assert_eq!(names_in(fresh.as_path()), Vec::<String>::new(), "and it is empty");
    for index in 0..5 {
        assert!(std::fs::symlink_metadata(fresh.join(format!("w{index}"))).is_err(), "w{index} is no stand-in of the new worktree");
        assert_eq!(inode_of(trash_dir.join(entry_name(last)).join(format!("w{index}")).as_path()), BTRFS_STAND_IN_INODE, "w{index} in the trash is the stand-in btrfs showed");
    }
    remove(root.as_path());
}

/// A worktree folder that holds one entry of someone else's stays in the snapshot and is never moved whole:
/// an empty
///  directory,
/// a directory holding data,
/// a file and a link all keep their place and their bytes,
/// and only the stand-ins go.
/// Why:
///   The inode number and the directory type are the whole test for a stand-in,
/// and every other entry may be
///  someone's data.
#[test]
fn btrfs_a_worktree_folder_that_holds_someone_elses_entries_stays_and_only_the_stand_ins_go() {
    let Some(root) = btrfs_fixture("placeholders-mixed-folder") else { return };
    let repo: PathBuf = main_with_nested_subvolumes(root.as_path(), 2);
    let folder: PathBuf = repo.join(".claude/worktrees");
    std::fs::create_dir_all(folder.join("empty")).expect("an empty directory");
    std::fs::create_dir_all(folder.join("full")).expect("a directory holding data");
    std::fs::write(folder.join("full/data"), b"someone's data").expect("a file in it");
    std::fs::write(folder.join("plain"), b"a file").expect("a file");
    std::fs::create_dir_all(root.join("someone")).expect("a directory outside the folder");
    std::os::unix::fs::symlink(root.join("someone"), folder.join("link")).expect("a link");
    let (stage, _trash) = snapshot_into(root.as_path(), repo.as_path(), ".claude/worktrees/topic", true);
    let kept: PathBuf = stage.join(".claude/worktrees");
    assert_eq!(names_in(kept.as_path()), vec!["empty", "full", "link", "plain"], "every entry of someone else's stays and the stand-ins go");
    assert!(kept.join("empty").is_dir(), "an ordinary empty directory stays");
    assert_eq!(std::fs::read(kept.join("full/data")).expect("the data"), b"someone's data", "a directory that holds something keeps it");
    assert_eq!(std::fs::read(kept.join("plain")).expect("the file"), b"a file", "a file keeps its bytes");
    assert!(std::fs::symlink_metadata(kept.join("link")).expect("the link").is_symlink(), "a link stays a link");
    remove(root.as_path());
}

/// A source whose Git does not ignore the place a trash is made removes the stand-ins where they are:
/// nothing is made
///  that a later command could stage,
/// and nothing is handed over.
#[test]
fn btrfs_a_source_with_no_ignored_trash_place_removes_the_stand_ins_in_place() {
    let Some(root) = btrfs_fixture("placeholders-no-trash-place") else { return };
    let repo: PathBuf = main_with_nested_subvolumes(root.as_path(), 2);
    let (stage, trash) = snapshot_into(root.as_path(), repo.as_path(), ".claude/worktrees/topic", false);
    assert_eq!(trash, None, "there is no trash to hand over");
    assert!(stage.join(".claude/worktrees").is_dir(), "the folder stays");
    assert_eq!(names_in(stage.join(".claude/worktrees").as_path()), Vec::<String>::new(), "and its stand-ins are removed where they were");
    assert!(std::fs::symlink_metadata(stage.join(TRASH_PARENT)).is_err(), "no trash is made where Git does not ignore it");
    remove(root.as_path());
}

/// The stand-ins join the trash the leave-out already made:
/// one manifest names the directories it set aside and the
///  folder the stand-ins went in,
/// in the one trash the service is told about.
#[test]
fn btrfs_the_stand_ins_join_the_trash_the_leave_out_made() {
    let Some(root) = btrfs_fixture("placeholders-join-trash") else { return };
    let repo: PathBuf = main_with_nested_subvolumes(root.as_path(), 3);
    with_run_state(repo.as_path());
    let (stage, trash) = snapshot_into(root.as_path(), repo.as_path(), ".claude/worktrees/topic", true);
    let manifest: TrashManifest = trash.expect("a trash to hand over");
    assert_eq!(manifest.entries.len(), 4, "the incremental directory, Git's directory for the new worktree, the worktree folder and the snapshot's own .git");
    let trash_dir: PathBuf = stage.join(TRASH_PARENT).join(manifest.name.as_str());
    assert_eq!(names_in(stage.join(TRASH_PARENT).as_path()), vec![manifest.name.clone()], "the one trash the add hands over");
    assert_eq!(names_in(trash_dir.join(entry_name(2)).as_path()), vec!["w0", "w1", "w2"], "the folder is the last entry");
    remove(root.as_path());
}

/// The snapshot of a repository with nested subvolumes,
/// taken up to but not including the stand-in sweep of the snapshot rung.
/// Why:
///   A control that plants a directory between the check of a worktree folder and its rename needs the rung's
///  steps below the sweep,
/// and only the seam of the sweep writes there.
struct BeforeTheSweep {
    stage: PathBuf,
    source: SnapshotSource,
    plan: AddPlan,
    root: File,
    hand_over: HandOver,
}

/// The snapshot of `repo` at `destination`,
/// taken as the rung takes it and left before the stand-in sweep.
fn snapshot_before_the_sweep(root: &Path, repo: &Path, destination: &str) -> BeforeTheSweep {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", "topic", destination]);
    let made: PathBuf = std::fs::canonicalize(repo.join(destination)).expect("destination");
    let plan = plan_to(made);
    let mut source: SnapshotSource = choose_snapshot_source(&git_context(), &main_identity(repo), &plan).expect("a snapshot source");
    source.trash_ignored = true;
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    snapshot_create_at(source.root.as_path(), stage.as_path()).expect("the snapshot");
    std::fs::set_permissions(&stage, std::fs::Permissions::from_mode(0o700)).expect("a private stage");
    let held: File = open_root(stage.as_path()).expect("the stage");
    let pruned = prune_snapshot(stage.as_path(), &source).expect("the leave-out");
    let hand_over: HandOver = HandOver::starting(pruned.trash);
    return BeforeTheSweep { stage, source, plan, root: held, hand_over };
}

/// A directory a person made in place of a stand-in after the check stays where it is:
/// the folder is looked at again in the trash,
/// and a folder that no longer holds only empty stand-ins goes back
///  to its place,
/// where its stand-ins are removed one by one.
/// Why:
///   The check of a folder and its rename are two steps,
/// and a directory a person made in that window is never
///  removed with the stand-ins it replaced,
/// whatever the folder held when it was checked (`never discard user data`).
#[test]
fn btrfs_a_directory_made_between_the_check_and_the_take_is_not_deleted() {
    let Some(root) = btrfs_fixture("placeholders-made-after-the-check") else { return };
    let repo: PathBuf = main_with_nested_subvolumes(root.as_path(), 2);
    let mut prepared: BeforeTheSweep = snapshot_before_the_sweep(root.as_path(), repo.as_path(), ".claude/worktrees/topic");
    let planted: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let folder: PathBuf = prepared.stage.join(".claude/worktrees");
    let mut seams: FoldSeams<'_> = FoldSeams {
        before_take: &mut || {
            if planted.replace(true) {
                return;
            }
            std::fs::remove_dir(folder.join("w0")).expect("the stand-in goes");
            std::fs::create_dir(folder.join("w0")).expect("a person's directory in its place");
            std::fs::write(folder.join("w0/keep"), b"someone's data").expect("a person's file");
        },
        before_put_back: &mut do_nothing,
    };
    remove_snapshot_placeholders_with(&prepared.root, &prepared.source, &prepared.plan, &mut prepared.hand_over, &mut seams).expect("the sweep");
    assert_eq!(std::fs::read(folder.join("w0/keep")).expect("the person's file is still where they put it"), b"someone's data", "the directory a person made after the check is never taken with the folder");
    assert_eq!(names_in(folder.as_path()), vec!["w0"], "and the stand-in beside it goes as before");
    remove(root.as_path());
}

/// A folder that cannot go back to the name it came from is kept beside the trash,
/// the trash's own entries keep their names,
/// and the stage's cleanup never takes it.
/// Why:
///   The name a taken folder goes back to can be taken by a directory somebody made meanwhile,
/// and a put back
///  that fails leaves the folder at a trash entry no manifest names;
/// the next directory this add sets aside takes the
///  same entry name,
/// so the snapshot rung fails and the stage, with that entry, is destroyed
///  (`never discard user data`).
#[test]
fn btrfs_a_folder_whose_put_back_is_blocked_is_kept_where_the_stage_cleanup_cannot_take_it() {
    let Some(root) = btrfs_fixture("placeholders-put-back-blocked") else { return };
    let repo: PathBuf = main_with_nested_subvolumes(root.as_path(), 2);
    let mut prepared: BeforeTheSweep = snapshot_before_the_sweep(root.as_path(), repo.as_path(), ".claude/worktrees/topic");
    let folder: PathBuf = prepared.stage.join(".claude/worktrees");
    let trash_parent: PathBuf = prepared.stage.join(TRASH_PARENT);
    let planted: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let occupied: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let mut seams: FoldSeams<'_> = FoldSeams {
        before_take: &mut || {
            if planted.replace(true) {
                return;
            }
            std::fs::remove_dir(folder.join("w0")).expect("the stand-in goes");
            std::fs::create_dir(folder.join("w0")).expect("a person's directory in its place");
            std::fs::write(folder.join("w0/keep"), b"someone's data").expect("a person's file");
        },
        before_put_back: &mut || {
            if occupied.replace(true) {
                return;
            }
            std::fs::create_dir(folder.as_path()).expect("a directory where the folder was");
        },
    };
    remove_snapshot_placeholders_with(&prepared.root, &prepared.source, &prepared.plan, &mut prepared.hand_over, &mut seams).expect("the sweep");
    let kept: Vec<String> = names_in(trash_parent.as_path()).into_iter().filter(|name: &String| { return name.starts_with(KEPT_PREFIX); }).collect();
    assert_eq!(kept.len(), 1, "the folder is kept in one place beside the trash, and nowhere else: {kept:?}");
    let kept_folder: PathBuf = trash_parent.join(kept[0].as_str());
    assert_eq!(std::fs::read(kept_folder.join("w0/keep")).expect("the person's file is kept"), b"someone's data", "the bytes somebody put in the folder survive the put back that could not happen");
    assert_eq!(names_in(folder.as_path()), Vec::<String>::new(), "the directory that took the name is left exactly as it was made");
    let moved = prepared.hand_over.take_entry(&prepared.root, &prepared.source, &prepared.root, OsStr::new(".git"), Path::new(".git"));
    assert!(moved.is_ok(), "the next directory the add sets aside finds its trash entry name free: {moved:?}");
    let made = identity_of(&prepared.root).expect("the stage's identity");
    discard_stage(prepared.stage.as_path(), Some(made));
    assert!(prepared.stage.is_dir(), "the cleanup leaves the stage that holds what the add kept");
    assert_eq!(std::fs::read(kept_folder.join("w0/keep")).expect("the kept folder outlives the cleanup"), b"someone's data", "the cleanup that would remove the stage takes nothing of the folder that was kept");
    remove(root.as_path());
}
