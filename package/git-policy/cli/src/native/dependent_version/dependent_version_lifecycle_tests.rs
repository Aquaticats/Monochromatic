//! What: Controls for the planner's view of a candidate state over real Git: the tracked
//!       listing, the overlaid bytes, `HEAD`'s copy, and every refusal.
//! Why: A file listed wrongly, a correction not read back, or a `HEAD` copy read from the
//!      wrong place would plan a different ripple than the installed wrapper does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const view = lifecycleWorkspace(store, corrected, []); expect(await view.trackedPaths()).toEqual([...]);
//! ```
#![cfg(unix)]

/// Import the view under test and its mode mapping.
use super::{lifecycle_workspace, tracked_mode};
/// The modes and object names entries carry.
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
/// The store the view reads through.
use crate::candidate_store::CandidateStore;
/// The planner's seam and its types.
use crate::dependent_version_content::{
    ContentUnavailable, TrackedMode, TrackedPath, WorkspaceContent,
};
/// A corrected file.
use crate::direct_fix_install::InstallChange;
/// The object name of a revision.
use crate::git_metadata::strip_git_line;
/// Shared fixture directories and real Git.
use crate::test_support::{REAL_GIT, fixture, git, remove, repository};
/// Corrected files by pathname.
use std::collections::BTreeMap;
/// Owned operating-system text.
use std::ffi::OsString;
/// Unix permission bits, for the executable file.
use std::os::unix::fs::PermissionsExt;
/// Borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// Shared byte handles.
use std::rc::Rc;

/// A store over real Git selecting `directory` with `-C`.
fn store(directory: &Path) -> CandidateStore {
    return CandidateStore::new(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), directory.as_os_str().to_os_string()],
        &[],
    );
}

/// The object a revision expression names.
fn object(repository: &Path, revision: &str) -> ObjectId {
    let output: std::process::Output = git(repository, &["rev-parse", "--verify", revision]);
    return parse_object_id(strip_git_line(output.stdout.as_slice())).expect("object name");
}

/// The failure a read returned, which the control requires.
fn refusal<T: std::fmt::Debug>(result: Result<T, ContentUnavailable>) -> ContentUnavailable {
    match result {
        Ok(value) => panic!("expected a refusal, got {value:?}"),
        Err(error) => return error,
    }
}

/// A repository whose `HEAD` holds a manifest, an executable and a link; the index then
/// stages a changed manifest and a new file, and the worktree changes the manifest again.
fn staged_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::create_dir_all(repo.join("package/module/a")).expect("directories");
    std::fs::write(
        repo.join("package/module/a/package.json"),
        b"{\"version\":\"1.0.0\"}\n",
    )
    .expect("manifest");
    std::fs::write(repo.join("run.sh"), b"#!/bin/sh\n").expect("script");
    std::fs::set_permissions(repo.join("run.sh"), std::fs::Permissions::from_mode(0o755))
        .expect("executable");
    std::os::unix::fs::symlink("run.sh", repo.join("link")).expect("link");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(
        repo.join("package/module/a/package.json"),
        b"{\"version\":\"1.1.0\"}\n",
    )
    .expect("raised manifest");
    std::fs::write(repo.join("new.txt"), b"new\n").expect("new file");
    git(repo.as_path(), &["add", "--all"]);
    std::fs::write(
        repo.join("package/module/a/package.json"),
        b"{\"version\":\"9.9.9\"}\n",
    )
    .expect("unstaged change");
    return repo;
}

/// The listing holds every index entry with its mode from any directory; bytes are the
/// index's, not the worktree's; `HEAD` reads its own copy, and a new file has none.
#[test]
fn reads_the_index_and_head_from_any_directory() {
    let root: PathBuf = fixture("lifecycle-reads");
    let repo: PathBuf = staged_repository(root.as_path());
    let mut subject_store: CandidateStore = store(repo.join("package").as_path());
    let corrected: BTreeMap<Vec<u8>, InstallChange> = BTreeMap::new();
    let candidates: [Vec<u8>; 1] = [b"new.txt".to_vec()];
    let mut view = lifecycle_workspace(&mut subject_store, &corrected, &candidates);
    assert_eq!(
        view.tracked_paths().expect("listing"),
        vec![
            TrackedPath {
                path: b"link".to_vec(),
                mode: TrackedMode::Symlink
            },
            TrackedPath {
                path: b"new.txt".to_vec(),
                mode: TrackedMode::Regular
            },
            TrackedPath {
                path: b"package/module/a/package.json".to_vec(),
                mode: TrackedMode::Regular,
            },
            TrackedPath {
                path: b"run.sh".to_vec(),
                mode: TrackedMode::Executable
            },
        ]
    );
    assert_eq!(
        view.candidate_bytes(b"package/module/a/package.json")
            .expect("index bytes"),
        b"{\"version\":\"1.1.0\"}\n"
    );
    assert_eq!(
        view.candidate_bytes(b"link").expect("link bytes"),
        b"run.sh"
    );
    assert_eq!(
        view.base_bytes(b"package/module/a/package.json")
            .expect("head bytes"),
        Some(b"{\"version\":\"1.0.0\"}\n".to_vec())
    );
    assert_eq!(view.base_bytes(b"new.txt").expect("new file"), None);
    assert_eq!(
        view.index_entry(b"run.sh").expect("index entry"),
        Some((CandidateMode::Executable, object(repo.as_path(), ":run.sh")))
    );
    assert_eq!(
        view.head_entry(b"package/module/a/package.json")
            .expect("head entry"),
        Some((
            CandidateMode::Regular,
            object(repo.as_path(), "HEAD:package/module/a/package.json")
        ))
    );
    assert_eq!(view.index_entry(b"missing").expect("no entry"), None);
    assert_eq!(view.head_entry(b"new.txt").expect("no head entry"), None);
    let untracked: ContentUnavailable = refusal(view.candidate_bytes(b"untracked.txt"));
    assert_eq!(untracked.path, Some(b"untracked.txt".to_vec()));
    assert!(
        untracked.reason.contains("not a tracked file"),
        "{}",
        untracked.reason
    );
    remove(root.as_path());
}

