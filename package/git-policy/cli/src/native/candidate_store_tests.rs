//! What:
//!  Disposable-repository controls for candidate versions and bytes against real Git 2.56.0.
//! Why:
//!  Candidates must describe exactly what Git would record (staged or committed),
//!      never the live worktree,
//!  for every kind of entry:
//!  non-UTF-8 names and content,
//!      symbolic links,
//!  submodules,
//!  deletions,
//!  renames and empty files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const version = await store.version({ kind: 'staged-against-head' });
//! // expect(await store.bytes(version.candidateAtPath('a.txt'))).toEqual(stagedBytes);
//! ```
#![cfg(unix)]

/// Import the store under test and shared fixtures.
use super::CandidateStore;
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
use crate::candidate_record::CandidateChange;
use crate::candidate_version::{Candidate, CandidateSource, CandidateVersion};
use crate::git_metadata::strip_git_line;
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A store over real Git selecting one repository with `-C`.
pub(super) fn store_with(program: &Path, directory: &Path) -> CandidateStore {
    return CandidateStore::new(
        program,
        &[OsString::from("-C"), directory.as_os_str().to_os_string()],
        &[],
    );
}

/// A store over the real Git executable.
pub(super) fn store(directory: &Path) -> CandidateStore {
    return store_with(Path::new(REAL_GIT), directory);
}

/// The object name a revision expression resolves to.
pub(super) fn rev_parse(repository: &Path, revision: &str) -> ObjectId {
    let output: std::process::Output = git(repository, &["rev-parse", "--verify", revision]);
    return parse_object_id(strip_git_line(output.stdout.as_slice())).expect("object name");
}

/// Write a worktree file whose name may be any bytes.
pub(super) fn write(repository: &Path, name: &[u8], content: &[u8]) {
    std::fs::write(repository.join(OsStr::from_bytes(name)), content).expect("worktree file");
}

/// The candidate at a pathname,
///  which the control requires to exist.
/// `'version` says the returned candidate is borrowed from the version,
///  not from the pathname.
pub(super) fn at<'version>(
    version: &'version CandidateVersion,
    path: &[u8],
) -> &'version Candidate {
    match version.candidate_at_path(path) {
        Some(candidate) => return candidate,
        None => panic!("no candidate at {:?}", String::from_utf8_lossy(path)),
    }
}

/// The sorted pathnames of a version.
pub(super) fn paths(version: &CandidateVersion) -> Vec<Vec<u8>> {
    let mut names: Vec<Vec<u8>> = Vec::new();
    for candidate in version.candidates() {
        names.push(candidate.path.clone());
    }
    names.sort();
    return names;
}

/// A base commit,
///  then one staged change of every kind;
///  returns the repository.
pub(super) fn staged_changes(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    write(repo.as_path(), b"keep.txt", b"unchanged\n");
    write(repo.as_path(), b"modify.txt", b"before\n");
    write(repo.as_path(), b"delete.txt", b"to be deleted\n");
    write(repo.as_path(), b"rename-old.txt", b"moved content\n");
    write(repo.as_path(), b"mode.sh", b"#!/bin/sh\n");
    write(repo.as_path(), b"type.txt", b"a file that becomes a link\n");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    write(repo.as_path(), b"modify.txt", b"after\n");
    git(repo.as_path(), &["rm", "--quiet", "delete.txt"]);
    git(repo.as_path(), &["mv", "rename-old.txt", "rename-new.txt"]);
    std::fs::set_permissions(repo.join("mode.sh"), std::fs::Permissions::from_mode(0o755))
        .expect("make the script executable");
    std::fs::remove_file(repo.join("type.txt")).expect("remove file before linking");
    std::os::unix::fs::symlink("keep.txt", repo.join("type.txt")).expect("type change");
    write(repo.as_path(), b"empty.txt", b"");
    write(
        repo.as_path(),
        b"n\xff\xfeame.bin",
        b"\0\xff content \xfe\n",
    );
    std::os::unix::fs::symlink("target with space", repo.join("link")).expect("symlink");
    git(repo.as_path(), &["add", "--all"]);
    let head: ObjectId = rev_parse(repo.as_path(), "HEAD");
    git(
        repo.as_path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            format!("160000,{},sub", head.as_str()).as_str(),
        ],
    );
    return repo;
}

