//! What:
//!  The request one attempt of a commit sends to the service:
//!  the amended commit landing as a new one,
//! the commit Git builds,
//! or the
//!  commit the service builds from the changes the preparation captured.
//! Why:
//!  The commit loop reached its line bound,
//! and this decision is the one stretch of it that reads only values the loop already
//!  holds,
//! so it stands in a file of its own and the loop calls it.
//!  It also reports the message Git typed into the commit it built,
//!  which the loop keeps for the person when the branch moves and the commit cannot be made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { sent, firstParent, typed } = theRequestOf({ amendment, embeddedInUse, tree, ready, invocation, nativeInputs, fixedMessage, id });
//! ```

/// The commit's request shape.
use crate::service_protocol::CommitRequest;
/// The commit the service builds from captured changes,
/// and the one Git built.
use crate::service_commit_requests_and_landing_report::{request_for_embedded, request_for_git_built};
/// The amended commit of an amend whose branch moved.
use crate::service_amend_replayed::{AmendedCommit, the_amendment_lands_as_new};
/// The arguments a rebuilt commit is started with.
use crate::service_commit_args::with_fixed_message;
/// The message embedded in the command line,
/// and the arguments after the subcommand.
use crate::service_commit_input_notes_and_embedded_commit::{Embedded, arguments_after_subcommand};
/// The commit Git builds in the scratch repository.
use crate::service_native::{BuildInput, GitBuilt, build_with_git};
/// What a preparation handed back.
use crate::service_prepare::{NativeInputs, Ready};
/// How a stopped transaction ends.
use crate::commit_transaction::{CommitInvocation, TransactionStop};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The values the request depends on,
///  named once so the call site stays short.
/// Why:
///   Every one of them belongs to the attempt that is being sent,
/// so passing them as one object keeps the loop readable and the borrows plain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestOf = { amendment, embeddedInUse, tree, ready, invocation, nativeInputs, fixedMessage, id };
/// ```
pub(crate) struct RequestOf<'a> {
    /// The amended commit,
    /// when this attempt is an amend that lands as a new commit.
    pub amendment: Option<&'a AmendedCommit>,
    /// The commit embedded in the command line,
    /// when the service builds it.
    pub embedded_in_use: Option<&'a Embedded>,
    /// The tree the embedded commit lands,
    /// when one was written.
    pub tree: &'a Option<String>,
    /// What the preparation handed back.
    pub ready: &'a Ready,
    /// The invocation whose command line names the arguments and the repository.
    pub invocation: &'a CommitInvocation<'a>,
    /// The private index,
    /// the capture and the arguments of this attempt.
    pub native_inputs: &'a NativeInputs,
    /// The file the person's message was kept in after an editor ran.
    pub fixed_message: Option<&'a Path>,
    /// The identity this commit's requests carry.
    pub id: &'a str,
}

/// What:
///  The request this attempt sends,
///  the first parent of the commit it lands,
///  and the message Git typed into the commit it built.
/// Why:
///   The three routes end in one of two requests:
/// a commit named whole,
/// which Git built or which the amended commit became,
///  and a commit the service builds from the changes and the message.
///  The message is reported because the loop keeps it for the person when the branch moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theRequestOf(fields: RequestOf): [CommitRequest, string, string?];
/// ```
pub(crate) fn the_request_of(fields: RequestOf<'_>) -> Result<(CommitRequest, String, Option<String>), TransactionStop> {
    let RequestOf { amendment, embedded_in_use, tree, ready, invocation, native_inputs, fixed_message, id } = fields;
    if let Some(found) = amendment {
        // The amended commit's own fields, on the tree the replay settled, with the branch as it is now as the single
        //  parent.
        let (sent, parent, subject) =
            the_amendment_lands_as_new(found, ready, invocation, native_inputs, id, arguments_after_subcommand(invocation.arguments, invocation.subcommand_index))?;
        return Ok((sent, parent, Some(subject)));
    }
    let Some(made) = embedded_in_use else {
        let layout_prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
        let rebuilt_arguments: Option<Vec<OsString>> = match fixed_message {
            Some(file) => with_fixed_message(native_inputs.arguments.as_slice(), layout_prefix, file),
            None => None,
        };
        let built: GitBuilt = build_with_git(&BuildInput {
            context: invocation.context,
            cwd: invocation.cwd,
            capture: &native_inputs.capture,
            scratch: ready.scratch.0.as_path(),
            commit_index: native_inputs.commit_index.as_path(),
            arguments: rebuilt_arguments.as_deref().unwrap_or(native_inputs.arguments.as_slice()),
            pathspecs: native_inputs.pathspecs.as_slice(),
            mode: native_inputs.mode,
            staged_all: native_inputs.staged_all,
            selected_privately: native_inputs.selected_privately,
        })?;
        let parent: String = built.parents.first().cloned().unwrap_or_default();
        let sent: CommitRequest = request_for_git_built(ready, native_inputs, id, built.commit, invocation.region.amend);
        return Ok((sent, parent, Some(built.message)));
    };
    let sent: CommitRequest = request_for_embedded(ready, native_inputs, id, made, tree);
    // The new commit's first parent: the tip it was made on, or for an amend the replaced commit's own parent.
    let parent: String = match made.source.as_ref() {
        Some(replaced) => replaced.parents.first().cloned().unwrap_or_default(),
        None => ready.base.clone(),
    };
    return Ok((sent, parent, None));
}
