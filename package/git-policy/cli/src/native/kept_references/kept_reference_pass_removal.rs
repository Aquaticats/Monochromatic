//! What:
//!  The conditional removal of one kept reference,
//!  and Git's words when the reference did not name the commit the pass
//!  listed.
//! Why:
//!  A reference another command wrote between the listing and this removal
//!  names a commit this pass never judged,
//!  and that commit may be the only one its name holds,
//!  so the old value is the condition of the removal,
//!  which is what the owner's rule "never discard user data" comes to on a
//!  reference two passes may both have listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removed = await git(['update-ref', '--no-deref', '-d', name, old]); // refused when the value moved
//! ```

/// The Git context and the request one update-ref is run as.
use crate::transaction_git::{
    GitContext,
    GitRequest,
};
/// The Git start that answers with the removal's own words.
use crate::transaction_git_awaiting::run_git_awaiting;
/// The common Git directory the reference is removed from.
use std::path::Path;
/// What:
///  Remove one kept reference only while it still names the commit the pass listed,
///  and answer Git's words when it did not.
/// Why:
///  A reference another command wrote between the listing and this removal names a commit this pass
///  never judged,
///  and that commit may be the only one its name holds,
///  so the old value is the condition of the removal
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = await git(['update-ref', '--no-deref', '-d', name, old]); // refused when the value moved
/// ```
pub(super) async fn remove_only_while_it_names(
    git: &GitContext,
    common_dir: &Path,
    name: &str,
    commit: &str,
) -> Option<String> {
    let request: GitRequest = GitRequest::new(
        common_dir,
        &["update-ref", "--no-deref", "-d", name, commit],
    );
    return match run_git_awaiting(git, &request).await {
        Ok(output) if output.succeeded() => None,
        Ok(output) => Some(
            String::from_utf8_lossy(output.stderr.as_slice())
                .trim()
                .to_owned(),
        ),
        Err(error) => Some(error.to_string()),
    };
}
