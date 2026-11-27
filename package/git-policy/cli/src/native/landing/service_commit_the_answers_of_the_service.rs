//! What:
//!  What one commit does with the answer the service gave:
//!  a landed commit and the words after it,
//!  a commit held behind a reference lock,
//!  the attempt the service sent back to be prepared again,
//!  the branch that moved under the attempt,
//!  and the failure a client met.
//! Why:
//!  Every arm is one ending of a command,
//!  and each one either keeps the person's commit or says why it did not,
//!  so the five stand together where a reader can see them all:
//!  a commit that landed,
//!  a commit that was written and kept behind a reference lock,
//!  an attempt that is prepared again,
//!  and the two ways the service says nothing landed.
//!  They sit in a file of their own because the commit command's file was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said: WhatTheServiceSaid = theAnswerOfTheService(answer)?;
//! ```

use crate::commit_timing::CommitTiming;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionResult,
    TransactionStop,
};
use crate::service_client::Submitted;
use crate::service_commit_after_a_moved_branch::{
    MovedBranch,
    after_a_moved_branch,
};
use crate::service_commit_attempt_values::TheAttempt;
use crate::service_commit_input_notes_and_embedded_commit::Embedded;
use crate::service_commit_rebuild_ceiling::{
    AfterAPrepareAgain,
    after_a_prepare_again,
};
use crate::service_landed_aftermath::{
    LandedAftermath,
    ServiceCommit,
    the_landed_commit,
};
use crate::service_messages::ClientFailure;
use crate::service_prepare::{
    HandedBackTree,
    NativeInputs,
    Ready,
};
use crate::service_protocol::CommitRequest;
use crate::standard_error_off_the_calling_thread::StandardErrorOffTheCallingThread;
use std::io::Write;
use std::time::Instant;

/// What became of one commit when the service answered.
pub(crate) enum WhatTheServiceSaid {
    /// The commit landed,
    /// or was written and kept behind a reference lock,
    /// and its ending is the command's.
    Landed(ServiceCommit),
    /// This attempt did not land,
    /// and the next one starts from what it wrote into the attempt's values.
    Rebuild,
}

/// Everything one answer of the service is read with:
/// the values the attempt carries,
/// the timing record the landing adds to,
/// and what this attempt built.
pub(crate) struct TheAnswerOfTheService<'a> {
    /// The values this command carries from attempt to attempt.
    pub(crate) values: &'a mut TheAttempt,
    /// The timing record the landing's own stretch is added to.
    pub(crate) timing: &'a mut CommitTiming,
    /// The command line this commit was made for.
    pub(crate) invocation: &'a CommitInvocation<'a>,
    /// What the service answered,
    /// or the failure the client met.
    pub(crate) submitted: Result<Submitted, ClientFailure>,
    /// The attempt this answer belongs to,
    /// as the preparation left it.
    /// It is a mutable one because the two answers that prepare the commit again hand its scratch tree back to this command.
    pub(crate) ready: &'a mut Ready,
    /// The request this attempt sent.
    pub(crate) sent: &'a CommitRequest,
    /// The parent the commit this attempt made has.
    pub(crate) first_parent: String,
    /// The message the person typed,
    /// when the command read one.
    pub(crate) typed: Option<String>,
    /// Whether Git built this attempt's commit in a scratch repository.
    pub(crate) native: bool,
    /// Whether this attempt lands an amend on the branch as it is now.
    pub(crate) lands_as_new: bool,
    /// The number of this attempt,
    /// counted from zero.
    pub(crate) attempt: usize,
}

