//! What:
//!  The commit an amend makes on the branch as it is now:
//!  where it is written and how it is signed,
//!  the fields an amend carries into it,
//!  the two ways Git is asked to write it:
//!  whole by `hash-object`,
//!  or signed by `commit-tree -S`,
//!  the identities that go with a signed one,
//!  and the request that lands it as a commit Git built.
//! Why:
//!   `git commit --amend` would replace the commit the branch moved past,
//!  so the commit is written again from the amended commit's own fields,
//!  and a signed one has to be signed by Git with the key the person chose.
//!  They live beside the fields they read and write,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceAmendReplayedTheCommitMadeAgain.ts
//! const rebuilt = (fields, site, tree, parent): string => fields.signed ? signedCommit(...) : wholeCommit(...);
//! ```

use crate::commit_transaction::{
    TransactionStop,
    failed,
};
use crate::diagnostic_log::warn;
use crate::private_storage::write_private_file;
use crate::service_amend::split_author;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git_checked,
};
use std::ffi::OsString;
use std::path::Path;
/// The name of the file the message is written in,
/// inside the commit's own scratch directory.
const MESSAGE_FILE_NAME: &str = "amended-message";

/// What:
///  Where the amended commit is written,
/// and how it is signed.
/// Why:
///   A Git start needs the process's own settings,
/// the directory the person ran the command in,
/// a private place for the message,
///  and the signing option the command line named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BuildSite = { context, cwd, scratch, signing?: string };
/// ```
pub struct BuildSite<'a> {
    /// How to run Git.
    pub context: &'a GitContext,
    /// The directory the command was given in.
    pub cwd: &'a Path,
    /// The commit's own scratch directory,
    /// removed with the commit.
    pub scratch: &'a Path,
    /// The `-S` option the command line named,
    /// such as `-S` or `-SABCDEF`,
    /// which chooses the key of a commit made again.
    pub signing: Option<&'a str>,
}

/// What:
///  The fields an amend carries into a commit made on the branch as it is now.
/// Why:
///   An amended commit keeps the message and the authorship of the commit it replaces,
/// and a commit on the branch as it is now is written without `--amend`,
/// so those fields travel here instead of through Git's own amend.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AmendedCommit = { author, committer, message, extraHeaders, signed, quiet };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AmendedCommit {
    /// The author line,
    /// `Name <email> seconds zone`.
    pub(super) author: String,
    /// The committer line,
    /// `Name <email> seconds zone`.
    pub(super) committer: String,
    /// The message after Git's clean-up,
    /// as stored:
    /// bytes,
    /// because a commit's message is stored as the bytes Git was given.
    pub(super) message: Vec<u8>,
    /// Header lines besides tree,
    /// parents,
    /// identities and the signature (another encoding,
    /// a merge tag),
    /// in the order the commit carries them.
    pub(super) extra_headers: Vec<String>,
    /// Whether the replaced commit was signed,
    /// so the new one is signed too.
    pub(super) signed: bool,
    /// Whether `--quiet` was given,
    /// which silences the summary line of the commit that lands.
    pub(super) quiet: bool,
}

/// The fields of a commit made again,
/// and whether it carries a signature.
impl AmendedCommit {
    /// Whether the commit whose fields these are carries a signature,
    /// so a commit made again is signed again.
    pub(crate) fn is_signed(&self) -> bool {
        return self.signed;
    }
}

/// What:
///  The commit made on the branch as it is now:
/// its tree,
/// that branch as its one parent,
/// and the amended commit's own fields.
/// Why:
///  `git commit --amend` cannot make it:
/// it would replace the commit the branch moved past.
/// An unsigned commit is written whole by `git hash-object`,
/// which keeps every header line;
/// a signed one is made again by `git commit-tree -S`,
/// which signs the exact identities and message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await amended.rebuiltOnto(site, tree, parent) // the commit's object name
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the message cannot be written or Git cannot write the commit.
pub(crate) fn rebuilt_onto(
    amended: &AmendedCommit,
    site: &BuildSite<'_>,
    tree: &str,
    parent: &str,
) -> Result<String, TransactionStop> {
    let message_file = site.scratch.join(MESSAGE_FILE_NAME);
    if let Err(error) = write_private_file(message_file.as_path(), amended.message.as_slice()) {
        return Err(failed(format!(
            "writing the message of the amended commit failed: {error}"
        )));
    }
    if amended.signed {
        return signed_commit(amended, site, tree, parent, message_file.as_path());
    }
    return whole_commit(amended, site, tree, parent);
}

/// The object name in a Git reply,
/// with the line break removed.
fn reply_name(stdout: &[u8]) -> String {
    return String::from_utf8_lossy(stdout).trim().to_string();
}