/// The staged version lists every changed path with its mode,
///  change and object,
///  and nothing unchanged.
#[test]
fn staged_version_describes_every_kind_of_entry() {
    let root: PathBuf = fixture("store-staged");
    let repo: PathBuf = staged_changes(root.as_path());
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(
        paths(&version),
        [
            b"delete.txt".to_vec(),
            b"empty.txt".to_vec(),
            b"link".to_vec(),
            b"mode.sh".to_vec(),
            b"modify.txt".to_vec(),
            b"n\xff\xfeame.bin".to_vec(),
            b"rename-new.txt".to_vec(),
            b"rename-old.txt".to_vec(),
            b"sub".to_vec(),
            b"type.txt".to_vec(),
        ]
    );
    for (path, mode, change) in [
        (
            b"delete.txt".as_slice(),
            CandidateMode::Regular,
            CandidateChange::Deleted,
        ),
        (b"empty.txt", CandidateMode::Regular, CandidateChange::Added),
        (b"link", CandidateMode::Symlink, CandidateChange::Added),
        (
            b"mode.sh",
            CandidateMode::Executable,
            CandidateChange::Modified,
        ),
        (
            b"modify.txt",
            CandidateMode::Regular,
            CandidateChange::Modified,
        ),
        (
            b"n\xff\xfeame.bin",
            CandidateMode::Regular,
            CandidateChange::Added,
        ),
        (
            b"rename-new.txt",
            CandidateMode::Regular,
            CandidateChange::Added,
        ),
        (
            b"rename-old.txt",
            CandidateMode::Regular,
            CandidateChange::Deleted,
        ),
        (b"sub", CandidateMode::Gitlink, CandidateChange::Added),
        (
            b"type.txt",
            CandidateMode::Symlink,
            CandidateChange::Modified,
        ),
    ] {
        let candidate: &Candidate = at(&version, path);
        assert_eq!(candidate.mode, mode, "{:?}", String::from_utf8_lossy(path));
        assert_eq!(
            candidate.change,
            change,
            "{:?}",
            String::from_utf8_lossy(path)
        );
        // A deleted path has no object; every other entry names one.
        assert_eq!(
            candidate.object.is_none(),
            change == CandidateChange::Deleted
        );
    }
    // A rename without rename detection is a deletion and an addition of the same object.
    assert_eq!(
        at(&version, b"rename-new.txt").object,
        Some(rev_parse(repo.as_path(), "HEAD:rename-old.txt"))
    );
    remove(root.as_path());
}

/// Bytes are exactly what the index names:
///  text,
///  binary,
///  empty,
///  link target,
///  submodule commit name,
///  and nothing for a deletion.
#[test]
fn staged_bytes_are_exact_for_every_kind_of_entry() {
    let root: PathBuf = fixture("store-bytes");
    let repo: PathBuf = staged_changes(root.as_path());
    let head: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    for (path, expected) in [
        (b"modify.txt".as_slice(), b"after\n".as_slice()),
        (b"empty.txt", b""),
        (b"n\xff\xfeame.bin", b"\0\xff content \xfe\n"),
        (b"link", b"target with space"),
        (b"type.txt", b"keep.txt"),
        (b"mode.sh", b"#!/bin/sh\n"),
        (b"rename-new.txt", b"moved content\n"),
        (b"delete.txt", b""),
        (b"rename-old.txt", b""),
        (b"sub", head.as_str().as_bytes()),
    ] {
        let bytes: Rc<[u8]> = subject.bytes(at(&version, path)).expect("candidate bytes");
        assert_eq!(&bytes[..], expected, "{:?}", String::from_utf8_lossy(path));
    }
    remove(root.as_path());
}

/// A deletion and a submodule entry are answered without the object reader;
///  a blob starts it.
#[test]
fn reader_starts_only_for_blob_content() {
    let root: PathBuf = fixture("store-lazy");
    let repo: PathBuf = staged_changes(root.as_path());
    let head: ObjectId = rev_parse(repo.as_path(), "HEAD");
    let mut subject: CandidateStore = store(repo.as_path());
    assert!(subject.reader.is_none());
    // Listing against a named commit needs no `HEAD` lookup, so it starts no reader either.
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstCommit(head.clone()))
        .expect("staged version");
    assert!(subject.reader.is_none());
    assert!(
        subject
            .bytes(at(&version, b"delete.txt"))
            .expect("deleted")
            .is_empty()
    );
    assert_eq!(
        &subject.bytes(at(&version, b"sub")).expect("gitlink")[..],
        head.as_str().as_bytes()
    );
    assert!(subject.reader.is_none());
    assert_eq!(
        &subject.bytes(at(&version, b"modify.txt")).expect("blob")[..],
        b"after\n"
    );
    assert!(subject.reader.is_some());
    remove(root.as_path());
}

/// Candidate bytes are the staged bytes even when the worktree file was changed,
///  emptied or removed afterwards.
#[test]
fn bytes_ignore_the_live_worktree() {
    let root: PathBuf = fixture("store-isolation");
    let repo: PathBuf = repository(root.as_path(), "repo");
    write(repo.as_path(), b"changed.txt", b"staged one\n");
    write(repo.as_path(), b"removed.txt", b"staged two\n");
    write(repo.as_path(), b"unstaged.txt", b"never staged\n");
    git(repo.as_path(), &["add", "changed.txt", "removed.txt"]);
    write(repo.as_path(), b"changed.txt", b"worktree only\n");
    std::fs::remove_file(repo.join("removed.txt")).expect("remove worktree file");
    let mut subject: CandidateStore = store(repo.as_path());
    let version: Rc<CandidateVersion> = subject
        .version(&CandidateSource::StagedAgainstHead)
        .expect("staged version");
    assert_eq!(
        paths(&version),
        [b"changed.txt".to_vec(), b"removed.txt".to_vec()]
    );
    assert_eq!(
        &subject.bytes(at(&version, b"changed.txt")).expect("staged")[..],
        b"staged one\n"
    );
    assert_eq!(
        &subject.bytes(at(&version, b"removed.txt")).expect("staged")[..],
        b"staged two\n"
    );
    remove(root.as_path());
}
