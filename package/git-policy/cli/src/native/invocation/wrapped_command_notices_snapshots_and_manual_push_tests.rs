//! Controls for what every command says about the worktree's pushes:
//! the held ones,
//! whose notice must reach the person without any notice directory existing yet,
//! and the words it uses.

use super::*;
use crate::push_failure_notice::CommitNamer;
/// The fixtures,
/// the real Git the fixtures run,
/// and the overlay that isolates its configuration.
use crate::test_support::{fixture, git, git_text, isolated_overlay, remove};
use crate::work_item::{WorkItem, WorkKind};

/// A commit name as it is,
/// so a control reads the line the record holds.
struct AsIs;

impl CommitNamer for AsIs {
    fn name_of(&self, commit: &str) -> String {
        return commit.to_string();
    }
}

/// A held push alone says its line:
/// no push-failure notice directory and no orphan-notice directory exists,
/// and the recording is what makes the one the notice is read from.
#[test]
fn a_held_push_is_said_with_no_notice_directory() {
    let root: PathBuf = fixture("notices-held-push");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let oid: String = String::from("1".repeat(40));
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: oid.clone(),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "no notice directory exists before the push is held");
    crate::work_item::hold(work_dir.as_path(), &item).expect("the push is held");
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "the hold wrote no second file: the record is the state");
    let said: Vec<String> = crate::service_handoff::held_push_notices(git_dir.as_path());
    assert_eq!(said.len(), 1, "one line: {said:?}");
    assert!(said[0].contains(oid.as_str()), "{}", said[0]);
    assert!(said[0].contains("next push"), "{}", said[0]);
    // The words claim only what is known: the push is held, and the next push of its branch carries it.
    assert!(!said[0].contains("stopped"), "the line does not claim the command stopped: {}", said[0]);
    assert!(!said[0].contains("checks"), "and not that its checks ran or not: {}", said[0]);
    // The state stands while the record stands, so the line stands with it and cannot disagree with it.
    let again: Vec<String> = crate::service_handoff::held_push_notices(git_dir.as_path());
    assert_eq!(again, said, "the line is the record's: {again:?}");
    assert!(
        crate::work_item::state_directory(work_dir.as_path(), crate::work_item::HELD).join(item.id()).exists(),
        "the record stays held"
    );
    remove(root.as_path());
}

/// A worktree with nothing held and no notice says nothing at all.
#[test]
fn a_worktree_with_nothing_held_says_nothing() {
    let root: PathBuf = fixture("notices-held-push-quiet");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    assert!(crate::service_handoff::held_push_notices(git_dir.as_path()).is_empty(), "a worktree with nothing held says nothing");
    assert!(push_notices_for(git_dir.as_path(), git_dir.as_path(), &AsIs).is_empty(), "and nothing failed either");
    assert!(!notice_directory(git_dir.as_path()).is_dir(), "the check made no directory either");
    remove(root.as_path());
}

/// A released push is not said:
/// the record is gone,
/// so the line is gone with it,
/// and no second file had to be cleared.
#[test]
fn a_released_push_is_not_said() {
    let root: PathBuf = fixture("notices-held-push-released");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("2".repeat(40)),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    crate::work_item::hold(work_dir.as_path(), &item).expect("the push is held");
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path()).len(), 1);
    assert!(crate::work_item::promote(work_dir.as_path(), item.id().as_str()), "the producer released it");
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path()), Vec::<String>::new(), "a released push is not said");
    remove(root.as_path());
}

/// A blocked push is not said:
/// the verdict removed the record,
/// so nothing reports it as held.
#[test]
fn a_blocked_push_is_not_said() {
    let root: PathBuf = fixture("notices-held-push-blocked");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let work_dir: PathBuf = crate::service_work::work_directory(git_dir.as_path());
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repo.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("3".repeat(40)),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    crate::work_item::hold(work_dir.as_path(), &item).expect("the push is held");
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path()).len(), 1);
    crate::work_item::block(work_dir.as_path(), item.id().as_str());
    assert_eq!(crate::service_handoff::held_push_notices(git_dir.as_path()), Vec::<String>::new(), "a blocked push is not said");
    remove(root.as_path());
}

/// What:
///  A push from a bare repository runs the discovery's queries in that repository,
///  not in the directory the command was typed in.
/// Why:
///   Every query of the discovery names no repository of its own,
///  so Git resolves the one its working directory is in.
///  A bare repository has no worktree,
///  and a push from one through `-C` starts in a directory that need not be a repository at all:
///  `for-each-ref` then failed with `not a git repository`,
///  and the push stopped as `content-unavailable` before Git ran (`Refs #6273`).
///  The control also asks the same query from the fixture root,
///  which is no repository,
///  so the directory it names is proven to be the difference the answer rests on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(manualPushDirectory(bareIdentity)).toBe(bareGitDir);
/// expect(trackingCommits({ cwd: manualPushDirectory(bareIdentity) })).resolves.toEqual(new Map());
/// await expect(trackingCommits({ cwd: fixtureRoot })).rejects.toThrow('not a git repository');
/// ```
#[test]
fn a_push_from_a_bare_repository_looks_at_that_repository() {
    let root: PathBuf = fixture("manual-push-bare-directory");
    let work: PathBuf = crate::test_support::repository(root.as_path(), "work");
    let bare: PathBuf = root.join("bare.git");
    git(bare.parent().expect("the fixture root"), &["init", "--quiet", "--bare", bare.to_str().expect("a text path")]);
    git(work.as_path(), &["push", "--quiet", bare.to_str().expect("a text path"), "main:main"]);
    // The identity the invocation's own location query answers for that repository.
    let asked: Vec<OsString> = crate::repository_location::location_query_arguments(&[]);
    let answered: std::process::Output = crate::test_support::git_output(bare.as_path(), asked.as_slice());
    let location: crate::repository_location::RepositoryLocation =
        crate::repository_location::parse_location_output(answered.status.success(), answered.stdout.as_slice()).expect("the location");
    assert!(
        matches!(location.identity, WorktreeIdentity::BareRepository { .. }),
        "a bare repository has no worktree: the identity is {:?}",
        location.identity
    );
    let directory: &Path = manual_push_directory(&location.identity).expect("a bare repository names the directory its queries run in");
    // The bare repository's own directory, which Git printed as its absolute common one.
    let named: PathBuf = PathBuf::from(git_text(bare.as_path(), &["rev-parse", "--path-format=absolute", "--git-common-dir"]));
    assert_eq!(directory, named.as_path(), "the queries run in the repository the push names");
    let push: PushGit = PushGit {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        cwd: directory.to_path_buf(),
        environment: isolated_overlay(),
        lfs: false,
    };
    let tracking: std::collections::BTreeMap<String, Vec<String>> =
        crate::manual_push_range::tracking_commits(&push, &[String::from("origin")]).expect("the bare repository's tracking refs are read there");
    assert!(tracking.get("origin").expect("the remote asked for").is_empty(), "it has no remote-tracking refs");
    // The positive control: the same query in the fixture root, which is no repository, fails.
    let elsewhere: PushGit = PushGit { cwd: root.clone(), ..push };
    let refused: crate::manual_push_range::ManualPushRangeError =
        crate::manual_push_range::tracking_commits(&elsewhere, &[String::from("origin")]).expect_err("a directory that is no repository cannot answer");
    assert!(refused.message.contains("not a git repository"), "{}", refused.message);
    remove(root.as_path());
}
