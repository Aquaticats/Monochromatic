//! Controls for the batch's side reference written by the landing itself:
//! the file Git writes where Git writes it,
//! the shapes this writer must never take,
//! and the settings that leave the write to Git.
//! Why:
//!  The write this replaces is Git's,
//!  so a control here holds two things at once:
//!  a reference that appears wherever Git would put it,
//!  and a decline,
//! with nothing left behind,
//! for every shape whose write Git would make differently.

use super::*;
use crate::test_support::{fixture, git_text, repository};
use std::path::{Path, PathBuf};

/// The batch every control here writes.
const THE_BATCH: u64 = 12;

/// The side reference of `THE_BATCH`.
fn the_name() -> String {
    return format!("{PREFIX}{THE_BATCH}");
}

/// What:
///  A repository with one commit,
///  and a Git directory of its own to write a reference in.
/// Why:
///   The reference is written into one worktree's Git directory,
///  so a control starts from a repository whose commit the reference can name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); return { repo, gitDir: join(repo, '.git'), tip: revParse('HEAD') };
/// ```
fn a_repository(name: &str) -> (PathBuf, PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let tip: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let git_dir: PathBuf = repo.join(".git");
    return (repo, git_dir, tip);
}

/// The bytes of a file,
/// or nothing when it is not there.
fn bytes_of(path: &Path) -> Option<Vec<u8>> {
    return std::fs::read(path).ok();
}

/// What:
///  The plan one configuration listing gives,
///  in the `git config --list -z --show-scope` form of `(scope, record)` pairs.
/// Why:
///   The settings that leave the write to Git are the ones a control has to build by hand,
///  and a listing is exactly what the landing's own configuration read parses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planOf = (records) => thePlanOf(ConfigSnapshot.parse(records.map(([s, t]) => `${s}\0${t}\0`).join(''), true))
/// ```
fn plan_of(records: &[(&str, &str)]) -> SideReferenceWritePlan {
    let mut listing: Vec<u8> = Vec::new();
    for (scope, record) in records {
        listing.extend_from_slice(scope.as_bytes());
        listing.push(0);
        listing.extend_from_slice(record.as_bytes());
        listing.push(0);
    }
    let Some(snapshot) = ConfigSnapshot::parse(listing.as_slice(), true) else { panic!("the control's listing is well formed") };
    return the_plan_of(Some(&snapshot));
}

/// What:
///  The reference the landing writes names the commit,
///  where Git writes it,
///  and holds what Git's write holds.
/// Why:
///  This is the whole result of the change:
///  the same file,
/// the same bytes,
/// the same place,
///  without a Git start the person waits on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeSideReference(gitDir, name, tip, 'none'); assert.equal(readFile(join(gitDir, name)), tip + '\n')
/// ```
#[test]
fn the_landing_writes_the_batchs_reference_where_git_writes_it() {
    let (repo, git_dir, tip) = a_repository("a-side-reference-written-here");
    let answer: SideReferenceWrite = write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::None);
    assert_eq!(answer, SideReferenceWrite::Written, "the landing writes the batch's own reference");
    let reference: PathBuf = git_dir.join(the_name());
    assert_eq!(bytes_of(reference.as_path()), Some(format!("{tip}\n").into_bytes()), "the reference holds the commit Git's own write holds");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", the_name().as_str()]), tip, "Git reads the reference as it reads its own");
    assert!(!git_dir.join(format!("{PREFIX}{THE_BATCH}.lock")).exists(), "no lock of this write is left behind");
}

/// What:
///  A written reference is overwritten by the next write of the same batch,
///  which is what a retry of the same batch does.
/// Why:
///   A landing may build one batch again,
///  and a reference that already names an earlier attempt must take the new commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeSideReference(gitDir, name, first, 'none'); await writeSideReference(gitDir, name, second, 'none'); assert.equal(await revParse(name), second)
/// ```
#[test]
fn a_reference_that_already_stands_takes_the_new_commit() {
    let (_root, git_dir, tip) = a_repository("a-side-reference-written-twice");
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::None), SideReferenceWrite::Written);
    let other: String = "1".repeat(tip.len());
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), other.as_str(), LockMark::None), SideReferenceWrite::Written);
    assert_eq!(bytes_of(git_dir.join(the_name()).as_path()), Some(format!("{other}\n").into_bytes()), "the second write is what the reference holds");
}

