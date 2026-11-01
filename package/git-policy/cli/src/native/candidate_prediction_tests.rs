//! What:
//!  Controls for both private-index candidate sources against real Git 2.56.0.
//! Why:
//!  `git add` candidates must be exactly what that command would stage,
//!  and a direct
//!      command's candidates exactly the selected worktree files,
//!  for every kind of entry,
//!      while the real index stays byte for byte as it was.
//!  Staged and worktree bytes are
//!      swapped between the two sides so a source that read the wrong side would fail.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const prepared = await prepareCandidates(git, ['-C', repo], [], { kind: 'add', region: ['--', 'a.txt'] });
//! // expect(await prepared.store.bytes(prepared.version.candidates[0])).toEqual(worktreeBytes);
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{CandidateRequest, PreparedCandidates, prepare_candidates};
use crate::candidate_error::{CandidateError, CandidateFailure};
use crate::candidate_object::CandidateMode;
use crate::candidate_private_index::PRIVATE_INDEX_PREFIX;
use crate::candidate_record::CandidateChange;
use crate::candidate_version::Candidate;
use crate::test_support::{REAL_GIT, fixture, git, git_output, remove, repository};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The arguments of a request,
///  written as text.
fn arguments(values: &[&str]) -> Vec<OsString> {
    let mut list: Vec<OsString> = Vec::new();
    for value in values {
        list.push(OsString::from(value));
    }
    return list;
}

/// Prepare through real Git with `-C directory` and an extra environment.
fn prepare_with(
    directory: &Path,
    overlay: &[(OsString, OsString)],
    request: &CandidateRequest,
) -> Result<PreparedCandidates, CandidateError> {
    return prepare_candidates(
        Path::new(REAL_GIT),
        &[OsString::from("-C"), directory.as_os_str().to_os_string()],
        overlay,
        request,
    );
}

/// The prediction of `git add` with these arguments,
///  which the control requires to succeed.
fn add(directory: &Path, values: &[&str]) -> PreparedCandidates {
    return prepare_with(directory, &[], &CandidateRequest::Add(arguments(values)))
        .expect("prediction");
}

/// The projection of these pathspecs,
///  which the control requires to succeed.
fn direct(directory: &Path, values: &[&str]) -> PreparedCandidates {
    return prepare_with(directory, &[], &CandidateRequest::Direct(arguments(values)))
        .expect("projection");
}

/// The failure of a preparation,
///  which the control requires.
fn refused(directory: &Path, request: &CandidateRequest) -> CandidateError {
    match prepare_with(directory, &[], request) {
        Ok(prepared) => panic!(
            "expected a failure, got {} candidates",
            prepared.version.candidates().len()
        ),
        Err(error) => return error,
    }
}

/// Each candidate's pathname,
///  as text,
///  with its change.
fn listed(prepared: &PreparedCandidates) -> Vec<(String, CandidateChange)> {
    let mut found: Vec<(String, CandidateChange)> = Vec::new();
    for candidate in prepared.version.candidates() {
        found.push((
            String::from_utf8_lossy(candidate.path.as_slice()).into_owned(),
            candidate.change,
        ));
    }
    return found;
}

/// The bytes of the candidate at `path`.
fn bytes(prepared: &mut PreparedCandidates, path: &[u8]) -> Vec<u8> {
    let candidate: Candidate = prepared
        .version
        .candidate_at_path(path)
        .expect("candidate")
        .clone();
    let read: Rc<[u8]> = prepared.store.bytes(&candidate).expect("bytes");
    return read.to_vec();
}

/// The real index file's bytes.
fn index_bytes(repo: &Path) -> Vec<u8> {
    return std::fs::read(repo.join(".git/index")).expect("real index");
}

/// The entries of the Git directory left by a private index,
///  which must be none.
fn private_leftovers(git_dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(git_dir).expect("Git directory") {
        let name: String = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.starts_with(PRIVATE_INDEX_PREFIX) {
            found.push(name);
        }
    }
    return found;
}

/// A committed base:
///  `tracked.txt`,
///  `remove.txt`,
///  `link` and `keep.txt`.
fn base(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    std::fs::write(repo.join("remove.txt"), b"to remove\n").expect("remove");
    std::fs::write(repo.join("keep.txt"), b"kept\n").expect("keep");
    std::os::unix::fs::symlink("keep.txt", repo.join("link")).expect("link");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    return repo;
}

