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
use crate::git_directory_roots::{CommandRepository, ProvenFolder};
use crate::lock_holder_records::{CHILD_KIND, SELF_KIND, folder_of, record_text};
use crate::record_proof_test_support::{a_dead_process, make_pid_file_after_the_lock};
use crate::test_support::{fixture, make_it_a_git_directory, remove};
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
    // What: Both folders are Git directories by Git's own test, so the proof the pass takes is a proof of a repository.
    // Why:  The pass now answers nothing for a folder Git would refuse, and a fixture without `HEAD` would make every
    //       control here pass because no pass was made at all
    //       (a review of `551084db3c`, finding 3).
    make_it_a_git_directory(git_dir.as_path());
    make_it_a_git_directory(common.as_path());
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
    // What: The real `refs` folder goes and the link takes its place, so the repository is still one Git accepts while
    //       the walk meets a link at its first component.
    // Why:  The proof answers whether Git accepts the folder, and Git follows a link at `refs`;
    //       the refusal under test is the walk's
    //       (a review of `551084db3c`,
    // finding 2).
    std::fs::remove_dir(common.join("refs")).expect("the real reference folder");
    std::os::unix::fs::symlink(outside.as_path(), common.join("refs").as_path()).expect("the link at refs");
    let persons_file: PathBuf = outside.join("important.lock");
    let persons_bytes: &[u8] = b"a person's data outside every repository\n";
    std::fs::write(persons_file.as_path(), persons_bytes).expect("the person's file");
    make_pid_file_after_the_lock(persons_file.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert_eq!(std::fs::read(persons_file.as_path()).expect("the person's file is still there"), persons_bytes, "a root that is a link is not searched");
    remove(root.as_path());
}

/// What:
///  One folder of a proven Git directory the pass searches,
///  and the lock Git leaves in it.
/// Why:
///  Every one of these folders is reached from the proof by the same walk,
///  so each needs the same two controls:
///  the folder real, where its lock is cleared,
///  and a link at every component in turn, where nothing beyond the link is touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootCase = { control: string; base: 'git-directory' | 'common-directory'; components: string[]; lock: string; everyLock: boolean };
/// ```
struct RootCase {
    /// The name this row's fixtures are filed under.
    control: &'static str,
    /// Which of the proof's two folders the components start at.
    base: ProvenFolder,
    /// The components from there,
    ///  each of which the walk opens as the folder it is named as.
    components: &'static [&'static str],
    /// The lock Git leaves there,
    ///  as a path below the last component.
    lock: &'static str,
    /// Whether the folder is read whole,
    ///  in which case every `*.lock` in it is a lock.
    every_lock: bool,
}

/// What:
///  Every folder the pass searches,
///  with the lock Git's own source writes in it.
/// Why:
///  A row here that no candidate names would be a folder nothing proves,
///  and a row whose lock the pass never clears would make its link arm pass for the wrong reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = [{ base: 'common-directory', components: ['refs'], lock: 'heads/main.lock', everyLock: true }, …];
/// ```
fn root_cases() -> Vec<RootCase> {
    return vec![
        RootCase { control: "refs", base: ProvenFolder::CommonDirectory, components: &["refs"], lock: "heads/main.lock", every_lock: true },
        RootCase { control: "worktrees", base: ProvenFolder::CommonDirectory, components: &["worktrees"], lock: "linked/index.lock", every_lock: true },
        RootCase { control: "sequencer", base: ProvenFolder::GitDirectory, components: &["sequencer"], lock: "todo.lock", every_lock: true },
        RootCase { control: "rebase-merge", base: ProvenFolder::GitDirectory, components: &["rebase-merge"], lock: "interactive.lock", every_lock: true },
        RootCase { control: "rebase-apply", base: ProvenFolder::GitDirectory, components: &["rebase-apply"], lock: "applying.lock", every_lock: true },
        RootCase { control: "info", base: ProvenFolder::GitDirectory, components: &["info"], lock: "sparse-checkout.lock", every_lock: false },
        RootCase { control: "reftable", base: ProvenFolder::CommonDirectory, components: &["reftable"], lock: "tables.list.lock", every_lock: false },
        RootCase { control: "objects", base: ProvenFolder::CommonDirectory, components: &["objects"], lock: "maintenance.lock", every_lock: false },
        RootCase { control: "objects-info", base: ProvenFolder::CommonDirectory, components: &["objects", "info"], lock: "alternates.lock", every_lock: false },
        RootCase { control: "objects-pack", base: ProvenFolder::CommonDirectory, components: &["objects", "pack"], lock: "multi-pack-index.lock", every_lock: false },
    ];
}

