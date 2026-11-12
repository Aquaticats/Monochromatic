//! What:
//!  Controls of what a removal does with the folder Git could not empty:
//!  the folder it finishes itself,
//!  the entry that appeared while the removal ran,
//!  and the record that the next command finishes.
//! Why:
//!  A folder nothing registers is the one outcome a removal must never leave,
//!  and never discarding a person's data is the rule that decides what stays:
//!  these controls build both states a filesystem produces,
//!  and the record a killed removal leaves.

use super::{LeftBehind, finish_after_git, finish_left_behind_folders, remember_before_git};
use crate::test_support::{fixture, remove};
use crate::worktree_removal_left_behind_record::{RECORD_FOLDER, record_before_git_with_owner};
use crate::worktree_removal_left_behind_record::{forget, read_record};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// One repository with a linked worktree registered in it,
/// so the record's name is a real registration's.
struct Scene {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The repository's common Git directory.
    common: PathBuf,
    /// The worktree's directory,
    /// which Git's own walk would have left.
    worktree: PathBuf,
    /// The registration's name below `worktrees`.
    id: String,
}

impl Scene {
    /// The path of the record this scene's removal writes.
    fn record(&self) -> PathBuf {
        return self.common.join(RECORD_FOLDER).join(self.id.as_str());
    }

    /// Take the registration away,
    /// as Git's own removal does whether or not its walk emptied the folder.
    fn unregister(&self) {
        std::fs::remove_dir_all(self.common.join("worktrees").join(self.id.as_str())).expect("the registration goes");
    }
}

/// What a scene holds:
/// a worktree with a tracked file and an ignored tree,
/// and the registration that names it.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("repo/.git");
    let worktree: PathBuf = root.join("worktree");
    let admin: PathBuf = common.join("worktrees/topic");
    for directory in [&admin, &worktree.join("node_modules/pkg")] {
        std::fs::create_dir_all(directory).expect("directory");
    }
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", admin.display())).expect("the pointer");
    std::fs::write(admin.join("gitdir"), format!("{}\n", worktree.join(".git").display())).expect("the back-link");
    std::fs::write(admin.join("commondir"), b"../..\n").expect("the common directory");
    std::fs::write(worktree.join("README.md"), b"readme\n").expect("tracked file");
    std::fs::write(worktree.join("node_modules/pkg/index.js"), b"x\n").expect("ignored file");
    return Scene { root, common, worktree, id: String::from("topic") };
}

/// What:
///  A removal that took the registration and left an empty folder finishes it,
///  and says so once.
/// Why:
///   This is the common case of a walk something defeated:
///  everything the removal asked for is gone,
///  so the folder is the removal's own and nothing of the person's is lost with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remembered = rememberBeforeGit(common, worktree); scene.unregister(); expect(finishAfterGit(remembered)).toEqual({ kind: 'removed' });
/// ```
#[test]
fn a_folder_the_removal_emptied_is_removed() {
    let scene: Scene = scene("left-behind-removed");
    let remembered = remember_before_git(scene.common.as_path(), scene.worktree.as_path()).expect("the folder is recorded");
    scene.unregister();
    std::fs::remove_file(scene.worktree.join("README.md")).expect("the walk took the tracked file");
    std::fs::remove_dir_all(scene.worktree.join("node_modules")).expect("the walk took the ignored tree");
    assert_eq!(finish_after_git(&remembered), LeftBehind::Removed);
    assert!(!scene.worktree.exists(), "the folder is gone");
    assert!(!scene.record().exists(), "the record is gone with it");
    remove(scene.root.as_path());
}

/// What:
///  A removal that took the registration and left the folder holding everything finishes it too:
///  what the record names is the removal's own.
/// Why:
///   Git's walk stops at the first entry it cannot delete,
///  so a folder it gave up on holds whole directories the walk never reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scene.unregister(); expect(finishAfterGit(remembered)).toEqual({ kind: 'removed' });
/// ```
#[test]
fn a_folder_the_removal_never_reached_is_removed() {
    let scene: Scene = scene("left-behind-whole");
    let remembered = remember_before_git(scene.common.as_path(), scene.worktree.as_path()).expect("the folder is recorded");
    scene.unregister();
    assert_eq!(finish_after_git(&remembered), LeftBehind::Removed);
    assert!(!scene.worktree.exists(), "the folder is gone with everything the removal asked for");
    remove(scene.root.as_path());
}