/// `git add` candidates are the staged changes it would make,
///  with worktree bytes,
///  and the real index is untouched.
#[test]
fn add_candidates_are_what_the_add_would_stage() {
    let root: PathBuf = fixture("predict-add");
    let repo: PathBuf = base(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked, edited\n").expect("edit");
    std::fs::write(repo.join("new.txt"), b"new\n").expect("new");
    std::fs::remove_file(repo.join("remove.txt")).expect("remove");
    std::fs::remove_file(repo.join("link")).expect("relink");
    std::os::unix::fs::symlink("new.txt", repo.join("link")).expect("link target");
    std::fs::write(repo.join("untouched.txt"), b"not named\n").expect("untouched");
    let before: Vec<u8> = index_bytes(repo.as_path());
    let mut prepared: PreparedCandidates = add(
        repo.as_path(),
        &[
            "--",
            "tracked.txt",
            "new.txt",
            "remove.txt",
            "link",
            "keep.txt",
        ],
    );
    // Paths the index held come first, in index order; the new path follows.
    assert_eq!(
        listed(&prepared),
        vec![
            (String::from("link"), CandidateChange::Modified),
            (String::from("remove.txt"), CandidateChange::Deleted),
            (String::from("tracked.txt"), CandidateChange::Modified),
            (String::from("new.txt"), CandidateChange::Added),
        ]
    );
    assert_eq!(bytes(&mut prepared, b"tracked.txt"), b"tracked, edited\n");
    assert_eq!(bytes(&mut prepared, b"new.txt"), b"new\n");
    assert_eq!(bytes(&mut prepared, b"link"), b"new.txt");
    assert_eq!(bytes(&mut prepared, b"remove.txt"), b"");
    let link: Candidate = prepared
        .version
        .candidate_at_path(b"link")
        .expect("link")
        .clone();
    assert_eq!(link.mode, CandidateMode::Symlink);
    let deleted: Candidate = prepared
        .version
        .candidate_at_path(b"remove.txt")
        .expect("deletion")
        .clone();
    assert_eq!(deleted.object, None);
    assert_eq!(deleted.mode, CandidateMode::Regular);
    assert_eq!(
        index_bytes(repo.as_path()),
        before,
        "the real index is unchanged"
    );
    assert_eq!(private_leftovers(repo.join(".git").as_path()).len(), 1);
    drop(prepared);
    assert_eq!(
        private_leftovers(repo.join(".git").as_path()),
        Vec::<String>::new()
    );
    assert_eq!(index_bytes(repo.as_path()), before);
    remove(root.as_path());
}

/// The add's candidate holds the worktree bytes it would stage,
///  never what is already staged,
///  in both directions.
#[test]
fn add_candidates_read_the_worktree_not_the_staged_copy() {
    let root: PathBuf = fixture("predict-add-swap");
    let repo: PathBuf = base(root.as_path(), "repo");
    // Staged dirty, worktree clean: `git add` would stage the clean bytes.
    std::fs::write(repo.join("tracked.txt"), b"STAGED DIRTY\n").expect("dirty");
    git(repo.as_path(), &["add", "tracked.txt"]);
    std::fs::write(repo.join("tracked.txt"), b"worktree clean\n").expect("clean");
    // A second file staged dirty that this add does not name is not a candidate.
    std::fs::write(repo.join("keep.txt"), b"STAGED DIRTY ELSEWHERE\n").expect("other");
    git(repo.as_path(), &["add", "keep.txt"]);
    let mut clean: PreparedCandidates = add(repo.as_path(), &["--", "tracked.txt"]);
    assert_eq!(
        listed(&clean),
        vec![(String::from("tracked.txt"), CandidateChange::Modified)]
    );
    assert_eq!(bytes(&mut clean, b"tracked.txt"), b"worktree clean\n");
    drop(clean);
    // Staged clean, worktree dirty: `git add` would stage the dirty bytes.
    std::fs::write(repo.join("tracked.txt"), b"staged clean\n").expect("clean");
    git(repo.as_path(), &["add", "tracked.txt"]);
    std::fs::write(repo.join("tracked.txt"), b"WORKTREE DIRTY\n").expect("dirty");
    let mut dirty: PreparedCandidates = add(repo.as_path(), &["--", "tracked.txt"]);
    assert_eq!(bytes(&mut dirty, b"tracked.txt"), b"WORKTREE DIRTY\n");
    drop(dirty);
    remove(root.as_path());
}

/// Additions without content,
///  a reverted file,
///  a dry run and a never-committed removal follow the installed wrapper.
#[test]
fn intent_to_add_reverts_dry_runs_and_uncommitted_removals() {
    let root: PathBuf = fixture("predict-add-kinds");
    let repo: PathBuf = base(root.as_path(), "repo");
    std::fs::write(repo.join("intent.txt"), b"intended content\n").expect("intent");
    let mut intent: PreparedCandidates = add(repo.as_path(), &["--intent-to-add", "intent.txt"]);
    assert_eq!(
        listed(&intent),
        vec![(String::from("intent.txt"), CandidateChange::Added)]
    );
    assert_eq!(
        bytes(&mut intent, b"intent.txt"),
        b"",
        "an intent records no content"
    );
    drop(intent);
    git(repo.as_path(), &["add", "--intent-to-add", "intent.txt"]);
    let mut realized: PreparedCandidates = add(repo.as_path(), &["intent.txt"]);
    assert_eq!(
        listed(&realized),
        vec![(String::from("intent.txt"), CandidateChange::Added)]
    );
    assert_eq!(bytes(&mut realized, b"intent.txt"), b"intended content\n");
    drop(realized);
    // Staged away from HEAD, then reverted in the worktree: re-adding restores HEAD's entry, still a candidate.
    std::fs::write(repo.join("tracked.txt"), b"changed\n").expect("change");
    git(repo.as_path(), &["add", "tracked.txt"]);
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("revert");
    let mut reverted: PreparedCandidates = add(repo.as_path(), &["tracked.txt"]);
    assert_eq!(
        listed(&reverted),
        vec![(String::from("tracked.txt"), CandidateChange::Modified)]
    );
    assert_eq!(bytes(&mut reverted, b"tracked.txt"), b"tracked\n");
    drop(reverted);
    // A dry run stages nothing.
    std::fs::write(repo.join("dry.txt"), b"dry\n").expect("dry");
    let dry: PreparedCandidates = add(repo.as_path(), &["--dry-run", "dry.txt"]);
    assert_eq!(listed(&dry), Vec::new());
    drop(dry);
    // A staged file that was never committed and is then deleted leaves nothing to check.
    git(repo.as_path(), &["add", "dry.txt"]);
    std::fs::remove_file(repo.join("dry.txt")).expect("delete");
    let gone: PreparedCandidates = add(repo.as_path(), &["dry.txt"]);
    assert_eq!(listed(&gone), Vec::new());
    drop(gone);
    remove(root.as_path());
}

/// During a merge conflict,
///  adding another path ignores the conflict,
///  and adding the conflicted path resolves it.
#[test]
fn a_conflict_elsewhere_does_not_stop_the_prediction() {
    let root: PathBuf = fixture("predict-add-conflict");
    let repo: PathBuf = base(root.as_path(), "repo");
    git(repo.as_path(), &["switch", "--quiet", "--create", "side"]);
    std::fs::write(repo.join("tracked.txt"), b"side\n").expect("side");
    git(
        repo.as_path(),
        &["commit", "--quiet", "--all", "--message=side"],
    );
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("tracked.txt"), b"main\n").expect("main");
    git(
        repo.as_path(),
        &["commit", "--quiet", "--all", "--message=main"],
    );
    let merge: std::process::Output = git_output(repo.as_path(), &["merge", "--quiet", "side"]);
    assert!(!merge.status.success(), "the merge conflicts");
    std::fs::write(repo.join("other.txt"), b"other\n").expect("other");
    let mut other: PreparedCandidates = add(repo.as_path(), &["other.txt"]);
    assert_eq!(
        listed(&other),
        vec![(String::from("other.txt"), CandidateChange::Added)]
    );
    assert_eq!(bytes(&mut other, b"other.txt"), b"other\n");
    drop(other);
    std::fs::write(repo.join("tracked.txt"), b"resolved\n").expect("resolve");
    let mut resolved: PreparedCandidates = add(repo.as_path(), &["tracked.txt"]);
    assert_eq!(
        listed(&resolved),
        vec![(String::from("tracked.txt"), CandidateChange::Modified)]
    );
    assert_eq!(bytes(&mut resolved, b"tracked.txt"), b"resolved\n");
    drop(resolved);
    remove(root.as_path());
}