/// What:
///  Reads the service's answer and says what the command does with it.
/// Why:
///  The arms below are the two ends of a commit:
///  it landed and the person is told what came after it,
///  or nothing landed and the attempt is prepared again,
///  so the loop that sent the request asks one question here and never reads an answer itself.
///  A commit held behind a reference lock is a landed one:
///  the owner's rule is that commits never fail,
///  and the words of the hold go to the thread that writes standard error,
///  because the exit status already says the result and a reader that stopped draining must not keep the command from
///  returning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// switch (theAnswerOfTheService(answer)) { case landed: return landed; case rebuild: continue }
/// ```
///
/// # Errors
///
/// `TransactionStop` when a preparation that did not land cannot be rebuilt,
/// when the failure a client met ends the command,
/// and when the branch moved under an attempt whose change cannot be carried onto it.
pub(crate) fn the_answer_of_the_service(
    answer: TheAnswerOfTheService<'_>
) -> Result<WhatTheServiceSaid, TransactionStop> {
    let TheAnswerOfTheService {
        values,
        timing,
        invocation,
        submitted,
        ready,
        sent,
        first_parent,
        typed,
        native,
        lands_as_new,
        attempt,
    } = answer;
    // The prepared inputs and the commit the command line spelled are read here rather than handed in, so the loop that sends
    //  the request holds no borrow of this attempt while this answer writes the values.
    let native_inputs: &NativeInputs = &ready.inputs;
    let embedded_in_use: Option<&Embedded> = if native || lands_as_new {
        None
    } else {
        values.embedded.as_ref()
    };
    return match submitted {
        Ok(Submitted::Landed {
            commit,
            ..
        }) => {
            let after_landing: Instant = Instant::now();
            let outcome: ServiceCommit = the_landed_commit(LandedAftermath {
                invocation,
                ready: &*ready,
                events: ready.events.clone(),
                native_inputs,
                commit: Some(commit.as_str()),
                first_parent,
                fixed_message: values.fixed_message.as_ref(),
                embedded_in_use,
                amended_as_new: if lands_as_new {
                    values.amended.as_ref()
                } else {
                    None
                },
                unlanded: values.unlanded.as_slice(),
                request_id: values.id.as_str(),
                // Nothing looks for a commit that an answer named.
                journal: None,
                config: values.config.clone(),
            });
            timing.after_landing += after_landing.elapsed();
            Ok(WhatTheServiceSaid::Landed(outcome))
        }
        Ok(Submitted::Held {
            words,
            ..
        }) => {
            // The commit is written and kept, and a reference lock keeps its branch where it is: the owner's rule is that
            //  commits never fail, so the command says what became of it and exits 0 (`SPEC.md`, "A commit held behind a
            //  reference lock"). Nothing is pushed and nothing is reviewed here: no branch names the commit yet, and the
            //  command that made it is over before its gate could run.
            // What: The held commit's words go to the thread that writes standard error.
            // Why:  The exit status already says the result, and a reader that stopped draining must
            //       not keep this command from returning.
            let mut notes: StandardErrorOffTheCallingThread<fn(&[u8])> =
                StandardErrorOffTheCallingThread::new();
            let _written = writeln!(
                notes,
                "{}: {words}",
                crate::wrapper_names_old_and_new::PROGRAM.current
            );
            Ok(WhatTheServiceSaid::Landed(ServiceCommit {
                result: TransactionResult {
                    events: Vec::new(),
                    exit_code: 0,
                    landed_oid: None,
                    uses_lfs: false,
                },
                handoff: None,
            }))
        }
        Ok(Submitted::PrepareAgain {
            landed,
            ..
        }) => {
            // The service did not land this attempt, so it left the tree to this command, which reads the change it
            //  captured from the tree's private index here (`retry_of`) and releases the tree once the arm ends.
            //  Starting the removal before that read took the private index from under it: `git write-tree` then failed
            //  with a directory that was not there, or wrote the empty tree and made the commit a deletion of every file.
            let _released: HandedBackTree = HandedBackTree(ready.scratch.0.clone());
            ready.scratch.held_by_the_service(true);
            match after_a_prepare_again(
                native,
                sent.amend,
                ready,
                invocation,
                landed,
                attempt,
                values.first_build_began,
            ) {
                AfterAPrepareAgain::Stopped(stop) => Err(stop),
                AfterAPrepareAgain::Retry(next) => {
                    values.retry = Some(next);
                    Ok(WhatTheServiceSaid::Rebuild)
                }
            }
        }
        Ok(Submitted::Moved) => {
            // Nothing landed on this attempt's tree, so the service left it to this command: whatever this arm comes to, the
            //  tree is released beside the command once the change it captured has been read from it.
            let _released: HandedBackTree = HandedBackTree(ready.scratch.0.clone());
            ready.scratch.held_by_the_service(true);
            values.retry = Some(after_a_moved_branch(MovedBranch {
                invocation,
                ready: &*ready,
                sent,
                embedded: values.embedded.as_ref(),
                native_inputs,
                typed: typed.as_deref(),
                native,
                editor_possible: values.editor_possible,
                rewords: values.rewords,
                attempt,
                first_build_began: values.first_build_began,
                id: values.id.as_str(),
                amended: &mut values.amended,
                fixed_message: &mut values.fixed_message,
                reissued_line: &mut values.reissued_line,
                unlanded: &mut values.unlanded,
            })?);
            Ok(WhatTheServiceSaid::Rebuild)
        }
        Err(failure) => {
            let failed: crate::service_commit_after_a_failure::FailedAttempt<'_> =
                crate::service_commit_after_a_failure::FailedAttempt {
                    failure,
                    sent,
                    events: ready.events.as_slice(),
                    ready,
                    invocation,
                    native_inputs,
                    first_parent: first_parent.as_str(),
                    fixed_message: values.fixed_message.as_ref(),
                    embedded_in_use,
                    amended_as_new: if lands_as_new {
                        values.amended.as_ref()
                    } else {
                        None
                    },
                    unlanded: values.unlanded.as_slice(),
                    request_id: values.id.as_str(),
                    config: &values.config,
                };
            return match crate::service_commit_after_a_failure::answer_after_a_failure(failed) {
                Ok(commit) => Ok(WhatTheServiceSaid::Landed(commit)),
                Err(stop) => Err(stop),
            };
        }
    };
}
