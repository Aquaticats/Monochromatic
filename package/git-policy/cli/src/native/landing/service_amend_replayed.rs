//! What:
//!  The commit an amend makes when the branch moved while the amend was prepared:
//!  its own author,
//! committer,
//! message and signature,
//!  written on the tree the replay settled,
//! with the branch as it is now as its only parent.
//! Why:
//!  The owner chose exactly this (verbatim,
//! 2026-11-09):
//! "A:
//! land as a new commit (Recommended)",
//! rejecting both the refusal and a rewrite of the commit another session landed.
//! The amended tree is replayed onto the new tip by the same rule as an ordinary commit
//!  (`service_replay.rs`),
//! and the commit is written here,
//! because `git commit --amend` would replace the commit the branch moved past.
//! An unsigned commit is written whole,
//! so every header line it carried (another encoding,
//!  a merge tag) survives;
//! a signed one is signed again by `git commit-tree -S`,
//! which cannot keep those headers,
//! and the person is told.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const amended = await amendedCommit({ made }); // or storedCommitFields(raw)
//! const oid = await amended.rebuiltOnto({ context, cwd, scratch, signing }, tree, parent);
//! ```

use crate::commit_transaction::{CommitInvocation, TransactionStop, failed};
use crate::diagnostic_log::warn;
use crate::private_index_request::indexed_request;
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_prepare_ready_shapes::{NativeInputs, Ready};
use crate::service_protocol::CommitRequest;
use crate::private_storage::write_private_file;
use crate::service_amend::split_author;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::ffi::OsString;
use std::path::Path;

/// The name of the file the message is written in,
/// inside the commit's own scratch directory.
const MESSAGE_FILE_NAME: &str = "amended-message";

/// The header line that starts a signature,
/// which is dropped and made again.
const SIGNATURE_HEADERS: [&str; 2] = ["gpgsig ", "gpgsig-sha256 "];

/// The commit an amend replaces,
/// when the service built the amend itself.
/// Why:
///   The service-built amend kept the replaced commit in its fields,
/// and a command line the service did not build carries none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = embedded?.source;
/// ```
fn source_of<'a>(embedded: Option<&'a Embedded>) -> Option<&'a crate::service_amend::AmendSource> {
    let Some(made) = embedded else { return None };
    return made.source.as_ref();
}

/// The amend the service built,
/// when it replaced a commit.
/// Why:
///   Only such an amend has fields a moved branch can carry into a new commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = embedded?.source ? embedded : undefined;
/// ```
fn embedded_with_a_source<'a>(embedded: Option<&'a Embedded>) -> Option<&'a Embedded> {
    let Some(source) = source_of(embedded) else { return None };
    let _: &crate::service_amend::AmendSource = source;
    return embedded;
}

