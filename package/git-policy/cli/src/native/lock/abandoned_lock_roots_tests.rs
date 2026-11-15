//! What:
//!  Controls of the directories the pass searches and of the record folder it reads,
//!  for the settings that move a file Git locks out of the Git directory:
//!  `GIT_COMMON_DIR` and `GIT_INDEX_FILE`.
//! Why:
//!  The pass, the writer of a record and the walk that looks for locks must take one answer about where this
//!  command's locks and records are.
//!  With `GIT_COMMON_DIR` naming another directory the record was written in the common one and read out of the Git
//!  one, and the lock of a dead Git was never cleared
//!  (a review of `4e9b9fff1d`,
//! finding 1).
//!  With `GIT_INDEX_FILE` naming a file outside the Git directory the index lock was never walked and the dead
//!  writer's record was then removed as unused, so a later pass had nothing to prove the lock gone with
//!  (the same review,
//! finding 2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repository = { gitDir: A, commonDir: B, indexFile: join(alt, 'index') };
//! plantLock(join(A, 'index.lock'), deadPid); plantRecord(folderOf(B), deadPid);
//! meetTheLocks(repository); expect(existsSync(join(A, 'index.lock'))).toBe(false);
//! ```

use crate::abandoned_locks::meet_the_locks_of_the_repository_the_command_works_on;
use crate::git_directory_roots::CommandRepository;
use crate::lock_holder_records::{CHILD_KIND, SELF_KIND, folder_of, record_text};
use crate::record_proof_test_support::{a_dead_process, make_pid_file_after_the_lock};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  A repository whose Git directory and common directory are two directories,
///  with the record a killed Git leaves in the common one and the index file, shallow file and object directory of
///  the command.
///  `(PathBuf, CommandRepository, i64, PathBuf)` is the fixture root, the answer, the process that is gone, and the
///  record file.
/// Why:
///   Every row of the two controls differs only in which of Git's settings moves a file it locks,
///  so one fixture makes them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [root, repository, deadPid, record] = splitRepository('control-name');
/// ```
fn split_repository(control: &str) -> (PathBuf, CommandRepository, i64, PathBuf) {
    let root: PathBuf = fixture(control);
    let git_dir: PathBuf = root.join("worktree-git-dir");
    let common: PathBuf = root.join("common-git-dir");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    std::fs::create_dir_all(common.as_path()).expect("the common Git directory");
    let (gone, identity): (i64, String) = a_dead_process();
    let folder: PathBuf = folder_of(common.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    let record: PathBuf = folder.join(format!("{gone}.record"));
    std::fs::write(record.as_path(), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes())
        .expect("the record a killed command leaves");
    let repository: CommandRepository = CommandRepository {
        start: root.clone(),
        git_dir: Some(git_dir.clone()),
        common_dir: Some(common.clone()),
        index_file: Some(git_dir.join("index")),
        shallow_file: Some(git_dir.join("shallow")),
        object_dir: Some(common.join("objects")),
    };
    return (root, repository, gone, record);
}

/// What:
///  Plant the lock a killed Git leaves at `lock`,
///  with its PID file naming `gone`.
/// Why:
///   The proof begins with a lock and a PID file born after it,
///  and the record this repository holds explains it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(lock, 'a half-written file\n'); makePidFileAfterTheLock(lock, deadPid);
/// ```
fn plant_the_lock(lock: &Path, gone: i64) {
    if let Some(parent) = lock.parent() {
        std::fs::create_dir_all(parent).expect("the directory the lock lies in");
    }
    std::fs::write(lock, b"a half-written file a killed Git never renamed\n").expect("the lock a killed Git left");
    make_pid_file_after_the_lock(lock, gone);
}

/// What:
///  With `GIT_COMMON_DIR` naming a directory beside the Git directory,
///  the pass reads the record out of the common directory,
///  finds the lock standing in the Git directory,
///  and clears it.
/// Why:
///  The record folder belongs to the common directory,
///  and a pass that guessed it from a root read the wrong one,
///  so a provably abandoned lock survived and a commit failed on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// meetTheLocks(repository); expect(existsSync(join(gitDir, 'index.lock'))).toBe(false); expect(existsSync(record)).toBe(false);
/// ```
#[test]
fn the_record_of_a_common_directory_beside_the_git_directory_explains_a_lock_in_the_git_directory() {
    let (root, repository, gone, record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-common-dir");
    let git_dir: PathBuf = repository.git_dir.clone().expect("the Git directory");
    let lock: PathBuf = git_dir.join("index.lock");
    plant_the_lock(lock.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(!lock.exists(), "the lock the dead Git left in the Git directory is cleared on the record the common directory holds");
    assert!(!record.exists(), "and the record goes with the lock it explained");
    assert!(!folder_of(repository.common_dir.as_ref().expect("the common directory").as_path()).exists(), "and the folder with its last record");
    remove(root.as_path());
}

/// What:
///  With `GIT_INDEX_FILE` naming a file outside the Git directory,
///  the pass walks the directory that file lies in,
///  finds the lock standing beside it,
///  and clears it on the record,
///  keeping that record only until then.
/// Why:
///  A walk that searched only the Git and common directories never saw this lock,
///  and then removed the record that explained it as unused,
///  so the lock could never be cleared and a commit failed on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alt = join(root, 'alternate-index'); repository.indexFile = join(alt, 'index');
/// meetTheLocks(repository); expect(existsSync(join(alt, 'index.lock'))).toBe(false);
/// ```
#[test]
fn a_lock_beside_an_index_outside_the_git_directory_is_found_and_cleared() {
    let (root, mut repository, gone, record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-alternate-index");
    let alternate: PathBuf = root.join("alternate-index");
    let index_file: PathBuf = alternate.join("index");
    repository.index_file = Some(index_file.clone());
    let lock: PathBuf = alternate.join("index.lock");
    plant_the_lock(lock.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(!lock.exists(), "the lock beside the index Git was told to use is cleared on the record the common directory holds");
    assert!(!record.exists(), "and the record goes with the lock it explained, rather than being called unused");
    remove(root.as_path());
}

/// What:
///  A file of a person's named `important.lock` beside an alternate index is neither opened nor unlinked,
///  while the lock of that index itself is:
///  the pass touches the exact names Git locks and nothing else with the suffix.
/// Why:
///  The walk once took every `*.lock` in the directory of a selected index,
///  and deleted a file that held a person's data, because the name looked like a lock
///  (a review of `429503238b`,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(alternate, 'important.lock'), 'user data that is not an index');
/// plantLock(join(alternate, 'index.lock'), deadPid); meetTheLocks(repository);
/// expect(readFileSync(join(alternate, 'important.lock'))).toEqual('user data that is not an index'); expect(existsSync(join(alternate, 'index.lock'))).toBe(false);
/// ```
#[test]
fn a_persons_file_named_like_a_lock_beside_an_alternate_index_is_left_untouched() {
    let (root, mut repository, gone, _record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-a-person-file");
    let alternate: PathBuf = root.join("alternate-index");
    std::fs::create_dir_all(alternate.as_path()).expect("the directory of the alternate index");
    repository.index_file = Some(alternate.join("index"));
    let persons_file: PathBuf = alternate.join("important.lock");
    let persons_bytes: &[u8] = b"user data that is not an index\n";
    std::fs::write(persons_file.as_path(), persons_bytes).expect("the person's file");
    // A PID companion naming a dead recorded process makes it look like an abandoned lock to a walk that took any name
    //  with the suffix.
    make_pid_file_after_the_lock(persons_file.as_path(), gone);
    // The lock the selected index itself can leave is the one the pass may take.
    let index_lock: PathBuf = alternate.join("index.lock");
    plant_the_lock(index_lock.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert_eq!(std::fs::read(persons_file.as_path()).expect("the person's file is still there"), persons_bytes, "the person's file is read and unchanged");
    assert!(!index_lock.exists(), "the lock of the index Git was told to use is cleared on the record");
    remove(root.as_path());
}

/// What:
///  A symbolic link among the entries of `<common>/worktrees` is not a worktree,
///  and the pass judges nothing in the folder it points at.
/// Why:
///  `Path::is_dir` follows a link, so a link planted at that name made the pass build lock candidates in its target and
///  delete a person's `HEAD.lock` outside every repository
///  (a review of `3ac1254a73`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(outside, join(common, 'worktrees', 'not-a-worktree')); plantLock(join(outside, 'HEAD.lock'), deadPid);
/// meetTheLocks(repository); expect(readFileSync(join(outside, 'HEAD.lock'))).toEqual(bytes);
/// ```
#[test]
fn a_link_among_the_worktree_entries_is_not_a_worktree() {
    let (root, repository, gone, _record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-a-linked-worktree-entry");
    let common: PathBuf = repository.common_dir.clone().expect("the common directory");
    let outside: PathBuf = root.join("outside-every-repository");
    std::fs::create_dir_all(outside.as_path()).expect("the folder the link points at");
    std::fs::create_dir_all(common.join("worktrees").as_path()).expect("the worktrees folder");
    std::os::unix::fs::symlink(outside.as_path(), common.join("worktrees/not-a-worktree").as_path()).expect("the link");
    let persons_file: PathBuf = outside.join("HEAD.lock");
    let persons_bytes: &[u8] = b"a person's data outside every repository\n";
    std::fs::write(persons_file.as_path(), persons_bytes).expect("the person's file");
    make_pid_file_after_the_lock(persons_file.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert_eq!(std::fs::read(persons_file.as_path()).expect("the person's file is still there"), persons_bytes, "nothing outside the repository is judged or removed");
    remove(root.as_path());
}

/// What:
///  A pass in the main worktree clears the abandoned index lock of a linked worktree,
///  because that worktree's own locks are candidates wherever the pass runs.
/// Why:
///  The walk once held only the current worktree's index and shallow file,
///  so a pass in the main worktree never saw a linked worktree's `index.lock`,
///  removed the record that explained it as unneeded,
///  and left the lock for the worktree's own pass to find with no proof
///  (a review of `3ac1254a73`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdir(join(common, 'worktrees', 'defect')); plantLock(join(common, 'worktrees', 'defect', 'index.lock'), deadPid);
/// meetTheLocks(mainRepository); expect(existsSync(join(common, 'worktrees', 'defect', 'index.lock'))).toBe(false);
/// ```
#[test]
fn a_pass_in_the_main_worktree_clears_a_linked_worktrees_index_lock() {
    let (root, repository, gone, record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-a-linked-worktree-index");
    let common: PathBuf = repository.common_dir.clone().expect("the common directory");
    let linked: PathBuf = common.join("worktrees/defect");
    std::fs::create_dir_all(linked.as_path()).expect("the linked worktree's Git directory");
    let lock: PathBuf = locked_index_of(linked.as_path());
    plant_the_lock(lock.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(!lock.exists(), "the main worktree's pass clears the lock the linked worktree's dead Git left");
    assert!(!record.exists(), "and the record goes with the lock it explained, because the pass that removed it held that lock");
    remove(root.as_path());
}

/// What:
///  The name of the lock Git writes for the index in a Git directory.
/// Why:
///   The control names the lock the same way Git does,
///  so a change to the naming shows here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lockedIndexOf = (gitDir) => join(gitDir, 'index.lock');
/// ```
fn locked_index_of(git_dir: &Path) -> PathBuf {
    return git_dir.join("index").with_extension("lock");
}

/// What:
///  A `refs` that is a link is no root of this repository,
///  and the pass judges nothing in the folder it points at.
/// Why:
///  A walk names `refs` as a root and the walk's `read_dir` follows a link at the root itself,
///  so a link planted there let the pass read a folder outside every repository and unlink a person's
///  `important.lock`
///  (a review of `58cd4d9f7c`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(outside, join(common, 'refs')); plantLock(join(outside, 'important.lock'), deadPid);
/// meetTheLocks(repository); expect(readFileSync(join(outside, 'important.lock'))).toEqual(bytes);
/// ```
#[test]
fn a_linked_refs_root_is_not_walked() {
    let (root, repository, gone, _record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-a-linked-refs");
    let common: PathBuf = repository.common_dir.clone().expect("the common directory");
    let outside: PathBuf = root.join("outside-every-repository");
    std::fs::create_dir_all(outside.as_path()).expect("the folder the link points at");
    std::os::unix::fs::symlink(outside.as_path(), common.join("refs").as_path()).expect("the link at refs");
    let persons_file: PathBuf = outside.join("important.lock");
    let persons_bytes: &[u8] = b"a person's data outside every repository\n";
    std::fs::write(persons_file.as_path(), persons_bytes).expect("the person's file");
    make_pid_file_after_the_lock(persons_file.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert_eq!(std::fs::read(persons_file.as_path()).expect("the person's file is still there"), persons_bytes, "a root that is a link is not searched");
    remove(root.as_path());
}
