//! What:
//!  The shape every way out of a preparation shares:
//!  what a conclusion was prepared over,
//!  the merge,
//!  cherry-pick or revert state a landing will remove,
//!  and the parts each `Ready` of this preparation is built from.
//! Why:
//!   A retry compares the conclusion it replays against the state this preparation saw,
//!  so the state is read once,
//!  here,
//!  and not when a retry first needs it;
//!  and one record carries what the endings of a preparation share instead of every one of them
//!  being handed fifteen values.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [shape, statePresent] = theReadyShape(invocation, setup, capturedAt);
//! ```

use super::service_prepare_what_it_reads::Preparation;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionStop,
};
use crate::conclusion_cleanup::{
    CONCLUSION_STATE_FILES,
    SEQUENCER_DIRECTORY,
};
use crate::conclusion_copy::the_state_of_a_conclusion;
use crate::service_prepare_ready_shapes::ReadyShape;
use crate::service_protocol::Cleanup;
use crate::service_replay_conclusion::the_state_read_failure;
use crate::transaction_git::GitContext;
use crate::transaction_journal::RefFormat;
use std::path::Path;

/// What:
///  Builds the shape every ending of one preparation shares,
/// with whether a merge,
/// a cherry-pick or a revert left state this landing removes.
/// Why:
///   The shape holds the pathspecs,
/// the reference and the scratch state that every `Ready` needs,
///  so the endings pass one record rather than fifteen values,
///  and the state is read at the one moment a retry can be compared against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theReadyShape(invocation, setup, capturedAt): [ReadyShape, boolean]
/// ```
pub(crate) fn the_ready_shape(
    invocation: &CommitInvocation<'_>,
    setup: &Preparation,
    captured_at: u64,
) -> Result<(ReadyShape, bool), TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let region = invocation.region;
    let interactive: bool = region.interactive || region.patch;
    let reftable: bool = setup.capture.layout.ref_format == RefFormat::Reftable;
    // Whatever state a merge or a pick left (also a squash message with no merge in progress) is removed as native Git removes it.
    let mut state_present: bool = setup.selection.concludes_sequencer;
    for entry in CONCLUSION_STATE_FILES
        .iter()
        .chain([SEQUENCER_DIRECTORY].iter())
    {
        if state_present {
            break;
        }
        if setup.capture.layout.git_dir.join(entry).exists() {
            state_present = true;
        }
    }
    let cleanup: Option<Cleanup> = if state_present {
        Some(Cleanup {
            git_dir: setup.capture.layout.git_dir.clone(),
            scratch: setup.scratch.0.clone(),
            reftable,
        })
    } else {
        None
    };
    let selected_privately: bool = setup.selection.read_only;
    let fixed_parents: bool = setup.selection.concludes_sequencer || setup.unborn;
    // What a conclusion was prepared over, read here and not when a retry needs it: another command may conclude the
    //  operation while this commit waits, and the replay must compare against the state this preparation saw.
    let conclusion_state: Option<String> = the_state_of_a_conclusion(
        context,
        cwd,
        setup.capture.layout.git_dir.as_path(),
        setup.capture.layout.ref_format,
        setup.capture.layout.conclusion,
    )
    .map_err(the_state_read_failure)?;
    // What every `Ready` of this preparation shares; the closures that carried it are the methods of this record.
    let shape: ReadyShape = ReadyShape {
        pathspecs: setup.pathspecs.clone(),
        mode: setup.selection.mode,
        staged_all: setup.selection.staged_all,
        selected_privately,
        commit_index: setup.indexes.commit_index.clone(),
        reference: setup.reference.clone(),
        announced_base: setup.announced_base.clone(),
        cleanup: cleanup.clone(),
        interactive,
        fixed_parents,
        captured_at,
        conclusion_state,
    };
    // What: `(shape, state_present)` hands back the record every ending shares, and whether state must be removed after the landing.
    // Why: the caller builds the endings from one record, and the tail decides from the state whether it reads the changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [shape, statePresent];
    // ```
    return Ok((shape, state_present));
}