/// What:
///  A repository that marks its reference locks gets the mark beside the lock,
///  and it is gone once the write is done.
/// Why:
///   The wrapper judges a standing lock by the number that mark names,
///  so a lock a landing dies on must be readable exactly like one Git died on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeSideReference(gitDir, name, tip, 'pid'); assert.deepEqual(list(join(gitDir, 'refs/worktree/cli-git-service')), [batch])
/// ```
#[test]
fn a_marked_lock_leaves_no_mark_behind() {
    let (_root, git_dir, tip) = a_repository("a-side-reference-marked-lock");
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::Pid), SideReferenceWrite::Written);
    let directory: PathBuf = git_dir.join("refs").join("worktree").join(DIRECTORY);
    let left: Vec<String> = std::fs::read_dir(directory.as_path())
        .expect("the reference directory")
        .map(|entry| return entry.expect("an entry").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, vec![THE_BATCH.to_string()], "the mark and the lock are given back, and the reference stays");
}

/// What:
///  A name the wrapper does not own is refused,
///  and nothing is written.
/// Why:
///   A branch,
/// a tag and a name Git words its own refusal for are the person's,
///  so a landing that wrote one would be writing a reference Git never made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of ['refs/heads/main', 'refs/worktree/cli-git-service/x', 'refs/worktree/cli-git-service/']) assert.equal(await writeSideReference(gitDir, name, tip, 'none'), 'left-to-git')
/// ```
#[test]
fn a_name_the_wrapper_does_not_own_is_left_to_git() {
    let (repo, git_dir, tip) = a_repository("a-side-reference-other-name");
    for name in ["refs/heads/main", "refs/worktree/cli-git-service/x", "refs/worktree/cli-git-service/", "refs/worktree/cli-git-service/7a"] {
        assert_eq!(
            write_side_reference(git_dir.as_path(), name, tip.as_str(), LockMark::None),
            SideReferenceWrite::LeftToGit,
            "a name the wrapper does not own is Git's to write: {name}"
        );
    }
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "--verify", "--quiet", "refs/heads/main"]), tip, "the branch the control named is untouched");
}

/// What:
///  A commit name that is not a whole object name is refused,
///  and nothing is written.
/// Why:
///   Git refuses to write a reference at a name it cannot parse,
///  so this writer must not write one either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const tip of ['', 'abc', 'A'.repeat(40), '0'.repeat(40), '1'.repeat(41)]) assert.equal(await writeSideReference(gitDir, name, tip, 'none'), 'left-to-git')
/// ```
#[test]
fn an_object_name_that_is_not_whole_is_left_to_git() {
    let (_root, git_dir, tip) = a_repository("a-side-reference-unwhole-name");
    for wrong in [String::new(), String::from("abc"), "A".repeat(40), "0".repeat(40), "1".repeat(41)] {
        assert_eq!(
            write_side_reference(git_dir.as_path(), the_name().as_str(), wrong.as_str(), LockMark::None),
            SideReferenceWrite::LeftToGit,
            "an object name Git would refuse is Git's to refuse: {wrong}"
        );
    }
    assert!(!git_dir.join(the_name()).exists(), "nothing is written for a name Git would refuse");
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::None), SideReferenceWrite::Written);
}

/// What:
///  A lock that already stands is never taken,
///  and its bytes are left as they were.
/// Why:
///   An exclusive create is what proves no other writer holds the reference,
///  so a standing lock belongs to somebody else and this landing must let Git report it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(`${name}.lock`, 'theirs'); assert.equal(await writeSideReference(gitDir, name, tip, 'none'), 'left-to-git'); assert.equal(await readFile(`${name}.lock`), 'theirs')
/// ```
#[test]
fn a_standing_lock_is_left_to_git() {
    let (_root, git_dir, tip) = a_repository("a-side-reference-standing-lock");
    let directory: PathBuf = git_dir.join("refs").join("worktree").join(DIRECTORY);
    std::fs::create_dir_all(directory.as_path()).expect("the reference directory");
    let lock: PathBuf = directory.join(format!("{THE_BATCH}.lock"));
    std::fs::write(lock.as_path(), "theirs\n").expect("a lock another writer made");
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::None), SideReferenceWrite::LeftToGit);
    assert_eq!(bytes_of(lock.as_path()), Some(b"theirs\n".to_vec()), "the standing lock is untouched");
    assert!(!directory.join(THE_BATCH.to_string()).exists(), "no reference is written over a standing lock");
}