/// What:
///  The folder a row's components name below the repository's Git directory or its common one.
/// Why:
///   The fixtures place their files by the same names the candidate table uses,
///  so a row cannot plant a lock the pass was never told to look for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = join(case.base === 'git-directory' ? gitDir : commonDir, ...case.components);
/// ```
fn folder_of_a_root(repository: &CommandRepository, case: &RootCase) -> PathBuf {
    let start: &Path = match case.base {
        ProvenFolder::GitDirectory => repository.git_dir.as_deref().expect("the Git directory"),
        ProvenFolder::CommonDirectory => repository.common_dir.as_deref().expect("the common directory"),
    };
    let mut folder: PathBuf = start.to_path_buf();
    for component in case.components {
        folder = folder.join(component);
    }
    return folder;
}

/// What:
///  Plant the lock Git leaves, its PID file naming a recorded process that is gone,
///  and a person's files beside it.
/// Why:
///   The lock must be one the pass would clear if it reached the folder,
///  so the link arm below proves the walk refused the link rather than that the lock looked held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(folder, lock), 'half-written\n'); makePidFileAfterTheLock(join(folder, lock), gone);
/// writeFileSync(join(folder, 'important.lock'), 'a person's data\n'); makePidFileAfterTheLock(join(folder, 'important.lock'), gone);
/// writeFileSync(join(folder, 'notes.txt'), 'a person's notes\n');
/// ```
fn plant_the_lock_and_a_persons_files(folder: &Path, lock: &str, gone: i64) {
    let lock_path: PathBuf = folder.join(lock);
    std::fs::create_dir_all(lock_path.parent().unwrap_or(folder)).expect("the folder the lock lies in");
    std::fs::write(lock_path.as_path(), b"half-written\n").expect("the lock a killed Git left");
    make_pid_file_after_the_lock(lock_path.as_path(), gone);
    let persons_lock: PathBuf = folder.join("important.lock");
    std::fs::write(persons_lock.as_path(), b"a person's data\n").expect("the person's file");
    make_pid_file_after_the_lock(persons_lock.as_path(), gone);
    std::fs::write(folder.join("notes.txt"), b"a person's notes\n").expect("the person's notes");
}

/// What:
///  Each folder the pass searches is cleared when it is really there,
///  and nothing beyond it is touched when a link stands at any of its components.
/// Why:
///  `read_dir` follows a link at the path it is given and `symlink_metadata` follows a link in every component but the
///  last,
///  so a link at `worktrees`, at `refs`, or at any folder below them sent the pass outside the repository:
///  a wrapped `git status` deleted a person's `HEAD.lock` from `<outside>/user-owned-directory`
///  (a review of `551084db3c`,
/// finding 2),
///  and the same sentence held for every other root one folder to the side.
///  Each row's real arm is the positive control of its link arms:
///  the same lock, the same PID file, cleared when no link is in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const row of rows) {
///   const real = splitRepository(`real-${row.control}`); plantTheLockAndAPersonsFiles(folderOfARoot(real, row), row.lock, real.gone);
///   meetTheLocks(real.repository); expect(existsSync(realLock)).toBe(false);
///   for (let prefix = 1; prefix <= row.components.length; prefix += 1) { /* link at that component, outside folder holds the same files */ expect(outsideStillThere).toBe(true) }
/// }
/// ```
#[test]
fn a_root_is_cleared_when_it_is_really_there_and_never_reached_through_a_link() {
    for case in root_cases() {
        // What: The folder real, with Git's lock in it and a person's files beside it.
        // Why:  This arm is the positive control of the link arms: it shows the plant is one the pass clears.
        a_root_with_no_link_in_the_way(&case);
        // What: A link at each component in turn, and the same lock and person's files beyond it.
        // Why:  Any of these components could be the link that sends the walk outside the repository.
        for prefix in 1..=case.components.len() {
            a_root_reached_through_a_link(&case, prefix);
        }
    }
}

