//! What:
//!  The facts one audit line carries about a deleted branch:
//!  its name,
//!  the commit it pointed at,
//!  and
//!       whether that commit is reachable from the `HEAD` the command ran beside.
//! Why:
//!  The owner chose "Allow,
//! add audit line" for `git branch -D` from the main worktree,
//!  so the line has to
//!       carry enough to restore the branch:
//!  a name plus the commit is exactly `git branch <name> <commit>`.
//!  The merged answer is one
//!       `git merge-base --is-ancestor` call,
//!  and it tells a reader whether the commit was already safe elsewhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // branchFacts({ context, name }) => [{ name: 'branch', value: 'gone' }, { name: 'commit', value: 'abc...' }]
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::git_metadata::{MetadataOutput, run_metadata_git};
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes,
///  `&OsStr` its borrowed form and `Path` a borrowed path.
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What:
///  The global options a query of this repository carries:
///  a `-C` naming the worktree when one is known,
///  else the
///       invocation's own options,
///  which already select it.
/// Why:
///   The audit reads the repository the command targets,
///  never the directory the wrapper happens to run in,
///  and
///       no second `-C` is put in front of one the invocation gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prefix = root ? ['-C', root] : context.globalPrefix;
/// ```
fn prefix_for(context: &GitContext, root: Option<&Path>) -> Vec<OsString> {
    let Some(directory) = root else {
        return context.global_prefix.clone();
    };
    return vec![
        OsString::from("-C"),
        directory.as_os_str().to_os_string(),
    ];
}

/// What:
///  The commit one local branch points at,
/// or nothing when there is no such branch.
/// Why:
///   A deletion of a name that does not exist deletes nothing,
///  and Git answers that itself;
///  the record
///       then has no branch to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit([...global, 'rev-parse', '--verify', '--quiet', `refs/heads/${name}`]);
/// ```
pub fn branch_commit(context: &GitContext, root: Option<&Path>, name: &OsStr) -> Option<String> {
    let mut reference: OsString = OsString::from("refs/heads/");
    reference.push(name);
    let mut arguments: Vec<OsString> = prefix_for(context, root);
    for token in ["rev-parse", "--verify", "--quiet"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(reference);
    let output: MetadataOutput =
        run_metadata_git(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice()).ok()?;
    if !output.success {
        return None;
    }
    let commit: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    if commit.is_empty() {
        return None;
    }
    return Some(commit);
}

/// What:
///  Whether a commit is an ancestor of the command's `HEAD`,
/// which is what "merged" means to Git's own `git branch -d`.
///  `Option<bool>` is "the answer,
/// or nothing when Git cannot say".
/// Why:
///   A merged branch can be brought back from the branch it is merged into,
/// and an unmerged one cannot,
///  so
///       this is the one word that says how much the deletion costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit([...global, 'merge-base', '--is-ancestor', commit, 'HEAD']);
/// ```
pub fn merged_into_head(context: &GitContext, root: Option<&Path>, commit: &str) -> Option<bool> {
    let mut arguments: Vec<OsString> = prefix_for(context, root);
    for token in ["merge-base", "--is-ancestor"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(OsString::from(commit));
    arguments.push(OsString::from("HEAD"));
    let output: MetadataOutput =
        run_metadata_git(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice()).ok()?;
    // 0 is "yes", 1 is "no", and anything else is a repository Git could not read, which is no answer.
    return match output.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        Some(_) | None => None,
    };
}

/// What:
///  The facts one record of a deleted branch carries,
/// best effort,
/// one record per branch.
/// Why:
///   `git branch -D one two` deletes two refs,
///  and each needs its own commit if either is to be restored,
///  so
///       each gets its own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deletionFacts({ context, name }): [string, AuditFact[]] | undefined;
/// ```
pub fn deletion_facts(context: &GitContext, root: Option<&Path>, name: &OsStr) -> Option<Vec<AuditFact>> {
    let commit: String = branch_commit(context, root, name)?;
    let mut facts: Vec<AuditFact> = vec![
        AuditFact::Text("branch", name.to_string_lossy().into_owned()),
        AuditFact::Text("commit", commit.clone()),
    ];
    if let Some(merged) = merged_into_head(context, root, commit.as_str()) {
        facts.push(AuditFact::Flag("merged", merged));
    }
    return Some(facts);
}

/// What:
///  Restore advice a person can run,
///  as one text fact,
///  so a reader of the line does not have to look up the command.
/// Why:
///   The whole point of the line is that the branch comes back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git branch ${name} ${commit}`
/// ```
pub fn restore_command(name: &OsStr, commit: &str) -> String {
    return format!("git branch {} {commit}", name.to_string_lossy());
}

/// The facts and their real-Git controls stay out of the release executable.
#[cfg(test)]
#[path = "branch_delete_audit_tests.rs"]
mod tests;