/// Names that are not UTF-8,
///  a subdirectory,
///  a repository without commits and a linked worktree all work.
#[test]
fn names_directories_unborn_heads_and_linked_worktrees() {
    let root: PathBuf = fixture("predict-add-places");
    let repo: PathBuf = base(root.as_path(), "repo");
    let name: &[u8] = b"n\xffme \x01.txt";
    std::fs::write(repo.join(OsStr::from_bytes(name)), b"bytes\n").expect("odd name");
    let mut odd: PreparedCandidates = add(repo.as_path(), &["--all"]);
    assert_eq!(odd.version.candidates().len(), 1);
    assert_eq!(odd.version.candidates()[0].path, name.to_vec());
    assert_eq!(bytes(&mut odd, name), b"bytes\n");
    drop(odd);
    // From a subdirectory, staging a path above it is still found, named from the top level.
    std::fs::create_dir(repo.join("sub")).expect("sub");
    std::fs::write(repo.join("sub/inner.txt"), b"inner\n").expect("inner");
    let below: PreparedCandidates = add(repo.join("sub").as_path(), &["inner.txt", "../keep.txt"]);
    assert_eq!(
        listed(&below),
        vec![(String::from("sub/inner.txt"), CandidateChange::Added)]
    );
    drop(below);
    std::fs::write(repo.join("keep.txt"), b"kept, edited\n").expect("edit");
    let above: PreparedCandidates = add(repo.join("sub").as_path(), &["../keep.txt"]);
    assert_eq!(
        listed(&above),
        vec![(String::from("keep.txt"), CandidateChange::Modified)]
    );
    drop(above);
    // Without any commit, every entry is an addition.
    let unborn: PathBuf = root.join("unborn");
    std::fs::create_dir(&unborn).expect("unborn");
    git(
        unborn.as_path(),
        &["init", "--quiet", "--initial-branch=main"],
    );
    std::fs::write(unborn.join("first.txt"), b"first\n").expect("first");
    let mut first: PreparedCandidates = add(unborn.as_path(), &["first.txt"]);
    assert_eq!(
        listed(&first),
        vec![(String::from("first.txt"), CandidateChange::Added)]
    );
    assert_eq!(bytes(&mut first, b"first.txt"), b"first\n");
    drop(first);
    // A linked worktree has its own index, and the private copy is made beside it.
    let linked: PathBuf = root.join("linked");
    git(
        repo.as_path(),
        &[
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("--quiet"),
            linked.as_os_str(),
        ],
    );
    std::fs::write(linked.join("tracked.txt"), b"linked edit\n").expect("linked edit");
    let worktree_index: PathBuf = repo.join(".git/worktrees/linked/index");
    let index_before: Vec<u8> = std::fs::read(&worktree_index).expect("linked index");
    let mut in_linked: PreparedCandidates = add(linked.as_path(), &["tracked.txt"]);
    assert_eq!(
        private_leftovers(repo.join(".git/worktrees/linked").as_path()).len(),
        1
    );
    assert_eq!(bytes(&mut in_linked, b"tracked.txt"), b"linked edit\n");
    drop(in_linked);
    assert_eq!(
        std::fs::read(&worktree_index).expect("linked index"),
        index_before
    );
    remove(root.as_path());
}

