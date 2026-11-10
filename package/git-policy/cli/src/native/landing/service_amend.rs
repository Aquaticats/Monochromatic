//! What:
//!  The facts of the commit an amend replaces,
//! read once from the repository:
//! its parents,
//! its first parent's tree,
//!  its author and its message,
//! and the author line an amend makes of them.
//! Why:
//!  An amend keeps the parents of the commit it replaces,
//! keeps its message when no message option is given and keeps
//!  its author unless the command line says otherwise.
//! All of that is arithmetic on the replaced commit's own record,
//!  so the wrapper embeds it;
//! each rule is compared field by field with native `git commit --amend` in
//!  `binary_service_amend_tests.rs`.
//! A commit with a header the amend would have to carry (another encoding,
//! a merge
//!  tag) returns nothing and native Git handles the line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = await readAmendSource(context, cwd); // undefined when native git must amend
//! ```

use crate::commit_transaction::{TransactionStop, failed};
use crate::service_commit_args::ServiceCommitArgs;
use crate::service_message::{author_overrides, identity};
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::Path;

/// What an amend reads from the commit it replaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmendSource {
    /// The commit the amend replaces,
    /// which is also the branch tip the amend must find when it lands.
    pub base: String,
    /// Its parents in order;
    /// the amended commit has the same ones.
    pub parents: Vec<String>,
    /// The tree of its first parent,
    /// or nothing for a root commit.
    pub parent_tree: Option<String>,
    /// Its author line,
    /// `Name <email> seconds zone`.
    pub author: String,
    /// Its message,
    /// exactly as stored.
    pub message: String,
}

/// The parts of a stored commit object an amend reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredCommit {
    /// The parents in order.
    pub parents: Vec<String>,
    /// The author line without its `author ` label.
    pub author: String,
    /// The message after the blank line.
    pub message: String,
}

/// What:
///  Reads a stored commit object,
/// or nothing when it has a header an amend would have to carry over.
/// Why:
///   An amend keeps extra headers (a merge tag,
/// for one) and the encoding of the commit it replaces.
/// The signature is
///  the exception:
/// native Git drops it,
/// because the amended commit is not the one that was signed.
/// Anything else is not
///  carried here,
/// so the line stays native.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseStoredCommit(raw) // undefined when an unknown header is present
/// ```
pub fn parse_stored_commit(raw: &str) -> Option<StoredCommit> {
    let (header, message) = raw.split_once("\n\n")?;
    let mut parents: Vec<String> = Vec::new();
    let mut author: Option<String> = None;
    let mut in_signature: bool = false;
    let mut has_tree: bool = false;
    for line in header.lines() {
        if in_signature && line.starts_with(' ') {
            continue;
        }
        in_signature = false;
        if line.starts_with("tree ") {
            has_tree = true;
        } else if let Some(parent) = line.strip_prefix("parent ") {
            parents.push(parent.to_string());
        } else if let Some(found) = line.strip_prefix("author ") {
            author = Some(found.to_string());
        } else if line.starts_with("committer ") {
            // The committer is made again at the amend.
        } else if line.starts_with("gpgsig ") || line.starts_with("gpgsig-sha256 ") {
            in_signature = true;
        } else {
            return None;
        }
    }
    if !has_tree {
        return None;
    }
    return Some(StoredCommit { parents, author: author?, message: message.to_string() });
}

/// The text of a Git query that must succeed for the line to be covered.
fn query(context: &GitContext, cwd: &Path, arguments: &[&str]) -> Result<Option<String>, TransactionStop> {
    let output = match run_git(context, &GitRequest::new(cwd, arguments)) {
        Ok(finished) => finished,
        Err(error) => return Err(failed(format!("reading the commit to amend failed: {error:?}"))),
    };
    return Ok(if output.code == Some(0) { String::from_utf8(output.stdout).ok() } else { None });
}

/// What:
///  Reads the commit `HEAD` names,
/// the one an amend replaces.
/// Why:
///   Resolved once,
/// before the message and the author are made,
/// so everything an amend takes from the old commit comes
///  from the same commit;
/// the preparation then checks that the branch is still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readAmendSource(context, cwd)
/// ```
pub fn read_amend_source(context: &GitContext, cwd: &Path) -> Result<Option<AmendSource>, TransactionStop> {
    let Some(named) = query(context, cwd, &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])? else { return Ok(None) };
    let base: String = named.trim().to_string();
    let Some(raw) = query(context, cwd, &["cat-file", "commit", base.as_str()])? else { return Ok(None) };
    let Some(stored) = parse_stored_commit(raw.as_str()) else { return Ok(None) };
    let parent_tree: Option<String> = match stored.parents.first() {
        Some(parent) => {
            let Some(tree) = query(context, cwd, &["rev-parse", "--verify", "--quiet", format!("{parent}^{{tree}}").as_str()])? else {
                return Ok(None);
            };
            Some(tree.trim().to_string())
        }
        None => None,
    };
    return Ok(Some(AmendSource { base, parents: stored.parents, parent_tree, author: stored.author, message: stored.message }));
}

/// The name,
/// the address and the raw date of an identity line,
/// as `git var` writes it.
/// Why:
///   An amended commit's author and committer reach `git commit-tree` as its own variables,
/// and both lines have exactly these three parts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [name, email, date] = line.match(/^(.*) <(.*)> (\d+ [+-]\d+)$/).slice(1);
/// ```
pub fn split_author(line: &str) -> Option<(String, String, String)> {
    let (name, rest) = line.rsplit_once(" <")?;
    let (email, date) = rest.split_once("> ")?;
    return Some((name.to_string(), email.to_string(), date.to_string()));
}

/// What:
///  The author line of an amended commit.
/// Why:
///   Native Git keeps the replaced commit's name,
/// address and date;
/// `--author` replaces the name and address,
/// `--date`
///  the date,
/// and `--reset-author` makes the committer the author with a new date.
/// Git's own `var` makes the line
///  whenever a part is replaced,
/// so a date or an identity is read as Git reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await amendedAuthor(context, cwd, source, parsed)
/// ```
pub fn amended_author(context: &GitContext, cwd: &Path, source: &AmendSource, parsed: &ServiceCommitArgs) -> Result<Option<String>, TransactionStop> {
    if parsed.reset_author {
        return identity(context, cwd, "GIT_AUTHOR_IDENT", &[]);
    }
    let replaced: Vec<(&'static str, String)> = author_overrides(parsed);
    if replaced.is_empty() {
        return Ok(Some(source.author.clone()));
    }
    let Some((name, email, date)) = split_author(source.author.as_str()) else { return Ok(None) };
    let mut parts: Vec<(&'static str, String)> = vec![("GIT_AUTHOR_NAME", name), ("GIT_AUTHOR_EMAIL", email), ("GIT_AUTHOR_DATE", date)];
    for (key, value) in replaced {
        for slot in parts.iter_mut() {
            if slot.0 == key {
                slot.1 = value;
                break;
            }
        }
    }
    return identity(context, cwd, "GIT_AUTHOR_IDENT", parts.as_slice());
}

/// Amend controls stay out of the release executable.
#[cfg(test)]
#[path = "service_amend_tests.rs"]
mod tests;
