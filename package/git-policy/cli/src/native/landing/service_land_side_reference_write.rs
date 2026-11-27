//! What:
//!  The face a landing's code calls for its own side references:
//!  the answer of one Git step,
//!  the write itself and the words of a refusal,
//!  the write named for each reference of the wrapper's own,
//!  the commit a write did not prove,
//!  and the tip a batch built that its reference did not hold.
//! Why:
//!  Each part is a file of its own beside this one,
//!  named for what it holds,
//!  and the names callers already use are re-exported here,
//!  so no caller changed when the parts moved.
//!  The file was over the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`),
//!  and the parts are listed in the folder's file list before it,
//!  in the order each uses the others.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export * from './the_answer_of_one_git_step_of_a_side_reference'
//! // ... and one line per part below.
//! ```

/// The answer of one Git step,
/// and the sentence of a step that was ended at its ceiling.
pub(crate) use crate::the_answer_of_one_git_step_of_a_side_reference::{
    StepAnswer,
    answering,
    answering_awaiting,
    take_ceiling_step,
};

/// The sentence a step that was ended at its ceiling is kept by,
/// which only a control of the landing names here.
#[cfg(test)]
pub(crate) use crate::the_answer_of_one_git_step_of_a_side_reference::keep_the_ceiling_step;

/// The write itself:
/// what became of it,
/// and whether the reference's own value names the commit.
pub(crate) use crate::the_write_of_a_side_reference::{
    ASideRefWrite,
    refused_words_of,
    the_batch_reference_names,
    the_refusal_line_of,
};

/// The write named for each reference of the wrapper's own.
pub(crate) use crate::the_side_references_a_commit_is_kept_on::{
    keep_on_a_side_ref,
    keep_on_the_scratch_reference,
    keep_the_batches_commit,
};

/// The same write awaited on the runtime,
/// which only a control names here.
#[cfg(test)]
pub(crate) use crate::the_side_references_a_commit_is_kept_on::keep_on_the_scratch_reference_awaiting;

/// The commit a write did not prove,
/// the sentence the person reads about it,
/// and letting go of the reference it was held on.
pub(crate) use crate::the_commit_a_side_reference_did_not_keep::{
    AKeptCommit,
    keep_an_unlanded_commit,
    keep_what_a_side_reference_write_left,
    let_go_of_a_reference_only_while_it_names,
    remember_the_tip_a_side_reference_does_not_hold,
};

/// The two writes of that commit that only a control names here.
#[cfg(test)]
pub(crate) use crate::the_commit_a_side_reference_did_not_keep::{
    keep_unlanded,
    keep_unlanded_awaiting,
};