/// Literal pathspec mode from the environment or the prefix leaves the prediction complete.
#[test]
fn literal_pathspec_mode_cannot_empty_the_prediction() {
    let root: PathBuf = fixture("predict-add-literal");
    let repo: PathBuf = base(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"edited\n").expect("edit");
    let literal_environment: Vec<(OsString, OsString)> =
        vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1"))];
    let from_environment: PreparedCandidates = prepare_with(
        repo.as_path(),
        literal_environment.as_slice(),
        &CandidateRequest::Add(arguments(&["tracked.txt"])),
    )
    .expect("prediction");
    assert_eq!(
        listed(&from_environment),
        vec![(String::from("tracked.txt"), CandidateChange::Modified)]
    );
    drop(from_environment);
    let from_prefix: PreparedCandidates = prepare_candidates(
        Path::new(REAL_GIT),
        &[
            OsString::from("-C"),
            repo.as_os_str().to_os_string(),
            OsString::from("--literal-pathspecs"),
        ],
        &[],
        &CandidateRequest::Add(arguments(&["tracked.txt"])),
    )
    .expect("prediction");
    assert_eq!(
        listed(&from_prefix),
        vec![(String::from("tracked.txt"), CandidateChange::Modified)]
    );
    drop(from_prefix);
    remove(root.as_path());
}

