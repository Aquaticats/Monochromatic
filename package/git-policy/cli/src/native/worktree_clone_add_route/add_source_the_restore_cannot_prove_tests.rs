//! Controls for the source shapes the fill's restore cannot prove:
//!  an untracked Git repository of the source's own that the copy's clean keeps,
//!  and a directory where the index records a file,
//!  are both reasons the route declines a source before the claim,
//!  while a repository an ignore rule covers,
//!  or an ordinary uncommitted change,
//!  is no reason.
//!
//! Why the source is read through the real check:
//!  the reasons are asked of the source the preflight proved,
//!  so a control that built the value by hand would prove nothing about the
//!  answers the route gets;
//!  every fixture is a real repository whose Git answers.
//!
//! Why the last control is a repository an ignore rule covers:
//!  the restore carries every ignored path into the new worktree,
//!  so a repository under an ignored directory is the person's and serves the
//!  route;
//!  a question that declined it would send an add to the ladder for no reason.

use super::reasons_the_restore_cannot_prove;
use crate::test_support::{
    fixture,
    git,
    git_context,
    remove,
    repository,
};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_snapshot_source::{
    SnapshotSource,
    source_or_dirty,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  A main checkout with one committed file and one uncommitted edit,
///  and its path.
/// Why:
///  Only a source whose files the fill would restore is asked these questions,
///  and this is the least a source can hold to be one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = aDirtyCheckout('shapes');
/// ```
fn a_dirty_checkout(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    let repo: PathBuf =
        std::fs::canonicalize(repository(root.as_path(), "main")).expect("the checkout's path");
    std::fs::write(repo.join("tracked.txt"), b"first\n").expect("the tracked file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("tracked.txt"), b"first\nedited\n").expect("the uncommitted edit");
    return repo;
}

/// What:
///  The source the route's preflight proves of a main checkout.
/// Why:
///  The reasons are asked of the value the preflight hands the steps,
///  never of a fixture assembled by the control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = sourceOrDirty(context, mainIdentity(repo));
/// ```
fn the_source(repo: &Path) -> SnapshotSource {
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        worktree_root: repo.to_path_buf(),
    };
    return source_or_dirty(&git_context(), &identity)
        .expect("a checkout with an ordinary change is a source the route reads");
}

/// A repository of the source's own at `path`,
/// with one committed file inside it.
fn a_repository_at(path: &Path) {
    std::fs::create_dir_all(path).expect("the repository's directory");
    git(path, &["init", "--quiet"]);
    std::fs::write(path.join("inside.txt"), b"inside\n").expect("the repository's own file");
    git(path, &["add", "--", "inside.txt"]);
    git(path, &["commit", "--quiet", "--message=inside"]);
}

/// What:
///  An untracked directory holding a Git repository of its own is a reason,
///  named by the directory the proof would still call untracked.
/// Why:
///  `git clean --force -d` keeps a repository it meets without a word,
///  and the proof lists what the clean left as untracked,
///  so the restore ends after the clone is made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reasonsTheRestoreCannotProve(source)).toContain('nested/');
/// ```
#[test]
fn an_untracked_repository_is_a_reason() {
    let repo: PathBuf = a_dirty_checkout("shapes-untracked-repository");
    a_repository_at(repo.join("nested").as_path());
    let reasons: Vec<String> =
        reasons_the_restore_cannot_prove((&git_context(), &the_source(repo.as_path())))
            .expect("Git answers");
    assert_eq!(
        reasons.len(),
        1,
        "one repository is one reason: {reasons:?}"
    );
    assert!(
        reasons[0].contains("nested/") && reasons[0].contains("clean"),
        "the reason names the directory and the clean that keeps it: {reasons:?}"
    );
    remove(repo.parent().expect("the fixture root"));
}

/// What:
///  A repository two directories down is a reason all the same,
///  named by the untracked directory above it.
/// Why:
///   Git's one-force clean reports the repository it skips and keeps the
///   directory above it,
///  and the proof then names that directory as untracked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await aRepositoryAt('pkg/nested'); expect(reasonsTheRestoreCannotProve(source)).toContain('pkg/');
/// ```
#[test]
fn a_repository_below_an_untracked_directory_is_a_reason() {
    let repo: PathBuf = a_dirty_checkout("shapes-repository-below");
    a_repository_at(repo.join("pkg").join("nested").as_path());
    let reasons: Vec<String> =
        reasons_the_restore_cannot_prove((&git_context(), &the_source(repo.as_path())))
            .expect("Git answers");
    assert_eq!(
        reasons.len(),
        1,
        "one repository is one reason: {reasons:?}"
    );
    assert!(
        reasons[0].contains("pkg/"),
        "the reason names the untracked directory the proof lists: {reasons:?}"
    );
    remove(repo.parent().expect("the fixture root"));
}

/// What:
///  A repository an ignore rule covers is no reason,
///  because the restore carries every ignored path into the new worktree.
/// Why:
///  The owner's rule is that a person's ignored cache,
///  local configuration and environment files stay,
///  and a repository kept in one is the person's too;
///  the route serves that source today and must go on serving it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile('.gitignore', 'cache/\n'); await aRepositoryAt('cache/nested');
/// expect(reasonsTheRestoreCannotProve(source)).toEqual([]);
/// ```
#[test]
fn an_ignored_repository_is_no_reason() {
    let repo: PathBuf = a_dirty_checkout("shapes-ignored-repository");
    std::fs::write(repo.join(".gitignore"), b"cache/\n").expect("the ignore rule");
    git(repo.as_path(), &["add", "--", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
    a_repository_at(repo.join("cache").join("nested").as_path());
    let reasons: Vec<String> =
        reasons_the_restore_cannot_prove((&git_context(), &the_source(repo.as_path())))
            .expect("Git answers");
    assert_eq!(
        reasons,
        Vec::<String>::new(),
        "an ignored repository is carried, not refused: {reasons:?}"
    );
    remove(repo.parent().expect("the fixture root"));
}

/// What:
///  A directory where the index records a file is a reason.
/// Why:
///  The restore removes the files its reset cannot see as files or links,
///  and a directory cannot be removed that way,
///  so the restore ends where Git's own checkout would have written the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rmSync('tracked.txt'); mkdirSync('tracked.txt'); writeFileSync('tracked.txt/x', 'x');
/// expect(reasonsTheRestoreCannotProve(source)).toContain('tracked.txt');
/// ```
#[test]
fn a_directory_where_the_index_records_a_file_is_a_reason() {
    let repo: PathBuf = a_dirty_checkout("shapes-directory-at-a-file");
    std::fs::remove_file(repo.join("tracked.txt")).expect("the file goes");
    std::fs::create_dir(repo.join("tracked.txt")).expect("the directory takes its name");
    std::fs::write(repo.join("tracked.txt").join("inside.txt"), b"inside\n")
        .expect("a file inside it");
    let reasons: Vec<String> =
        reasons_the_restore_cannot_prove((&git_context(), &the_source(repo.as_path())))
            .expect("Git answers");
    assert_eq!(reasons.len(), 1, "one directory is one reason: {reasons:?}");
    assert!(
        reasons[0].contains("tracked.txt") && reasons[0].contains("directory"),
        "the reason names the path and what stands there: {reasons:?}"
    );
    remove(repo.parent().expect("the fixture root"));
}

/// What:
///  A checkout whose only fault is an ordinary uncommitted change is asked
///  nothing and is no reason.
/// Why:
///  This is the source the fill restores every day,
///  and a reason here would send every such add to the ladder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reasonsTheRestoreCannotProve(aDirtyCheckout())).toEqual([]);
/// ```
#[test]
fn an_ordinary_change_is_no_reason() {
    let repo: PathBuf = a_dirty_checkout("shapes-ordinary-change");
    let reasons: Vec<String> =
        reasons_the_restore_cannot_prove((&git_context(), &the_source(repo.as_path())))
            .expect("Git answers");
    assert_eq!(
        reasons,
        Vec::<String>::new(),
        "an ordinary change is what the restore is for: {reasons:?}"
    );
    remove(repo.parent().expect("the fixture root"));
}

/// What:
///  A clean checkout is asked nothing,
///  because the fill restores nothing of it.
/// Why:
///  The two questions cost Git runs and a walk of the untracked directories,
///  and a clean source has none of the shapes they look for.
///  The tracked file is aged before Git records it,
///  so the entry is not one Git has to settle by content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a clean checkout, a repository of its own outside it, and no reason
/// expect(reasonsTheRestoreCannotProve(aCleanCheckout())).toEqual([]);
/// ```
#[test]
fn a_clean_checkout_is_asked_nothing() {
    let root: PathBuf = fixture("shapes-clean-checkout");
    let repo: PathBuf =
        std::fs::canonicalize(repository(root.as_path(), "main")).expect("the checkout's path");
    std::fs::write(repo.join("tracked.txt"), b"first\n").expect("the tracked file");
    crate::test_support::age(repo.join("tracked.txt").as_path());
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    git(repo.as_path(), &["update-index", "--refresh"]);
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        worktree_root: repo.clone(),
    };
    let source: SnapshotSource =
        source_or_dirty(&git_context(), &identity).expect("a clean checkout is a source");
    assert!(source.dirty.is_none(), "the fixture's checkout is clean");
    let reasons: Vec<String> = reasons_the_restore_cannot_prove((&git_context(), &source))
        .expect("a clean source is not asked");
    assert_eq!(reasons, Vec::<String>::new());
    remove(root.as_path());
}

/// What:
///  An untracked path the account cannot remove is no reason.
/// Why:
///  This is the boundary of the two questions:
///  a clean that cannot unlink a path (`warning: failed to remove ...:
///  Permission denied`,
///  exit 1) is a step failure,
///  not a refusal of the proof,
///  and no read of the source can rule out every such failure (the immutable
///  bit,
///  a full disk,
///  a filesystem that stops answering).
///  The settle of plan step 14 is that class's answer,
///  and the restore's own control shows the clean ending for this fixture
///  (`worktree::worktree_dirty_restore::tests::a_restore_ends_at_the_clean_it_cannot_finish`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chmodSync('d', 0o500); expect(reasonsTheRestoreCannotProve(source)).toEqual([]);
/// ```
#[test]
fn a_path_the_clean_cannot_remove_is_no_reason() {
    let repo: PathBuf = a_dirty_checkout("shapes-unwritable-directory");
    let held: PathBuf = repo.join("d");
    std::fs::create_dir(held.as_path()).expect("the untracked directory");
    std::fs::write(held.join("held.txt"), b"held\n").expect("a file inside it");
    std::fs::set_permissions(
        held.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o500),
    )
    .expect("the directory loses its write permission");
    let reasons: Vec<String> =
        reasons_the_restore_cannot_prove((&git_context(), &the_source(repo.as_path())))
            .expect("Git answers");
    std::fs::set_permissions(
        held.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("the fixture gives the permission back");
    assert_eq!(
        reasons,
        Vec::<String>::new(),
        "a step failure is the settle's, not this question's: {reasons:?}"
    );
    remove(repo.parent().expect("the fixture root"));
}
