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
    let at_the_commit: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), "refs/heads/main", made.as_str());
    let not_at_the_commit: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), "refs/heads/main", "0000000000000000000000000000000000000000");
    let no_such_branch: WhatTheBranchShows = what_the_branch_shows(&context, root.as_path(), "refs/heads/absent", other.as_str());
    crate::test_support::remove(root.as_path());
    assert_eq!(at_the_commit, WhatTheBranchShows::Made, "the branch is at the commit this command prepared");
    assert_eq!(not_at_the_commit, WhatTheBranchShows::NotMade, "the branch is not at that commit");
    assert_eq!(no_such_branch, WhatTheBranchShows::Unread, "a branch that cannot be read is not a fact either way");
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