/// A replay Git refuses,
///  or a Git that cannot start,
///  is a failure that names Git's reason and leaves nothing behind.
#[test]
fn failures_report_git_and_leave_the_index_alone() {
    let root: PathBuf = fixture("predict-failures");
    let repo: PathBuf = base(root.as_path(), "repo");
    let before: Vec<u8> = index_bytes(repo.as_path());
    let missing: CandidateError = refused(
        repo.as_path(),
        &CandidateRequest::Add(arguments(&["no-such-file.txt"])),
    );
    assert_eq!(missing.failure, CandidateFailure::GitFailed);
    assert!(
        missing.message.starts_with(
            "cli-git could not predict what git add stages: git add failed: fatal: pathspec"
        ),
        "{}",
        missing.message
    );
    let direct_missing: CandidateError = refused(
        repo.as_path(),
        &CandidateRequest::Direct(arguments(&["no-such-file.txt"])),
    );
    assert_eq!(direct_missing.failure, CandidateFailure::GitFailed);
    assert!(
        direct_missing
            .message
            .starts_with("cli-git could not read the selected worktree files: git add failed:"),
        "{}",
        direct_missing.message
    );
    assert_eq!(index_bytes(repo.as_path()), before);
    assert_eq!(
        private_leftovers(repo.join(".git").as_path()),
        Vec::<String>::new()
    );
    // Outside a repository Git cannot name an index.
    let outside: CandidateError = refused(
        root.as_path(),
        &CandidateRequest::Add(arguments(&["file.txt"])),
    );
    assert_eq!(outside.failure, CandidateFailure::GitFailed);
    assert!(
        outside
            .message
            .starts_with("cli-git could not predict what git add stages: git rev-parse failed:"),
        "{}",
        outside.message
    );
    let not_started: Result<PreparedCandidates, CandidateError> = prepare_candidates(
        root.join("no-git").as_path(),
        &[],
        &[],
        &CandidateRequest::Direct(arguments(&[":/"])),
    );
    let Err(start) = not_started else {
        panic!("a missing Git cannot prepare anything");
    };
    assert_eq!(start.failure, CandidateFailure::GitNotStarted);
    assert!(
        start
            .message
            .starts_with("cli-git could not read the selected worktree files: git rev-parse could not be started:"),
        "{}",
        start.message
    );
    remove(root.as_path());
}

