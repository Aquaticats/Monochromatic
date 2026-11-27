//! What:
//!  The merge commit of an isolated merge:
//! the commit with its parents in Git's order,
//! signed as the options and the settings ask.
//! Why:
//!   `git commit-tree` signs only when it is given `-S` and never reads `commit.gpgSign`,
//! so a merge on a signed branch must not be the one unsigned commit.
//!  `merge_isolation_apply.rs` writes the merge into the checkout,
//! and this commit is a file of its own so that one stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const oid = await commitMerge({ session, shape, config, ends, message });
//! ```

/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// The settings read for the merge.
use crate::merge_isolation_run_types::MergeConfig;
/// The command-line reading,
/// including how the commit is signed.
use crate::merge_options::{
    MergeShape,
    Signing,
};
/// One Git command's output.
use crate::transaction_git::GitOutput;

/// What:
///  Create the merge commit with its parents in Git's order,
/// signing it as the options and settings ask.
/// Why:
///   `commit-tree` signs only when given `-S` and never reads `commit.gpgSign`;
/// a merge on a signed branch must not
///  be the one unsigned commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ session, shape, config, parents, tree, message }): Promise<string>;
/// ```
pub fn commit_merge(
    session: &Session,
    shape: &MergeShape,
    config: &MergeConfig,
    ends: (&str, &str, &str),
    message: Vec<u8>,
) -> Result<String, String> {
    let (head, target, tree): (&str, &str, &str) = ends;
    let signing: Option<String> = match &shape.signing {
        Signing::Never => None,
        Signing::Sign(None) => Some(String::from("-S")),
        Signing::Sign(Some(key)) => Some(format!("-S{}", String::from_utf8_lossy(key))),
        Signing::Default => {
            if config.sign_commits {
                Some(String::from("-S"))
            } else {
                None
            }
        }
    };
    let mut arguments: Vec<&str> = vec!["commit-tree", tree, "-p", head, "-p", target];
    if let Some(flag) = signing.as_deref() {
        arguments.push(flag);
    }
    let created: GitOutput = session.run_with_input(arguments.as_slice(), message)?;
    if !created.succeeded() {
        return Err(created.error_text());
    }
    return Ok(String::from_utf8_lossy(&created.stdout).trim().to_owned());
}
