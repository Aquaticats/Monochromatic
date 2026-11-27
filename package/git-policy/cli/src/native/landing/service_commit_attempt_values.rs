//! What:
//!  The values one commit command carries from the first attempt of its loop to the last:
//!  the address it reaches the service at,
//!  the identity of the request,
//!  what the person's line asked for,
//!  and the records a rebuild is made from.
//! Why:
//!  The commit command runs its attempts in one loop,
//!  and every attempt hands the next one what it learned:
//!  the message a moved branch made it keep,
//!  the line a rebuild reads,
//!  the captured change the next preparation replays,
//!  and the commits Git built that the branch moving made useless.
//!  They stand in one record instead of a dozen bindings beside the loop,
//!  so a step that reads or writes them says so in its signature,
//!  and the file that holds the order the steps run in stays short.
//!  It sits in a file of its own because the commit command's file was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const the: TheAttempt = theAttempt(invocation, timing);
//! ```

use crate::commit_timing::CommitTiming;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionStop,
    failed,
};
use crate::random_id::random_uuid;
use crate::service_amend_replayed::AmendedCommit;
use crate::service_client::ClientConfig;
use crate::service_commit_args::opens_editor;
use crate::service_commit_input_notes_and_embedded_commit::{
    Embedded,
    arguments_after_subcommand,
    managed_client_config,
    route_for_this_worktree,
    standard_input_for,
};
use crate::service_provision::ProvisionHost;
use crate::service_replay::Captured;
use crate::service_server::Bind;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

/// The values one commit command carries from the first attempt of its loop to the last.
pub(crate) struct TheAttempt {
    /// The address this command reaches the service at.
    pub(crate) address: Bind,
    /// The host this command is to provision the service on,
    /// when none answers.
    pub(crate) provisioning: Option<ProvisionHost>,
    /// The arguments after the subcommand,
    /// which every reader of the person's line asks for.
    pub(crate) after: Vec<OsString>,
    /// The identity of this commit,
    /// which joins its timing record to the service's journal entry for it.
    pub(crate) id: String,
    /// The bytes this command was given on its standard input.
    pub(crate) standard_input: Option<Vec<u8>>,
    /// Whether this command has asked for the service's socket activation yet.
    pub(crate) provisioned: bool,
    /// Whether an editor may have run,
    /// which decides what a moved branch costs the person:
    /// a message to keep or a signature to redo.
    pub(crate) editor_possible: bool,
    /// Whether this line asks for the one fixup whose commit Git makes from the branch and not from the index:
    /// such a commit is empty by design,
    /// and the commit a moved branch makes of it is written from the fields Git stored.
    pub(crate) rewords: bool,
    /// The client configuration of this command,
    /// which keeps the address for the build that asks the service for a finished shadow before anything is created.
    pub(crate) config: ClientConfig,
    /// The commit the command line spells,
    /// once the preparation has decided Git may not build it.
    pub(crate) embedded: Option<Embedded>,
    /// Whether the command line has been read into that commit yet.
    pub(crate) decided: bool,
    /// The message the person typed once the editor has run,
    /// kept in a file so a commit moved by a tip that changed is built again from it.
    pub(crate) fixed_message: Option<PathBuf>,
    /// The command line that rebuild reads,
    /// made once from what the preparation captured;
    /// nothing while no rebuild is due.
    pub(crate) reissued_line: Option<Vec<OsString>>,
    /// The message of the command as the last preparation settled it,
    /// kept for the person when the commit cannot be made.
    pub(crate) last_message: Option<String>,
    /// What the previous preparation captured when the service sent the commit back because the branch moved:
    /// the next preparation merges it onto the branch as it is and never reads the worktree again.
    pub(crate) retry: Option<Captured>,
    /// The branch (or `HEAD`) the first preparation found,
    /// which every later preparation must find again.
    pub(crate) first_reference: Option<crate::ref_name::RefName>,
    /// The commits Git built that the branch moving made useless:
    /// Git printed a summary for each,
    /// so the wrapper names them once the commit that did land is known.
    pub(crate) unlanded: Vec<String>,
    /// The fields of the commit the wrapper writes itself instead of building the line again,
    /// kept from the attempt that prepared it.
    /// An amend whose branch moved lands them as a new commit on the branch as it is now
    /// (`service_amend_replayed.rs`),
    /// because `git commit --amend` would replace the commit that moved the branch;
    /// a `--fixup=reword:` commit lands them on the branch's own tree,
    /// because Git takes the index out of that command's commit and running the line again would ask the person for the
    /// message a second time.
    pub(crate) amended: Option<AmendedCommit>,
    /// The first build's start:
    /// the ceiling on rebuilding is counted from it.
    pub(crate) first_build_began: Instant,
}

/// What:
///  Reads what one commit command starts with,
///  before its first attempt is prepared.
/// Why:
///  The route to the service,
/// the identity of the request and the client configuration are the same for every attempt,
///  and the records a rebuild is made from start empty.
///  A route that finds no service,
///  and an identity that cannot be made,
///  end the command here,
///  before anything was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const the = theAttempt(invocation, timing); // throws TransactionStop
/// ```
///
/// # Errors
///
/// `TransactionStop` when no service can be reached or started,
/// and when an identity for this commit cannot be made.
pub(crate) fn the_attempt(
    invocation: &CommitInvocation<'_>,
    timing: &mut CommitTiming,
) -> Result<TheAttempt, TransactionStop> {
    let (address, provisioning): (Bind, Option<ProvisionHost>) =
        route_for_this_worktree(invocation, timing)?;
    // The line's own arguments are copied once here, so every later step reads them from the record this command carries
    //  instead of borrowing them from the invocation.
    let after: Vec<OsString> =
        arguments_after_subcommand(invocation.arguments, invocation.subcommand_index).to_vec();
    let id: String = match random_uuid() {
        Ok(made) => made,
        Err(error) => {
            return Err(failed(format!(
                "an identity for this commit could not be made: {error:?}"
            )));
        }
    };
    // The identity is the key that joins this commit's timing record to the service's journal entry for it.
    timing.request = Some(id.clone());
    let standard_input: Option<Vec<u8>> = standard_input_for(invocation)?;
    // The address is kept for the build, which asks the service for a finished shadow before anything is created.
    let config: ClientConfig = managed_client_config(address.clone(), provisioning.as_ref());
    let editor_possible: bool = opens_editor(after.as_slice());
    let rewords: bool = crate::service_commit_line_reissued::rewords_a_commit(
        invocation.arguments,
        invocation.subcommand_index,
    );
    return Ok(TheAttempt {
        address,
        provisioning,
        after,
        id,
        standard_input,
        provisioned: false,
        editor_possible,
        rewords,
        config,
        embedded: None,
        decided: false,
        fixed_message: None,
        reissued_line: None,
        last_message: None,
        retry: None,
        first_reference: None,
        unlanded: Vec::new(),
        amended: None,
        first_build_began: Instant::now(),
    });
}