/// A direct scope holds every selected worktree file with worktree bytes,
///  tracked and new,
///  never ignored ones.
#[test]
fn direct_candidates_are_every_selected_worktree_file() {
    let root: PathBuf = fixture("project-direct");
    let repo: PathBuf = base(root.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), b"ignored.txt\n").expect("ignore");
    std::fs::write(repo.join("ignored.txt"), b"ignored\n").expect("ignored");
    std::fs::write(repo.join("new.txt"), b"new\n").expect("new");
    std::fs::remove_file(repo.join("remove.txt")).expect("remove");
    // Staged dirty, worktree clean, and the reverse.
    std::fs::write(repo.join("tracked.txt"), b"STAGED DIRTY\n").expect("dirty");
    git(repo.as_path(), &["add", "tracked.txt"]);
    std::fs::write(repo.join("tracked.txt"), b"worktree clean\n").expect("clean");
    std::fs::write(repo.join("keep.txt"), b"staged clean\n").expect("clean");
    git(repo.as_path(), &["add", "keep.txt"]);
    std::fs::write(repo.join("keep.txt"), b"WORKTREE DIRTY\n").expect("dirty");
    let before: Vec<u8> = index_bytes(repo.as_path());
    let mut everything: PreparedCandidates = direct(repo.as_path(), &[":/"]);
    assert_eq!(
        listed(&everything),
        vec![
            (String::from(".gitignore"), CandidateChange::Added),
            (String::from("keep.txt"), CandidateChange::Modified),
            (String::from("link"), CandidateChange::Modified),
            (String::from("new.txt"), CandidateChange::Added),
            (String::from("tracked.txt"), CandidateChange::Modified),
        ]
    );
    assert_eq!(bytes(&mut everything, b"tracked.txt"), b"worktree clean\n");
    assert_eq!(bytes(&mut everything, b"keep.txt"), b"WORKTREE DIRTY\n");
    assert_eq!(bytes(&mut everything, b"link"), b"keep.txt");
    drop(everything);
    let one: PreparedCandidates = direct(repo.as_path(), &["new.txt"]);
    assert_eq!(
        listed(&one),
        vec![(String::from("new.txt"), CandidateChange::Added)]
    );
    drop(one);
    // A scope that only removes a file selects nothing.
    let removed: PreparedCandidates = direct(repo.as_path(), &["remove.txt"]);
    assert_eq!(listed(&removed), Vec::new());
    drop(removed);
    assert_eq!(
        index_bytes(repo.as_path()),
        before,
        "the real index is unchanged"
    );
    assert_eq!(
        private_leftovers(repo.join(".git").as_path()),
        Vec::<String>::new()
    );
    remove(root.as_path());
}

/// `StagedAtBase` marks additions against the recorded base, not against a `HEAD` another landing moved;
///  an unborn base marks every entry added, and no pathspec means no candidate.
#[test]
fn staged_at_base_compares_with_the_recorded_base_and_not_the_moved_head() {
    let root: PathBuf = fixture("predict-staged-at-base");
    let repo: PathBuf = base(root.as_path(), "repo");
    let recorded: String = String::from_utf8_lossy(git(repo.as_path(), &["rev-parse", "HEAD"]).stdout.as_slice())
        .trim()
        .to_string();
    std::fs::write(repo.join("fresh.txt"), b"fresh\n").expect("fresh");
    git(repo.as_path(), &["add", "--", "fresh.txt"]);
    // Another landing moves HEAD by committing the same path.
    git(repo.as_path(), &["commit", "--quiet", "--message=moved", "--only", "--", "fresh.txt"]);
    let scope: Vec<OsString> = vec![OsString::from(":(top,literal)fresh.txt")];
    let live: Vec<(String, CandidateChange)> =
        listed(&prepare_with(repo.as_path(), &[], &CandidateRequest::Staged(scope.clone())).expect("staged"));
    assert_eq!(live, vec![(String::from("fresh.txt"), CandidateChange::Modified)]);
    let object = crate::candidate_object::parse_object_id(recorded.as_bytes()).expect("object name");
    let at_base: Vec<(String, CandidateChange)> = listed(
        &prepare_with(
            repo.as_path(),
            &[],
            &CandidateRequest::StagedAtBase { base: Some(object), pathspecs: scope.clone() },
        )
        .expect("at base"),
    );
    assert_eq!(at_base, vec![(String::from("fresh.txt"), CandidateChange::Added)]);
    let unborn: Vec<(String, CandidateChange)> = listed(
        &prepare_with(
            repo.as_path(),
            &[],
            &CandidateRequest::StagedAtBase {
                base: None,
                pathspecs: vec![OsString::from(":(top,literal)tracked.txt")],
            },
        )
        .expect("unborn"),
    );
    assert_eq!(unborn, vec![(String::from("tracked.txt"), CandidateChange::Added)]);
    let nothing = prepare_with(
        repo.as_path(),
        &[],
        &CandidateRequest::StagedAtBase { base: None, pathspecs: Vec::new() },
    )
    .expect("empty scope");
    assert!(nothing.version.candidates().is_empty());
    remove(root.as_path());
}
