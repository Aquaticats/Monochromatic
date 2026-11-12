//! Controls for what the branch shows once the commit service went silent.

use super::*;

/// What:
///  The branch is read once before the silence message,
/// so the message states what it shows.
/// Why:
///   The 2026-11-09T07:44:20Z incident printed "it is not known whether the commit was made"
/// while the service still held the commit and landed it 73 s later.
/// One bounded read of the branch turns that into a fact and waits for nothing.
#[test]
fn what_the_branch_shows_is_read_from_the_branch() {
    let root: std::path::PathBuf = crate::test_support::repository(crate::test_support::fixture_root().as_path(), "service-commit-branch-shows");
    let made: String = crate::test_support::git_text(root.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let other: String = crate::test_support::git_text(root.as_path(), &["rev-parse", "HEAD~0"]).trim().to_string();
    let context: crate::transaction_git::GitContext = crate::test_support::git_context();
    // An object that is not there at all: Git answers 128, which is neither "yes" nor "no", so nothing is claimed.
    let at_the_commit: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), made.as_str());
    let no_such_commit: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), "0000000000000000000000000000000000000000");
    let no_such_branch: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/absent"), other.as_str());
    crate::test_support::remove(root.as_path());
    assert_eq!(at_the_commit, WhatTheBranchShows::Made, "the branch holds the commit this command prepared");
    assert_eq!(no_such_commit, WhatTheBranchShows::Unread, "a commit that is not there is not a fact either way");
    assert_eq!(no_such_branch, WhatTheBranchShows::Unread, "a branch that cannot be read is not a fact either way");
}

/// What:
///  A commit a later commit sits on top of was still made by this command.
/// Why:
///   Finding 5 of the third review:
/// the service can land the prepared commit and a concurrent command then advances the branch,
/// so the tip differs although the prepared commit is in the branch's history.
/// Tip inequality establishes only that the prepared commit is not the tip.
#[test]
fn a_commit_a_later_commit_sits_on_top_of_was_made() {
    let root: std::path::PathBuf = crate::test_support::repository(crate::test_support::fixture_root().as_path(), "service-branch-shows-on-top");
    let earlier: String = crate::test_support::git_text(root.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    crate::test_support::git(root.as_path(), &["commit", "--quiet", "--allow-empty", "--message=later"]);
    let later: String = crate::test_support::git_text(root.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let off_the_branch: String = {
        // A commit that is not on the branch at all.
        crate::test_support::git(root.as_path(), &["checkout", "--quiet", "--detach", "HEAD~1"]);
        crate::test_support::git(root.as_path(), &["commit", "--quiet", "--allow-empty", "--message=elsewhere"]);
        let made: String = crate::test_support::git_text(root.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
        crate::test_support::git(root.as_path(), &["checkout", "--quiet", "main"]);
        made
    };
    let context: crate::transaction_git::GitContext = crate::test_support::git_context();
    let on_top: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), earlier.as_str());
    let the_tip: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), later.as_str());
    let elsewhere: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), off_the_branch.as_str());
    crate::test_support::remove(root.as_path());
    assert_eq!(on_top, WhatTheBranchShows::Made, "a commit the branch's tip sits on top of was made by this command");
    assert_eq!(the_tip, WhatTheBranchShows::Made, "the tip itself is the commit this command prepared");
    assert_eq!(elsewhere, WhatTheBranchShows::NotMade, "a commit the branch does not hold was not made by this command");
}

/// The three answers say different things,
/// and the one that says the commit was made offers nothing to fix.
#[test]
fn the_silence_message_states_what_the_branch_shows() {
    let made: String = crate::service_messages::render(&crate::service_messages::ClientFailure::ServiceSilentAfterPreparing {
        shows: crate::service_messages::WhatTheBranchShows::Made,
    });
    let not_made: String = crate::service_messages::render(&crate::service_messages::ClientFailure::ServiceSilentAfterPreparing {
        shows: crate::service_messages::WhatTheBranchShows::NotMade,
    });
    let unread: String = crate::service_messages::render(&crate::service_messages::ClientFailure::ServiceSilentAfterPreparing {
        shows: crate::service_messages::WhatTheBranchShows::Unread,
    });
    assert!(made.contains("the commit was made"), "{made}");
    assert!(!made.contains("it is not known whether the commit was made"), "{made}");
    assert!(not_made.contains("it was not made") && not_made.contains("may still land"), "{not_made}");
    assert!(unread.contains("it is not known whether the commit was made"), "{unread}");
}

/// What:
///  A silence the branch shows as made is not a failure at all.
/// Why:
///   Finding 6 of the third review:
/// a commit that landed and was confirmed by the branch read still went down the failure path,
/// so the command exited nonzero and skipped the landing report and the handoff.
/// The owner's rule is that commits never fail.
#[test]
fn a_silence_the_branch_shows_as_made_is_not_a_failure() {
    let root: std::path::PathBuf = crate::test_support::repository(crate::test_support::fixture_root().as_path(), "service-branch-shows-made");
    let made: String = crate::test_support::git_text(root.as_path(), &["rev-parse", "HEAD"]).trim().to_string();
    let context: crate::transaction_git::GitContext = crate::test_support::git_context();
    let at_the_commit: Option<crate::service_messages::ClientFailure> =
        the_silent_service_after_a_prepare(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), Some(made.as_str()));
    let elsewhere: Option<crate::service_messages::ClientFailure> =
        the_silent_service_after_a_prepare(&context, root.as_path(), &crate::ref_name::RefName::from_text("refs/heads/main"), Some("0000000000000000000000000000000000000000"));
    crate::test_support::remove(root.as_path());
    assert!(at_the_commit.is_none(), "the command has nothing to report when the branch holds its commit");
    assert!(elsewhere.is_some(), "a commit the branch does not show still has to be reported");
}
