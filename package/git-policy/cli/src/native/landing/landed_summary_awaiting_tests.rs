//! Controls for the awaited face of the landed commit's name:
//! it answers what the synchronous face answers,
//! where the synchronous face cannot answer at all,
//!  and a reading of the object store that gives no answer asks Git rather than the last resort.

use super::*;
use crate::blocking_jobs_within_the_budget::a_controls_answer::{AStatesAnswer, state_the_answer};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// What:
///  A repository whose `core.abbrev` asks for a name longer than the last resort,
///  the commit they name and their object store.
/// Why:
///  A control must tell a name Git answered from the seven characters the last resort gives,
/// so the
///  abbreviation it asks for is longer than seven.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = initRepo(); commit(repo); git(repo, ['config', 'core.abbrev', '13']); return { repo, commit: git(repo, ['rev-parse', 'HEAD']) };
/// ```
fn a_repository_whose_name_git_answers(name: &str) -> (PathBuf, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "one\n").expect("file");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["config", "core.abbrev", "13"]);
    let commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    return (root, commit);
}

/// The snapshot of what a Git command line set for the repository,
/// with the scope of each entry.
fn snapshot_with(abbreviation: &str) -> ConfigSnapshot {
    let root: PathBuf = fixture("landed-summary-awaiting-snapshot");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let listing: Vec<u8> = git_text(repo.as_path(), &["-c", format!("core.abbrev={abbreviation}").as_str(), "config", "--list", "-z", "--show-scope"]).into_bytes();
    remove(root.as_path());
    return ConfigSnapshot::parse(listing.as_slice(), true).expect("a snapshot");
}

/// What:
///  The awaited face gives the name the synchronous face gives,
/// with the files answering and with Git answering,
/// and it
///  gives it inside the runtime,
/// where the synchronous face cannot start its Git at all.
/// Why:
///  The two faces print one person's landed commit,
/// so a difference in the name they print is one commit described two
///  ways;
/// the name Git answers is what pins that the awaited face's child ran and was awaited rather than
///  abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const config of [snapshot, undefined]) assert.equal(await shortNameAwaiting(...), shortName(...));
/// ```
#[test]
fn the_awaited_name_answers_what_the_synchronous_one_answers_inside_the_runtime() {
    let (root, commit) = a_repository_whose_name_git_answers("landed-summary-awaiting-agrees");
    let repo: PathBuf = root.join("repo");
    let context: GitContext = git_context();
    let objects: PathBuf = repo.join(".git/objects");
    let snapshot: ConfigSnapshot = snapshot_with("12");
    let from_the_files_expected: String = git_text(repo.as_path(), &["-c", "core.abbrev=12", "rev-parse", "--short", commit.as_str()]).trim().to_string();
    let from_git_expected: String = git_text(repo.as_path(), &["rev-parse", "--short", commit.as_str()]).trim().to_string();
    for (label, config, expected) in [
        ("the files answer", Some(&snapshot), from_the_files_expected.as_str()),
        ("Git answers", None, from_git_expected.as_str()),
    ] {
        let synchronous: String = short_name(&context, repo.as_path(), objects.as_path(), config, commit.as_str());
        let awaited: String =
            crate::lazy_runtime::run_on_runtime(short_name_awaiting(&context, repo.as_path(), objects.as_path(), config, commit.as_str())).expect("the runtime");
        assert_eq!(synchronous, expected, "{label}: the synchronous face is the one the two faces are compared on");
        assert_eq!(awaited, synchronous, "{label}: the two faces answer one name for the same commit");
    }
    remove(root.as_path());
}

/// What:
///  A reading of the object store that gives no answer asks Git,
///  and the name is Git's rather than the last resort.
/// Why:
///  A task of the runtime may not read the object store,
/// so the reading is a job of the pool with a ceiling;
/// a job that outlasted its
///  ceiling answered nothing,
/// and nothing is what the files answer when they cannot describe the name,
/// so the caller asks
///  Git.
/// A reading nobody answered must not fall through to the seven characters:
/// that is a name Git never
///  printed,
/// and the person copies it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using _stated = stateTheAnswer('noAnswer'); const name = await shortNameAwaiting({ config: snapshotWith('12') }); assert.equal(name, gitShortName(repo, commit))
/// ```
#[test]
fn a_reading_of_the_object_store_that_gives_no_answer_asks_git() {
    let (root, commit) = a_repository_whose_name_git_answers("landed-summary-awaiting-no-answer");
    let repo: PathBuf = root.join("repo");
    let context: GitContext = git_context();
    let objects: PathBuf = repo.join(".git/objects");
    let snapshot: ConfigSnapshot = snapshot_with("12");
    let expected: String = git_text(repo.as_path(), &["rev-parse", "--short", commit.as_str()]).trim().to_string();
    let _stated = state_the_answer(AStatesAnswer::NoAnswer);
    let awaited: String = crate::lazy_runtime::run_on_runtime(short_name_awaiting(&context, repo.as_path(), objects.as_path(), Some(&snapshot), commit.as_str()))
        .expect("the runtime");
    assert_eq!(awaited, expected, "a reading of the object store that gave no answer is answered by Git's own name, not by the last resort");
    assert_ne!(awaited.len(), LAST_RESORT_LENGTH, "the seven characters Git did not print are not the name the person is given");
    remove(root.as_path());
}