/// What:
///  A symbolic reference at the batch's name is left to Git.
/// Why:
///   Replacing a symbolic reference with a plain file would be a write Git never makes for a name that
///  resolves through it,
///  so the shape is Git's to read and to decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink('../heads/main', join(dir, '12')); assert.equal(await writeSideReference(gitDir, name, tip, 'none'), 'left-to-git')
/// ```
#[test]
fn a_symbolic_reference_at_the_name_is_left_to_git() {
    let (_root, git_dir, tip) = a_repository("a-side-reference-symbolic");
    let directory: PathBuf = git_dir.join("refs").join("worktree").join(DIRECTORY);
    std::fs::create_dir_all(directory.as_path()).expect("the reference directory");
    let reference: PathBuf = directory.join(THE_BATCH.to_string());
    std::os::unix::fs::symlink("../heads/main", reference.as_path()).expect("a symbolic reference");
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::None), SideReferenceWrite::LeftToGit);
    assert!(std::fs::symlink_metadata(reference.as_path()).expect("still there").file_type().is_symlink(), "the symbolic reference is untouched");
}

/// What:
///  A path this writer cannot make is left to Git.
/// Why:
///   A `refs` directory that is a file refuses every reference write,
///  and Git words that refusal;
///  this writer must not report a write of its own for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(join(gitDir, 'refs', 'worktree'), 'in the way'); assert.equal(await writeSideReference(gitDir, name, tip, 'none'), 'left-to-git')
/// ```
#[test]
fn a_path_that_cannot_be_made_is_left_to_git() {
    let (_root, git_dir, tip) = a_repository("a-side-reference-path-in-the-way");
    std::fs::write(git_dir.join("refs").join("worktree"), "in the way\n").expect("a file where a directory belongs");
    assert_eq!(write_side_reference(git_dir.as_path(), the_name().as_str(), tip.as_str(), LockMark::None), SideReferenceWrite::LeftToGit);
    assert_eq!(bytes_of(git_dir.join("refs").join("worktree").as_path()), Some(b"in the way\n".to_vec()), "the file in the way is untouched");
}

/// What:
///  A repository whose settings Git's own write would obey keeps Git's write.
/// Why:
///   A reference store that is flushed,
/// a shared repository's modes and a lock mark this module cannot read are
///  writes this writer cannot reproduce,
///  and a configuration read with the caller's own global options proves nothing about the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(thePlanOf(undefined), 'left-to-git'); assert.equal(thePlanOf(withFsyncReference), 'left-to-git')
/// ```
#[test]
fn the_settings_that_leave_the_write_to_git() {
    assert_eq!(the_plan_of(None), SideReferenceWritePlan::LeftToGit, "a configuration that could not be read is Git's");
    assert_eq!(plan_of(&[]), SideReferenceWritePlan::Here(LockMark::None), "a plain repository with no setting of its own is written here");
    assert_eq!(plan_of(&[("local", "core.lockfilepid\ntrue")]), SideReferenceWritePlan::Here(LockMark::Pid), "a repository that marks its locks gets the mark");
    assert_eq!(plan_of(&[("local", "core.lockfilepid\nfalse")]), SideReferenceWritePlan::Here(LockMark::None), "a repository that marks none gets none");
    assert_eq!(plan_of(&[("local", "core.lockfilepid\nmaybe")]), SideReferenceWritePlan::LeftToGit, "a mark this module cannot read is Git's");
    assert_eq!(plan_of(&[("local", "core.fsync\nreference")]), SideReferenceWritePlan::LeftToGit, "a reference store that is flushed is Git's");
    assert_eq!(plan_of(&[("local", "core.fsync\nindex")]), SideReferenceWritePlan::Here(LockMark::None), "a set that names no reference is written here");
    assert_eq!(plan_of(&[("local", "core.fsync\nall")]), SideReferenceWritePlan::LeftToGit, "a set that names every component names the reference");
    assert_eq!(plan_of(&[("local", "core.sharedrepository\ngroup")]), SideReferenceWritePlan::LeftToGit, "a shared repository's modes are Git's");
    assert_eq!(plan_of(&[("local", "extensions.refstorage\nreftable")]), SideReferenceWritePlan::LeftToGit, "a storage that is not plain files is Git's");
    assert_eq!(
        the_plan_of(Some(&ConfigSnapshot::parse(b"", false).expect("an empty listing"))),
        SideReferenceWritePlan::LeftToGit,
        "a configuration read with the caller's own global options proves nothing about the store"
    );
}