/// What:
///  An entry that appeared while the removal was running stays,
///  the folder stays with it,
///  and it is named;
///  once it is gone the next command finishes the removal.
/// Why:
///   That entry is somebody's,
///  and an unregistered folder is only acceptable while a record points at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(join(worktree, 'notes.txt')); expect(finishAfterGit(remembered)).toMatchObject({ kind: 'kept', names: ['notes.txt'] });
/// expect(worktree).toExist(); remove(join(worktree, 'notes.txt')); finishLeftBehindFolders(common); expect(worktree).not.toExist();
/// ```
#[test]
fn an_entry_that_appeared_during_the_removal_stays_and_the_next_command_finishes_the_folder() {
    let scene: Scene = scene("left-behind-stranger");
    let remembered = remember_before_git(scene.common.as_path(), scene.worktree.as_path()).expect("the folder is recorded");
    scene.unregister();
    std::fs::remove_file(scene.worktree.join("README.md")).expect("the walk took the tracked file");
    std::fs::remove_dir_all(scene.worktree.join("node_modules")).expect("the walk took the ignored tree");
    std::fs::write(scene.worktree.join("notes.txt"), b"mine\n").expect("something wrote during the removal");
    let outcome: LeftBehind = finish_after_git(&remembered);
    let LeftBehind::Kept { names, reason } = &outcome else { panic!("a folder holding a stranger is kept: {outcome:?}") };
    assert_eq!(names, &vec![String::from("notes.txt")]);
    assert!(reason.contains("appeared"), "{reason}");
    assert_eq!(std::fs::read(scene.worktree.join("notes.txt")).expect("the stranger stays"), b"mine\n");
    assert!(scene.record().is_file(), "the record stays so a command finishes the removal");
    // The next command finds the folder and finishes it once nothing of somebody else's is in it.
    std::fs::remove_file(scene.worktree.join("notes.txt")).expect("the person takes their file");
    let problems: Vec<String> = finish_left_behind_folders(scene.common.as_path());
    assert_eq!(problems.len(), 1, "the finished removal is said once: {problems:?}");
    assert!(problems[0].starts_with("monochromatic-git-policy-cli: Git could not empty"), "{problems:?}");
    assert!(!scene.worktree.exists(), "the folder is gone");
    assert!(!scene.record().exists(), "the record is gone");
    remove(scene.root.as_path());
}