/// What:
///  The row's folder really there,
///  with Git's lock and a person's files in it:
///  the lock goes and the person's files stay.
/// Why:
///   This is the positive control of the row's link arms:
///  the same lock,
///  with the same PID file naming the same gone process,
///  is cleared when no link stands in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, repository, gone } = splitRepository(`real-${row.control}`); plantTheLockAndAPersonsFiles(folderOfARoot(repository, row), row.lock, gone);
/// meetTheLocks(repository); expect(existsSync(realLock)).toBe(false); expect(existsSync(notes)).toBe(true);
/// ```
fn a_root_with_no_link_in_the_way(case: &RootCase) {
    let control: String = format!("root-real-{}", case.control);
    let (root, repository, gone, _record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository(control.as_str());
    let folder: PathBuf = folder_of_a_root(&repository, case);
    std::fs::create_dir_all(folder.as_path()).expect("the folder the row names");
    plant_the_lock_and_a_persons_files(folder.as_path(), case.lock, gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(!folder.join(case.lock).exists(), "the lock Git left in `{}`, reached with no link in the way, is cleared", case.control);
    assert!(folder.join("notes.txt").exists(), "a person's file in `{}` is not a lock and is left alone", case.control);
    assert_eq!(
        folder.join("important.lock").exists(),
        !case.every_lock,
        "in `{}` a `.lock` file is a lock only where the folder is read whole",
        case.control
    );
    remove(root.as_path());
}

/// What:
///  A link at the `prefix`th component of the row,
///  with the row's lock and a person's files in the folder it points at:
///  none of them is touched,
///  because the folder beyond a link is no folder of this repository.
/// Why:
///   `read_dir` follows a link at the path it is given and `symlink_metadata` follows a link in every component but the
///  last,
///  so a link at `worktrees` or at `refs` sent the pass outside the repository and it deleted a person's file there
///  (a review of `551084db3c`,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, repository, gone } = splitRepository(`linked-${row.control}-at-${prefix}`);
/// symlink(outside, join(base, ...row.components.slice(0, prefix))); plantTheLockAndAPersonsFiles(join(outside, ...row.components.slice(prefix)), row.lock, gone);
/// meetTheLocks(repository); expect(existsSync(join(outside, ...rest, row.lock))).toBe(true);
/// ```
fn a_root_reached_through_a_link(case: &RootCase, prefix: usize) {
    let control: String = format!("root-linked-{}-at-{}", case.control, case.components[prefix - 1]);
    let (root, repository, gone, _record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository(control.as_str());
    // The components before the link are real folders, as they are in a repository.
    let mut reached: PathBuf = match case.base {
        ProvenFolder::GitDirectory => repository.git_dir.clone().expect("the Git directory"),
        ProvenFolder::CommonDirectory => repository.common_dir.clone().expect("the common directory"),
    };
    for component in &case.components[..prefix - 1] {
        reached = reached.join(component);
        std::fs::create_dir_all(reached.as_path()).expect("the component before the link");
    }
    let linked: PathBuf = reached.join(case.components[prefix - 1]);
    if linked.exists() {
        std::fs::remove_dir_all(linked.as_path()).expect("the real folder gives way to the link");
    }
    let outside: PathBuf = root.join(format!("outside-every-repository-{prefix}"));
    std::fs::create_dir_all(outside.as_path()).expect("the folder the link points at");
    let mut beyond: PathBuf = outside.clone();
    for component in &case.components[prefix..] {
        beyond = beyond.join(component);
    }
    plant_the_lock_and_a_persons_files(beyond.as_path(), case.lock, gone);
    std::os::unix::fs::symlink(outside.as_path(), linked.as_path()).expect("the link");
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(beyond.join(case.lock).exists(), "a link at `{}` is no folder of this repository, so the lock beyond it is untouched", linked.display());
    assert!(beyond.join("important.lock").exists(), "and a person's file beyond `{}` is untouched", linked.display());
    assert!(beyond.join("notes.txt").exists(), "and so are the person's notes beyond `{}`", linked.display());
    remove(root.as_path());
}

/// What:
///  A folder an option names as the Git directory that is no repository gets no pass and no record:
///  the lock standing in it is untouched and no record folder is made.
/// Why:
///  `--git-dir` was believed before Git said the folder was a repository,
///  so a wrapped `git status` wrote a record into an ordinary folder and the next run deleted a person's
///  `refs/user-data.lock` with its PID companion,
///  while Git refused the folder both times
///  (a review of `551084db3c`,
/// finding 3).
///  The positive control is the `refs` row of
///  `a_root_is_cleared_when_it_is_really_there_and_never_reached_through_a_link`:
///  the same lock in a folder that is a repository is cleared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(join(ordinary, 'refs'), { recursive: true }); writeFileSync(join(ordinary, 'refs/user-data.lock'), bytes); writeFileSync(join(ordinary, 'refs/user-data~pid.lock'), `pid ${gone}`);
/// meetTheLocks({ gitDir: ordinary, commonDir: ordinary, … }); expect(existsSync(join(ordinary, 'refs/user-data.lock'))).toBe(true); expect(existsSync(folderOf(ordinary))).toBe(false)
/// ```
#[test]
fn a_folder_named_as_the_git_directory_that_is_no_repository_gets_no_pass_and_no_record() {
    let root: PathBuf = fixture("lock-roots-a-folder-that-is-no-repository");
    let ordinary: PathBuf = root.join("not-a-repository");
    std::fs::create_dir_all(ordinary.join("refs").as_path()).expect("an ordinary folder with a references folder");
    // What: The record a killed command of an older release left in that folder, and the lock its number explains.
    // Why:  Without a record naming the number, the pass keeps the lock whatever it thinks of the folder, and this
    //       control would pass for the wrong reason.
    let (gone, identity): (i64, String) = a_dead_process();
    let folder: PathBuf = folder_of(ordinary.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    let record: PathBuf = folder.join("record-of-the-gone-holder");
    std::fs::write(record.as_path(), record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes()).expect("the record");
    let lock: PathBuf = ordinary.join("refs/user-data.lock");
    std::fs::write(lock.as_path(), b"a person's data in a folder that is no repository\n").expect("the person's file");
    make_pid_file_after_the_lock(lock.as_path(), gone);
    let repository: CommandRepository = CommandRepository {
        start: root.clone(),
        git_dir: Some(ordinary.clone()),
        common_dir: Some(ordinary.clone()),
        index_file: Some(ordinary.join("index")),
        shallow_file: Some(ordinary.join("shallow")),
        object_dir: Some(ordinary.join("objects")),
    };
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(lock.exists(), "a folder Git refuses is no place this pass opens or unlinks");
    assert!(ordinary.join("refs/user-data~pid.lock").exists(), "and the PID companion beside it is untouched");
    assert!(record.exists(), "and the record of the process that is gone is left where it was, because no pass was made");
    remove(root.as_path());
}

/// What:
///  A root the walk refuses because it is a link leaves the walk incomplete,
///  so the record of the process that left a lock beyond that link is not removed.
/// Why:
///  The proof follows `refs` as Git does and accepts the folder,
///  while the walk refuses a link at the root:
///  a refusal that did not say "I could not see inside" was read as a complete walk,
///  and the record of the lock sitting beyond the link was unlinked as litter,
///  so after the person replaced the link with the real folder no pass could ever clear that lock
///  (a review of `b0ea691cc1`,
/// finding 4).
///  The positive control is the `refs` row of
///  `a_root_is_cleared_when_it_is_really_there_and_never_reached_through_a_link`:
///  the same lock in a folder reached without a link is cleared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlink(linkedRefs, join(common, 'refs')); writeFileSync(join(linkedRefs, 'heads/main.lock'), bytes); writePidCompanion(gone);
/// meetTheLocks(repository); expect(existsSync(lock)).toBe(true); expect(existsSync(record)).toBe(true)
/// ```
#[test]
fn a_root_refused_as_a_link_leaves_the_walk_incomplete_so_its_record_stays() {
    let (root, repository, gone, record): (PathBuf, CommandRepository, i64, PathBuf) = split_repository("lock-roots-a-link-keeps-its-record");
    let common: PathBuf = repository.common_dir.clone().expect("the common directory");
    let linked: PathBuf = root.join("linked-references");
    std::fs::create_dir_all(linked.join("heads").as_path()).expect("the folder the link points at");
    std::fs::remove_dir_all(common.join("refs").as_path()).expect("the real reference folder gives way to the link");
    std::os::unix::fs::symlink(linked.as_path(), common.join("refs").as_path()).expect("the link at refs");
    let lock: PathBuf = linked.join("heads/main.lock");
    std::fs::write(lock.as_path(), b"a half-written reference\n").expect("the abandoned lock beyond the link");
    make_pid_file_after_the_lock(lock.as_path(), gone);
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(lock.exists(), "a root reached through a link is no folder this pass opens");
    assert!(record.exists(), "and the record of the process that left the lock beyond it is not called litter");
    remove(root.as_path());
}

/// What:
///  A folder whose `HEAD` real Git refuses gets no pass,
///  even when its `objects` and `refs` folders,
///  a record of a gone process,
///  and a lock that record explains are all there.
/// Why:
///  The proof read the `HEAD` text loosely,
///  so `ref: \0\n` passed as a symbolic reference while Git refused the folder as no repository,
///  and this pass unlinked `refs/heads/user-data.lock` there first
///  (a review of `b0ea691cc1`,
/// finding 1;
/// the comparison over generated `HEAD` contents is `repo_read/git_directory_proof_tests.rs`).
///  The positive control is the `refs` row of
///  `a_root_is_cleared_when_it_is_really_there_and_never_reached_through_a_link`:
///  the same lock in a folder with a `HEAD` Git accepts is cleared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(join(ordinary, 'HEAD'), 'ref: \u0000\n'); writeFileSync(join(ordinary, 'refs/heads/user-data.lock'), bytes); writePidCompanion(gone);
/// meetTheLocks({ gitDir: ordinary, commonDir: ordinary, … }); expect(existsSync(lock)).toBe(true)
/// ```
#[test]
fn a_folder_whose_head_real_git_refuses_gets_no_pass() {
    let root: PathBuf = fixture("lock-roots-a-head-git-refuses");
    let ordinary: PathBuf = root.join("not-a-repository");
    std::fs::create_dir_all(ordinary.join("objects").as_path()).expect("an object store");
    std::fs::create_dir_all(ordinary.join("refs/heads").as_path()).expect("a references folder");
    std::fs::write(ordinary.join("HEAD"), b"ref: \0\n").expect("the HEAD real Git refuses");
    // The record of a gone process and the lock its number explains, so a pass that ran would clear it.
    let (gone, identity): (i64, String) = a_dead_process();
    let folder: PathBuf = folder_of(ordinary.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    std::fs::write(
        folder.join("record-of-the-gone-holder"),
        record_text(&[(SELF_KIND, gone, identity.clone()), (CHILD_KIND, gone, identity)]).as_bytes(),
    )
    .expect("the record");
    let lock: PathBuf = ordinary.join("refs/heads/user-data.lock");
    std::fs::write(lock.as_path(), b"a person's data\n").expect("the person's file");
    make_pid_file_after_the_lock(lock.as_path(), gone);
    let repository: CommandRepository = CommandRepository {
        start: root.clone(),
        git_dir: Some(ordinary.clone()),
        common_dir: Some(ordinary.clone()),
        index_file: Some(ordinary.join("index")),
        shallow_file: Some(ordinary.join("shallow")),
        object_dir: Some(ordinary.join("objects")),
    };
    meet_the_locks_of_the_repository_the_command_works_on(&repository);
    assert!(lock.exists(), "a folder Git refuses is no place this pass opens or unlinks");
    assert!(ordinary.join("refs/heads/user-data~pid.lock").exists(), "and the PID companion beside it is untouched");
    remove(root.as_path());
}
