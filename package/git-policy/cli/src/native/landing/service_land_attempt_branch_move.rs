//! What:
//!  The one branch write of a landing attempt:
//!  the removal of the batch's scratch reference,
//!  the transaction that moves the branch,
//!  and the words that say what the write is doing while it runs.
//! Why:
//!  The write is never ended by the wrapper,
//!  so everything a person can read about it stands in one place
//!  (`service_busy.rs`,
//! `AGENTS.md` rules `SYB` and `STB`),
//!  and a write Git never started ends the batch here
//!  before the body judges what a refusal means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const moved = moveTheBranch(the, built); // throws the code the write ended with
//! ```

use crate::service_land_attempt_batch_built::WhatTheBatchBuilt;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_git_steps_and_import_stream::run_words;
use crate::service_land_hold_a_batch::{
    DropTheBatch,
    end_a_batch_the_write_never_started,
};
use crate::service_protocol::FailureCode;
use crate::transaction_git::GitOutput;
use std::time::Instant;

/// What:
///  Writes the transaction that moves the branch,
///  or ends the batch when Git never started.
/// Why:
///  A branch that did not move leaves nothing of the batch landing,
///  so what the batch took is given back here,
///  and a step ended at a ceiling keeps the commit the reference holds first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveTheBranch(the: TheAttempt, built: WhatTheBatchBuilt): GitOutput
/// ```
///
/// # Errors
///
/// The code the write ended with,
/// after the batch is ended and the person is told what became of its commit.
pub(crate) fn move_the_branch(
    the: &mut TheAttempt<'_>,
    built: &WhatTheBatchBuilt,
) -> Result<GitOutput, FailureCode> {
    crate::service_fault::reach("before-ref");
    // Every batch holds its commits on the scratch branch until this move, a commit Git built included: the branch write
    //  removes it in the same transaction (`service_land_objects_of_batch.rs`). A reference this attempt did not prove is not
    //  this landing's to remove: the name may hold another command's commit, so the delete is left out and the branch move
    //  names the commit without it (`service_land_without_its_scratch_reference.rs`).
    let removal: String =
        crate::service_land_without_its_scratch_reference::the_removal_of_the_scratch_reference(
            the.scratch.as_str(),
        );
    // A detached head is the `HEAD` file itself and is updated as such. The update follows a symbolic `HEAD` as native Git's
    //  own commit does, because a compare that does not follow it replaces a head attached to a branch since the last look
    //  (measured: `option no-deref` with `update HEAD <new> <old>` replaced `ref: refs/heads/B` at the old name with the
    //  detached new one). A head that became a branch at another value fails the compare; at the same value the branch it
    //  names takes the commit, and the head is read again below.
    let transaction: Vec<u8> =
        crate::service_land_branch_move_transaction::reference_move_transaction(
            &the.first.reference,
            built.new.as_str(),
            the.old.as_str(),
            removal.as_str(),
        );
    // The message is one argument of `-m`, never a line of the transaction, so no byte of the commit's message can change
    // the grammar Git reads on standard input (`AGENTS.md` rules `SYB` and `STB`).
    let moving: Vec<std::ffi::OsString> = vec![
        std::ffi::OsString::from("update-ref"),
        std::ffi::OsString::from("-m"),
        built.reflog.clone(),
        std::ffi::OsString::from("--stdin"),
    ];
    let ref_began: Instant = Instant::now();
    // The write is never ended by the wrapper (owner, 2026-11-04: "Never cut a branch write (Recommended)"), so a write that never
    //  returns holds this thread; the clients that queue behind it are told what it is stuck in, and the commit it makes is on
    //  the scratch branch until the update renames the reference (`service_busy.rs`).
    let moved: GitOutput = {
        let described: String = if the.first.commit.is_some() {
            format!(
                "it moves {} to {new}",
                the.first.reference.spoken(),
                new = built.new
            )
        } else {
            format!(
                "it moves {} to {new}; the commit is also at {scratch} until the update ends",
                the.first.reference.spoken(),
                new = built.new,
                scratch = the.scratch
            )
        };
        let _described = crate::service_busy_step_record_on_this_thread::detail(described);
        match run_words(
            &the.git,
            the.repository,
            moving.as_slice(),
            Some(transaction),
        ) {
            Ok(output) => output,
            // Git never started, so the branch did not move and nothing of this batch lands: what the batch took is given
            //  back, so no record of a landing that did not happen outlives it. A step ended at a ceiling keeps the commit
            //  the reference holds first (`land_batch_timed`), so that reference is left for it.
            Err(code) => {
                let drop: DropTheBatch<'_> = DropTheBatch {
                    git: &the.git,
                    context: the.context,
                    repository: the.repository,
                    seq: the.seq,
                    intent: &built.intent,
                    scratch: the.scratch.as_str(),
                };
                end_a_batch_the_write_never_started(&drop, code, the.announce);
                return Err(code);
            }
        }
    };
    the.timing.ref_update += ref_began.elapsed();
    return Ok(moved);
}
