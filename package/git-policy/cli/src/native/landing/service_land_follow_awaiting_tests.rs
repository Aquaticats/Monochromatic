//! Controls for the awaited steps of the landing's follow-up:
//! the index a reset installs is the one the synchronous reset installs,
//! the derived names are
//!  the ones it reads,
//! and both are done inside the runtime,
//! where the synchronous faces cannot start Git at all.

use super::tests::{differs, index_behind};
use super::*;
use crate::test_support::{git_context, git_output, remove};
use std::path::PathBuf;

/// What:
///  The entries the paths of a commit take in the index,
///  whichever face reset them,
///  and an entry list that names a path whose name is not
///  text.
/// Why:
///  The two faces install the index one landing hands the person,
/// so a difference here is a commit whose index shows
///  another commit's entries once the landing is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(await resetEntriesAwaiting(git, repo, index, second, [b'new.txt']), resetEntries(git, repo, index, second, [b'new.txt']));
/// ```
#[test]
fn the_awaited_reset_installs_what_the_synchronous_one_installs() {
    let names: Vec<Vec<u8>> = vec![b"not-utf8-\xff\xfe.txt".to_vec(), b"plain.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("follow-awaiting-reset", names.as_slice());
    assert!(differs(repo.as_path(), second.as_str()), "the index starts behind the commit");
    let synchronous: Result<(), FailureCode> = reset_entries(&git_context(), repo.as_path(), Some(repo.join(".git/index").as_path()), second.as_str(), names.as_slice());
    assert_eq!(synchronous, Ok(()), "the synchronous face resets the entries of a commit it built");
    assert!(!differs(repo.as_path(), second.as_str()), "the synchronous reset installs the commit's entries");
    remove(root.as_path());
    let (awaited_root, awaited_repo, _awaited_first, awaited_second) = index_behind("follow-awaiting-reset-awaited", names.as_slice());
    assert!(differs(awaited_repo.as_path(), awaited_second.as_str()), "the index starts behind the commit");
    let awaited: Result<(), FailureCode> = crate::lazy_runtime::run_on_runtime(reset_entries_awaiting(
        &git_context(),
        awaited_repo.as_path(),
        Some(awaited_repo.join(".git/index").as_path()),
        awaited_second.as_str(),
        names.as_slice(),
    ))
    .expect("the runtime");
    assert_eq!(awaited, synchronous, "both faces answer one thing about the entries of a commit");
    assert!(!differs(awaited_repo.as_path(), awaited_second.as_str()), "the awaited reset installs the commit's entries inside the runtime");
    remove(awaited_root.as_path());
}

/// What:
///  The derived names of a batch are the bytes the synchronous face reads,
///  for a name that is not text and for a
///  branch that did not exist,
///  and the awaited face reads them inside the runtime.
/// Why:
///  The names are the pathspecs of the index update,
/// so a face that read another list would leave another set of
///  entries behind;
/// the empty old value is the root commit's arm,
/// which lists the whole tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(await derivedNamesAwaiting(git, repo, old, new), derivedNames(git, repo, old, new));
/// ```
#[test]
fn the_awaited_derived_names_are_the_ones_the_synchronous_face_reads() {
    let names: Vec<Vec<u8>> = vec![b"not-utf8-\xff\xfe.txt".to_vec(), b"plain.txt".to_vec()];
    let (root, repo, first, second) = index_behind("follow-awaiting-derived", names.as_slice());
    let zero: String = "0".repeat(first.len());
    for (label, old) in [("a branch that moved", first.as_str()), ("a branch that did not exist", zero.as_str())] {
        let synchronous: Vec<u8> = derived_names(&git_context(), repo.as_path(), old, second.as_str()).expect("the names the batch changed");
        let awaited: Vec<u8> = crate::lazy_runtime::run_on_runtime(derived_names_awaiting(&git_context(), repo.as_path(), old, second.as_str()))
            .expect("the runtime")
            .expect("the names the batch changed");
        assert_eq!(awaited, synchronous, "{label}: the two faces read one list of names");
        assert!(!synchronous.is_empty(), "{label}: the batch changed paths, so the list names them");
    }
    remove(root.as_path());
}

/// What:
///  The failure a refused reset stands for is the one the synchronous reset stands for.
/// Why:
///  The code is what the client is told (a repository that is full,
/// an index another program holds),
/// so a face that
///  classified the same refusal differently tells the person about another problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = await resetEntriesAwaiting(contextWithMissingGit, repo, index, commit, names); assert.deepEqual(refused, resetEntries(contextWithMissingGit, ...));
/// ```
#[test]
fn a_refused_reset_is_classified_as_the_synchronous_face_classifies_it() {
    let names: Vec<Vec<u8>> = vec![b"plain.txt".to_vec()];
    let (root, repo, _first, second) = index_behind("follow-awaiting-refused", names.as_slice());
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent-git-for-a-control");
    let synchronous: Result<(), FailureCode> = reset_entries(&context, repo.as_path(), Some(repo.join(".git/index").as_path()), second.as_str(), names.as_slice());
    let awaited: Result<(), FailureCode> =
        crate::lazy_runtime::run_on_runtime(reset_entries_awaiting(&context, repo.as_path(), Some(repo.join(".git/index").as_path()), second.as_str(), names.as_slice()))
            .expect("the runtime");
    assert_eq!(synchronous, Err(FailureCode::Unavailable), "a Git that cannot be run is a step that cannot run");
    assert_eq!(awaited, synchronous, "both faces classify a step that could not run the same way");
    assert!(!git_output(repo.as_path(), &["diff", "--cached", "--quiet", second.as_str()]).status.success(), "nothing was installed by a step that could not run");
    remove(root.as_path());
}
