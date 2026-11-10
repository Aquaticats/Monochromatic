//! What:
//!  The pieces of a service commit that need no loop:
//! the request for a commit Git built,
//! the request for a commit the service
//!  built from the embedded message,
//! and what a landed commit leaves for the person.
//! Why:
//!  `service_commit.rs` holds the retry loop;
//! these steps take what the loop knows as parameters and use nothing from it,
//! so
//!  they are a file the loop uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! requestForGitBuilt(...); requestForEmbedded(...); reportLanding(...)
//! ```

/// The marker lines a committed file still holds,
/// and the line about an operation left open.
use crate::conflict_markers::{open_operation_line, paths_with_markers};
/// The commit an amend makes when the branch moved under it.
use crate::service_amend_replayed::AmendedCommit;
/// The kind of operation a commit concludes.
use crate::transaction_journal::Conclusion;
use crate::commit_transaction::{CommitInvocation, TransactionStop};
use crate::landed_summary::{amend_lands_as_new_note, rebuilt_note, short_name, subject_of, summary_line};
use crate::service_prepare::{NativeInputs, Ready};
use crate::service_protocol::CommitRequest;
use crate::worktree_completion::install_added_worktree_files;
use std::io::Write;
use std::path::PathBuf;
use crate::service_commit_input_notes_and_embedded_commit::{Embedded, print_note, print_summary};

/// What:
///  The request for a commit that Git built in the shadow,
/// or that the wrapper wrote whole:
/// the built commit is named whole and the service lands it.
/// Why:
///  The service moves the branch to a commit it did not make,
/// so the request carries no changes and no message,
/// and `amend` says whether the commit replaces the branch's last commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const requestForGitBuilt = (ready, inputs, id, builtCommit, amend) => ({ id, ...identity(ready, inputs), changes: [], commit: builtCommit, amend, cleanup: ready.cleanup });
/// ```
pub(crate) fn request_for_git_built(ready: &Ready, native_inputs: &NativeInputs, id: &str, built_commit: String, amend: bool) -> CommitRequest {
    return CommitRequest {
        id: id.to_string(),
        repository: ready.repository.clone(),
        common_dir: ready.common_dir.clone(),
        git_dir: native_inputs.capture.layout.git_dir.clone(),
        reference: ready.reference.clone(),
        base: ready.base.clone(),
        // The service names the index entries from the commit itself, so no list of changes travels.
        changes: Vec::new(),
        // The commit is Git's whole: message, identities and tree are in it.
        message: String::new(),
        author: None,
        committer: String::new(),
        amend,
        tree: None,
        commit: Some(built_commit),
        cleanup: ready.cleanup.clone(),
        index: native_inputs.capture.layout.real_index_path.clone(),
        scratch: ready.scratch.0.clone(),
        captured: ready.captured_at,
    };
}

/// What:
///  The request for a commit the service builds from the changes and the message embedded in the command line.
/// Why:
///  The service builds the commit itself,
/// so the request carries the changes,
/// the message and the identities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const requestForEmbedded = (ready, inputs, id, made, tree) => ({ id, ...identity(ready, inputs), changes: ready.changes, message: made.message });
/// ```
pub(crate) fn request_for_embedded(ready: &Ready, native_inputs: &NativeInputs, id: &str, made: &Embedded, tree: &Option<String>) -> CommitRequest {
    return CommitRequest {
        id: id.to_string(),
        repository: ready.repository.clone(),
        common_dir: ready.common_dir.clone(),
        git_dir: native_inputs.capture.layout.git_dir.clone(),
        reference: ready.reference.clone(),
        base: ready.base.clone(),
        changes: ready.changes.clone().unwrap_or_default(),
        message: made.message.clone(),
        author: Some(made.author.clone()),
        committer: made.committer.clone(),
        amend: tree.is_some(),
        tree: tree.clone(),
        commit: None,
        cleanup: None,
        index: native_inputs.capture.layout.real_index_path.clone(),
        scratch: ready.scratch.0.clone(),
        captured: ready.captured_at,
    };
}