/// What:
///  Writes the commit object whole,
/// every header line the amended commit carries included.
/// Why:
///   No Git primitive writes a commit with headers of its own choosing,
/// and `git hash-object -t commit -w` stores the bytes it is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git('hash-object', '-t', 'commit', '-w', '--stdin', { input: headerAndMessage })
/// ```
fn whole_commit(
    amended: &AmendedCommit,
    site: &BuildSite<'_>,
    tree: &str,
    parent: &str,
) -> Result<String, TransactionStop> {
    let mut raw: String = String::new();
    raw.push_str(format!("tree {tree}\n").as_str());
    raw.push_str(format!("parent {parent}\n").as_str());
    raw.push_str(format!("author {}\n", amended.author).as_str());
    raw.push_str(format!("committer {}\n", amended.committer).as_str());
    for line in &amended.extra_headers {
        raw.push_str(line);
        raw.push('\n');
    }
    raw.push('\n');
    let mut object: Vec<u8> = raw.into_bytes();
    object.extend_from_slice(amended.message.as_slice());
    let mut request: GitRequest =
        GitRequest::new(site.cwd, &["hash-object", "-t", "commit", "-w", "--stdin"]);
    request.input = Some(object);
    let written = run_git_checked(site.context, &request)?;
    return Ok(reply_name(written.stdout.as_slice()));
}

/// What:
///  Makes the commit again with the amended commit's identities and message,
/// and signs it.
/// Why:
///   No primitive signs an object written whole,
/// and `git commit-tree -S` reads the identities from the environment and the message from a file.
/// A header line it cannot carry is dropped,
/// so the person is told which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git('commit-tree', signing, '-p', parent, '-F', messageFile, tree, { env: identities })
/// ```
fn signed_commit(
    amended: &AmendedCommit,
    site: &BuildSite<'_>,
    tree: &str,
    parent: &str,
    message_file: &Path,
) -> Result<String, TransactionStop> {
    if !amended.extra_headers.is_empty() {
        warn(
            "serviceCommit",
            format!(
                "the amended commit carried header lines ({}) that a signed commit made again cannot keep, so the commit that lands has none of them",
                amended.extra_headers.join("; ")
            )
            .as_str(),
        );
    }
    let mut request: GitRequest = GitRequest::new(site.cwd, &["commit-tree"]);
    request
        .arguments
        .push(OsString::from(site.signing.unwrap_or("-S")));
    request.arguments.push(OsString::from("-p"));
    request.arguments.push(OsString::from(parent));
    request.arguments.push(OsString::from("-F"));
    request
        .arguments
        .push(message_file.as_os_str().to_os_string());
    request.arguments.push(OsString::from(tree));
    identity_variables(
        amended.author.as_str(),
        amended.committer.as_str(),
        &mut request,
    );
    let written = run_git_checked(site.context, &request)?;
    return Ok(reply_name(written.stdout.as_slice()));
}

/// What:
///  The author and committer a Git start reads.
/// Why:
///   `git commit-tree` takes neither identity from its arguments,
/// and both lines are already in the shape Git's variables read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// request.env.push(...identityVariables(author), ...identityVariables(committer))
/// ```
fn identity_variables(
    author: &str,
    committer: &str,
    request: &mut GitRequest,
) {
    variables_of(author, "GIT_AUTHOR", request);
    variables_of(committer, "GIT_COMMITTER", request);
}

/// Adds the three variables of one identity line.
fn variables_of(
    line: &str,
    prefix: &str,
    request: &mut GitRequest,
) {
    let Some((name, email, date)) = split_author(line) else {
        return;
    };
    request.environment.push((
        OsString::from(format!("{prefix}_NAME")),
        OsString::from(name),
    ));
    request.environment.push((
        OsString::from(format!("{prefix}_EMAIL")),
        OsString::from(email),
    ));
    request.environment.push((
        OsString::from(format!("{prefix}_DATE")),
        OsString::from(date),
    ));
}

impl AmendedCommit {
    /// Whether the summary line of this commit is printed,
    /// which `--quiet` silences.
    pub(crate) fn quiet(&self) -> bool {
        return self.quiet;
    }

    /// The subject of the message,
    /// as the summary line names the commit.
    pub(crate) fn subject(&self) -> String {
        return crate::landed_summary::subject_of(
            String::from_utf8_lossy(self.message.as_slice()).as_ref(),
        );
    }

    /// The author line,
    /// `Name <email> seconds zone`,
    /// which a commit made again reads to supply the authorship the person kept.
    pub(crate) fn authored_by(&self) -> &str {
        return self.author.as_str();
    }
}

/// What:
///  The `-S` option the command line named,
/// nothing when it named none.
/// Why:
///   A commit made again is signed with the key the person chose,
/// and `git commit-tree` takes `-S` and `-S<key>` in the same spelling the command line uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arguments.find((token) => token.startsWith('-S') && !token.startsWith('--'))
/// ```
pub(crate) fn signing_option(arguments: &[OsString]) -> Option<&str> {
    for token in arguments {
        let Some(text) = token.to_str() else { continue };
        if text.starts_with("-S") && !text.starts_with("--") {
            return Some(text);
        }
    }
    return None;
}