/// Whether a header line starts a signature,
/// which is dropped and made again.
/// Why:
///   The spellings a signature may use are named once,
/// and a reader of the stored object asks this question of each header line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SIGNATURE_HEADERS.some((prefix) => line.startsWith(prefix))
/// ```
fn begins_with_a_signature(line: &str) -> bool {
    for prefix in SIGNATURE_HEADERS {
        if line.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

/// Header lines an amended commit does not carry into the new one:
/// the tree and the parents are the replay's,
/// the identities and the signature are made again.
fn is_replaced_header(line: &str) -> bool {
    return line.starts_with("tree ") || line.starts_with("parent ") || line.starts_with("author ") || line.starts_with("committer ");
}

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
    author: String,
    /// The committer line,
    /// `Name <email> seconds zone`.
    committer: String,
    /// The message after Git's clean-up,
    /// as stored:
    /// bytes,
    /// because a commit's message is stored as the bytes Git was given.
    message: Vec<u8>,
    /// Header lines besides tree,
    /// parents,
    /// identities and the signature (another encoding,
    /// a merge tag),
    /// in the order the commit carries them.
    extra_headers: Vec<String>,
    /// Whether the replaced commit was signed,
    /// so the new one is signed too.
    signed: bool,
    /// Whether `--quiet` was given,
    /// which silences the summary line of the commit that lands.
    quiet: bool,
}

/// What:
///  The amended commit of a command line the service built itself.
/// Why:
///   Such a commit never carries a signature
/// (`commit.gpgsign` and `-S` make Git build the commit instead),
/// and has no header line beyond the ones the amend makes again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// amendedFromEmbedded({ author, committer, message, quiet })
/// ```
pub(crate) fn amended_from_embedded(made: &Embedded) -> AmendedCommit {
    return AmendedCommit {
        author: made.author.clone(),
        committer: made.committer.clone(),
        message: with_final_line_break(made.message.as_bytes()),
        extra_headers: Vec::new(),
        signed: false,
        quiet: made.quiet,
    };
}

/// What:
///  The message with exactly one line break at its end,
/// nothing added when it already ends with one.
/// Why:
///   A commit's message ends with a line break whatever the command line gave,
/// and a rebuild without one would store a message Git never writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bytes.at(-1) === 0x0a ? bytes : Buffer.concat([bytes, Buffer.from('\n')])
/// ```
fn with_final_line_break(bytes: &[u8]) -> Vec<u8> {
    if bytes.ends_with(b"\n") {
        return bytes.to_vec();
    }
    // `mut` appends the one line break the message lacks.
    let mut made: Vec<u8> = bytes.to_vec();
    made.push(b'\n');
    return made;
}

/// What:
///  Where the blank line that ends a commit's header block starts,
/// nothing when the bytes hold none.
/// Why:
///   The header block ends there,
/// and the message is every byte after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// raw.indexOf('\n\n')
/// ```
fn blank_line_at(raw: &[u8]) -> Option<usize> {
    for (at, window) in raw.windows(2).enumerate() {
        if window == b"\n\n" {
            return Some(at);
        }
    }
    return None;
}

/// What:
///  The amended commit of a commit Git built,
/// read from the object it wrote.
/// Why:
///   That object holds the authorship,
/// the committer,
/// the message and any header line the commit carries,
///  and it is the one record of what the amend was to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// storedCommitFields(rawCommitText) // undefined when the text is not a commit
/// ```
pub(crate) fn stored_commit_fields(raw: &[u8]) -> Option<AmendedCommit> {
    let split: usize = blank_line_at(raw)?;
    let header: &str = std::str::from_utf8(raw.get(..split)?).ok()?;
    let message: Vec<u8> = raw.get(split + 2..)?.to_vec();
    // `mut` collects the parts the header block names.
    let mut author: Option<String> = None;
    let mut committer: Option<String> = None;
    let mut extra_headers: Vec<String> = Vec::new();
    let mut signed: bool = false;
    // A signature's own lines begin with a space and belong to the signature's header.
    let mut in_signature: bool = false;
    for line in header.lines() {
        if in_signature && line.starts_with(' ') {
            continue;
        }
        in_signature = false;
        if is_replaced_header(line) {
            if let Some(found) = line.strip_prefix("author ") {
                author = Some(found.to_string());
            }
            if let Some(found) = line.strip_prefix("committer ") {
                committer = Some(found.to_string());
            }
            continue;
        }
        let signature: bool = begins_with_a_signature(line);
        if signature {
            signed = true;
            in_signature = true;
            continue;
        }
        // A merge tag's own lines begin with a space and are carried as they are.
        extra_headers.push(line.to_string());
    }
    return Some(AmendedCommit {
        author: author?,
        committer: committer?,
        message,
        extra_headers,
        signed,
        quiet: false,
    });
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
pub(crate) fn rebuilt_onto(amended: &AmendedCommit, site: &BuildSite<'_>, tree: &str, parent: &str) -> Result<String, TransactionStop> {
    let message_file = site.scratch.join(MESSAGE_FILE_NAME);
    if let Err(error) = write_private_file(message_file.as_path(), amended.message.as_slice()) {
        return Err(failed(format!("writing the message of the amended commit failed: {error}")));
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
fn whole_commit(amended: &AmendedCommit, site: &BuildSite<'_>, tree: &str, parent: &str) -> Result<String, TransactionStop> {
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
    let mut request: GitRequest = GitRequest::new(site.cwd, &["hash-object", "-t", "commit", "-w", "--stdin"]);
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
fn signed_commit(amended: &AmendedCommit, site: &BuildSite<'_>, tree: &str, parent: &str, message_file: &Path) -> Result<String, TransactionStop> {
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
    request.arguments.push(OsString::from(site.signing.unwrap_or("-S")));
    request.arguments.push(OsString::from("-p"));
    request.arguments.push(OsString::from(parent));
    request.arguments.push(OsString::from("-F"));
    request.arguments.push(message_file.as_os_str().to_os_string());
    request.arguments.push(OsString::from(tree));
    identity_variables(amended.author.as_str(), amended.committer.as_str(), &mut request);
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
fn identity_variables(author: &str, committer: &str, request: &mut GitRequest) {
    variables_of(author, "GIT_AUTHOR", request);
    variables_of(committer, "GIT_COMMITTER", request);
}

/// Adds the three variables of one identity line.
fn variables_of(line: &str, prefix: &str, request: &mut GitRequest) {
    let Some((name, email, date)) = split_author(line) else { return };
    request.environment.push((OsString::from(format!("{prefix}_NAME")), OsString::from(name)));
    request.environment.push((OsString::from(format!("{prefix}_EMAIL")), OsString::from(email)));
    request.environment.push((OsString::from(format!("{prefix}_DATE")), OsString::from(date)));
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
        return crate::landed_summary::subject_of(String::from_utf8_lossy(self.message.as_slice()).as_ref());
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

/// What the tree of an amend's commit is.
/// Why:
///   An amend the service builds carries the tree of its private index,
/// one that leaves the replaced commit as it was is Git's to refuse,
/// and a commit the service does not build as an amend carries no tree of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AmendsTree = { kind: 'tree'; tree: string } | { kind: 'empty-against-the-parent' } | { kind: 'nothing' };
/// ```
pub(crate) enum AmendsTree {
    /// The commit's tree is the one the private index holds.
    Tree(String),
    /// The amended tree equals the replaced commit's parent tree,
    /// which Git refuses as an empty amend itself.
    EmptyAgainstTheParent,
    /// No tree to write.
    Nothing,
}

/// What:
///  The tree an amend's commit records,
///  and nothing when the amend is carried by the service as it always was.
/// Why:
///   An amend the service builds is made from the commit `HEAD` named when the message was read;
/// when the branch left it,
/// the amend becomes a new commit on the branch as it is now,
///  whose tree is the replay the preparation already settled in the private index.
/// A tree equal to the replaced commit's parent tree is an amend Git refuses as empty against its first parent
/// (a merge excepted) unless `--allow-empty` says it is meant,
/// and Git then says so itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = theAmendsTree(ready, inputs, embedded, landsAsNew); // undefined when the amendment is an amend
/// ```
///
/// # Errors
///
/// `TransactionStop` when the private index cannot be written.
pub(crate) fn the_amends_tree(
    ready: &Ready,
    invocation: &CommitInvocation<'_>,
    native_inputs: &NativeInputs,
    embedded: Option<&Embedded>,
    lands_as_new: bool,
) -> Result<AmendsTree, TransactionStop> {
    if lands_as_new || ready.needs_native {
        return Ok(AmendsTree::Nothing);
    }
    let Some(source) = source_of(embedded) else { return Ok(AmendsTree::Nothing) };
    let written = run_git_checked(
        invocation.context,
        &indexed_request(
            invocation.cwd,
            &["write-tree"],
            Some(native_inputs.commit_index.as_path()),
            Some(native_inputs.capture.layout.object_directory.as_path()),
        ),
    )?;
    let written_tree: String = String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string();
    let against: &str = source.parent_tree.as_deref().unwrap_or(native_inputs.capture.layout.empty_tree_oid.as_str());
    if source.parents.len() < 2 && !invocation.region.allow_empty && written_tree == against {
        return Ok(AmendsTree::EmptyAgainstTheParent);
    }
    return Ok(AmendsTree::Tree(written_tree));
}

/// What:
///  The commit an amend makes when the branch moved,
///  as the request that lands it,
///  its first parent,
///  and the subject the summary line names.
/// Why:
///   `git commit --amend` would replace the commit the branch moved past,
/// so the commit is written from the amended commit's own fields on the tree the replay settled
/// (`rebuilt_onto`),
/// and the request lands it as a commit Git built,
/// which no longer replaces anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { sent, firstParent, subject } = theAmendmentLandsAsNew(...);
/// ```
///
/// # Errors
///
/// `TransactionStop` when the replayed tree cannot be written or the commit cannot be made.
pub(crate) fn the_amendment_lands_as_new(
    fields: &AmendedCommit,
    ready: &Ready,
    invocation: &CommitInvocation<'_>,
    native_inputs: &NativeInputs,
    id: &str,
    arguments: &[std::ffi::OsString],
) -> Result<(CommitRequest, String, String), TransactionStop> {
    let settled: String = crate::service_replay::captured_tree(
        invocation.context,
        invocation.cwd,
        native_inputs.commit_index.as_path(),
        native_inputs.capture.layout.object_directory.as_path(),
    )?;
    let site: BuildSite<'_> = BuildSite { context: invocation.context, cwd: invocation.cwd, scratch: ready.scratch.0.as_path(), signing: signing_option(arguments) };
    let made: String = rebuilt_onto(fields, &site, settled.as_str(), ready.base.as_str())?;
    let subject: String = fields.subject();
    let sent: CommitRequest = crate::service_commit_requests_and_landing_report::request_for_git_built(ready, native_inputs, id, made, false);
    return Ok((sent, ready.base.clone(), subject));
}

/// What:
///  What the amend of a request carries into the commit a moved branch makes of it.
/// Why:
///   A command line the service built the amend from kept the fields already,
/// and one Git built the amend for wrote them into the commit object,
/// which is read back here instead of being guessed from the configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fields = amended?.source ? amendedFromEmbedded(amended) : storedCommitFields(await git(['cat-file', 'commit', sent.commit]));
/// ```
///
/// # Errors
///
/// `TransactionStop` when the prepared commit cannot be read back.
pub(crate) fn amended_fields(invocation: &CommitInvocation<'_>, sent: &CommitRequest, embedded: Option<&Embedded>) -> Result<AmendedCommit, TransactionStop> {
    if let Some(made) = embedded_with_a_source(embedded) {
        return Ok(amended_from_embedded(made));
    }
    let Some(commit) = sent.commit.as_deref() else {
        return Err(failed(String::from("the amend prepared no commit whose message and authorship a later tip could keep")));
    };
    let read = run_git_checked(invocation.context, &GitRequest::new(invocation.cwd, &["cat-file", "commit", commit]))?;
    return match stored_commit_fields(read.stdout.as_slice()) {
        Some(fields) => Ok(fields),
        None => Err(failed(String::from("the commit this amend prepared could not be read back, so it cannot be made on the branch as it is now"))),
    };
}

/// Controls for the commit an amend makes stay out of the release executable.
#[cfg(test)]
#[path = "service_amend_replayed_tests.rs"]
mod tests;