/// A corrected file is read back from the correction, every other file from the index.
#[test]
fn corrections_are_read_before_the_index() {
    let root: PathBuf = fixture("lifecycle-overlay");
    let repo: PathBuf = staged_repository(root.as_path());
    let mut subject_store: CandidateStore = store(repo.as_path());
    let mut corrected: BTreeMap<Vec<u8>, InstallChange> = BTreeMap::new();
    corrected.insert(
        b"new.txt".to_vec(),
        InstallChange {
            path: b"new.txt".to_vec(),
            mode: CandidateMode::Regular,
            original: Rc::from(&b"new\n"[..]),
            replacement: Rc::from(&b"corrected\n"[..]),
        },
    );
    let mut view = lifecycle_workspace(&mut subject_store, &corrected, &[]);
    assert_eq!(
        view.candidate_bytes(b"new.txt").expect("corrected"),
        b"corrected\n"
    );
    assert_eq!(
        view.candidate_bytes(b"run.sh").expect("index"),
        b"#!/bin/sh\n"
    );
    remove(root.as_path());
}

/// Before the first commit nothing has a `HEAD` copy.
#[test]
fn an_unborn_head_has_no_copies() {
    let root: PathBuf = fixture("lifecycle-unborn");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(
        repo.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    std::fs::write(repo.join("a.txt"), b"a\n").expect("file");
    git(repo.as_path(), &["add", "a.txt"]);
    let mut subject_store: CandidateStore = store(repo.as_path());
    let corrected: BTreeMap<Vec<u8>, InstallChange> = BTreeMap::new();
    let mut view = lifecycle_workspace(&mut subject_store, &corrected, &[]);
    assert_eq!(view.base_bytes(b"a.txt").expect("no head"), None);
    assert_eq!(view.candidate_bytes(b"a.txt").expect("index"), b"a\n");
    remove(root.as_path());
}

/// An unmerged entry, a candidate the listing lacks, and listings Git refuses each stop
/// the plan, naming the path when one is concerned.
#[test]
fn refuses_listings_that_are_not_the_candidate_state() {
    let root: PathBuf = fixture("lifecycle-refusals");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("c.txt"), b"base\n").expect("base");
    git(repo.as_path(), &["add", "c.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    std::fs::write(repo.join("c.txt"), b"side\n").expect("side");
    git(
        repo.as_path(),
        &["commit", "--quiet", "--all", "--message=side"],
    );
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("c.txt"), b"main\n").expect("main");
    git(
        repo.as_path(),
        &["commit", "--quiet", "--all", "--message=main"],
    );
    let _ = crate::test_support::git_output(repo.as_path(), &["merge", "--quiet", "side"]);
    let corrected: BTreeMap<Vec<u8>, InstallChange> = BTreeMap::new();
    let mut conflicted_store: CandidateStore = store(repo.as_path());
    let mut conflicted = lifecycle_workspace(&mut conflicted_store, &corrected, &[]);
    let unmerged: ContentUnavailable = refusal(conflicted.tracked_paths());
    assert_eq!(unmerged.path, Some(b"c.txt".to_vec()));
    assert!(unmerged.reason.contains("unmerged"), "{}", unmerged.reason);
    git(repo.as_path(), &["merge", "--abort"]);
    let mut unlisted_store: CandidateStore = store(repo.as_path());
    let candidates: [Vec<u8>; 1] = [b"gone.txt".to_vec()];
    let mut unlisted = lifecycle_workspace(&mut unlisted_store, &corrected, &candidates);
    let missing: ContentUnavailable = refusal(unlisted.tracked_paths());
    assert_eq!(missing.path, Some(b"gone.txt".to_vec()));
    assert!(
        missing.reason.contains("does not hold"),
        "{}",
        missing.reason
    );
    let outside: PathBuf = root.join("not-a-repository");
    std::fs::create_dir(&outside).expect("plain directory");
    let mut failing_store: CandidateStore = store(outside.as_path());
    let mut failing = lifecycle_workspace(&mut failing_store, &corrected, &[]);
    assert_eq!(refusal(failing.tracked_paths()).path, None);
    assert_eq!(refusal(failing.base_bytes(b"c.txt")).path, None);
    remove(root.as_path());
}

/// The four Git modes map onto the planner's four.
#[test]
fn maps_every_mode() {
    assert_eq!(tracked_mode(CandidateMode::Regular), TrackedMode::Regular);
    assert_eq!(
        tracked_mode(CandidateMode::Executable),
        TrackedMode::Executable
    );
    assert_eq!(tracked_mode(CandidateMode::Symlink), TrackedMode::Symlink);
    assert_eq!(tracked_mode(CandidateMode::Gitlink), TrackedMode::Submodule);
}