/// What:
///  What a landed commit leaves for the person:
/// the saved message file removed,
/// the summary line of a commit the service built,
///  the note about builds that did not land,
/// the warnings about conflict markers that were committed,
/// the line about an open
///  merge or cherry-pick,
/// and the files a policy added in the worktree.
/// Why:
///  They follow the landing and change nothing the landing decided,
/// so the commit loop hands them to one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportLanding(invocation, ready, nativeInputs, commit, firstParent, fixedMessage, embeddedInUse, amendedAsNew, unlanded)
/// ```
pub(crate) fn report_landing(
    invocation: &CommitInvocation<'_>,
    ready: &Ready,
    native_inputs: &NativeInputs,
    commit: &str,
    first_parent: &str,
    fixed_message: Option<&PathBuf>,
    embedded_in_use: Option<&Embedded>,
    amended_as_new: Option<&AmendedCommit>,
    unlanded: &[String],
) -> Result<(), TransactionStop> {
    if let Some(file) = fixed_message {
        // The commit was made from the saved message; the file has done its work.
        let _removed = std::fs::remove_file(file);
    }
    // A commit the service built itself has had no `git commit` to print its summary, so one line is printed once the
    //  branch moved, and it names the commit that landed. A commit Git built was printed by Git, as Git prints it, with
    //  the options the person typed and no option of the wrapper's.
    if let Some(made) = embedded_in_use
        && !made.quiet
    {
        let name: String = short_name(
            invocation.context,
            invocation.cwd,
            native_inputs.capture.layout.object_directory.as_path(),
            native_inputs.capture.layout.config_snapshot.as_ref(),
            commit,
        );
        print_summary(summary_line(ready.reference.as_str(), first_parent, name.as_str(), subject_of(made.message.as_str()).as_str()).as_str());
    }
    // A commit the wrapper wrote itself, from the fields of the commit the person's line prepared: its own summary line
    //  (unless `--quiet`), and, for an amend, the words that say what it became and that the commit it named was left as it is.
    //  A `--fixup=reword:` commit names no amendment, so it says nothing more than its summary.
    if let Some(amended) = amended_as_new {
        let name: String = short_name(
            invocation.context,
            invocation.cwd,
            native_inputs.capture.layout.object_directory.as_path(),
            native_inputs.capture.layout.config_snapshot.as_ref(),
            commit,
        );
        if !amended.quiet() {
            print_summary(summary_line(ready.reference.as_str(), first_parent, name.as_str(), amended.subject().as_str()).as_str());
        }
        if invocation.region.amend {
            print_note(amend_lands_as_new_note(ready.reference.trim_start_matches("refs/heads/"), name.as_str()).as_str());
        }
    }
    // A commit Git built again printed a summary for each build; one line names the builds that did not land and the one that
    //  did, whichever of the two the wrapper then wrote the landed commit from. A commit the wrapper wrote from the fields of
    //  such a build (`--fixup=reword:` on a moved branch) names its own build that way too.
    if !unlanded.is_empty() {
        let mut lost: Vec<String> = Vec::new();
        for full in unlanded {
            lost.push(abbreviated_name(invocation, native_inputs, full.as_str()));
        }
        print_note(rebuilt_note(lost.as_slice(), abbreviated_name(invocation, native_inputs, commit).as_str()).as_str());
    }
    // The owner: "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will be committed."
    //  The claim is made for a path whose committed blob holds marker lines, read from the blob and the marker size
    //  the attributes give; a path whose blob holds none is resolved, and nothing is said about markers for it.
    for path in paths_with_markers(invocation.context, ready.repository.as_path(), commit, ready.committed_conflicted.as_slice()) {
        let name: String = String::from_utf8_lossy(path.as_slice()).into_owned();
        // Terminal output error: a warning is advisory; a failed write must not fail a commit that landed.
        let _warned = writeln!(
            std::io::stderr(),
            "monochromatic-git-policy-cli: warning: {name} still has conflict markers, and they were committed. To fix it, remove the \
             markers from {name} and make a follow-up commit: git commit -m \"<message>\" -- {name}"
        );
    }
    // A commit of named paths leaves the merge or cherry-pick open, whether or not markers were committed.
    if let Some(kind) = ready.leaves_open {
        let _noted = write!(std::io::stderr(), "{}", open_operation_line(kind == Conclusion::Merge));
    }
    // The corrections a policy made reach the worktree once the commit has landed, as the earlier landing did.
    install_added_worktree_files(
        invocation.context,
        invocation.cwd,
        ready.repository.as_path(),
        ready.worktree_records.as_slice(),
        Some(native_inputs.capture.layout.object_directory.as_path()),
    )
    .map_err(message_stop)?;
    return Ok(());
}

/// What:
///  A failure with a message as the stop that ends the commit with that message.
/// Why:
///  `map_err` hands over the failure itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const messageStop = (error: { 0: string }) => new TransactionStop(error[0]);
/// ```
pub(crate) fn message_stop(error: crate::recovery_error::RecoveryError) -> TransactionStop {
    return TransactionStop::Message(error.0);
}

/// What:
///  The name Git gives a commit in this repository as it is now,
/// as short as Git writes it.
/// Why:
///  The notes after a landing name commits by their short names,
/// asked of Git once per name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const abbreviatedName = (full: string) => shortName(context, cwd, objectDirectory, config, full);
/// ```
pub(crate) fn abbreviated_name(invocation: &CommitInvocation<'_>, native_inputs: &NativeInputs, full: &str) -> String {
    return short_name(
        invocation.context,
        invocation.cwd,
        native_inputs.capture.layout.object_directory.as_path(),
        native_inputs.capture.layout.config_snapshot.as_ref(),
        full,
    );
}
