//! Controls for the awaited read of the branch:
//! it answers what the synchronous read answers,
//! where the synchronous read cannot ask Git at
//!  all,
//! and it keeps the honest "not known" for both.

use super::*;
use crate::test_support::{fixture, git_context, remove, repository};
use std::path::PathBuf;

/// What:
///  The answers one read of the branch gives through both faces,
///  so a control compares them without asking which is which.
/// Why:
///  The awaited face must answer what the synchronous one answers;
/// a difference here is a silence message that
///  claims another thing about the person's commit once the command is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [sync, awaited] = bothAnswers(context, repo, reference, prepared);
/// ```
fn both_answers(context: &GitContext, repo: &Path, reference: &crate::ref_name::RefName, prepared: &str) -> (WhatTheBranchShows, WhatTheBranchShows) {
    // The read takes the bytes Git printed for the name, so the control's text becomes one of those here.
    let reference: crate::ref_name::RefName = crate::ref_name::RefName::from_text(reference);
    let synchronous: WhatTheBranchShows = what_the_branch_shows(context, repo, &reference, prepared);
    let awaited: WhatTheBranchShows =
        crate::lazy_runtime::run_on_runtime(what_the_branch_shows_awaiting(context, repo, &reference, prepared)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  Both faces answer one thing about a commit the branch holds,
///  about a commit it does not hold and about a commit no
///  object store has,
///  and the awaited face asks Git inside the runtime,
/// where the synchronous face cannot start it at all.
/// Why:
///  The three answers tell "made",
/// "not made" and "not known" apart,
/// and the silence message is what the person reads:
/// a face that
///  folded any two of them together reports a made commit as unknown or an unknown one as unmade.
/// The commit the branch
///  holds is what pins that the awaited child ran and was awaited rather than abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const prepared of [made, elsewhere, missing]) assert.equal(...bothAnswers(prepared));
/// assert.equal((await bothAnswers(made))[1], 'made');
/// ```
#[test]
fn the_awaited_branch_read_answers_what_the_synchronous_one_answers() {
    let root: PathBuf = fixture("service-branch-shows-awaiting");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let made: String = crate::test_support::git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let elsewhere: String = crate::test_support::git_text(repo.as_path(), &["commit-tree", "-m", "elsewhere", "HEAD^{tree}"]).trim().to_string();
    let cases: [(&str, &str, WhatTheBranchShows); 3] = [
        ("a commit the branch holds", made.as_str(), WhatTheBranchShows::Made),
        ("a commit the branch does not hold", elsewhere.as_str(), WhatTheBranchShows::NotMade),
        ("a commit no object store has", "0000000000000000000000000000000000000000", WhatTheBranchShows::Unread),
    ];
    let main: crate::ref_name::RefName = crate::ref_name::RefName::from_text("refs/heads/main");
    for (label, prepared, expected) in cases {
        let (synchronous, awaited) = both_answers(&context, repo.as_path(), &main, prepared);
        assert_eq!(synchronous, expected, "{label}: the synchronous read is what the two faces are compared on");
        assert_eq!(awaited, synchronous, "{label}: the two faces answer one thing about the branch");
    }
    remove(root.as_path());
}

/// What:
///  The failure a silent service is reported with is the one the synchronous face builds,
///  for a branch that holds
///  the commit,
///  one that does not,
///  and a request that left no prepared commit at all.
/// Why:
///  The failure is the words the person reads after a silence,
/// so the awaited face must word it the same;
/// a branch that
///  holds the commit is no failure at all (the owner's rule is that commits never fail),
/// and that arm is the one a
///  face that could not read the branch would get wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const prepared of [made, elsewhere, undefined]) assert.deepEqual(await theSilentServiceAfterAPrepareAwaiting(...), theSilentServiceAfterAPrepare(...));
/// ```
#[test]
fn the_awaited_silence_reports_what_the_synchronous_one_reports() {
    let root: PathBuf = fixture("service-branch-shows-awaiting-silence");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let made: String = crate::test_support::git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let elsewhere: String = crate::test_support::git_text(repo.as_path(), &["commit-tree", "-m", "elsewhere", "HEAD^{tree}"]).trim().to_string();
    let cases: [(&str, Option<&str>, Option<ClientFailure>); 3] = [
        ("a branch that holds the commit", Some(made.as_str()), None),
        (
            "a branch that does not hold the commit",
            Some(elsewhere.as_str()),
            Some(ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::NotMade }),
        ),
        ("a request with no prepared commit", None, Some(ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::Unread })),
    ];
    let main: crate::ref_name::RefName = crate::ref_name::RefName::from_text("refs/heads/main");
    for (label, prepared, expected) in cases {
        let synchronous: Option<ClientFailure> = the_silent_service_after_a_prepare(&context, repo.as_path(), &main, prepared);
        let awaited: Option<ClientFailure> = crate::lazy_runtime::run_on_runtime(the_silent_service_after_a_prepare_awaiting(
            &context,
            repo.as_path(),
            &main,
            prepared,
        ))
        .expect("the runtime");
        assert_eq!(synchronous, expected, "{label}: the synchronous report is what the two faces are compared on");
        assert_eq!(awaited, synchronous, "{label}: the two faces report one silence the same way");
    }
    remove(root.as_path());
}