/// What:
///  A removal whose process died before Git returned,
///  and whose owner is gone,
///  is finished by the next command.
/// Why:
///   A kill is the state a person cannot clean up after,
///  and no registration names the folder once Git ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordBeforeGit(common, id, worktree, '1:ended'); unregister(); expect(finishLeftBehindFolders(common)).toHaveLength(1);
/// ```
#[test]
fn a_removal_whose_process_ended_is_finished_by_the_next_command() {
    let scene: Scene = scene("left-behind-killed");
    record_before_git_with_owner(scene.common.as_path(), OsStr::new(scene.id.as_str()), scene.worktree.as_path(), Some("1:ended")).expect("the record of a removal that ended");
    scene.unregister();
    let problems: Vec<String> = finish_left_behind_folders(scene.common.as_path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(!scene.worktree.exists(), "the folder is gone");
    assert!(!scene.record().exists());
    remove(scene.root.as_path());
}

/// What:
///  A removal that is still running is left alone by every other command.
/// Why:
///   A second command that finished the removal would delete what the running one is still proving,
///  and the running one's own record has nothing left to mark.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberBeforeGit(common, worktree); expect(finishLeftBehindFolders(common)).toEqual([]); expect(worktree).toExist();
/// ```
#[test]
fn a_removal_that_is_still_running_is_left_alone() {
    let scene: Scene = scene("left-behind-running");
    let remembered = remember_before_git(scene.common.as_path(), scene.worktree.as_path()).expect("the folder is recorded");
    scene.unregister();
    let problems: Vec<String> = finish_left_behind_folders(scene.common.as_path());
    assert!(problems.is_empty(), "a removal whose process is alive is not finished by another: {problems:?}");
    assert!(scene.worktree.is_dir(), "the folder stands");
    drop(remembered);
    remove(scene.root.as_path());
}

/// What:
///  A removal Git refused leaves nothing to finish:
///  the registration is still there and the folder is where it belongs.
/// Why:
///   A dirty worktree is Git's own refusal,
///  and a finish that removed it would delete a worktree the person was told stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(finishAfterGit(remembered)).toEqual({ kind: 'nothing' }); expect(worktree).toExist(); expect(record).not.toExist();
/// ```
#[test]
fn a_removal_git_refused_leaves_the_folder_where_it_belongs() {
    let scene: Scene = scene("left-behind-refused");
    let remembered = remember_before_git(scene.common.as_path(), scene.worktree.as_path()).expect("the folder is recorded");
    assert_eq!(finish_after_git(&remembered), LeftBehind::Nothing);
    assert!(scene.worktree.join("README.md").is_file(), "the worktree stands");
    assert!(!scene.record().exists(), "nothing is left to finish");
    remove(scene.root.as_path());
}

/// What:
///  A folder that appeared with an entry the record never saw is not removed,
///  even when nothing registers it.
/// Why:
///   The record names what the removal saw,
///  and a folder whose every entry is new is another directory at that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(finishLeftBehindFolders(common)).toHaveLength(1); expect(read(join(worktree, 'mine'))).toBe('mine');
/// ```
#[test]
fn a_folder_of_entries_the_record_never_saw_is_kept_and_named() {
    let scene: Scene = scene("left-behind-replaced");
    // The record of a removal whose process ended, which is the state a next command acts on.
    record_before_git_with_owner(scene.common.as_path(), OsStr::new(scene.id.as_str()), scene.worktree.as_path(), Some("1:ended")).expect("the folder is recorded");
    scene.unregister();
    std::fs::remove_dir_all(scene.worktree.as_path()).expect("the folder the record names goes");
    std::fs::create_dir_all(scene.worktree.as_path()).expect("another directory takes the name");
    std::fs::write(scene.worktree.join("mine"), b"mine\n").expect("its own file");
    let problems: Vec<String> = finish_left_behind_folders(scene.common.as_path());
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("`mine`"), "the file is named: {problems:?}");
    assert_eq!(std::fs::read(scene.worktree.join("mine")).expect("the file stays"), b"mine\n");
    let bytes: Vec<u8> = std::fs::read(scene.record()).expect("the record stays");
    assert!(read_record(bytes.as_slice()).is_ok(), "the record still names the folder");
    forget(scene.common.as_path(), OsStr::new(scene.id.as_str()));
    remove(scene.root.as_path());
}

/// What:
///  A directory the repository does not register is not recorded at all.
/// Why:
///   Only Git's own removal of a registered worktree can leave a folder nothing points at,
///  and a record for anything else would let a command remove a directory no removal asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(rememberBeforeGit(common, join(root, 'nothing'))).toBeUndefined();
/// ```
#[test]
fn a_directory_the_repository_does_not_register_is_not_recorded() {
    let scene: Scene = scene("left-behind-unregistered");
    let other: PathBuf = scene.root.join("nothing");
    std::fs::create_dir_all(other.as_path()).expect("directory");
    assert!(remember_before_git(scene.common.as_path(), other.as_path()).is_none());
    assert!(!scene.common.join(RECORD_FOLDER).exists(), "no record was written");
    assert!(Path::new(scene.worktree.as_path()).is_dir(), "the registered worktree is untouched");
    remove(scene.root.as_path());
}
